FROM rust:1.85-slim-bookworm AS builder

# Install OpenSSL and musl libc for alpine
RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    musl-tools \
    make \
    perl

WORKDIR /app
COPY Cargo.toml Cargo.lock .env ./
COPY src ./src/

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo fetch
RUN cargo build --release --target x86_64-unknown-linux-musl --verbose

# ----------------
FROM alpine:3.21
RUN apk add --no-cache openssl
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/str-proc ./
COPY .env ./
EXPOSE 8000
CMD ["./str-proc"]