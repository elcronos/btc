#!/usr/bin/env bash
set -euo pipefail

echo "=== Building BTC ==="
cargo build --release

echo ""
echo "=== Installing to ~/.cargo/bin ==="
cp target/release/btc ~/.cargo/bin/btc

# Ad-hoc sign for macOS (prevents "killed" on Apple Silicon)
if [ "$(uname)" = "Darwin" ]; then
    echo "=== Signing binary (macOS) ==="
    codesign -s - --force ~/.cargo/bin/btc
fi

echo ""
echo "[OK] btc installed to ~/.cargo/bin/btc"
echo "     Version: $(~/.cargo/bin/btc --version)"
echo ""
echo "Run 'btc setup' in a project directory to get started."
