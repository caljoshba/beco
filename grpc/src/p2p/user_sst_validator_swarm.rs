use std::error::Error;

use libp2p::{identify, ping, rendezvous, Multiaddr, Swarm, SwarmBuilder};

use super::{behaviour::BecoBehaviour, P2P};
use libp2p::gossipsub;
use libp2p::rendezvous::Cookie;
use libp2p::swarm::THandlerErr;
use libp2p::{multiaddr::Protocol, swarm::SwarmEvent};
use std::time::Duration;

use crate::p2p::{rendezvous_peer_id, USER_NAMESPACE};

use super::behaviour::BecoBehaviourEvent;

impl P2P {
    pub fn create_swarm(&self) -> Result<Swarm<BecoBehaviour>, Box<dyn Error>> {
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
        };

        self.subscribe_to_topics(&mut behaviour);

        // Create a Swarm to manage peers and events
        let mut swarm = SwarmBuilder::with_existing_identity(self.keys.clone())
            .with_tokio()
            .with_tcp(
                Default::default(),
                (libp2p_tls::Config::new, libp2p::noise::Config::new),
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_dns()?
            .with_behaviour(|_key| behaviour)?
            .build();
        let addr = format!("/ip4/0.0.0.0/tcp/{}", self.config.p2p_port)
            .parse::<Multiaddr>()
            .unwrap();
        swarm.listen_on(addr.clone())?;
        swarm.add_external_address(
            format!(
                "/ip4/{}/tcp/{}",
                self.config.external_host, self.config.external_p2p_port
            )
            .parse::<Multiaddr>()
            .unwrap(),
        );
        swarm.dial(self.rendezvous_address.clone())?;
        println!("Peer ID: {:?}", self.peer_id);
        Ok(swarm)
    }

    #[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
    pub async fn discover_rendzvous(
        &self,
        swarm: &mut Swarm<BecoBehaviour>,
        namespace: String,
        cookie: Option<Cookie>,
    ) {
        swarm.behaviour_mut().rendezvous.discover(
            Some(rendezvous::Namespace::new(namespace).unwrap()),
            cookie,
            None,
            rendezvous_peer_id().await.clone(),
        );
    }

    #[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
    pub async fn register(&self, swarm: &mut Swarm<BecoBehaviour>) {
        let result = swarm.behaviour_mut().rendezvous.register(
            rendezvous::Namespace::new(USER_NAMESPACE.to_string()).unwrap(),
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

    #[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
    pub async fn process_swarm_events(
        &self,
        event: SwarmEvent<BecoBehaviourEvent, THandlerErr<BecoBehaviour>>,
        swarm: &mut Swarm<BecoBehaviour>,
        cookie: &mut Option<rendezvous::Cookie>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                println!("{error:?}");
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                println!("{error:?}");
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
                    self.discover_rendzvous(swarm, USER_NAMESPACE.to_string(), None)
                        .await;
                    // swarm.behaviour_mut().rendezvous.discover(
                    //     Some(rendezvous::Namespace::new(USER_NAMESPACE.to_string()).unwrap()),
                    //     None,
                    //     None,
                    //     rendezvous_peer_id().await.clone(),
                    // );
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
}
