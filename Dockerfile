FROM rust:1.75-slim-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -ms /bin/bash vscode \
    && mkdir -p /home/vscode/.cargo/registry \
    && chown -R vscode:vscode /home/vscode

# Set up working directory
WORKDIR /workspace

# Switch to non-root user
USER vscode

# Set environment variables
ENV RUST_BACKTRACE=1
ENV CARGO_HOME=/home/vscode/.cargo

# Pre-create Cargo directories
RUN mkdir -p /home/vscode/.cargo/registry \
    && mkdir -p /home/vscode/.cargo/git

# Keep container running
CMD ["sleep", "infinity"] 