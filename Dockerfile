FROM rust:1.95.0-slim-trixie@sha256:319f354c6f068b01c19292555033a0e6133b4249a893c479aec3b0b8ab2f660a AS builder
WORKDIR /app

RUN USER=root \
    apt-get update && \
    apt-get install -y openssl \
    make \
    g++ \
    perl \
    ca-certificates && \
    update-ca-certificates --fresh

ENV CXX="g++" CC="gcc"
COPY . .

RUN cargo build --release --verbose

# ---------------------------------
# Arm architecture image for deployment machine
FROM gcr.io/distroless/cc-debian13@sha256:d4ee60642acd6531185413d51c4c9a709f07dbd69fb400788950bc6983b80574
# Copy ca-certs and openssl 
COPY --from=builder /etc/ssl/certs /etc/ssl/certs
COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl.so.3    /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypto.so.3 /usr/lib/x86_64-linux-gnu/

USER nonroot
WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /app/target/release/strategy-processor ./
EXPOSE 8000
CMD ["./strategy-processor"]