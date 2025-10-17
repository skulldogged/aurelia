# Mobile Support

Aurelia now includes basic mobile support for Android and iOS platforms using Tauri 2.

## Prerequisites

### Android Development

To build for Android, you need:

1. **Android Studio** with the following components:
   - Android SDK Platform 33 or higher
   - Android SDK Build-Tools
   - NDK (Side by side)

2. **Environment Variables**:
   ```bash
   export ANDROID_HOME=$HOME/Android/Sdk
   export NDK_HOME=$ANDROID_HOME/ndk/<version>
   ```

3. **Rust Android Targets**:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```

### iOS Development (macOS only)

To build for iOS, you need:

1. **Xcode** (from the Mac App Store)
2. **Rust iOS Targets**:
   ```bash
   rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
   ```

## Building and Running

### Android

Initialize Android project (first time only):
```bash
bun run tauri android init
```

Run in development mode:
```bash
bun run tauri android dev
```

Build release APK:
```bash
bun run tauri android build
```

### iOS (macOS only)

Initialize iOS project (first time only):
```bash
bun run tauri ios init
```

Run in development mode:
```bash
bun run tauri ios dev
```

Build release:
```bash
bun run tauri ios build
```

## Mobile-Specific Changes

The following features are disabled on mobile platforms:

1. **Discord Rich Presence** - Not available on mobile platforms
2. **Window Vibrancy Effects** - Mobile platforms use standard window decorations
3. **System Tray** - Not applicable on mobile platforms

All core music playback, library management, and streaming features work on mobile.

## Configuration

Mobile-specific configurations are in:
- `src-tauri/tauri.android.conf.json` - Android-specific settings
- `src-tauri/tauri.ios.conf.json` - iOS-specific settings

These files override the base `tauri.conf.json` for their respective platforms.

## Notes

- The app uses responsive layouts that work on mobile screens
- Custom window controls are only shown on desktop
- Touch gestures and mobile UI patterns should be considered for future enhancements
