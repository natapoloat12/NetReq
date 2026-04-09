# Build Stage
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build for release using nightly for edition2024 and unstable features
RUN cargo build --release

# Final Stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/anydesk-access-backend .

# Copy CA-certs from builder for secure API calls + install curl for healthchecks
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

EXPOSE 5051
CMD ["./anydesk-access-backend"]
