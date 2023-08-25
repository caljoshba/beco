#![allow(dead_code)]
#![allow(unused_variables)]

mod xrpl;
mod evm;
mod traits;
mod errors;
mod entry;
mod enums;
mod chain;
mod permissioms;
mod user;
mod server;
mod state;
mod proto;
mod p2p;


use entry::Entry;
use p2p::P2P;
use tokio::sync::{OnceCell, mpsc::{self, Receiver, Sender}};
use tonic::transport::Server;

use server::BecoImplementation;
use proto::beco::beco_server::BecoServer;
use serde_json::Value;

mod beco_proto {
    include!("proto/beco.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("beco_descriptor");
}

static ENTRY: OnceCell<Entry> = OnceCell::const_new();

async fn get_entry(tx_p2p: Sender<Value>, rx_p2p: Receiver<Value>, tx_grpc: Sender<Value>, rx_grpc: Receiver<Value>) -> &'static Entry {
    ENTRY.get_or_init(|| async { Entry::new(tx_p2p, rx_p2p, tx_grpc, rx_grpc) }).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://tokio.rs/tokio/tutorial/channels
    let (tx_p2p, mut rx_p2p) = mpsc::channel::<Value>(32);
    let (tx_grpc, mut rx_grpc) = mpsc::channel::<Value>(32);
    let entry = get_entry(tx_p2p, rx_p2p, tx_grpc, rx_grpc).await;
    let mut swarm = P2P::new(entry).create_swarm().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    tokio::spawn(async move {
        P2P::loop_swarm(&mut swarm).await;
    });
    let addr = "127.0.0.1:9001".parse()?;
    let wallet = BecoImplementation::new(entry);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(beco_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let _ = Server::builder()
        .add_service(BecoServer::new(wallet))
        .add_service(reflection_service)
        .serve(addr).await;
    Ok(())
}
