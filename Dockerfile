# Stage 1: Build the application
FROM rust:1.90-slim-bookworm AS builder

# Install musl tools for static linking (optional, but highly recommended for minimal images)
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Copy manifest files first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Build only dependencies
#RUN cargo fetch --locked --target x86_64-unknown-linux-musl

# Copy source code and build the release binary
COPY src ./src
RUN CARGO_INCREMENTAL=0 RUSTFLAGS="-C strip=debuginfo" cargo build --release --locked --target x86_64-unknown-linux-musl

# Stage 2: Create a minimal runtime image
FROM scratch

# Copy the statically linked binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/meshlet ./meshlet

EXPOSE 8000

CMD ["./meshlet"]
