#![allow(dead_code)]
#![allow(unused_variables)]

mod keys;
mod xrpl;
mod evm;
mod traits;
mod errors;
mod enums;
mod user;
mod wallet;
mod response;
mod server;

mod test;


use tonic::transport::Server;

use server::WalletImplementation;
use wallet::keys_server::KeysServer;

mod wallet_proto {
    include!("wallet.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("wallet_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9001".parse()?;
    let wallet = WalletImplementation::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(wallet_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let _ = Server::builder()
        .add_service(KeysServer::new(wallet))
        .add_service(reflection_service)
        .serve(addr).await;
    Ok(())
}
