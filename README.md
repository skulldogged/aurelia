# Aurelia

Aurelia is a native mobile music client for Jellyfin.

- Android is built with Kotlin, Jetpack Compose, and Media3.
- iOS is built with SwiftUI and AVFoundation.
- Shared Jellyfin, library, cache, and lyrics behavior lives in Rust and is exposed to both apps through UniFFI.

## Repository layout

```text
apps/mobile/android/      Android application
apps/mobile/ios/          iOS application and Swift package
crates/aurelia-core/      Shared domain, persistence, and Jellyfin logic
crates/aurelia-lyrics/    Lyrics parsing and models
crates/uniffi-bindgen/    Mobile binding-generation CLI wrapper
```

## Quick start

Enter the Nix development shell when using NixOS:

```bash
nix develop
```

Build and test Android:

```bash
cd apps/mobile/android
./gradlew testDebugUnitTest assembleDebug
```

Build the Rust core and run its tests:

```bash
cargo test --workspace
```

iOS builds require macOS with Xcode:

```bash
./apps/mobile/ios/build-rust.sh
open apps/mobile/ios/Aurelia.xcworkspace
```

See [docs/BUILDING.md](docs/BUILDING.md) and [docs/TESTING.md](docs/TESTING.md) for the complete workflows.
