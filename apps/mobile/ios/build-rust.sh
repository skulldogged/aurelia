#!/bin/bash
set -eo pipefail

# Build Rust aurelia-core for iOS targets and generate Swift UniFFI bindings.
# Usage: ./build-rust.sh [--release]
#
# Prerequisites:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CORE_CRATE="aurelia-core"
OUT_DIR="$SCRIPT_DIR/AureliaCore"
SOURCES_DIR="$OUT_DIR/Sources"
FRAMEWORK_DIR="$OUT_DIR/AureliaCoreFFI.xcframework"

PROFILE="debug"
CARGO_FLAGS=()
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS=("--release")
fi

TARGET_DIR="$PROJECT_ROOT/target"
# Keep all cargo outputs deterministic for this script so generated Swift bindings
# and XCFramework slices always come from the same build artifacts.
export CARGO_TARGET_DIR="$TARGET_DIR"

# Cleanup temp directories on exit
TEMP_DIRS=()
cleanup() {
    for dir in "${TEMP_DIRS[@]}"; do
        rm -rf "$dir"
    done
}
trap cleanup EXIT

ensure_rust_target() {
    local target="$1"
    if ! rustup target list --installed | grep -qx "$target"; then
        echo "==> Installing missing Rust target: $target"
        rustup target add "$target"
    fi
}

IOS_DEVICE_TARGET="aarch64-apple-ios"
IOS_SIM_TARGETS=("aarch64-apple-ios-sim" "x86_64-apple-ios")
HOST_ARCH="$(uname -m)"

ensure_rust_target "$IOS_DEVICE_TARGET"
for IOS_SIM_TARGET in "${IOS_SIM_TARGETS[@]}"; do
    ensure_rust_target "$IOS_SIM_TARGET"
done

echo "==> Building aurelia-core for iOS device (aarch64-apple-ios)..."
IPHONEOS_DEPLOYMENT_TARGET=18.0 \
cargo build -p "$CORE_CRATE" --target "$IOS_DEVICE_TARGET" "${CARGO_FLAGS[@]}"

for IOS_SIM_TARGET in "${IOS_SIM_TARGETS[@]}"; do
    echo "==> Building aurelia-core for iOS simulator ($IOS_SIM_TARGET)..."
    IPHONEOS_DEPLOYMENT_TARGET=18.0 \
    cargo build -p "$CORE_CRATE" --target "$IOS_SIM_TARGET" "${CARGO_FLAGS[@]}"
done

# Build macOS target for SwiftPM tests (host architecture)
if [[ "$HOST_ARCH" == "arm64" ]]; then
    MACOS_TARGET="aarch64-apple-darwin"
else
    MACOS_TARGET="x86_64-apple-darwin"
fi
ensure_rust_target "$MACOS_TARGET"
echo "==> Building aurelia-core for macOS ($MACOS_TARGET)..."
MACOSX_DEPLOYMENT_TARGET=13.0 cargo build -p "$CORE_CRATE" --target "$MACOS_TARGET" "${CARGO_FLAGS[@]}"

# Generate Swift bindings from the host build
echo "==> Building aurelia-core for host (binding generation)..."
cargo build -p "$CORE_CRATE" "${CARGO_FLAGS[@]}"

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
HEADER_FILES=("$SOURCES_DIR/aurelia_coreFFI.h" "$SOURCES_DIR/aurelia_lyricsFFI.h")
MODULEMAP_FILES=("$SOURCES_DIR/aurelia_coreFFI.modulemap" "$SOURCES_DIR/aurelia_lyricsFFI.modulemap")

echo "==> Creating XCFramework..."
rm -rf "$FRAMEWORK_DIR"

DEVICE_LIB="$TARGET_DIR/$IOS_DEVICE_TARGET/$PROFILE/libaurelia_core.a"
SIM_ARM64_LIB="$TARGET_DIR/aarch64-apple-ios-sim/$PROFILE/libaurelia_core.a"
SIM_X86_64_LIB="$TARGET_DIR/x86_64-apple-ios/$PROFILE/libaurelia_core.a"
MACOS_LIB="$TARGET_DIR/$MACOS_TARGET/$PROFILE/libaurelia_core.a"

# xcodebuild requires a single library definition per platform variant.
# Merge arm64 + x86_64 iOS simulator static libs into one universal archive.
SIM_UNIVERSAL_DIR=$(mktemp -d)
TEMP_DIRS+=("$SIM_UNIVERSAL_DIR")
SIM_LIB="$SIM_UNIVERSAL_DIR/libaurelia_core.a"
lipo -create "$SIM_ARM64_LIB" "$SIM_X86_64_LIB" -output "$SIM_LIB"

# Create temporary directories for headers
DEVICE_HEADERS=$(mktemp -d)
SIM_HEADERS=$(mktemp -d)
MACOS_HEADERS=$(mktemp -d)
TEMP_DIRS+=("$DEVICE_HEADERS" "$SIM_HEADERS" "$MACOS_HEADERS")

for dir in "$DEVICE_HEADERS" "$SIM_HEADERS" "$MACOS_HEADERS"; do
    cp "${HEADER_FILES[@]}" "$dir/"
    for f in "${MODULEMAP_FILES[@]}"; do cat "$f" >> "$dir/module.modulemap"; echo "" >> "$dir/module.modulemap"; done
done

xcodebuild -create-xcframework \
    -library "$DEVICE_LIB" -headers "$DEVICE_HEADERS" \
    -library "$SIM_LIB" -headers "$SIM_HEADERS" \
    -library "$MACOS_LIB" -headers "$MACOS_HEADERS" \
    -output "$FRAMEWORK_DIR"

# Ensure SwiftPM can read module maps copied from mktemp-created header dirs.
chmod -R a+rX "$FRAMEWORK_DIR"

echo "==> Done! XCFramework at: $FRAMEWORK_DIR"
echo "    Swift bindings at: $SOURCES_DIR/AureliaCore.swift"
