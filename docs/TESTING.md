# Testing Aurelia

## Rust

```bash
cargo test --workspace
```

Run the matching package while iterating:

```bash
cargo test -p aurelia-core
cargo test -p aurelia-lyrics
```

## Android

```bash
cd apps/mobile/android
./gradlew ktlintCheck testDebugUnitTest
```

Instrumentation tests require a connected device or emulator:

```bash
./gradlew connectedDebugAndroidTest
```

## iOS

iOS validation requires macOS and Xcode:

```bash
./apps/mobile/ios/build-rust.sh
cd apps/mobile/ios/AureliaCore
swift test
```

The CI workflow performs Rust checks on Linux, builds and tests Android on Linux, and builds and tests iOS on macOS.
