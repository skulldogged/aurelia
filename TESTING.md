# Testing Guide

This repository has tests across Rust, shared Vue/TS, web + desktop apps, and mobile.

## Quick Commands

- `bun run test:js` — run all JS/TS tests (shared + web + desktop)
- `bun run test:rust` — run all Rust tests
- `bun run test` — JS + Rust
- `bun run test:android` — Android JVM unit tests
- `bun run test:android:ui` — Android instrumentation tests (requires device/emulator)
- `bun run test:ios` — iOS Swift package tests (requires XCFramework)

## JS/TS (Shared + Web + Desktop)

- Shared:
  - `bun --cwd apps/shared test`
- Web frontend:
  - `bun --cwd apps/web/frontend test`
- Desktop frontend:
  - `bun --cwd apps/desktop test`

Each uses Vitest with the same conventions:
- Test files live in `tests/` or alongside source as `*.spec.ts`/`*.test.ts`.
- DOM environment: `happy-dom`.

## Rust

Run all Rust tests:

```bash
cargo test --workspace
```

Core crates contain unit tests inside modules, plus backend integration tests in `apps/web/backend/tests`.

## Web Backend (Axum)

Backend API route tests live in:
- `apps/web/backend/tests/api_routes.rs`

These are included in `cargo test --workspace`.

## Desktop (Tauri)

Desktop UI tests are Vitest-based and run with the JS test suite.

## Android

Unit tests (JVM):

```bash
cd apps/mobile/android
./gradlew test
```

UI tests (instrumentation):

```bash
cd apps/mobile/android
./gradlew connectedAndroidTest
```

You need a running emulator or attached device for instrumentation tests.

## iOS

The iOS Swift package tests require the UniFFI XCFramework to exist. Generate it first:

```bash
cd apps/mobile/ios
./build-rust.sh
```

Then run:

```bash
cd apps/mobile/ios/AureliaCore
swift test
```
