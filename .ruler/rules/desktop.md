# Desktop Guidelines (Vue + Electron)

## Architecture

The desktop app is a **thin Electron shell** that imports all business logic from `apps/shared/` via `@aurelia/shared`. Native audio, library, and integrations run in the local Axum backend (`aurelia-web-backend`).

```
apps/desktop/electron/
├── src/              # Shell only (App.vue, router, assets)
├── electron/         # Main process + preload
└── scripts/          # Dev/build helpers that spawn the Rust backend
```

## Coding Standards

### Vue & TypeScript

- **Structure**: `<script setup>` (Composition API) -> `<template>` -> `<style>`.
- **State**: Use `ref`/`reactive`. PascalCase components.
- **Styling**: Prefer Tailwind utilities (v4). Use `<style scoped>` only when necessary.
- **Imports**: Import shared logic from `@aurelia/shared`:
  ```ts
  import { usePlayerStore } from '@aurelia/shared'
  import { Button } from '@aurelia/shared'
  ```

### Where Code Lives

| What | Where | Import |
|------|-------|--------|
| Pinia stores | `apps/shared/src/stores/` | `from '@aurelia/shared'` |
| Composables | `apps/shared/src/composables/` | `from '@aurelia/shared'` |
| UI components | `apps/shared/src/components/ui/` | `from '@aurelia/shared'` |
| Utilities | `apps/shared/src/lib/` | `from '@aurelia/shared'` |
| Shared pages | `apps/shared/src/pages/` | `from '@aurelia/shared'` |
| Desktop-only components | `apps/desktop/electron/src/` | Relative import |

### Electron & Rust

- **Communication**: Call the local Axum backend over HTTP (`/api`) and WebSocket (`/ws`). Use generated effects, not Electron IPC, for playback.
- **Window/tray/Last.fm callback**: Electron preload bridge (`window.aureliaDesktop`).
- **Audio**: Keep the Rust player (rodio + symphonia + rustfft). Electron uses `RustAudioPlayerImpl`; the browser uses Web Audio. Do not fall back to Web Audio in Electron.

### UX/Accessibility

- Maintain ARIA accessibility.
- Respect reduced motion preferences.

## Available Stores (in shared package)

- `usePlayerStore` - Audio player state (494 lines, comprehensive)
- `useLibraryStore` - Music library
- `useAuthStore` - Authentication
- `usePlaylistStore` - Playlist management
- `useThemeStore` - Theme/colors
- `useHomeStore` - Home page data
- `useLastFmStore` - Last.fm integration
- `useListenBrainzStore` - ListenBrainz integration
- `useSystemTrayStore` - System tray state
- `useAccentColorStore` - Accent colors

## Available Composables (in shared package)

- `useAudioEngine` - Audio playback abstraction
- `usePlayerControls` - Player controls
- `usePlayerSession` - Session management
- `useLibrary` - Library operations
- `useAuth` - Authentication flow
- `useDiscordPresence` - Discord Rich Presence
- `useSystemTray` - Tray integration
- `useVisualizerData` - Audio visualization
- `useSession` - Session management
- And 10+ more...
