#!/bin/bash
set -e

echo "Setting up LotaBots development environment..."

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME"/.cargo/env
fi

# Update Rust
rustup update

# Check cargo fmt and clippy
rustup component add rustfmt
rustup component add clippy

# Install development tools
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-audit

# Create necessary directories
mkdir -p .cargo

# Create cargo config
cat > .cargo/config.toml << EOL
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[alias]
serve = "watch -x run"
lint = "clippy -- -D warnings"
fmt = "fmt --all"
EOL

echo "Development environment setup complete!"
echo
echo "Next steps:"
echo "1. Set up your environment variables (copy .env.example to .env)"
echo "2. Run 'cargo build' to build all services"
echo "3. Run 'cargo test' to verify everything works"
echo
echo "Development commands:"
echo "- cargo serve    : Run with auto-reload on changes"
echo "- cargo lint    : Run clippy with strict settings"
echo "- cargo fmt     : Format all code" 