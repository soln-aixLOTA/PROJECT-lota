FROM rust:latest as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Set DATABASE_URL for SQLx during build
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}
ENV SQLX_OFFLINE=true

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim as production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libtesseract5 \
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

# Create config directory
RUN mkdir -p /etc/document-automation/config

# Copy the binary from builder
COPY --from=builder /app/target/release/document-automation /usr/local/bin/

# Set working directory
WORKDIR /usr/local/bin

# Set the entrypoint
ENTRYPOINT ["document-automation"] 