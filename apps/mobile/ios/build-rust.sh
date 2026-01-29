#!/bin/bash
set -euo pipefail

# Build Rust aurelia-core for iOS targets and generate Swift UniFFI bindings.
# Usage: ./build-rust.sh [--release]
#
# Prerequisites:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CORE_CRATE="aurelia-core"
OUT_DIR="$SCRIPT_DIR/AureliaCore"
SOURCES_DIR="$OUT_DIR/Sources"
FRAMEWORK_DIR="$OUT_DIR/AureliaCoreFFI.xcframework"

PROFILE="debug"
CARGO_FLAGS=""
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
fi

TARGET_DIR="$PROJECT_ROOT/target"

echo "==> Building aurelia-core for iOS device (aarch64-apple-ios)..."
cargo build -p "$CORE_CRATE" --target aarch64-apple-ios $CARGO_FLAGS

echo "==> Building aurelia-core for iOS simulator (aarch64-apple-ios-sim)..."
cargo build -p "$CORE_CRATE" --target aarch64-apple-ios-sim $CARGO_FLAGS

# Generate Swift bindings from the host build
echo "==> Building aurelia-core for host (binding generation)..."
cargo build -p "$CORE_CRATE" $CARGO_FLAGS

HOST_LIB="$TARGET_DIR/$PROFILE/libaurelia_core.dylib"
if [[ ! -f "$HOST_LIB" ]]; then
    # Try .so on Linux
    HOST_LIB="$TARGET_DIR/$PROFILE/libaurelia_core.so"
fi

echo "==> Generating Swift UniFFI bindings..."
mkdir -p "$SOURCES_DIR"
cargo run -p uniffi-bindgen -- generate \
    --library "$HOST_LIB" \
    --language swift \
    --out-dir "$SOURCES_DIR" \
    --no-format

mv "$SOURCES_DIR/aurelia_core.swift" "$SOURCES_DIR/AureliaCore.swift"

# Move the generated header and modulemap into place for the XCFramework
HEADER_FILE="$SOURCES_DIR/aurelia_coreFFI.h"
MODULEMAP_FILE="$SOURCES_DIR/aurelia_coreFFI.modulemap"

echo "==> Creating XCFramework..."
rm -rf "$FRAMEWORK_DIR"

DEVICE_LIB="$TARGET_DIR/aarch64-apple-ios/$PROFILE/libaurelia_core.a"
SIM_LIB="$TARGET_DIR/aarch64-apple-ios-sim/$PROFILE/libaurelia_core.a"

# Create temporary directories for headers
DEVICE_HEADERS=$(mktemp -d)
SIM_HEADERS=$(mktemp -d)
cp "$HEADER_FILE" "$DEVICE_HEADERS/"
cp "$MODULEMAP_FILE" "$DEVICE_HEADERS/module.modulemap"
cp "$HEADER_FILE" "$SIM_HEADERS/"
cp "$MODULEMAP_FILE" "$SIM_HEADERS/module.modulemap"

xcodebuild -create-xcframework \
    -library "$DEVICE_LIB" -headers "$DEVICE_HEADERS" \
    -library "$SIM_LIB" -headers "$SIM_HEADERS" \
    -output "$FRAMEWORK_DIR"

rm -rf "$DEVICE_HEADERS" "$SIM_HEADERS"

echo "==> Done! XCFramework at: $FRAMEWORK_DIR"
echo "    Swift bindings at: $SOURCES_DIR/aurelia_core.swift"
