#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

mod chain;
mod config;
mod entry;
mod enums;
mod errors;
mod evm;
mod implement;
mod organisation;
mod p2p;
mod permissions;
mod proto;
mod requests;
mod server;
mod traits;
mod user;
mod utils;
mod xrpl;

use config::Config;
use entry::Entry;
use envconfig::Envconfig;
use futures::future::join;
use p2p::P2P;
use proto::beco::beco_server::BecoServer;
use serde_json::Value;
use server::BecoImplementation;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    OnceCell,
};
use tonic::transport::Server;

mod beco_proto {
    include!("proto/beco.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("beco_descriptor");
}

static ENTRY: OnceCell<Arc<Entry>> = OnceCell::const_new();
#[cfg(feature = "user")]
async fn get_entry(
    tx_p2p: Sender<Value>,
    tx_grpc: Sender<Value>,
    rx_grpc: Receiver<Value>,
) -> &'static Arc<Entry> {
    ENTRY
        .get_or_init(|| async { Arc::new(Entry::new(tx_p2p, tx_grpc, rx_grpc)) })
        .await
}
#[cfg(feature = "user")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init_from_env().unwrap();
    // https://tokio.rs/tokio/tutorial/channels
    let (tx_p2p, rx_p2p) = mpsc::channel::<Value>(32);
    let (tx_grpc, rx_grpc) = mpsc::channel::<Value>(32);
    let entry = get_entry(tx_p2p, tx_grpc, rx_grpc).await;
    let p2p_server = async move {
        tokio::spawn(async move {
            P2P::new(entry, rx_p2p).loop_swarm().await;
        });
    };

    let addr: std::net::SocketAddr = ([0, 0, 0, 0], config.grpc_port).into();
    let grpc_server = async move {
        let wallet = BecoImplementation::new(entry.clone());

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(beco_proto::FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();
        let _ = Server::builder()
            .add_service(BecoServer::new(wallet))
            .add_service(reflection_service)
            .serve(addr)
            .await;
    };

    println!("Serving grpc and p2p");
    let _ret = join(p2p_server, grpc_server).await;

    Ok(())
}
