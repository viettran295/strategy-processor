FROM rust:1.85-slim-bookworm AS chef
WORKDIR /app
# Install OpenSSL and musl libc for alpine
RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    musl-tools \
    make \
    perl
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef

# ------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json .
# Build and cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --verbose

# ----------------
FROM alpine:3.21
RUN apk add --no-cache openssl
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/str-proc ./
COPY .env ./
EXPOSE 8000
CMD ["./str-proc"]