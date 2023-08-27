use either::Either;
use futures::prelude::*;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport, transport::upgrade::Version},
    gossipsub::{self}, identify, identity,
    multiaddr::Protocol,
    noise, ping,
    pnet::{PnetConfig, PreSharedKey},
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport, Swarm,
};
use serde_json::Value;
use tokio::sync::mpsc::Receiver;
use std::{time::Duration, path::Path, fs, error::Error, str::FromStr, env, sync::Arc};

use crate::{entry::Entry, enums::data_value::{ProcessRequest, DataRequestType}};

// https://github.com/libp2p/rust-libp2p/blob/master/examples/ipfs-private/src/main.rs

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "BecoBehaviourEvent")]
pub struct BecoBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}

impl BecoBehaviour {
    pub fn gossipsub(&self) -> &gossipsub::Behaviour {
        &self.gossipsub
    }
}

pub enum BecoBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for BecoBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BecoBehaviourEvent::Gossipsub(event)
    }
}

impl From<identify::Event> for BecoBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        BecoBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for BecoBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        BecoBehaviourEvent::Ping(event)
    }
}

pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    entry: &'static Arc<Entry>,
    rx_p2p: Receiver<Value>,
    propose_gossip_sub: gossipsub::IdentTopic,
    corroborate_gossip_sub: gossipsub::IdentTopic,
    validated_gossip_sub: gossipsub::IdentTopic,
    load_user_gossip_sub: gossipsub::IdentTopic,
}

