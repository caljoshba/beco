#![allow(dead_code)]
#![allow(unused_variables)]

mod xrpl;
mod evm;
mod traits;
mod errors;
mod enums;
mod chain;
mod response;
mod server;
mod proto;

mod test;


use tonic::transport::Server;

use server::BecoImplementation;
use proto::beco::beco_server::BecoServer;

mod beco_proto {
    include!("proto/beco.rs");
    // include!("proto/account.rs");
    // include!("proto/user.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("beco_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9001".parse()?;
    let wallet = BecoImplementation::default();

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
