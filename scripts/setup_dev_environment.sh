#!/bin/bash
set -e

# Function to install dependencies on macOS
install_macos_dependencies() {
    echo "Installing dependencies on macOS..."
    brew update
    brew install openssl pkg-config
}

# Function to install dependencies on Linux
install_linux_dependencies() {
    echo "Installing dependencies on Linux..."
    sudo apt-get update
    sudo apt-get install -y build-essential libssl-dev pkg-config
}

# Detect the operating system and install dependencies
if [[ "$(uname)" == "Darwin" ]]; then
    # macOS
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    install_macos_dependencies
elif [[ "$(expr substr $(uname -s) 1 5)" == "Linux" ]]; then
    # Linux
    install_linux_dependencies
else
    echo "Unsupported operating system. Please install dependencies manually."
    exit 1
fi

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install development tools
rustup component add clippy rustfmt
cargo install cargo-watch cargo-audit

# Set up git hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Create development configuration
cp config/rapidmq.yaml config/rapidmq_dev.yaml

echo "Development environment setup complete!"