& cargo run --release --bin rendezvous --features="rendezvous")
& cargo run --release --bin server --features="grpc"
& PORT=9002 P2P=7001 cargo run --release --bin server --features="grpc"
& P2P=7003 cargo run --release --bin validator --features="validator"