#!/bin/bash
set -eo pipefail

# Build iOS IPA and .app bundle from apps/mobile/ios/
# Usage: ./build-ios.sh [--release]
#
# Creates:
#   - build/Aurelia.ipa (iOS app package)
#   - build/Aurelia.app (iOS app bundle)

cd "$(dirname "$0")"

PROFILE="debug"
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
fi

echo "==> Installing JS dependencies..."
bun install

echo "==> Generating TypeScript & Kotlin Bindings..."
bun run bindings:generate

echo "==> Building Rust Framework..."
./apps/mobile/ios/build-rust.sh --release

echo "==> Building iOS App (Unsigned)..."
xcodebuild archive \
    -workspace apps/mobile/ios/Aurelia.xcworkspace \
    -scheme Aurelia \
    -configuration Release \
    -destination "generic/platform=iOS" \
    -archivePath build/ios.xcarchive \
    CODE_SIGN_IDENTITY="" \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO

echo "==> Packaging iOS IPA..."
mkdir -p build/ios-payload/Payload
cp -r build/ios.xcarchive/Products/Applications/Aurelia.app build/ios-payload/Payload/
cd build/ios-payload
zip -r ../Aurelia.ipa Payload
cd ../..

echo "==> Copying .app bundle..."
cp -r build/ios.xcarchive/Products/Applications/Aurelia.app build/Aurelia.app

echo ""
echo "==> Build complete!"
echo "    IPA: build/Aurelia.ipa"
echo "    App: build/Aurelia.app"
