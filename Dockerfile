FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app

RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    make \
    g++ \
    perl

ENV CXX="g++" CC="gcc"
COPY . .

RUN cargo build --release --verbose

# ----------------
FROM debian:bookworm-slim
RUN USER=root \
    apt-get update && \
    apt-get install -y openssl ca-certificates && \
    update-ca-certificates --fresh && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app

ARG TWEL_DATA_KEY 
ENV TWEL_DATA_KEY=$TWEL_DATA_KEY
ENV RUST_LOG=debug

COPY --from=builder /app/target/release/str-proc ./
EXPOSE 8000
CMD ["./str-proc"]