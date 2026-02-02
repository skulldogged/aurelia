# Desktop Guidelines (Vue + Tauri)

## Architecture

The desktop app is a **thin shell** that imports all business logic from `apps/shared/` via `@aurelia/shared`.

```
apps/desktop/
├── src/              # Shell only (App.vue, router, assets)
│   ├── components/   # Desktop-specific chrome (top bars, etc.)
│   ├── router/
│   └── main.ts
└── src-tauri/        # Rust backend
    └── src/
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
| Desktop-only components | `apps/desktop/src/components/` | Relative import |

### Tauri & Rust

- **Communication**: Call Rust via `@tauri-apps/api`. Always handle errors (try/catch) with user-facing messages.
- **Security**: Validate payloads. Follow Tauri security practices.
- **Structure**: Rust modules in `apps/desktop/src-tauri/src`.
- **Audio**: rodio with symphonia backend, rustfft for visualization.

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
