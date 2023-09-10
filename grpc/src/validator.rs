#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

mod chain;
mod enums;
mod errors;
mod evm;
mod implement;
mod p2p;
mod permissions;
mod proto;
mod utils;
mod traits;
mod user;
mod xrpl;

use p2p::P2P;

#[cfg(feature = "validator")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    P2P::new().loop_swarm().await;

    Ok(())
}
