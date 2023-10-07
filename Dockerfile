FROM rust AS base

WORKDIR /code

RUN apt-get update && apt-get install -y protobuf-compiler

COPY ./grpc/Cargo.toml ./grpc/Cargo.lock ./
COPY ./grpc/build.rs ./
COPY ./grpc/proto ./proto
COPY ./grpc/src ./src

RUN cargo build --release

FROM base AS rendezvous

CMD [ "cargo", "run", "--release", "--bin", "rendezvous", "--features=rendezvous" ]

FROM base AS grpc

CMD [ "cargo", "run", "--release", "--bin", "grpc", "--features=grpc" ]

FROM base AS validator

CMD [ "cargo", "run", "--release", "--bin", "validator", "--features=validator" ]

FROM base AS sst

CMD [ "cargo", "run", "--release", "--bin", "sst", "--features=sst" ]


