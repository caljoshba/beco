# BECO
Blockchain Everywhere Connector

## Install protoc

https://grpc.io/docs/protoc-installation/

    sudo apt-get install -y protobuf-compiler

## Examples of grpc

https://konghq.com/blog/engineering/building-grpc-apis-with-rust
https://protobuf.dev/programming-guides/proto3/#enum-value-options

cargo run --release --bin server --features="grpc"
PEER=/ip4/127.0.0.1/tcp/38743 PORT=other cargo run --release --bin server --features="grpc"
PEER=/ip4/127.0.0.1/tcp/38743 PORT=other cargo run --release --bin validator --features="validator"