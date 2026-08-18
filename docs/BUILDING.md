# Building Aurelia

## Shared prerequisites

- Rust stable
- Java 17
- Android SDK 36 and build tools 36.0.0
- Android NDK 29.0.14206865
- `cargo-ndk`

On NixOS, `nix develop` supplies the Rust toolchain, Java, Android SDK/NDK, and `cargo-ndk`.

## Rust core

```bash
cargo build --workspace
```

The workspace contains `aurelia-core`, `aurelia-lyrics`, and the UniFFI binding generator used by both mobile builds.

## Android

```bash
cd apps/mobile/android
./gradlew assembleDebug
```

Gradle builds `aurelia-core` for the Android ABIs, regenerates the Kotlin UniFFI bindings, and packages the native libraries automatically. Release builds use:

```bash
./gradlew assembleRelease
```

The debug APK is written under `apps/mobile/android/app/build/outputs/apk/debug/`.

## Desktop prototype

The desktop prototype uses the mainline GPUI revision pinned in
`apps/desktop/Cargo.toml`. It does not depend on `gpui-component`.

On NixOS, enter the development shell so Fontconfig, Wayland, X11, and Vulkan
libraries are available, then run:

```bash
nix develop
cargo run -p aurelia-desktop
```

The desktop app authenticates directly through `aurelia-core`, stores its
session in Aurelia's application-data directory, and performs the same smart
library and favorites sync used by mobile. The home screen is populated from a
profile-specific cache after sync completes. Playback controls are still a UI
prototype and do not produce audio yet.

## iOS

iOS requires macOS and Xcode. Build the Rust XCFramework and regenerate Swift UniFFI bindings with:

```bash
./apps/mobile/ios/build-rust.sh
```

Use `--release` for optimized Rust libraries:

```bash
./apps/mobile/ios/build-rust.sh --release
```

Then open `apps/mobile/ios/Aurelia.xcworkspace` in Xcode, or build an unsigned archive from the command line:

```bash
xcodebuild archive \
  -workspace apps/mobile/ios/Aurelia.xcworkspace \
  -scheme Aurelia \
  -configuration Release \
  -destination 'generic/platform=iOS' \
  -archivePath build/ios.xcarchive \
  CODE_SIGN_IDENTITY='' \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO
```

## Generated bindings

Do not edit generated Kotlin or Swift UniFFI sources by hand. Android regenerates Kotlin bindings during Gradle pre-build; `apps/mobile/ios/build-rust.sh` regenerates Swift bindings and the XCFramework.
