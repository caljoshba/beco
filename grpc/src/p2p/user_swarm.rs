use super::config::Config;
use super::{behaviour::BecoBehaviour, P2P};

use std::sync::Arc;
use std::time::Duration;

use envconfig::Envconfig;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc::Receiver;

use crate::enums::data_value::{DataRequestType, ProcessRequest};

use crate::p2p::USER_NAMESPACE;

use crate::entry::Entry;

use libp2p::{gossipsub, identity, Multiaddr, PeerId, Swarm};

use crate::{enums::data_value::DataRequests, utils::calculate_hash};


impl P2P {
    pub fn new(entry: &'static Arc<Entry>, rx_p2p: Receiver<Value>) -> Self {
        let config = Config::init_from_env().unwrap();
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let propose_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::PROPOSE.to_string());
        let corroborate_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::CORROBORATE.to_string());
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        let load_user_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::LOAD.to_string());
        let new_user_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::NEW.to_string());
        let response_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::RESPONSE.to_string());
        let rendezvous_address = config
            .rendezvous_address
            .clone()
            .parse::<Multiaddr>()
            .unwrap();
        Self {
            keys,
            peer_id,
            entry,
            rx_p2p,
            propose_gossip_sub,
            corroborate_gossip_sub,
            validated_gossip_sub,
            load_user_gossip_sub,
            new_user_gossip_sub,
            response_gossip_sub,
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
        println!("Subscribing to {:?}", self.load_user_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.load_user_gossip_sub)
            .unwrap();
        println!("Subscribing to {:?}", self.new_user_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.new_user_gossip_sub)
            .unwrap();
        println!("Subscribing to {:?}", self.response_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.response_gossip_sub)
            .unwrap();
    }

    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().unwrap();
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        loop {
            tokio::select! {
                Some(message) = self.rx_p2p.recv() => {
                    let mut request: ProcessRequest = serde_json::from_value(message).unwrap();
                    request.originator_peer_id = Some(self.peer_id.to_string());
                    match request.status {
                        DataRequestType::IGNORED => { request.ignore_signatures.insert(self.peer_id.to_string()); },
                        DataRequestType::INVALID => { request.failed_signatures.insert(self.peer_id.to_string()); },
                        DataRequestType::VALID => { request.validated_signatures.insert(self.peer_id.to_string()); },
                        _ => {},
                    };
                    let gossip_sub = match request.status {
                        DataRequestType::PROPOSE => { &self.propose_gossip_sub },
                        DataRequestType::CORROBORATE => { &self.corroborate_gossip_sub },
                        DataRequestType::IGNORED => { &self.corroborate_gossip_sub },
                        DataRequestType::INVALID => { &self.corroborate_gossip_sub },
                        DataRequestType::VALID => { &self.corroborate_gossip_sub },
                        DataRequestType::NOTFOUND => { &self.corroborate_gossip_sub },
                        DataRequestType::VALIDATED => { &self.validated_gossip_sub },
                        DataRequestType::FAILED => { &self.validated_gossip_sub },
                        DataRequestType::LOAD => { &self.load_user_gossip_sub },
                        DataRequestType::NEW => { &self.new_user_gossip_sub},
                        DataRequestType::RESPONSE => { &self.response_gossip_sub},
                        DataRequestType::FETCH => { &self.load_user_gossip_sub},
                    };
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(gossip_sub.clone(), serde_json::to_vec(&request).unwrap())
                    {
                        println!("Publish error: {e:?}");
                    }
                    println!("Mesage sent with status: {}", request.status);
                }
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        self.discover_rendzvous(&mut swarm, USER_NAMESPACE.to_string(), cookie.clone()).await;
                        // swarm.behaviour_mut().rendezvous.discover(
                        //     Some(rendezvous::Namespace::new(USER_NAMESPACE.to_string()).unwrap()),
                        //     cookie.clone(),
                        //     None,
                        //     rendezvous_peer_id().await.clone()
                        // );
                    }
                }
                // need a mechanism to routinely go around and check the queues process the next one if something went wrong
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
            DataRequestType::CORROBORATE => {
                let response = self.entry.corroborate(&mut process_request).await;
            }
            DataRequestType::VALIDATED => {
                let hash = calculate_hash(&process_request.request);
                let response = self
                    .entry
                    .update(
                        process_request.request,
                        process_request.calling_user,
                        process_request.user_id.clone(),
                    )
                    .await;
                if response.is_ok() {
                    self.entry
                        .success_event(hash, Some(process_request.user_id), process_request.status)
                        .await;
                    self.entry.ping_event(&hash).await;
                } else {
                    self.entry
                        .fail_event(hash, Some(process_request.user_id))
                        .await;
                    self.entry.ping_event(&hash).await;
                }
            }
            DataRequestType::FAILED | DataRequestType::NOTFOUND => {
                let hash = calculate_hash(&process_request.request);
                self.entry
                    .fail_event(hash, Some(process_request.user_id))
                    .await;
                self.entry.ping_event(&hash).await;
            }
            DataRequestType::RESPONSE => {
                if process_request.originator_hash.is_none()
                    || process_request.originator_peer_id.is_none()
                    || process_request.originator_peer_id.unwrap() != self.peer_id.to_string()
                {
                    println!("not the same originator");
                    return;
                }
                let hash = process_request.originator_hash.unwrap();
                match process_request.request {
                    DataRequests::LoadUser(user_option) => {
                        if user_option.is_none() {
                            if self.entry.does_event_exist(hash).await {
                                self.entry.fail_event(hash, None).await;
                                self.entry.ping_event(&hash).await;
                            }
                            return;
                        }
                        let user = user_option.unwrap();
                        self.entry.load_user(user.clone()).await;
                        self.entry
                            .success_event(hash, Some(user.id), process_request.status)
                            .await;
                        self.entry.ping_event(&hash).await;
                    }
                    _ => {}
                }
            }
            DataRequestType::LOAD => match process_request.request {
                DataRequests::LoadUser(user_option) => {
                    let hash = process_request.originator_hash.unwrap();
                    if user_option.is_none() {
                        if self.entry.does_event_exist(hash).await {
                            self.entry.fail_event(hash, None).await;
                            self.entry.ping_event(&hash).await;
                        }
                        return;
                    }
                    let user = user_option.unwrap();
                    if self.entry.is_user_loaded(&user).await {
                        self.entry.load_user(user.clone()).await;
                    }
                    if hash != 0 && self.entry.does_event_exist(hash).await {
                        self.entry.fail_event(hash, Some(user.id)).await;
                        self.entry.ping_event(&hash).await;
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
}
