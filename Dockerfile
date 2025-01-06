FROM rust:latest as builder

WORKDIR /usr/src/app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libtesseract-dev \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Set environment variables for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/document_automation
ENV SQLX_OFFLINE=true
ENV SQLX_OFFLINE_DIR=/usr/src/app/.sqlx

# Create SQLx data directory
RUN mkdir -p /usr/src/app/.sqlx

# Build the application
RUN cargo build --release --package document-automation

# Create the runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libtesseract4 \
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

# Create config directory
RUN mkdir -p /etc/document-automation/config

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/document-automation /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["document-automation"] 