use super::config::Config;
use super::{behaviour::BecoBehaviour, P2P};

use std::collections::HashMap;
use std::time::Duration;

use envconfig::Envconfig;
use futures::StreamExt;
use tokio::sync::RwLock;

use crate::enums::data_value::{DataRequestType, ProcessRequest};

use crate::p2p::USER_NAMESPACE;

use libp2p::{gossipsub, identity, Multiaddr, PeerId, Swarm};

use crate::utils::calculate_hash;

use chrono::Utc;

impl P2P {
    pub fn new() -> Self {
        let config = Config::init_from_env().unwrap();
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let propose_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::PROPOSE.to_string());
        let corroborate_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::CORROBORATE.to_string());
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        let rendezvous_address = config
            .rendezvous_address
            .clone()
            .parse::<Multiaddr>()
            .unwrap();
        Self {
            keys,
            peer_id,
            propose_gossip_sub,
            corroborate_gossip_sub,
            validated_gossip_sub,
            proposals_processing: RwLock::new(HashMap::new()),
            proposal_queues: RwLock::new(HashMap::new()),
            config,
            rendezvous_address,
        }
    }

    pub fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
        println!("Subscribing to {:?}", self.propose_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.propose_gossip_sub)
            .unwrap();
        println!("Subscribing to {:?}", self.corroborate_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.corroborate_gossip_sub)
            .unwrap();
        println!("Subscribing to {:?}", self.validated_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.validated_gossip_sub)
            .unwrap();
    }

    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().unwrap();
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        loop {
            tokio::select! {
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        self.discover_rendzvous(&mut swarm, USER_NAMESPACE.to_string(), cookie.clone()).await;
                        // swarm.behaviour_mut().rendezvous.discover(
                        //     Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                        //     cookie.clone(),
                        //     None,
                        //     rendezvous_peer_id().await.clone()
                        // )
                    }
                }
                // need a mechanism to routeinly go around and check the queues process the next one if something went wrong
                event = swarm.select_next_some() => {
                    self.process_swarm_events(event, &mut swarm, &mut cookie).await;
                }
            }
        }
    }

    pub async fn process_gossipsub(
        &self,
        message: gossipsub::Message,
        swarm: &mut Swarm<BecoBehaviour>,
    ) {
        let mut process_request: ProcessRequest =
            serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
        match process_request.status {
            DataRequestType::PROPOSE => {
                let datetime = Utc::now();
                process_request.datetime = Some(datetime);

                let hash = calculate_hash(&process_request);
                process_request.hash = hash;
                process_request.status = DataRequestType::CORROBORATE;

                let connections: usize = swarm.connected_peers().count();
                process_request.connected_peers = if connections > 2 {
                    connections - 2
                } else {
                    connections.clone()
                };

                {
                    let proposal_queue = &mut self.proposal_queues.write().await;
                    if let Some(user_queue) = proposal_queue.get_mut(&process_request.user_id) {
                        user_queue.write().await.push(process_request.clone());
                    } else {
                        proposal_queue.insert(
                            process_request.user_id.clone(),
                            RwLock::new(vec![process_request.clone()]),
                        );
                    }
                }
                self.process_next_request(&process_request.user_id, swarm)
                    .await;
            }
            DataRequestType::VALID | DataRequestType::INVALID | DataRequestType::IGNORED => {
                let hash = calculate_hash(&process_request);
                {
                    let proposals_processing = &mut self.proposals_processing.write().await;
                    if let Some(request) = proposals_processing.get_mut(&process_request.user_id) {
                        if request.hash != hash {
                            return;
                        }
                        for signature in process_request.validated_signatures.iter() {
                            request.validated_signatures.insert(signature.clone());
                        }
                        for signature in process_request.failed_signatures.iter() {
                            request.failed_signatures.insert(signature.clone());
                        }
                        for signature in process_request.ignore_signatures.iter() {
                            request.ignore_signatures.insert(signature.clone());
                        }
                    } else {
                        process_request.status = DataRequestType::NOTFOUND;
                        let _ = swarm.behaviour_mut().gossipsub.publish(
                            self.corroborate_gossip_sub.clone(),
                            serde_json::to_vec(&process_request).unwrap(),
                        );
                    }
                };
                self.check_if_validated(&mut process_request, swarm, hash)
                    .await;
            }
            _ => {}
        };
    }

    async fn check_if_validated(
        &self,
        process_request: &mut ProcessRequest,
        swarm: &mut Swarm<BecoBehaviour>,
        hash: u64,
    ) -> bool {
        let processed = {
            let proposals_processing = &mut self.proposals_processing.write().await;
            let failed_or_validated = if let Some(queued_request) =
                proposals_processing.get_mut(&process_request.user_id)
            {
                let validated_signatures_len = queued_request.validated_signatures.len();
                let validated_signatures_threshold = ((queued_request.connected_peers
                    - queued_request.ignore_signatures.len())
                    as f32
                    * 0.8)
                    .ceil() as usize;

                let failed_signatures_len = queued_request.failed_signatures.len();
                let failed_signatures_threshold = ((queued_request.connected_peers
                    - queued_request.ignore_signatures.len())
                    as f32
                    * 0.2)
                    .ceil() as usize;
                self.process_if_threshold_reached(
                    queued_request,
                    swarm,
                    DataRequestType::VALIDATED,
                    validated_signatures_len,
                    validated_signatures_threshold,
                    hash,
                )
                .await
                    || self
                        .process_if_threshold_reached(
                            queued_request,
                            swarm,
                            DataRequestType::FAILED,
                            failed_signatures_len,
                            failed_signatures_threshold,
                            hash,
                        )
                        .await
            } else {
                false
            };
            failed_or_validated
        };

        if processed {
            {
                self.proposals_processing
                    .write()
                    .await
                    .remove(&process_request.user_id);
            }
            self.process_next_request(&process_request.user_id, swarm)
                .await;
        }
        processed
    }

    async fn process_if_threshold_reached(
        &self,
        queued_request: &mut ProcessRequest,
        swarm: &mut Swarm<BecoBehaviour>,
        status: DataRequestType,
        signatures_len: usize,
        threshold: usize,
        hash: u64,
    ) -> bool {
        if signatures_len < threshold {
            return false;
        }
        queued_request.status = status.clone();
        let result = swarm.behaviour_mut().gossipsub.publish(
            self.validated_gossip_sub.clone(),
            serde_json::to_vec(&queued_request).unwrap(),
        );

        if let Err(e) = result {
            println!("Error pusblishing message: {e:?}");
            return false;
        }
        println!("sent message with status: {}", queued_request.status);
        return true;
    }

    async fn process_next_request(&self, user_id: &String, swarm: &mut Swarm<BecoBehaviour>) {
        let mut proposals_processing = self.proposals_processing.write().await;
        let mut proposal_queues = self.proposal_queues.write().await;
        if proposals_processing.get(user_id).is_none() {
            if let Some(requests) = proposal_queues.get_mut(user_id) {
                let request_lock = &mut requests.write().await;
                if request_lock.len() > 0 {
                    let next_request = request_lock.remove(0);
                    proposals_processing.insert(user_id.clone(), next_request.clone());
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(
                        self.corroborate_gossip_sub.clone(),
                        serde_json::to_vec(&next_request).unwrap(),
                    ) {
                        println!("Publish error: {e:?}");
                    }
                }
            }
        }
    }
}
