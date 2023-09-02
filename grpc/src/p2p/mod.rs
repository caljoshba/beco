mod behaviour;

#[cfg(not(feature = "rendezvous"))]
use crate::{
    entry::Entry,
    enums::data_value::{DataRequestType, ProcessRequest},
    p2p::behaviour::{BecoBehaviour, BecoBehaviourEvent},
    utils::{calculate_hash, ProposeEvent},
};
use chrono::prelude::*;
use either::Either;
use futures::{future::Lazy, prelude::*};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport, transport::upgrade::Version},
    gossipsub::{self, MessageId, PublishError},
    identify, identity,
    multiaddr::Protocol,
    noise, ping,
    pnet::{PnetConfig, PreSharedKey},
    rendezvous,
    swarm::{keep_alive, NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use serde_json::Value;
use std::{
    collections::HashMap, env, error::Error, fs, path::Path, str::FromStr, sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc::Receiver, OnceCell, RwLock, RwLockWriteGuard};

#[cfg(feature = "rendezvous")]
use crate::{
    entry::Entry,
    enums::data_value::{DataRequestType, ProcessRequest},
    p2p::behaviour::{RendezvousServerBehaviour, RendezvousServerBehaviourEvent},
    utils::{calculate_hash, ProposeEvent},
};

// https://github.com/libp2p/rust-libp2p/blob/master/examples/ipfs-private/src/main.rs
static RENDEZVOUS_POINT_ADDRESS: OnceCell<Multiaddr> = OnceCell::const_new();
async fn rendezvous_address() -> &'static Multiaddr {
    RENDEZVOUS_POINT_ADDRESS
        .get_or_init(|| async { "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap() })
        .await
}
const NAMESPACE: &str = "rendezvous";

#[cfg(feature = "grpc")]
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

#[cfg(feature = "validator")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    propose_gossip_sub: gossipsub::IdentTopic,
    corroborate_gossip_sub: gossipsub::IdentTopic,
    validated_gossip_sub: gossipsub::IdentTopic,
    proposals_processing: RwLock<HashMap<String, ProcessRequest>>,
    proposal_queues: RwLock<HashMap<String, RwLock<Vec<ProcessRequest>>>>,
}

#[cfg(feature = "orchestrator")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    validated_gossip_sub: gossipsub::IdentTopic,
    load_user_gossip_sub: gossipsub::IdentTopic,
}

#[cfg(feature = "rendezvous")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
}

