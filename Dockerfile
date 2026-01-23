# Multi-stage Docker build for letsopt gRPC server
# Stage 1: Build the Rust application
FROM rust:1.83-bookworm as builder

# Install build dependencies for COIN-OR CBC and HiGHS
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    protobuf-compiler \
    libprotobuf-dev \
    coinor-libcbc-dev \
    coinor-libclp-dev \
    coinor-libcoinutils-dev \
    coinor-libosi-dev \
    coinor-libcgl-dev \
    clang \
    libclang-dev \
    llvm-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock build.rs ./

# Copy proto files
COPY proto/ ./proto/

# Copy source code
COPY src/ ./src/

# Build the application in release mode with both solvers
# COIN-OR CBC and HiGHS will be compiled from source
RUN cargo build --release --bin letsopt-server

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libstdc++6 \
    coinor-libcbc3 \
    coinor-libclp1 \
    coinor-libcoinutils3v5 \
    coinor-libosi1v5 \
    coinor-libcgl1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 letsopt

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/letsopt-server /app/letsopt-server

# Change ownership
RUN chown -R letsopt:letsopt /app

# Switch to non-root user
USER letsopt

# Expose gRPC port
EXPOSE 50051

# Health check (optional - checks if port is listening)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD timeout 5s bash -c '</dev/tcp/localhost/50051' || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV GRPC_ADDRESS=0.0.0.0:50051

# Run the server
CMD ["/app/letsopt-server"]
