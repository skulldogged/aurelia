#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

RUN_IOS="auto"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-ios)
      RUN_IOS="false"
      shift
      ;;
    --ios)
      RUN_IOS="true"
      shift
      ;;
    --check)
      CHECK_ONLY="true"
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: scripts/generate-bindings.sh [--skip-ios|--ios] [--check]" >&2
      exit 1
      ;;
  esac
done

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"

if [[ "$(uname -s)" == "Darwin" ]]; then
  SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
  export SDKROOT
  export CFLAGS="-isysroot $SDKROOT"
fi

if [[ "$RUN_IOS" == "auto" ]]; then
  if [[ "$(uname -s)" == "Darwin" ]]; then
    RUN_IOS="true"
  else
    RUN_IOS="false"
  fi
fi

if [[ "$RUN_IOS" == "true" && "$(uname -s)" != "Darwin" ]]; then
  echo "iOS generation requires macOS. Re-run with --skip-ios on this platform." >&2
  exit 1
fi

SENTINEL_DIR="$ROOT_DIR/.bindings-cache"
SENTINEL_FILE="$SENTINEL_DIR/last-hash"
mkdir -p "$SENTINEL_DIR"

RUST_SOURCES=$(find "$ROOT_DIR/crates" -name "*.rs" -type f 2>/dev/null | sort)
CARGO_FILES=$(find "$ROOT_DIR" -name "Cargo.toml" -maxdepth 2 -type f 2>/dev/null | sort)
UNIFFI_CONFIGS=$(find "$ROOT_DIR/apps/mobile" -name "uniffi.toml" -type f 2>/dev/null | sort)

CURRENT_HASH=$(echo "$RUST_SOURCES $CARGO_FILES $UNIFFI_CONFIGS" | xargs cat 2>/dev/null | shasum -a 256 | cut -d' ' -f1)

if [[ -f "$SENTINEL_FILE" ]]; then
  STORED_HASH=$(cat "$SENTINEL_FILE")
  if [[ "$CURRENT_HASH" == "$STORED_HASH" ]]; then
    echo "==> Bindings up to date (no Rust source changes detected)"
    echo "$CURRENT_HASH" > "$SENTINEL_FILE"
    exit 0
  fi
fi

echo "==> Building aurelia-core and uniffi-bindgen"
cargo build -p aurelia-core
cargo build -p uniffi-bindgen

echo "==> Regenerating shared TypeScript bindings"
cargo run -p uniffi-bindgen -- all --out-dir apps/shared/src/generated

echo "==> Regenerating macro-generated API client/types"
cargo check -p aurelia-api --features web

HOST_LIB=""
for candidate in \
  "$CARGO_TARGET_DIR/debug/libaurelia_core.dylib" \
  "$CARGO_TARGET_DIR/debug/libaurelia_core.so" \
  "$CARGO_TARGET_DIR/debug/aurelia_core.dll"; do
  if [[ -f "$candidate" ]]; then
    HOST_LIB="$candidate"
    break
  fi
done

if [[ -z "$HOST_LIB" ]]; then
  echo "Could not find host aurelia-core dynamic library in $CARGO_TARGET_DIR/debug" >&2
  exit 1
fi

echo "==> Regenerating Android Kotlin UniFFI bindings"
cargo run -p uniffi-bindgen -- generate \
  --library "$HOST_LIB" \
  --language kotlin \
  --config apps/mobile/android/app/src/main/java/uniffi/aurelia_core/uniffi.toml \
  --out-dir apps/mobile/android/app/src/main/java \
  --no-format

if [[ "$RUN_IOS" == "true" ]]; then
  echo "==> Regenerating iOS Swift bindings/XCFramework"
  (cd apps/mobile/ios && ./build-rust.sh)

  if [[ -z "${XCODE_VERSION_ACTUAL:-}" ]]; then
      echo "==> Verifying iOS Swift package"
      swift build --package-path apps/mobile/ios/AureliaCore
  else
      echo "==> Skipping Swift package verification (running inside Xcode)"
  fi
else
  echo "==> Skipping iOS generation"
fi

echo "$CURRENT_HASH" > "$SENTINEL_FILE"
echo "==> Binding generation complete (cached hash updated)"
