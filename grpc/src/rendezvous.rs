#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

mod p2p;

use p2p::P2P;

#[cfg(feature = "rendezvous")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    P2P::new().loop_swarm().await;

    Ok(())
}
