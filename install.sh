#!/bin/bash

set -e

echo "🔍 Installing hunt..."

# Check if cargo is installed, if not install Rust
if ! command -v cargo &> /dev/null; then
    echo "🔧 Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Build the project
echo "📦 Building hunt..."
cargo build --release

# Create ~/.local/bin if it doesn't exist
BIN_DIR="$HOME/.local/bin"
if [ ! -d "$BIN_DIR" ]; then
    echo "📁 Creating $BIN_DIR..."
    mkdir -p "$BIN_DIR"
fi

# Copy binary to ~/.local/bin
echo "🔧 Installing binary to $BIN_DIR/hunt..."
cp target/release/hunt "$BIN_DIR/hunt"
chmod +x "$BIN_DIR/hunt"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "⚠️  $BIN_DIR is not in your PATH."
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.bashrc  # or source ~/.zshrc"
fi

echo "✅ hunt installed successfully!"
echo "Run 'hunt --help' to get started."

