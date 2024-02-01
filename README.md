# BECO

## Very much a WIP!

## What is this?

This is a blockchain/DLT inspired application, intended to work through permissioned mechanisms. This is not intended to be a public network for now.

The purpose is to provide users with an identity they can use across the web2/web3 space.
The identity is a very simple one for the moment, consisting purely of a first name, last name and middle names.

## Architecture



## How?

Using grpc nodes, a user's identity is stored across multiple instances which could be considered multi A-Z and is used for quick access to read and write. When you want to update your identity, the grpc node validates the request based on the requesting user's permissions before submitting a request across the rest of the network.

The request is first submitteed to the validator node (which is more of a guardian than a validator), that register's the request and applies a timestamp. It then re-submits the request via the P2P network to all other grpc nodes that can validate the request if they have the same user's already loaded. Once enough signatures are collected, validating the request, the request is sent to the `sst` (single source of truth) node which is connected to a postgres DB and updates the details in a merkle tree.

## Running the application

This docker setup is designed for a linux environment and may not work on Mac or Windows. I've also noticed that utilising docker, the response times have jumped from about 40ms to 200ms in some instances compared to bare metal.

    docker compose up rendezvous

Copy the peer ID that is output in the logs and update the value in [p2p/mod.rs](./grpc//src//p2p//mod.rs):

```rust
RENDEZVOUS_PEER_ID
    .get_or_init(|| async {
        "12D3KooWHDjkJgZrgjFxzmARH2DsJ6GYGV79NMuwLhvHSLYyw4Ai"
            .parse()
            .unwrap()
    })
    .await
```

Then you can start the other applications

    docker compose up -d postgres
    docker compose up -d validator
    docker compose up -d user

I use postman to connect and perform requests against the grpc node running on port `9001`. The grpc node also uses server reflection in order to ascertain the request shape.

https://libp2p.io/
https://github.com/libp2p/rust-libp2p/blob/master/examples/identify/src/main.rs