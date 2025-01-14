# Build stage
FROM rust:1.81 as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the application and db_check binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binaries
COPY --from=builder /usr/src/app/target/release/document-automation /usr/local/bin/document-automation
COPY --from=builder /usr/src/app/target/release/db_check /usr/local/bin/db_check

# Copy migrations directory
COPY --from=builder /usr/src/app/migrations /usr/local/bin/migrations

# Set environment variables
ENV RUST_LOG=info

EXPOSE 3000

CMD ["document-automation"]
