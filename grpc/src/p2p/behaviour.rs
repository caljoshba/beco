#[cfg(not(feature = "rendezvous"))]
use libp2p::{gossipsub, identify, ping, rendezvous, swarm::NetworkBehaviour};
#[cfg(feature = "rendezvous")]
use libp2p::{identify, ping, rendezvous, swarm::NetworkBehaviour};

#[cfg(not(feature = "rendezvous"))]
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "BecoBehaviourEvent")]
pub struct BecoBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub rendezvous: rendezvous::client::Behaviour,
}

#[cfg(not(feature = "rendezvous"))]
impl BecoBehaviour {
    pub fn gossipsub(&self) -> &gossipsub::Behaviour {
        &self.gossipsub
    }
}

#[cfg(not(feature = "rendezvous"))]
#[derive(Debug)]
pub enum BecoBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Identify(identify::Event),
    Ping(ping::Event),
    Rendezvous(rendezvous::client::Event),
}

#[cfg(not(feature = "rendezvous"))]
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

#[cfg(not(feature = "rendezvous"))]
impl From<rendezvous::client::Event> for BecoBehaviourEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        BecoBehaviourEvent::Rendezvous(event)
    }
}

#[cfg(feature = "rendezvous")]
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "BecoBehaviourEvent")]
pub struct BecoBehaviour {
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub rendezvous: rendezvous::server::Behaviour,
}

#[cfg(feature = "rendezvous")]
#[derive(Debug)]
pub enum BecoBehaviourEvent {
    Identify(identify::Event),
    Ping(ping::Event),
    Rendezvous(rendezvous::server::Event),
}

#[cfg(feature = "rendezvous")]
impl From<rendezvous::server::Event> for BecoBehaviourEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        BecoBehaviourEvent::Rendezvous(event)
    }
}
