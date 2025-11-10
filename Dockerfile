# Build stage - use Debian 12 (bookworm) to match distroless base
FROM rust:1.90-slim-bookworm AS builder

WORKDIR /usr/src/headwind

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage - using distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/headwind/target/release/headwind /app/headwind

# Distroless nonroot image runs as UID 65532 (nonroot user)
# No shell, no package managers - minimal attack surface

# Expose ports
EXPOSE 8080 8081 8082 9090

ENTRYPOINT ["/app/headwind"]
