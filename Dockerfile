# syntax=docker/dockerfile:1

########## Build stage ##########
# Pin the major version; any stable >= 1.75 (the crate's MSRV) works.
FROM rust:1-slim AS builder

WORKDIR /app

# Cache dependency compilation: copy the manifests first and build a
# placeholder crate, so later source changes reuse this dependency layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '' > src/lib.rs \
    && cargo build --release --locked

# Now build the real crate (deps are cached from the layer above).
COPY src ./src
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked

# Optional: run the offline test suite inside the image.
# RUN cargo test --release --locked

########## Runtime stage ##########
# bookworm on both stages keeps the builder's glibc <= the runtime's.
FROM debian:bookworm-slim

# CA certificates in case roots are loaded from the system; the client
# itself uses bundled webpki roots (rustls), so TLS works either way.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user.
COPY --from=builder --chown=10001:10001 \
    /app/target/release/massive /usr/local/bin/massive
USER 10001

# The demo reads MASSIVE_API_KEY (never bake secrets into the image):
#   docker run -e MASSIVE_API_KEY=... massive:latest          # AAPL REST demo
#   docker run -e MASSIVE_API_KEY=... massive:latest MSFT     # REST demo
#   docker run -e MASSIVE_API_KEY=... massive:latest ws AAPL  # WebSocket demo
ENTRYPOINT ["massive"]
