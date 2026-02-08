# Aurelia

Aurelia is a multi-platform Jellyfin music client monorepo.

## Applications

- Desktop (Tauri): `/Users/marshall/Projects/aurelia/apps/desktop/tauri`
- Desktop (native SwiftUI macOS): `/Users/marshall/Projects/aurelia/apps/desktop/macos`
- Web frontend: `/Users/marshall/Projects/aurelia/apps/web/frontend`
- Web backend (Axum): `/Users/marshall/Projects/aurelia/apps/web/backend`
- Shared Vue/TS package: `/Users/marshall/Projects/aurelia/apps/shared`
- Mobile clients:
  - Android: `/Users/marshall/Projects/aurelia/apps/mobile/android`
  - iOS: `/Users/marshall/Projects/aurelia/apps/mobile/ios`

## Core Rust Crates

- `aurelia-core`: `/Users/marshall/Projects/aurelia/crates/aurelia-core`
- `aurelia-api`: `/Users/marshall/Projects/aurelia/crates/aurelia-api`
- `aurelia-api-macros`: `/Users/marshall/Projects/aurelia/crates/aurelia-api-macros`
- `uniffi-bindgen`: `/Users/marshall/Projects/aurelia/crates/uniffi-bindgen`

## Quick Start

### Prerequisites

- Bun (v1+)
- Rust stable toolchain
- Node.js 20+
- Platform dependencies from `/Users/marshall/Projects/aurelia/BUILDING.md`

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
bun run dev:desktop:tauri

# Native macOS app (opens generated Xcode project)
bun run dev:desktop:macos
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
bun run build:desktop:tauri
bun run build:desktop:strict               # typecheck + build
bun run build:desktop:release              # fully optimized Rust release profile

# Native macOS build
bun run build:desktop:macos
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
bun run test:desktop:macos
bun run test:web
bun run test:android
bun run test:ios
```

Full testing guide: `/Users/marshall/Projects/aurelia/TESTING.md`

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
│   │   └── macos/
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

See `/Users/marshall/Projects/aurelia/CONTRIBUTING.md`.
