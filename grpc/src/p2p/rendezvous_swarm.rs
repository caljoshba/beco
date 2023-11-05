use std::error::Error;

use envconfig::Envconfig;
use libp2p::{identity, PeerId};
use libp2p::{identify, ping, rendezvous, swarm::SwarmEvent, Multiaddr, Swarm, SwarmBuilder};

use crate::p2p::behaviour::BecoBehaviourEvent;

use super::{behaviour::BecoBehaviour, P2P};

use futures::StreamExt;

use super::rendezvous_config::Config;

impl P2P {
    pub fn new() -> Self {
        let config = Config::init_from_env().unwrap();
        let keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keys.public());
        Self {
            keys,
            peer_id,
            config,
        }
    }

    pub fn create_swarm(&self) -> Result<Swarm<BecoBehaviour>, Box<dyn Error>> {
        let behaviour = BecoBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                self.keys.public(),
            )),
            ping: ping::Behaviour::new(ping::Config::new()),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
        };

        // Create a Swarm to manage peers and events
        let mut swarm = SwarmBuilder::with_existing_identity(self.keys.clone())
            .with_tokio()
            .with_tcp(
                Default::default(),
                (libp2p_tls::Config::new, libp2p::noise::Config::new),
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            // .with_other_transport(|_key| transport)?
            .with_dns()?
            .with_behaviour(|_key| behaviour)?
            .build();

        let addr = format!("/ip4/0.0.0.0/tcp/{}", self.config.p2p_port,)
            .parse::<Multiaddr>()
            .unwrap();
        let ext_addr = format!(
            "/ip4/{}/tcp/{}/p2p/{}",
            self.config.external_host,
            self.config.p2p_port,
            self.keys.public().to_peer_id()
        )
        .parse::<Multiaddr>()
        .unwrap();

        swarm.listen_on(addr.clone())?;
        swarm.add_external_address(ext_addr.clone());
        println!("Peer ID: {:?}", self.peer_id);

        Ok(swarm)
    }

    pub async fn loop_swarm(&mut self) {
        let mut swarm: Swarm<BecoBehaviour> = self.create_swarm().unwrap();
        loop {
            tokio::select! {
                // need to cleanup registrations when a connection closes
                event = swarm.select_next_some() => {
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
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("Connected to {}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            println!("Disconnected from {}", peer_id);
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
                            rendezvous::server::Event::PeerRegistered { peer, registration },
                        )) => {
                            println!(
                                "Peer {} registered for namespace '{}'",
                                peer,
                                registration.namespace
                            );
                        }
                        SwarmEvent::Behaviour(BecoBehaviourEvent::Rendezvous(
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
}
