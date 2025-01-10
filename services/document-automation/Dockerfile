FROM debian:bullseye-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libtesseract-dev \
    tesseract-ocr \
    curl \
    build-essential \
    ca-certificates

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Create app directory
WORKDIR /usr/src/app

# Copy Cargo files
COPY services/document-automation/Cargo.toml services/document-automation/Cargo.lock ./

# Copy source code
COPY services/document-automation/src ./src/

# Create .sqlx directory
RUN mkdir -p .sqlx

# Copy SQLx data
COPY services/document-automation/.sqlx ./.sqlx/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libtesseract4 \
    tesseract-ocr \
    && rm -rf /var/lib/apt/lists/*

# Create config directory
RUN mkdir -p /etc/document-automation/config

# Copy the binary
COPY --from=builder /usr/src/app/target/release/document-automation /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/document-automation"]
