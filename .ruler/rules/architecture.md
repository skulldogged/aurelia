# Architecture & Tech Stack

## Monorepo Overview

```
aurelia/
├── apps/
│   ├── desktop/          # Desktop app (Tauri + Vue)
│   │   ├── src/          # Thin shell - imports from @shared
│   │   └── src-tauri/    # Tauri/Rust backend
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
    ├── aurelia-core/     # Shared Rust library (uniffi bindings)
    ├── aurelia-api/      # API abstraction (Tauri/Axum impl)
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

### Desktop (Tauri)

Thin shell in `apps/desktop/src/` - imports everything from `@aurelia/shared`.

| Layer | Tech |
|-------|------|
| Shell | Tauri v2 (Rust backend in `src-tauri/src/`) |
| Build | Vite, Bun |
| Audio | rodio, symphonia, rustfft |
| Extras | Discord RPC, system tray, media controls |

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
| `aurelia-api` | API abstraction with Tauri/Axum implementations |
| `aurelia-api-macros` | Proc macros for generating API boilerplate |
| `uniffi-bindgen` | Generates TypeScript/Kotlin bindings |

## Key Patterns

- **Shared Package**: Vue code shared between desktop and web. Both are thin shells importing from `apps/shared/`. Mobile uses native code with uniffi bindings.
- **API Abstraction**: `aurelia-api` provides uniform API interface with separate Tauri and Axum implementations.
- **uniffi**: Rust core exposed to Android/iOS via generated bindings.
- **Bun**: Package management and scripts throughout.
