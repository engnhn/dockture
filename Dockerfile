# ==============================================================================
# Stage 1: Build Phase (Ultra-fast compilation using Alpine Rust toolchain)
# ==============================================================================
FROM rust:alpine as builder

# Install musl-dev and build tools
RUN apk add --no-cache musl-dev ca-certificates

WORKDIR /usr/src/dockture

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build optimized release binary
RUN cargo build --release

# ==============================================================================
# Stage 2: Final Minimal Runtime Image (~15MB Alpine footprint)
# ==============================================================================
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

# Create default configuration directory
RUN mkdir -p /root/.config/dockture

# Copy compiled binary from builder stage
COPY --from=builder /usr/src/dockture/target/release/dockture /usr/local/bin/dockture

# Expose volume mount points for Docker socket and config
VOLUME ["/var/run/docker.sock", "/root/.config/dockture"]

# Set default entrypoint and command
ENTRYPOINT ["dockture"]
CMD ["run"]
