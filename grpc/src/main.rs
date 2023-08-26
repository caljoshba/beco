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
use futures::future::join;
use p2p::P2P;
use tokio::sync::{OnceCell, mpsc::{self, Receiver, Sender}};
use tonic::transport::Server;

use server::BecoImplementation;
use proto::beco::beco_server::BecoServer;
use serde_json::Value;

use std::env;

mod beco_proto {
    include!("proto/beco.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("beco_descriptor");
}

static ENTRY: OnceCell<Entry> = OnceCell::const_new();

async fn get_entry(tx_p2p: Sender<Value>, tx_grpc: Sender<Value>, rx_grpc: Receiver<Value>) -> &'static Entry {
    ENTRY.get_or_init(|| async { Entry::new(tx_p2p, tx_grpc, rx_grpc) }).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://tokio.rs/tokio/tutorial/channels
    let (tx_p2p, mut rx_p2p) = mpsc::channel::<Value>(32);
    let (tx_grpc, mut rx_grpc) = mpsc::channel::<Value>(32);
    let entry = get_entry(tx_p2p, tx_grpc, rx_grpc).await;
    let p2p_server = async move {
        let mut swarm = P2P::new(entry).create_swarm().await.unwrap();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        tokio::spawn(async move {
            P2P::loop_swarm(&mut swarm, rx_p2p, entry).await;
        });
    };
    let port = if env::var("PORT").unwrap_or("default".to_string()) == "default".to_string() { 9001 } else { 9002 };
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    let grpc_server = async move {
        let wallet = BecoImplementation::new(entry);

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(beco_proto::FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();
        let _ = Server::builder()
            .add_service(BecoServer::new(wallet))
            .add_service(reflection_service)
            .serve(addr).await;
    };

    println!("Serving grpc and p2p");
    let _ret = join(p2p_server, grpc_server).await;
    
    
    Ok(())
}
