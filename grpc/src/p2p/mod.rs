mod behaviour;

#[cfg(any(
    feature = "grpc",
    feature = "validator",
    feature = "rendezvous",
    feature = "sst"
))]
use either::Either;
#[cfg(any(
    feature = "grpc",
    feature = "validator",
    feature = "rendezvous",
    feature = "sst"
))]
use futures::prelude::*;
#[cfg(any(feature = "grpc", feature = "validator", feature = "sst"))]
use libp2p::{
    core::{muxing::StreamMuxerBox, transport, transport::upgrade::Version},
    gossipsub, identify, identity,
    multiaddr::Protocol,
    noise, ping,
    pnet::{PnetConfig, PreSharedKey},
    rendezvous,
    swarm::{keep_alive, SwarmBuilder, SwarmEvent, THandlerErr},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
#[cfg(any(feature = "validator", feature = "sst"))]
use std::{env, error::Error, fs, path::Path, str::FromStr, time::Duration};
#[cfg(any(feature = "rendezvous", feature = "sst"))]
use tokio::sync::OnceCell;

#[cfg(any(feature = "grpc"))]
use crate::{
    entry::Entry,
    enums::data_value::{DataRequestType, ProcessRequest},
    p2p::behaviour::{BecoBehaviour, BecoBehaviourEvent},
    utils::calculate_hash,
};
#[cfg(any(feature = "grpc"))]
use serde_json::Value;
#[cfg(any(feature = "grpc"))]
use std::{env, error::Error, fs, path::Path, str::FromStr, sync::Arc, time::Duration};
#[cfg(any(feature = "grpc"))]
use tokio::sync::{mpsc::Receiver, OnceCell};

#[cfg(feature = "validator")]
use crate::{
    enums::data_value::{DataRequestType, ProcessRequest},
    p2p::behaviour::{BecoBehaviour, BecoBehaviourEvent},
    utils::calculate_hash,
};
#[cfg(any(feature = "validator"))]
use chrono::prelude::*;
#[cfg(any(feature = "validator"))]
use std::collections::HashMap;
#[cfg(any(feature = "validator"))]
use tokio::sync::{OnceCell, RwLock};

#[cfg(any(feature = "sst"))]
use crate::entry::Entry;
#[cfg(feature = "rendezvous")]
use crate::p2p::behaviour::{RendezvousServerBehaviour, RendezvousServerBehaviourEvent};
#[cfg(any(feature = "sst"))]
use crate::{
    enums::data_value::{DataRequestType, ProcessRequest},
    p2p::behaviour::{BecoBehaviour, BecoBehaviourEvent},
};
#[cfg(any(feature = "rendezvous"))]
use libp2p::{
    core::{muxing::StreamMuxerBox, transport, transport::upgrade::Version},
    identify, identity,
    multiaddr::Protocol,
    noise, ping,
    pnet::{PnetConfig, PreSharedKey},
    rendezvous,
    swarm::{keep_alive, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
#[cfg(any(feature = "rendezvous"))]
use std::{error::Error, fs, path::Path, str::FromStr, time::Duration};

// https://github.com/libp2p/rust-libp2p/blob/master/examples/ipfs-private/src/main.rs
static RENDEZVOUS_POINT_ADDRESS: OnceCell<Multiaddr> = OnceCell::const_new();
async fn rendezvous_address() -> &'static Multiaddr {
    RENDEZVOUS_POINT_ADDRESS
        .get_or_init(|| async { "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap() })
        .await
}
const NAMESPACE: &str = "rendezvous";

static RENDEZVOUS_PEER_ID: OnceCell<PeerId> = OnceCell::const_new();
async fn rendezvous_peer_id() -> &'static PeerId {
    RENDEZVOUS_PEER_ID
        .get_or_init(|| async {
            "12D3KooWGuC7iJJtcvH9NwnnZ5urrW4Sxgi2WzD5jTX3d4mgWPhs"
                .parse()
                .unwrap()
        })
        .await
}

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
    new_user_gossip_sub: gossipsub::IdentTopic,
    response_gossip_sub: gossipsub::IdentTopic,
}

#[cfg(feature = "validator")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    propose_gossip_sub: gossipsub::IdentTopic,
    corroborate_gossip_sub: gossipsub::IdentTopic,
    validated_gossip_sub: gossipsub::IdentTopic,
    // need to add an expiration datetime to each of these
    proposals_processing: RwLock<HashMap<String, ProcessRequest>>,
    proposal_queues: RwLock<HashMap<String, RwLock<Vec<ProcessRequest>>>>,
}

#[cfg(feature = "sst")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    entry: Entry,
    validated_gossip_sub: gossipsub::IdentTopic,
    load_gossip_sub: gossipsub::IdentTopic,
    new_user_gossip_sub: gossipsub::IdentTopic,
    response_gossip_sub: gossipsub::IdentTopic,
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
        let new_user_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::NEW.to_string());
        let response_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::RESPONSE.to_string());
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

    #[cfg(feature = "sst")]
    pub fn new() -> Self {
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        let entry = Entry::new();
        let validated_gossip_sub =
            gossipsub::IdentTopic::new(DataRequestType::VALIDATED.to_string());
        let load_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::LOAD.to_string());
        let new_user_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::NEW.to_string());
        let response_gossip_sub = gossipsub::IdentTopic::new(DataRequestType::RESPONSE.to_string());
        Self {
            keys,
            peer_id,
            entry,
            validated_gossip_sub,
            load_gossip_sub,
            new_user_gossip_sub,
            response_gossip_sub,
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

        let behaviour = RendezvousServerBehaviour {
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
        swarm.add_external_address(rendezvous_address().await.clone());
        println!("Peer ID: {:?}", self.peer_id);

        Ok(swarm)
    }

    #[cfg(feature = "rendezvous")]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        loop {
            tokio::select! {
                // need to cleanup registrations when a connection closes
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
                            // println!(
                            //     "Served peer {} with {} registrations",
                            //     enquirer,
                            //     registrations.len()
                            // );
                        }
                        other => {}
                    }
                }
            }
        }
    }

    #[cfg(any(feature = "grpc", feature = "validator", feature = "sst"))]
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
        let port = env::var("P2P")
            .unwrap_or("7000".into())
            .parse::<i32>()
            .unwrap();
        let addr = format!("/ip4/0.0.0.0/tcp/{port}")
            .parse::<Multiaddr>()
            .unwrap();
        swarm.listen_on(addr.clone())?;
        swarm.add_external_address(
            format!("/ip4/127.0.0.1/tcp/{port}")
                .parse::<Multiaddr>()
                .unwrap(),
        );
        swarm.dial(rendezvous_address().await.clone())?;
        println!("Peer ID: {:?}", self.peer_id);
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

    #[cfg(feature = "sst")]
    fn subscribe_to_topics(&self, behaviour: &mut BecoBehaviour) {
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

    #[cfg(feature = "grpc")]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        loop {
            tokio::select! {
                Some(message) = self.rx_p2p.recv() => {
                    let mut request: ProcessRequest = serde_json::from_value(message.clone()).unwrap();
                    request.originator_peer_id = Some(self.peer_id.to_string());
                    match request.status {
                        DataRequestType::IGNORED => { request.ignore_signatures.insert("me".into()); },
                        DataRequestType::INVALID => { request.failed_signatures.insert("me".into()); },
                        DataRequestType::VALID => { request.validated_signatures.insert("me".into()); },
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
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            cookie.clone(),
                            None,
                            rendezvous_peer_id().await.clone()
                        );
                    }
                }
                // need a mechanism to routinely go around and check the queues process the next one if something went wrong
                event = swarm.select_next_some() => {
                    self.process_swarm_events(event, &mut swarm, &mut cookie).await;
                }
            }
        }
    }

    #[cfg(any(feature = "validator"))]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        loop {
            tokio::select! {
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            cookie.clone(),
                            None,
                            rendezvous_peer_id().await.clone()
                        )
                    }
                }
                // need a mechanism to routeinly go around and check the queues process the next one if something went wrong
                event = swarm.select_next_some() => {
                    self.process_swarm_events(event, &mut swarm, &mut cookie).await;
                }
            }
        }
    }

    #[cfg(any(feature = "sst"))]
    pub async fn loop_swarm(&mut self) {
        let mut swarm = self.create_swarm().await.unwrap();
        let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
        let mut cookie = None;
        loop {
            tokio::select! {
                _ = discover_tick.tick() => {
                    if cookie.is_some() {
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            cookie.clone(),
                            None,
                            rendezvous_peer_id().await.clone()
                        )
                    }
                }
                // need a mechanism to routeinly go around and check the queues process the next one if something went wrong
                event = swarm.select_next_some() => {
                    self.process_swarm_events(event, &mut swarm, &mut cookie).await;
                }
            }
        }
    }

    #[cfg(any(feature = "grpc", feature = "validator", feature = "sst"))]
    async fn register(&self, swarm: &mut Swarm<BecoBehaviour>) {
        let result = swarm.behaviour_mut().rendezvous.register(
            rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap(),
            rendezvous_peer_id().await.clone(),
            None,
        );
        if let Err(error) = result {
            println!("Failed to register: {error}");
            return;
        } else {
            println!("registered with result: {result:?}");
        }
    }

    #[cfg(any(feature = "grpc", feature = "validator", feature = "sst"))]
    async fn process_swarm_events(
        &self,
        event: SwarmEvent<BecoBehaviourEvent, THandlerErr<BecoBehaviour>>,
        swarm: &mut Swarm<BecoBehaviour>,
        cookie: &mut Option<rendezvous::Cookie>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
            }
            SwarmEvent::IncomingConnection { local_addr, .. } => {
                println!("incoming connection: {local_addr:?}");
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                if peer_id == rendezvous_peer_id().await.clone() {
                    self.register(swarm).await;
                }
            }
            SwarmEvent::Behaviour(BecoBehaviourEvent::Identify(identify::Event::Received {
                ..
            })) => {}
            SwarmEvent::Behaviour(BecoBehaviourEvent::Ping(event)) => match event {
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
                    if peer == rendezvous_peer_id().await.clone() {
                        self.register(swarm).await;
                    }
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
                    if peer == rendezvous_peer_id().await.clone() {
                        self.register(swarm).await;
                    }
                    println!("ping: ping::Failure with {}: {error}", peer.to_base58());
                }
            },
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connection established: {peer_id:?}");
                if peer_id == rendezvous_peer_id().await.clone() {
                    self.register(swarm).await;
                    swarm.behaviour_mut().rendezvous.discover(
                        Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                        None,
                        None,
                        rendezvous_peer_id().await.clone(),
                    );
                }
            }
            SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                rendezvous::client::Event::Discovered {
                    registrations,
                    cookie: new_cookie,
                    ..
                },
            )) => {
                cookie.replace(new_cookie);

                for registration in registrations {
                    for address in registration.record.addresses() {
                        let peer = registration.record.peer_id();
                        println!("Discovered peer {} at {}", peer, address);

                        let p2p_suffix = Protocol::P2p(peer);
                        let address_with_p2p: Multiaddr =
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
                    namespace, rendezvous_node, ttl
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
                    rendezvous_node, namespace, error
                );
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
                self.process_gossipsub(message, swarm).await;
            }
            _ => {}
        }
    }

    #[cfg(feature = "validator")]
    async fn process_gossipsub(
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
                process_request.connected_peers = if connections > 1 {
                    connections - 1
                } else {
                    connections.clone()
                };

                let required_signatures = 1;
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

    #[cfg(feature = "validator")]
    async fn check_if_validated(
        &self,
        process_request: &mut ProcessRequest,
        swarm: &mut Swarm<BecoBehaviour>,
        hash: u64,
    ) -> bool {
        let processed = {
            let proposals_processing = &mut self.proposals_processing.write().await;
            if let Some(queued_request) = proposals_processing.get_mut(&process_request.user_id) {
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
                if self
                    .process_if_threshold_reached(
                        queued_request,
                        swarm,
                        DataRequestType::VALIDATED,
                        validated_signatures_len,
                        validated_signatures_threshold,
                        hash,
                    )
                    .await
                {
                    return true;
                }

                if self
                    .process_if_threshold_reached(
                        queued_request,
                        swarm,
                        DataRequestType::FAILED,
                        failed_signatures_len,
                        failed_signatures_threshold,
                        hash,
                    )
                    .await
                {
                    return true;
                }
            }
            false
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

    #[cfg(feature = "validator")]
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
        return true;
    }

    #[cfg(feature = "validator")]
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

    #[cfg(feature = "grpc")]
    async fn process_gossipsub(
        &self,
        message: gossipsub::Message,
        swarm: &mut Swarm<BecoBehaviour>,
    ) {
        use crate::enums::data_value::DataRequests;

        let mut process_request: ProcessRequest =
            serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
        match process_request.status {
            DataRequestType::CORROBORATE => {
                let response = self.entry.corroborate(&mut process_request).await;
                println!("{response:?}");
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
                println!("{response:?}");
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
                    DataRequests::LoadUser(user) => {
                        self.entry.load_user(user.clone()).await;
                        self.entry
                            .success_event(hash, Some(user.id), process_request.status)
                            .await;
                        self.entry.ping_event(&hash).await;
                    }
                    _ => {}
                }
            }
            _ => {}
        };
    }

    #[cfg(feature = "sst")]
    async fn process_gossipsub(
        &self,
        message: gossipsub::Message,
        swarm: &mut Swarm<BecoBehaviour>,
    ) {
        use std::collections::HashSet;

        use chrono::Utc;

        use crate::{enums::data_value::DataRequests, utils::calculate_hash};

        let process_request: ProcessRequest =
            serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
        match process_request.status {
            DataRequestType::NEW => match process_request.request {
                DataRequests::AddUser(request) => {
                    let (user, _) = self.entry.add_user(request).await;
                    let data_request = DataRequests::LoadUser(user.clone());
                    let hash = calculate_hash(&data_request);
                    let load_request = ProcessRequest {
                        validated_signatures: HashSet::new(),
                        failed_signatures: HashSet::new(),
                        ignore_signatures: HashSet::new(),
                        status: DataRequestType::RESPONSE,
                        request: data_request,
                        calling_user: process_request.calling_user,
                        user_id: user.id,
                        hash,
                        datetime: Some(Utc::now()),
                        connected_peers: 0,
                        originator_hash: process_request.originator_hash,
                        originator_peer_id: process_request.originator_peer_id,
                    };
                    let result = swarm.behaviour_mut().gossipsub.publish(
                        self.response_gossip_sub.clone(),
                        serde_json::to_vec(&load_request).unwrap(),
                    );

                    if let Err(e) = result {
                        println!("Error pusblishing message: {e:?}");
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
}
