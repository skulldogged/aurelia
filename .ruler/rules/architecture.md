# Architecture & Tech Stack

## Monorepo Overview

```
aurelia/
├── apps/
│   ├── desktop/electron/ # Desktop app (Electron + Vue)
│   │   ├── src/          # Thin shell - imports from @shared
│   │   └── electron/     # Electron main/preload; Rust lives in the local backend
│   ├── web/
│   │   ├── frontend/     # Web frontend (Vite + Vue)
│   │   └── backend/      # Axum server (Rust)
│   ├── mobile/
│   │   ├── android/      # Android app (Kotlin + Jetpack Compose)
│   │   └── ios/          # iOS app
│   └── shared/           # CORE: Shared Vue code
│       ├── src/
│       │   ├── stores/       # Pinia stores (11 stores)
│       │   ├── composables/  # Vue composables (20+)
│       │   ├── components/   # UI components
│       │   │   └── ui/       # shadcn-vue components
│       │   ├── lib/          # Utilities, API clients
│       │   └── pages/        # Vue Router pages
│       └── package.json
└── crates/
    ├── aurelia-core/     # Shared Rust library (audio, uniffi bindings)
    ├── aurelia-api/      # HTTP API abstraction (Axum impl)
    ├── aurelia-api-macros/ # Procedural macros
    └── uniffi-bindgen/   # Binding generator
```

## Tech Stack

### Frontend (Shared Package)

Vue code shared between **desktop and web only**. Mobile uses native Kotlin/Swift via uniffi bindings.

| Layer | Tech |
|-------|------|
| Framework | Vue 3.5, Composition API, `<script setup>`, TypeScript |
| Styling | Tailwind CSS v4, reka-ui, shadcn-vue patterns |
| State | Pinia stores (`apps/shared/src/stores`) |
| Logic | Composables (`apps/shared/src/composables`) |
| UI | shadcn-vue components (`apps/shared/src/components/ui`) |
| Utilities | `apps/shared/src/lib` |
| Routing | Vue Router 4 |

### Desktop (Electron)

Thin shell in `apps/desktop/electron/` - imports everything from `@aurelia/shared`. Native playback runs in the local Axum backend.

| Layer | Tech |
|-------|------|
| Shell | Electron (frameless window, tray, Last.fm callback) |
| Backend | Local `aurelia-web-backend` over HTTP + WebSocket |
| Build | Vite, Bun |
| Audio | Rust player (rodio, symphonia, rustfft) — not Web Audio |
| Extras | Discord RPC, system tray |

### Web

| Layer | Tech |
|-------|------|
| Frontend | Same shared package as desktop |
| Backend | Axum (Rust in `apps/web/backend/`) |
| API | OpenAPI-generated client |

### Mobile

| Platform | Tech |
|----------|------|
| Android | Kotlin, Jetpack Compose, Material 3, Media3/ExoPlayer |
| iOS | Native iOS |
| Shared | uniffi bindings to `aurelia-core` |

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `aurelia-core` | Domain logic, models, services, database |
| `aurelia-api` | API abstraction with Axum HTTP implementations |
| `aurelia-api-macros` | Proc macros for generating API boilerplate |
| `uniffi-bindgen` | Generates TypeScript/Kotlin bindings |

## Key Patterns

- **Shared Package**: Vue code shared between desktop and web. Both are thin shells importing from `apps/shared/`. Mobile uses native code with uniffi bindings.
- **API Abstraction**: `aurelia-api` generates Axum routes and a TypeScript HTTP client. Electron talks to the local backend the same way the web app does.
- **Desktop audio**: Electron uses `RustAudioPlayerImpl` (HTTP commands + `/ws` position/spectrum). The browser uses Web Audio.
- **uniffi**: Rust core exposed to Android/iOS via generated bindings.
- **Bun**: Package management and scripts throughout.
