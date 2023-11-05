mod behaviour;

#[cfg(any(feature = "user", feature = "validator", feature = "sst", feature = "rendezvous"))]
use libp2p::identity;

#[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
mod config;
#[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
mod user_sst_validator_swarm;

#[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
use config::Config;
#[cfg(any(feature = "user", feature = "validator", feature = "sst"))]
use libp2p::{gossipsub, Multiaddr, PeerId};

#[cfg(any(
    feature = "rendezvous",
    feature = "sst",
    feature = "user",
    feature = "validator"
))]
use tokio::sync::OnceCell;

#[cfg(feature = "rendezvous")]
mod rendezvous_config;
#[cfg(feature = "rendezvous")]
mod rendezvous_swarm;

#[cfg(feature = "rendezvous")]
use libp2p::PeerId;
#[cfg(feature = "rendezvous")]
use rendezvous_config::Config;

#[cfg(any(feature = "user"))]
mod user_swarm;

#[cfg(any(feature = "user"))]
use crate::entry::Entry;
#[cfg(any(feature = "user"))]
use serde_json::Value;
#[cfg(any(feature = "user"))]
use std::sync::Arc;
#[cfg(any(feature = "user"))]
use tokio::sync::mpsc::Receiver;

#[cfg(any(feature = "validator"))]
mod validator_swarm;

#[cfg(feature = "validator")]
use crate::enums::data_value::ProcessRequest;
#[cfg(any(feature = "validator"))]
use std::collections::HashMap;
#[cfg(any(feature = "validator"))]
use tokio::sync::RwLock;

#[cfg(any(feature = "sst"))]
mod sst_swarm;

#[cfg(any(feature = "sst"))]
use crate::merkle::SST;

// https://github.com/libp2p/rust-libp2p/blob/master/examples/ipfs-private/src/main.rs

const USER_NAMESPACE: &str = "user";
const ORG_NAMESPACE: &str = "organisation";
const GOV_NAMESPACE: &str = "government";

static RENDEZVOUS_PEER_ID: OnceCell<PeerId> = OnceCell::const_new();
async fn rendezvous_peer_id() -> &'static PeerId {
    RENDEZVOUS_PEER_ID
        .get_or_init(|| async {
            "12D3KooWKQhCTgmWTzH8hQYKEjzYeJ1LmFfEVjEwmvMuotQdkbKu"
                .parse()
                .unwrap()
        })
        .await
}

#[cfg(feature = "user")]
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
    config: Config,
    rendezvous_address: Multiaddr,
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
    config: Config,
    rendezvous_address: Multiaddr,
}

#[cfg(feature = "sst")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    sst: SST,
    validated_gossip_sub: gossipsub::IdentTopic,
    load_gossip_sub: gossipsub::IdentTopic,
    new_user_gossip_sub: gossipsub::IdentTopic,
    response_gossip_sub: gossipsub::IdentTopic,
    config: Config,
    rendezvous_address: Multiaddr,
}

#[cfg(feature = "rendezvous")]
pub struct P2P {
    keys: identity::Keypair,
    peer_id: PeerId,
    config: Config,
}
