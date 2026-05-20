# Aurelia

Aurelia is a multi-platform music client monorepo with first-class Jellyfin support.

## Backend Providers

- Supported providers: `Jellyfin`

## Applications

- Desktop (Tauri): `apps/desktop/tauri`
- Web frontend: `apps/web/frontend`
- Web backend (Axum): `apps/web/backend`
- Shared Vue/TS package: `apps/shared`
- Mobile clients:
  - Android: `apps/mobile/android`
  - iOS: `apps/mobile/ios`

## Core Rust Crates

- `aurelia-core`: `crates/aurelia-core`
- `aurelia-api`: `crates/aurelia-api`
- `aurelia-api-macros`: `crates/aurelia-api-macros`
- `uniffi-bindgen`: `crates/uniffi-bindgen`

## Quick Start

### Prerequisites

- Bun (v1+)
- Rust stable toolchain
- Node.js 20+
- Platform dependencies from `BUILDING.md`

### Install

```bash
git clone https://github.com/pupbrained/aurelia.git
cd aurelia
bun install
```

### Development Commands (from repo root)

```bash
# Web (frontend + backend)
bun run dev:web

# Desktop Tauri app
bun run dev:desktop
```

## Build Commands (from repo root)

```bash
# Web frontend + backend release build
bun run build:web
bun run build:web -- --skip-bindings      # skip binding generation
bun run build:web -- --force-bindings     # force binding generation
bun run build:web:strict                   # typecheck + build
bun run build:web:release                  # fully optimized Rust release profile

# Desktop Tauri build
bun run build:desktop
bun run build:desktop:strict               # typecheck + build
bun run build:desktop:release              # fully optimized Rust release profile
```

## Typecheck Commands (from repo root)

```bash
bun run typecheck
bun run typecheck:web
bun run typecheck:desktop
```

## Testing Commands (from repo root)

```bash
bun run test
bun run test:js
bun run test:rust
bun run test:desktop
bun run test:web
bun run test:android
bun run test:ios
```

Full testing guide: `TESTING.md`

## Monorepo Layout

```text
aurelia/
├── apps/
│   ├── shared/
│   ├── web/
│   │   ├── frontend/
│   │   └── backend/
│   ├── desktop/
│   │   ├── tauri/
│   │   │   └── src-tauri
│   └── mobile/
│       ├── android/
│       └── ios/
├── crates/
├── scripts/
├── BUILDING.md
├── CONTRIBUTING.md
└── TESTING.md
```

## Contributing

See `CONTRIBUTING.md`.
