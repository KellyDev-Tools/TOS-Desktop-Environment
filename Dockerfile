# Syntax: docker/dockerfile:1
# TOS Brain Daemon Dockerfile

FROM rust:1.80-slim-bullseye AS builder

WORKDIR /usr/src/tos
COPY . .

RUN cargo build --release --bin tos-brain

# --- Runtime Image ---
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/tos
COPY --from=builder /usr/src/tos/target/release/tos-brain /usr/local/bin/tos-brain

# The default Discovery Gate and IPC TCP/WS Ports
EXPOSE 7000
EXPOSE 7001

# Start the Brain Daemon headless
ENTRYPOINT ["tos-brain", "--headless"]
