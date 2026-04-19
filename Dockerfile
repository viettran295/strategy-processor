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
FROM gcr.io/distroless/cc-debian13@sha256:56aaf20ab2523a346a67c8e8f8e8dabe447447d0788b82284d14ad79cd5f93cc
# Copy ca-certs and openssl 
COPY --from=builder /etc/ssl/certs /etc/ssl/certs
COPY --from=builder /usr/lib/*-linux-gnu/libssl.so.3    /usr/lib/
COPY --from=builder /usr/lib/*-linux-gnu/libcrypto.so.3 /usr/lib/

USER nonroot
WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /app/target/release/strategy-processor ./
EXPOSE 8000
CMD ["./strategy-processor"]