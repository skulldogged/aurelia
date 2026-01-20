# Architecture & Tech Stack

## Overview

```
aurelia/
├── apps/
│   ├── desktop/          # Desktop app (Tauri + Vue)
│   │   ├── src/          # Vue 3 frontend
│   │   └── src-tauri/    # Tauri/Rust backend
│   └── mobile/
│       └── android/      # Android app (Kotlin, Jetpack Compose)
└── crates/
    └── aurelia-core/     # Shared Rust library (uniffi bindings)
```

## Tech Stack

### Desktop (Tauri + Vue)
| Layer | Tech |
|-------|------|
| Frontend | Vue 3, Composition API, `<script setup>`, TypeScript |
| Styling | Tailwind CSS v4, shadcn-vue patterns |
| Shell | Tauri v2 (Rust backend) |
| State | Pinia stores (`src/stores`), composables (`src/composables`) |
| Tooling | Vite, Bun (`bun run`, `bunx`), Vue Router 4 |

### Android (Native)
| Layer | Tech |
|-------|------|
| UI | Jetpack Compose, Material 3 |
| Language | Kotlin |
| Media | Media3 / ExoPlayer |
| State | ViewModel + StateFlow |
| Backend | uniffi bindings to shared Rust core |
| Build | Gradle, Android SDK |

### Shared
| Component | Tech |
|-----------|------|
| Core logic | Rust (`crates/aurelia-core/`), exposed via uniffi |
| API | Jellyfin server integration |
