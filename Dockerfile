FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
ARG ARCH

RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    make \
    g++ \
    perl
ENV CXX_"$ARCH"="g++" CXX="g++" CC_"$ARCH"="gcc" CC="gcc"
COPY . .
RUN rustup target add $ARCH
RUN cargo build --release --target $ARCH --verbose

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
ARG ARCH
ENV ARCH=$ARCH
ENV RUST_LOG=debug

COPY --from=builder /app/target/$ARCH/release/str-proc ./
EXPOSE 8000
CMD ["./str-proc"]