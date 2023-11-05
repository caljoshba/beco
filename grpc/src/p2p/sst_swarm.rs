use super::config::Config;
use super::{behaviour::BecoBehaviour, P2P};

use std::time::Duration;

use envconfig::Envconfig;

use crate::enums::data_value::{DataRequestType, DataRequests, ProcessRequest};

use crate::p2p::USER_NAMESPACE;

use futures::StreamExt;

use libp2p::{gossipsub, identity, Multiaddr, PeerId, Swarm};

use crate::utils::calculate_hash;

use crate::merkle::SST;

use chrono::{DateTime, Utc};

use std::collections::HashSet;

use libp2p::gossipsub::IdentTopic;

impl P2P {
    #[cfg(feature = "sst")]
    pub fn new() -> Self {
        let config = Config::init_from_env().unwrap();
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let sst = SST::new();
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        let load_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::LOAD.to_string());
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
            sst,
            validated_gossip_sub,
            load_gossip_sub,
            new_user_gossip_sub,
            response_gossip_sub,
            config,
            rendezvous_address,
        }
    }

    #[cfg(feature = "sst")]
    pub fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
        println!("Subscribing to {:?}", self.validated_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.validated_gossip_sub)
            .unwrap();
        println!("Subscribing to {:?}", self.load_gossip_sub);
        behaviour
            .gossipsub
            .subscribe(&self.load_gossip_sub)
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

    #[cfg(any(feature = "sst"))]
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

    #[cfg(feature = "sst")]
    pub async fn process_gossipsub(
        &self,
        message: gossipsub::Message,
        swarm: &mut Swarm<BecoBehaviour>,
    ) {
        let process_request: ProcessRequest =
            serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
        let result = self.sst.update(process_request.clone()).await;
        match process_request.status {
            DataRequestType::NEW => match process_request.request {
                DataRequests::AddUser(request) => {
                    let (user_option, user_id) = if result.is_err() {
                        (None, "".to_string())
                    } else {
                        let (user, _) = result.unwrap();
                        (Some(user.clone()), user.id)
                    };
                    let data_request = DataRequests::LoadUser(user_option.clone());
                    P2P::send_process_request(
                        swarm,
                        self.response_gossip_sub.clone(),
                        DataRequestType::RESPONSE,
                        data_request,
                        process_request.calling_user,
                        user_id,
                        Some(Utc::now()),
                        process_request.originator_hash,
                        process_request.originator_peer_id,
                    )
                }
                _ => {}
            },
            DataRequestType::FETCH => match process_request.request {
                DataRequests::FetchUser(request) => {
                    let user = self.sst.fetch_user(&request.user_id).await;
                    let (status, user_id) = if user.is_none() {
                        (DataRequestType::FAILED, "".to_string())
                    } else {
                        (DataRequestType::RESPONSE, user.clone().unwrap().id)
                    };

                    let data_request = DataRequests::LoadUser(user.clone());
                    P2P::send_process_request(
                        swarm,
                        self.response_gossip_sub.clone(),
                        status,
                        data_request,
                        process_request.calling_user,
                        user_id,
                        Some(Utc::now()),
                        process_request.originator_hash,
                        process_request.originator_peer_id,
                    )
                }
                _ => {}
            },
            _ => {}
        };
    }

    #[cfg(feature = "sst")]
    fn send_process_request(
        swarm: &mut Swarm<BecoBehaviour>,
        topic: IdentTopic,
        status: DataRequestType,
        data_request: DataRequests,
        calling_user: String,
        user_id: String,
        datetime: Option<DateTime<Utc>>,
        originator_hash: Option<u64>,
        originator_peer_id: Option<String>,
    ) {
        let hash = calculate_hash(&data_request);
        let load_request = ProcessRequest {
            validated_signatures: HashSet::new(),
            failed_signatures: HashSet::new(),
            ignore_signatures: HashSet::new(),
            status,
            request: data_request,
            calling_user: calling_user,
            user_id,
            hash,
            datetime: Some(Utc::now()),
            connected_peers: 0,
            originator_hash: originator_hash,
            originator_peer_id: originator_peer_id,
        };
        let result = swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, serde_json::to_vec(&load_request).unwrap());

        if let Err(e) = result {
            println!("Error pusblishing message: {e:?}");
        }
    }
}
