# Stage 1: Use cargo-chef to cache dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Stage 2: Prepare the recipe for caching dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies and the application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build the application
COPY . .
RUN cargo build --release --bin study_app_backend

# Stage 4: Runtime environment
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install CA certificates and debugging tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl openssl && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/study_app_backend /usr/local/bin

# Verify CA certificates (optional for debugging)
RUN ls -l /etc/ssl/certs/ && \
    curl https://example.com --insecure || echo "Debugging: Curl failed"

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/study_app_backend"]

