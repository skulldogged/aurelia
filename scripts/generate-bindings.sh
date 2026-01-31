#!/bin/bash
set -e

echo "🔧 Generating Aurelia TypeScript bindings..."

# Build the bindgen tool first
echo "  → Building bindgen tool..."
cargo build -p uniffi-bindgen

# Generate all TypeScript bindings
echo "  → Generating TypeScript types and client..."
cargo run -p uniffi-bindgen -- all --out-dir apps/shared/src/generated

echo "✅ All bindings generated successfully!"
