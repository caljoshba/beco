# BECO
Blockchain Everywhere Connector

## Install protoc

https://grpc.io/docs/protoc-installation/

    sudo apt-get install -y protobuf-compiler

## Examples of grpc

https://konghq.com/blog/engineering/building-grpc-apis-with-rust
https://protobuf.dev/programming-guides/proto3/#enum-value-options


## P2P
https://github.com/libp2p/rust-libp2p/blob/master/examples/rendezvous/src/main.rs
https://github.com/libp2p/specs/blob/master/rendezvous/README.md


cargo run --release --bin rendezvous --features="rendezvous"
cargo run --release --bin user --features="user"
P2P=7003 cargo run --release --bin validator --features="validator"
P2P=7004 cargo run --release --bin sst --features="sst"


PORT=9002 P2P=7001 cargo run --release --bin grpc --features="grpc"