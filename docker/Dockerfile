# Build stage
FROM rust:1.89 as builder

WORKDIR /app

# Configure rsproxy for faster builds in China
ENV RUSTUP_DIST_SERVER="https://rsproxy.cn"
ENV RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# Copy and setup cargo config for rsproxy
RUN mkdir -p ~/.cargo
COPY docker/cargo-config.toml ~/.cargo/config
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/mcp-gateway /app/mcp-gateway
EOF

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libcurl4 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/mcp-gateway /app/mcp-gateway

# Copy migrations and config from host
COPY migrations /app/migrations
COPY config /app/config

# Create necessary directories
RUN mkdir -p /app/logs

# Expose ports
EXPOSE 3000

CMD ["/app/mcp-gateway"]