impl P2P {
    #[cfg(feature = "grpc")]
    pub fn new(entry: &'static Arc<Entry>, rx_p2p: Receiver<Value>) -> Self {
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let propose_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::PROPOSE.to_string());
        let corroborate_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::CORROBORATE.to_string());
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
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

    #[cfg(feature = "validator")]
    pub fn new() -> Self {
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let propose_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::PROPOSE.to_string());
        let corroborate_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::CORROBORATE.to_string());
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        Self {
            keys,
            peer_id,
            propose_gossip_sub,
            corroborate_gossip_sub,
            validated_gossip_sub,
            proposals_processing: RwLock::new(HashMap::new()),
            proposal_queues: RwLock::new(HashMap::new()),
        }
    }

    #[cfg(feature = "rendezvous")]
    pub fn new() -> Self {
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        Self { keys, peer_id }
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

    #[cfg(feature = "rendezvous")]
    pub async fn create_swarm(&self) -> Result<Swarm<RendezvousServerBehaviour>, Box<dyn Error>> {
        let ipfs_path = Path::new("");
        // println!("using IPFS_PATH {ipfs_path:?}");
        let psk: Option<PreSharedKey> = self
            .get_psk(&ipfs_path)?
            .map(|text| PreSharedKey::from_str(&text))
            .transpose()?;

        // Create a random PeerId
        if let Some(psk) = psk {
            println!("using swarm key with fingerprint: {}", psk.fingerprint());
        }

        // Set up a an encrypted DNS-enabled TCP Transport over and Yamux protocol
        let transport = self.build_transport(self.keys.clone(), psk);

        let mut behaviour = RendezvousServerBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                self.keys.public(),
            )),
            ping: ping::Behaviour::new(ping::Config::new()),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            keep_alive: keep_alive::Behaviour,
        };

        // Create a Swarm to manage peers and events
        let mut swarm =
            SwarmBuilder::with_tokio_executor(transport, behaviour, self.peer_id).build();

        swarm.listen_on(rendezvous_address().await.clone())?;
        println!("Peer ID: {:?}", self.peer_id);
        // swarm.dial(rendezvous_address().await.clone())?;

        // Reach out to other nodes if specified
        // for to_dial in std::env::args().skip(1) {
        //     let addr: Multiaddr = self.parse_legacy_multiaddr(&to_dial)?;
        //     swarm.dial(addr)?;
        //     println!("Dialed {to_dial:?}")
        // }

        // Read full lines from stdin
        // let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

        Ok(swarm)
    }

    #[cfg(any(feature = "grpc", feature = "validator", feature = "orhcestrator"))]
    pub async fn create_swarm(&self) -> Result<Swarm<BecoBehaviour>, Box<dyn Error>> {
        // env_logger::init();

        // let ipfs_path = get_ipfs_path();
        let ipfs_path = Path::new("");
        // println!("using IPFS_PATH {ipfs_path:?}");
        let psk: Option<PreSharedKey> = self
            .get_psk(&ipfs_path)?
            .map(|text| PreSharedKey::from_str(&text))
            .transpose()?;

        if let Some(psk) = psk {
            println!("using swarm key with fingerprint: {}", psk.fingerprint());
        }

        // Set up a an encrypted DNS-enabled TCP Transport over and Yamux protocol
        let transport = self.build_transport(self.keys.clone(), psk);

        // We create a custom network behaviour that combines gossipsub, ping and identify.

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .max_transmit_size(262144)
            .build()
            .expect("valid config");
        let mut behaviour = BecoBehaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(self.keys.clone()),
                gossipsub_config,
            )
            .expect("Valid configuration"),
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                self.keys.public(),
            )),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(30))),
            rendezvous: rendezvous::client::Behaviour::new(self.keys.clone()),
            keep_alive: keep_alive::Behaviour,
        };

        self.subscribe_to_topics(&mut behaviour);

        // Create a Swarm to manage peers and events
        let mut swarm =
            SwarmBuilder::with_tokio_executor(transport, behaviour, self.peer_id).build();
        let port = env::var("P2P").unwrap_or("7000".into()).parse::<i32>().unwrap();
        let addr = format!("/ip4/0.0.0.0/tcp/{port}").parse::<Multiaddr>().unwrap();
        swarm.listen_on(addr.clone())?;
        swarm.add_external_address(format!("/ip4/127.0.0.1/tcp/{port}").parse::<Multiaddr>().unwrap());
        swarm.dial(rendezvous_address().await.clone())?;

        // Reach out to other nodes if specified
        // for to_dial in std::env::args().skip(1) {
        //     let addr: Multiaddr = self.parse_legacy_multiaddr(&to_dial)?;
        //     swarm.dial(addr)?;
        //     println!("Dialed {to_dial:?}")
        // }

        // Read full lines from stdin
        // let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

        Ok(swarm)
    }

    #[cfg(feature = "grpc")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
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
    }

    #[cfg(feature = "validator")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
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

    #[cfg(feature = "orchestrator")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
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
    }

    #[cfg(feature = "grpc")]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        // Listen on all interfaces and whatever port the OS assigns
        // if let Ok(peer) = env::var("PEER") {
        //     let addr = peer.parse::<Multiaddr>().unwrap();
        //     let _ = swarm.dial(addr);
        // }
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        let rendezous_peer_id = "12D3KooWCxV8czTueJYfzFsuJcjQPe6R3ZxJZdgcYeZYZS9Drqtw"
            .parse()
            .unwrap();
        loop {
            tokio::select! {
                // need another mechanism to go through and cleanup hanging requests
                Some(message) = self.rx_p2p.recv() => {
                    let request: ProcessRequest = serde_json::from_value(message.clone()).unwrap();
                    let gossip_sub = match request.status {
                        DataRequestType::PROPOSE => { &self.propose_gossip_sub },
                        DataRequestType::CORROBORATE => { &self.corroborate_gossip_sub },
                        DataRequestType::IGNORED => { &self.corroborate_gossip_sub },
                        DataRequestType::INVALID => { &self.corroborate_gossip_sub },
                        DataRequestType::VALID => { &self.corroborate_gossip_sub },
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
                    println!("Mesage sent with status: {}", request.status);
                }
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            cookie.clone(),
                            None,
                            rendezous_peer_id
                            )
                    }
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {address:?}");
                        }
                        SwarmEvent::IncomingConnection { local_addr, .. } => {
                            println!("incoming connection: {local_addr:?}");
                        }
                        // SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(event)) => {
                        //     println!("identify: {event:?}");
                        // }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(identify::Event::Received {
                            ..
                        })) => {
                            if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                                rendezvous::Namespace::from_static(NAMESPACE),
                                rendezous_peer_id.clone(),
                                None,
                            ) {
                                println!("Failed to register: {error}");
                                return;
                            }
                            println!("Connected");
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
                            let mut propose_request: ProcessRequest = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            match propose_request.status {
                                DataRequestType::CORROBORATE => {
                                    let response = self.entry.corroborate(&mut propose_request).await;
                                    println!("{response:?}");
                                },
                                DataRequestType::VALIDATED => {
                                    println!("\n\n\n\n\nGot new validated message");
                                    let hash = calculate_hash(&propose_request.request);
                                    let response = self.entry.update(propose_request.request, propose_request.calling_user, propose_request.user_id).await;
                                    println!("{response:?}");
                                    if response.is_ok() {
                                        self.entry.success_event(hash).await;
                                        self.entry.ping_event(&hash).await;
                                    }
                                },
                                DataRequestType::FAILED => {
                                    let hash = calculate_hash(&propose_request.request);
                                    self.entry.fail_event(hash).await;
                                    self.entry.ping_event(&hash).await;
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
                        SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezous_peer_id => {
                            println!(
                                "Connected to rendezvous point, discovering nodes in '{}' namespace ...",
                                NAMESPACE
                            );

                            swarm.behaviour_mut().rendezvous.discover(
                                Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                                None,
                                None,
                                rendezous_peer_id,
                            );
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(rendezvous::client::Event::Discovered {
                            registrations,
                            cookie: new_cookie,
                            ..
                        })) => {
                            cookie.replace(new_cookie);

                            for registration in registrations {
                                for address in registration.record.addresses() {
                                    let peer = registration.record.peer_id();
                                    println!("Discovered peer {} at {}", peer, address);

                                    let p2p_suffix = Protocol::P2p(peer);
                                    let address_with_p2p =
                                        if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                            address.clone().with(p2p_suffix)
                                        } else {
                                            address.clone()
                                        };

                                    swarm.dial(address_with_p2p).unwrap();
                                }
                            }
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                            rendezvous::client::Event::Registered {
                                namespace,
                                ttl,
                                rendezvous_node,
                            },
                        )) => {
                            println!(
                                "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                                namespace,
                                rendezvous_node,
                                ttl
                            );
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                            rendezvous::client::Event::RegisterFailed {
                                rendezvous_node,
                                namespace,
                                error,
                            },
                        )) => {
                            println!(
                                "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                                rendezvous_node,
                                namespace,
                                error
                            );
                        }
                        _ => {}
                    }
                    // _ = discover_tick.tick() => if cookie.is_some() {
                    //         swarm.behaviour_mut().rendezvous.discover(
                    //             Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                    //             cookie.clone(),
                    //             None,
                    //             rendezvous_point
                    //             )
                    //         } else {}

                    // _ = discover_tick.tick() => {
                    //     if cookie.is_some() {
                    //         swarm.behaviour_mut().rendezvous.discover(
                    //             Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                    //             cookie.clone(),
                    //             None,
                    //             rendezvous_point
                    //             )
                    //     }
                    // }
                }
            }
        }
    }

    #[cfg(feature = "validator")]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        // if let Ok(peer) = env::var("PEER") {
        //     let addr = peer.parse::<Multiaddr>().unwrap();
        //     let _ = swarm.dial(addr);
        // }
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        let rendezous_peer_id = "12D3KooWCxV8czTueJYfzFsuJcjQPe6R3ZxJZdgcYeZYZS9Drqtw"
            .parse()
            .unwrap();
        loop {
            tokio::select! {
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            cookie.clone(),
                            None,
                            rendezous_peer_id
                            )
                    }
                }
                // need a mechanism to routeinly go around and check the queues process the next one if somethign went wrong
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {address:?}");
                        }
                        SwarmEvent::IncomingConnection { local_addr, .. } => {
                            println!("incoming connection: {local_addr:?}");
                        }
                        // SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(event)) => {
                        //     println!("identify: {event:?}");
                        // }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(identify::Event::Received {
                            ..
                        })) => {
                            if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                                rendezvous::Namespace::from_static(NAMESPACE),
                                rendezous_peer_id.clone(),
                                None,
                            ) {
                                println!("Failed to register: {error}");
                                return;
                            }
                            println!("Connected");
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
                            let mut propose_request: ProcessRequest = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            match propose_request.status {
                                DataRequestType::PROPOSE => {
                                    println!("processing proposal");
                                    let datetime = Utc::now();
                                    propose_request.datetime = Some(datetime);
                                    let hash = calculate_hash(&propose_request);
                                    println!("{hash}");
                                    propose_request.hash = hash;
                                    propose_request.status = DataRequestType::CORROBORATE;
                                    let connections: &usize = &swarm.connected_peers().count();
                                    println!("{connections}");
                                    propose_request.connected_peers = if connections > &1 {
                                        connections - 1
                                    } else { connections.clone() };
                                    let required_signatures = 1;
                                    {
                                        let proposal_queue = &mut self.proposal_queues.write().await;
                                        if let Some(user_queue) = proposal_queue.get_mut(&propose_request.user_id) {
                                            user_queue.write().await.push(propose_request.clone());
                                        } else {
                                            proposal_queue.insert(propose_request.user_id.clone(), RwLock::new(vec![propose_request.clone()]));
                                        }
                                    }
                                    println!("sending proposal");
                                    self.process_next_request(&propose_request.user_id, hash, &mut swarm).await;
                                },
                                DataRequestType::VALID | DataRequestType::INVALID => {
                                    let hash = calculate_hash(&propose_request);
                                    let result = {
                                        let proposals = &mut self.proposals_processing.write().await;
                                        if let Some(request) = proposals.get_mut(&propose_request.user_id) {
                                            if request.hash != hash {
                                                return;
                                            }
                                            for signature in propose_request.validated_signatures.iter() {
                                                request.validated_signatures.insert(signature.clone());
                                            }
                                            for signature in propose_request.failed_signatures.iter() {
                                                request.failed_signatures.insert(signature.clone());
                                            }
                                            self.check_if_validated(request, &mut swarm, hash).await
                                        } else {
                                            println!("processing request not found for user {}, hash: {hash}", propose_request.user_id);
                                            println!("{:?}", proposals);
                                            return;
                                        }
                                    };
                                    if let Err(e) = result {
                                        println!("{e:?}");
                                    } else if result.unwrap().0.len() > 0 {
                                        println!("IGNORED");
                                        {
                                            self.proposals_processing.write().await.remove(&propose_request.user_id);
                                        }
                                        self.process_next_request(&propose_request.user_id, hash, &mut swarm).await;
                                    } else {
                                        println!("OTHER");
                                    };

                                    // } else {
                                    //     println!("processing request not found for user {}, hash: {hash}", propose_request.user_id);
                                    //     println!("{:?}", proposals);
                                    // }
                                },
                                DataRequestType::IGNORED => {
                                    let hash = calculate_hash(&propose_request);
                                    let result = {
                                        let proposals = &mut self.proposals_processing.write().await;
                                        if let Some(request) = proposals.get_mut(&propose_request.user_id) {
                                            if request.hash != hash {
                                                return;
                                            }
                                            for signature in propose_request.ignore_signatures.iter() {
                                                request.ignore_signatures.insert(signature.clone());
                                            }
                                            self.check_if_validated(request, &mut swarm, hash).await
                                        } else {
                                            Err(PublishError::Duplicate)
                                        }
                                    };
                                    if let Err(e) = result {
                                        println!("{e:?}");
                                    } else if result.unwrap().0.len() > 0 {
                                        println!("IGNORED");
                                        {
                                            self.proposals_processing.write().await.remove(&propose_request.user_id);
                                        }
                                        self.process_next_request(&propose_request.user_id, hash, &mut swarm).await;
                                    } else {
                                        println!("OTHER");
                                    };
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
                        SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezous_peer_id => {
                            println!(
                                "Connected to rendezvous point, discovering nodes in '{}' namespace ...",
                                NAMESPACE
                            );

                            swarm.behaviour_mut().rendezvous.discover(
                                Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                                None,
                                None,
                                rendezous_peer_id,
                            );
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(rendezvous::client::Event::Discovered {
                            registrations,
                            cookie: new_cookie,
                            ..
                        })) => {
                            cookie.replace(new_cookie);

                            for registration in registrations {
                                for address in registration.record.addresses() {
                                    let peer = registration.record.peer_id();
                                    println!("Discovered peer {} at {}", peer, address);

                                    let p2p_suffix = Protocol::P2p(peer);
                                    let address_with_p2p =
                                        if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                            address.clone().with(p2p_suffix)
                                        } else {
                                            address.clone()
                                        };

                                    swarm.dial(address_with_p2p).unwrap();
                                }
                            }
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                            rendezvous::client::Event::Registered {
                                namespace,
                                ttl,
                                rendezvous_node,
                            },
                        )) => {
                            println!(
                                "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                                namespace,
                                rendezvous_node,
                                ttl
                            );
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                            rendezvous::client::Event::RegisterFailed {
                                rendezvous_node,
                                namespace,
                                error,
                            },
                        )) => {
                            println!(
                                "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                                rendezvous_node,
                                namespace,
                                error
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[cfg(feature = "rendezvous")]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("Connected to {}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            println!("Disconnected from {}", peer_id);
                        }
                        SwarmEvent::Behaviour(RendezvousServerBehaviourEvent::Rendezvous(
                            rendezvous::server::Event::PeerRegistered { peer, registration },
                        )) => {
                            println!(
                                "Peer {} registered for namespace '{}'",
                                peer,
                                registration.namespace
                            );
                        }
                        SwarmEvent::Behaviour(RendezvousServerBehaviourEvent::Rendezvous(
                            rendezvous::server::Event::DiscoverServed {
                                enquirer,
                                registrations,
                            },
                        )) => {
                            println!(
                                "Served peer {} with {} registrations",
                                enquirer,
                                registrations.len()
                            );
                        }
                        other => {
                            println!("Unhandled {:?}", other);
                        }
                    }
                }
            }
        }
        // unimplemented!()
    }

    #[cfg(feature = "validator")]
    async fn check_if_validated(
        &self,
        queued_request: &mut ProcessRequest,
        swarm: &mut Swarm<BecoBehaviour>,
        hash: u64,
    ) -> Result<MessageId, PublishError> {
        println!("message to validate");
        println!("{queued_request:?}");
        if queued_request.validated_signatures.len()
            >= (((queued_request.connected_peers - queued_request.ignore_signatures.len()) as f32
                * 0.8)
                .ceil() as usize)
        {
            queued_request.status = DataRequestType::VALIDATED;
            swarm.behaviour_mut().gossipsub.publish(
                self.validated_gossip_sub.clone(),
                serde_json::to_vec(&queued_request).unwrap(),
            )
        } else if queued_request.failed_signatures.len()
            >= (((queued_request.connected_peers - queued_request.ignore_signatures.len()) as f32
                * 0.2)
                .ceil() as usize)
        {
            queued_request.status = DataRequestType::FAILED;
            swarm.behaviour_mut().gossipsub.publish(
                self.validated_gossip_sub.clone(),
                serde_json::to_vec(&queued_request).unwrap(),
            )
        } else {
            Ok(MessageId(vec![]))
        }
    }

    #[cfg(feature = "validator")]
    async fn process_next_request(
        &self,
        user_id: &String,
        hash: u64,
        swarm: &mut Swarm<BecoBehaviour>,
    ) {
        let mut proposals_processing = self.proposals_processing.write().await;
        let mut proposal_queues = self.proposal_queues.write().await;
        println!("boop");
        if proposals_processing.get(user_id).is_none() {
            if let Some(requests) = proposal_queues.get_mut(user_id) {
                let request_lock = &mut requests.write().await;
                if request_lock.len() > 0 {
                    let next_request = request_lock.remove(0);
                    proposals_processing.insert(user_id.clone(), next_request.clone());
                    println!("sending message: {next_request:?}");
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(
                        self.corroborate_gossip_sub.clone(),
                        serde_json::to_vec(&next_request).unwrap(),
                    ) {
                        println!("Publish error: {e:?}");
                    }
                }
            }
        }
        println!("here");
        println!("{proposals_processing:?}");
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