impl P2P {
    pub fn new(entry: &'static Arc<Entry>, rx_p2p: Receiver<Value>) -> Self {
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let propose_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::PROPOSE.to_string());
        let corroborate_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::CORROBORATE.to_string());
        let validated_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        let load_user_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::LOAD.to_string());
        Self {
            keys,
            peer_id,
            entry,
            rx_p2p,
            propose_gossip_sub,
            corroborate_gossip_sub,
            validated_gossip_sub,
            load_user_gossip_sub,
        }
    }

    pub fn build_transport(
        &self,
        key_pair: identity::Keypair,
        psk: Option<PreSharedKey>,
    ) -> transport::Boxed<(PeerId, StreamMuxerBox)> {
        let noise_config = noise::Config::new(&key_pair).unwrap();
        let yamux_config = yamux::Config::default();
    
        let base_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
        let maybe_encrypted = match psk {
            Some(psk) => Either::Left(
                base_transport.and_then(move |socket, _| PnetConfig::new(psk).handshake(socket)),
            ),
            None => Either::Right(base_transport),
        };
        maybe_encrypted
            .upgrade(Version::V1Lazy)
            .authenticate(noise_config)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed()
    }

    fn get_psk(&self, path: &Path) -> std::io::Result<Option<String>> {
        let swarm_key_file = path.join("swarm.key");
        match fs::read_to_string(swarm_key_file) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn strip_peer_id(&self, addr: &mut Multiaddr) {
        let last = addr.pop();
        match last {
            Some(Protocol::P2p(peer_id)) => {
                let mut addr = Multiaddr::empty();
                addr.push(Protocol::P2p(peer_id));
                println!("removing peer id {addr} so this address can be dialed by rust-libp2p");
            }
            Some(other) => addr.push(other),
            _ => {}
        }
    }

    fn parse_legacy_multiaddr(&self, text: &str) -> Result<Multiaddr, Box<dyn Error>> {
        let sanitized = text
            .split('/')
            .map(|part| if part == "ipfs" { "p2p" } else { part })
            .collect::<Vec<_>>()
            .join("/");
        let mut res = Multiaddr::from_str(&sanitized)?;
        self.strip_peer_id(&mut res);
        Ok(res)
    }

    pub async fn create_swarm(&self) -> Result<Swarm<BecoBehaviour>, Box<dyn Error>> {
        // env_logger::init();
    
        // let ipfs_path = get_ipfs_path();
        let ipfs_path = Path::new("");
        // println!("using IPFS_PATH {ipfs_path:?}");
        let psk: Option<PreSharedKey> = self.get_psk(&ipfs_path)?
            .map(|text| PreSharedKey::from_str(&text))
            .transpose()?;
    
        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("using random peer id: {local_peer_id:?}");
        if let Some(psk) = psk {
            println!("using swarm key with fingerprint: {}", psk.fingerprint());
        }
    
        // Set up a an encrypted DNS-enabled TCP Transport over and Yamux protocol
        let transport = self.build_transport(local_key.clone(), psk);
    
        // We create a custom network behaviour that combines gossipsub, ping and identify.

        let gossipsub_config = gossipsub::ConfigBuilder::default()
                .max_transmit_size(262144)
                .build()
                .expect("valid config");
        let mut behaviour = BecoBehaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            )
            .expect("Valid configuration"),
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                local_key.public(),
            )),
            ping: ping::Behaviour::new(ping::Config::new()),
        };

        self.subscribe_to_topics(&mut behaviour);
    
        // Create a Swarm to manage peers and events
        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();
    
        // Reach out to other nodes if specified
        // for to_dial in std::env::args().skip(1) {
        //     let addr: Multiaddr = self.parse_legacy_multiaddr(&to_dial)?;
        //     swarm.dial(addr)?;
        //     println!("Dialed {to_dial:?}")
        // }
    
        // Read full lines from stdin
        // let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
    
        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(swarm)
    }

    #[cfg(feature = "grpc")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
        println!("Subscribing to {:?}", self.propose_gossip_sub);
        behaviour.gossipsub.subscribe(&self.propose_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.corroborate_gossip_sub);
        behaviour.gossipsub.subscribe(&self.corroborate_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.validated_gossip_sub);
        behaviour.gossipsub.subscribe(&self.validated_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.load_user_gossip_sub);
        behaviour.gossipsub.subscribe(&self.load_user_gossip_sub).unwrap();
    }

    #[cfg(feature = "validator")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
        println!("Subscribing to {:?}", self.propose_gossip_sub);
        behaviour.gossipsub.subscribe(&self.propose_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.corroborate_gossip_sub);
        behaviour.gossipsub.subscribe(&self.corroborate_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.validated_gossip_sub);
        behaviour.gossipsub.subscribe(&self.validated_gossip_sub).unwrap();
    }

    #[cfg(feature = "orchestrator")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
        println!("Subscribing to {:?}", self.validated_gossip_sub);
        behaviour.gossipsub.subscribe(&self.validated_gossip_sub).unwrap();
        println!("Subscribing to {:?}", self.load_user_gossip_sub);
        behaviour.gossipsub.subscribe(&self.load_user_gossip_sub).unwrap();
    }

    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        if let Ok(peer) = env::var("PEER") {
            let addr = peer.parse::<Multiaddr>().unwrap();
            let _ = swarm.dial(addr);
        }
        loop {
            tokio::select! {
                Some(message) = self.rx_p2p.recv() => {
                    let request: ProcessRequest = serde_json::from_value(message.clone()).unwrap();
                    let gossip_sub = match request.status {
                        DataRequestType::PROPOSE => { &self.propose_gossip_sub },
                        DataRequestType::CORROBORATE => { &self.corroborate_gossip_sub },
                        DataRequestType::IGNORED => { &self.corroborate_gossip_sub },
                        DataRequestType::INVALID => { &self.corroborate_gossip_sub },
                        DataRequestType::VALIDATED => { &self.validated_gossip_sub },
                        DataRequestType::FAILED => { &self.validated_gossip_sub },
                        DataRequestType::LOAD => { &self.load_user_gossip_sub },
                    };
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(gossip_sub.clone(), serde_json::to_vec(&message).unwrap())
                    {
                        println!("Publish error: {e:?}");
                    }
                    println!("Mesage sent");
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {address:?}");
                        }
                        SwarmEvent::IncomingConnection { local_addr, .. } => {
                            println!("incoming connection: {local_addr:?}");
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(event)) => {
                            println!("identify: {event:?}");
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source: peer_id,
                            message_id: id,
                            message,
                        })) => {
                            println!(
                                "Got message: {} with id: {} from peer: {:?}",
                                String::from_utf8_lossy(&message.data),
                                id,
                                peer_id
                            );
                            let propose_request: ProcessRequest = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            match propose_request.status {
                                DataRequestType::PROPOSE => {
                                    let request = propose_request.request;
                                    let response = self.entry.propose(request, propose_request.calling_user, propose_request.user_id, DataRequestType::CORROBORATE, propose_request.hash).await;
                                    println!("{response:?}");
                                },
                                DataRequestType::VALIDATED => {
                                    let request = propose_request.request;
                                    let response = self.entry.update(request, propose_request.calling_user, propose_request.user_id, propose_request.hash).await;
                                    println!("{response:?}");
                                    if response.is_ok() {
                                        self.entry.success_event(propose_request.hash).await;
                                        self.entry.ping_event(&propose_request.hash).await;
                                    }
                                },
                                DataRequestType::FAILED => {
                                    self.entry.fail_event(propose_request.hash).await;
                                    self.entry.ping_event(&propose_request.hash).await;
                                },
                                _ => {},
                            };                          
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Ping(event)) => {
                            match event {
                                ping::Event {
                                    peer,
                                    result: Result::Ok(rtt),
                                    ..
                                } => {
                                    println!(
                                        "ping: rtt to {} is {} ms",
                                        peer.to_base58(),
                                        rtt.as_millis()
                                    );
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Timeout),
                                    ..
                                } => {
                                    println!("ping: timeout to {}", peer.to_base58());
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Unsupported),
                                    ..
                                } => {
                                    println!("ping: {} does not support ping protocol", peer.to_base58());
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Other { error }),
                                    ..
                                } => {
                                    println!("ping: ping::Failure with {}: {error}", peer.to_base58());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // pub async fn main(&self) -> Result<(), Box<dyn Error>> {
    //     // env_logger::init();
    
    //     // let ipfs_path = get_ipfs_path();
    //     let ipfs_path = Path::new("");
    //     // println!("using IPFS_PATH {ipfs_path:?}");
    //     let psk: Option<PreSharedKey> = self.get_psk(&ipfs_path)?
    //         .map(|text| PreSharedKey::from_str(&text))
    //         .transpose()?;
    
    //     // Create a random PeerId
    //     let local_key = identity::Keypair::generate_ed25519();
    //     let local_peer_id = PeerId::from(local_key.public());
    //     println!("using random peer id: {local_peer_id:?}");
    //     if let Some(psk) = psk {
    //         println!("using swarm key with fingerprint: {}", psk.fingerprint());
    //     }
    
    //     // Set up a an encrypted DNS-enabled TCP Transport over and Yamux protocol
    //     let transport = self.build_transport(local_key.clone(), psk);
    
    //     // Create a Gosspipsub topic
    //     let gossipsub_topic = gossipsub::IdentTopic::new("chat");
    
    //     // We create a custom network behaviour that combines gossipsub, ping and identify.
    //     #[derive(NetworkBehaviour)]
    //     #[behaviour(to_swarm = "BecoBehaviourEvent")]
    //     struct BecoBehaviour {
    //         gossipsub: gossipsub::Behaviour,
    //         identify: identify::Behaviour,
    //         ping: ping::Behaviour,
    //     }
    
    //     enum BecoBehaviourEvent {
    //         Gossipsub(gossipsub::Event),
    //         Identify(identify::Event),
    //         Ping(ping::Event),
    //     }
    
    //     impl From<gossipsub::Event> for BecoBehaviourEvent {
    //         fn from(event: gossipsub::Event) -> Self {
    //             BecoBehaviourEvent::Gossipsub(event)
    //         }
    //     }
    
    //     impl From<identify::Event> for BecoBehaviourEvent {
    //         fn from(event: identify::Event) -> Self {
    //             BecoBehaviourEvent::Identify(event)
    //         }
    //     }
    
    //     impl From<ping::Event> for BecoBehaviourEvent {
    //         fn from(event: ping::Event) -> Self {
    //             BecoBehaviourEvent::Ping(event)
    //         }
    //     }
    
    //     // Create a Swarm to manage peers and events
    //     let mut swarm = {
    //         let gossipsub_config = gossipsub::ConfigBuilder::default()
    //             .max_transmit_size(262144)
    //             .build()
    //             .expect("valid config");
    //         let mut behaviour = BecoBehaviour {
    //             gossipsub: gossipsub::Behaviour::new(
    //                 gossipsub::MessageAuthenticity::Signed(local_key.clone()),
    //                 gossipsub_config,
    //             )
    //             .expect("Valid configuration"),
    //             identify: identify::Behaviour::new(identify::Config::new(
    //                 "/ipfs/0.1.0".into(),
    //                 local_key.public(),
    //             )),
    //             ping: ping::Behaviour::new(ping::Config::new()),
    //         };
    
    //         println!("Subscribing to {gossipsub_topic:?}");
    //         behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
    //         SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build()
    //     };
    
    //     // Reach out to other nodes if specified
    //     // for to_dial in std::env::args().skip(1) {
    //     //     let addr: Multiaddr = self.parse_legacy_multiaddr(&to_dial)?;
    //     //     swarm.dial(addr)?;
    //     //     println!("Dialed {to_dial:?}")
    //     // }
    
    //     // Read full lines from stdin
    //     // let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
    
    //     // Listen on all interfaces and whatever port the OS assigns
    //     swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
        // Kick it off
        // loop {
        //     tokio::select! {
        //         line = stdin.select_next_some() => {
                    // if let Err(e) = swarm
                    //     .behaviour_mut()
                    //     .gossipsub
                    //     .publish(gossipsub_topic.clone(), line.expect("Stdin not to close").as_bytes())
                    // {
                    //     println!("Publish error: {e:?}");
                    // }
        //         },
        //         event = swarm.select_next_some() => {
        //             match event {
        //                 SwarmEvent::NewListenAddr { address, .. } => {
        //                     println!("Listening on {address:?}");
        //                 }
        //                 SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(event)) => {
        //                     println!("identify: {event:?}");
        //                 }
        //                 SwarmEvent::Behaviour(BecoBehaviourEvent::Gossipsub(gossipsub::Event::Message {
        //                     propagation_source: peer_id,
        //                     message_id: id,
        //                     message,
        //                 })) => {
        //                     println!(
        //                         "Got message: {} with id: {} from peer: {:?}",
        //                         String::from_utf8_lossy(&message.data),
        //                         id,
        //                         peer_id
        //                     )
        //                 }
        //                 SwarmEvent::Behaviour(BecoBehaviourEvent::Ping(event)) => {
        //                     match event {
        //                         ping::Event {
        //                             peer,
        //                             result: Result::Ok(rtt),
        //                             ..
        //                         } => {
        //                             println!(
        //                                 "ping: rtt to {} is {} ms",
        //                                 peer.to_base58(),
        //                                 rtt.as_millis()
        //                             );
        //                         }
        //                         ping::Event {
        //                             peer,
        //                             result: Result::Err(ping::Failure::Timeout),
        //                             ..
        //                         } => {
        //                             println!("ping: timeout to {}", peer.to_base58());
        //                         }
        //                         ping::Event {
        //                             peer,
        //                             result: Result::Err(ping::Failure::Unsupported),
        //                             ..
        //                         } => {
        //                             println!("ping: {} does not support ping protocol", peer.to_base58());
        //                         }
        //                         ping::Event {
        //                             peer,
        //                             result: Result::Err(ping::Failure::Other { error }),
        //                             ..
        //                         } => {
        //                             println!("ping: ping::Failure with {}: {error}", peer.to_base58());
        //                         }
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //     }
        // }
    // }
}