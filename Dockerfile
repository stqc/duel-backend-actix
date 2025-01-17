FROM rust as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
COPY upload.json .
RUN cargo chef prepare --recipe-path recipe.json


FROM rust as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
COPY . /app
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update && apt-get install -y ca-certificates
# RUN apt-get update \
#     && apt-get install -y libssl-dev \
#     && rm -rf /var/lib/apt/lists/*
COPY upload.json .
COPY --from=builder /app/target/release/hello_world_acix .


CMD ["./hello_world_acix"]