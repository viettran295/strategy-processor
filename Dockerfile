FROM rust:1.85-slim-bookworm AS chef
WORKDIR /app
ARG ARCH

# Install OpenSSL and musl libc for alpine
RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    musl-tools \
    make \
    perl

RUN rustup target add $ARCH
RUN cargo install cargo-chef

# Stage 3 - planner ------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4 - builder ------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json .

# Set compiler and linker compatible with architecture
# RUN if [ "$ARCH" = "aarch64-unknown-linux-musl" ]; then \
#         export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc; \
#         export CC=aarch64-linux-gnu-gcc; \
#     elif [ "$ARCH" = "x86_64-unknown-linux-musl" ]; then \
#         export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-gnu-gcc; \
#         export CC=x86_64-linux-gnu-gcc; \
#     fi && \

RUN cargo chef cook --release --target $ARCH --recipe-path recipe.json
COPY . .
RUN cargo build --release --target $ARCH --verbose

# ----------------
FROM alpine:3.21
RUN apk add --no-cache openssl strace
WORKDIR /app

ARG TWEL_DATA_KEY 
ENV TWEL_DATA_KEY=$TWEL_DATA_KEY
ARG ARCH
ENV ARCH=$ARCH
COPY --from=builder /app/target/$ARCH/release/str-proc ./
ENV RUST_LOG=debug

EXPOSE 8000
CMD ["./str-proc"]