# Native macOS app

This directory contains the native SwiftUI macOS app target for Aurelia.

## Canonical source layout

`AureliaMac/` is the single source of truth for app code.

- App code: `AureliaMac/`
- Tests: `Tests/`
- XcodeGen spec: `project.yml`
- Generated Xcode project: `AureliaMac.xcodeproj`

There is no parallel `Sources/` application tree.

## Build and run

From repo root:

- `bun run dev:desktop:macos` generates `AureliaMac.xcodeproj` (via XcodeGen) and opens it in Xcode.
- `bun run build:desktop:macos` generates the project and builds `AureliaMac` via `xcodebuild`.

Direct commands:

- `cd apps/desktop/macos && xcodegen --spec project.yml`
- `xcodebuild -project apps/desktop/macos/AureliaMac.xcodeproj -scheme AureliaMac -destination 'platform=macOS' build`
- `swift test --package-path apps/desktop/macos`

## Dependency

The app depends on shared `AureliaCore` at `apps/mobile/ios/AureliaCore`.
