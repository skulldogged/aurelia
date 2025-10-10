# Project Overview: Jellyfin Music Player

## Purpose
A modern desktop music player for Jellyfin, providing a beautiful native interface for browsing and playing music from a Jellyfin server. Features include full library management, playback controls, audio features like equalizer and gapless playback, lyrics support, Discord Rich Presence, fullscreen player, and system tray integration.

## Tech Stack
- **Frontend**: Vue 3 with Composition API and `<script setup>` syntax, fully typed with TypeScript
- **UI**: shadcn-vue components with Tailwind CSS v4 for styling
- **Desktop Framework**: Tauri v2 with Rust backend for native performance
- **State Management**: Pinia stores
- **Build Tools**: Vite for frontend bundling, Cargo for Rust compilation
- **Package Manager**: Bun (exclusive - do not use npm/pnpm/yarn)

## Key Dependencies
- @tauri-apps/api: Tauri frontend API
- @vueuse/core: Vue composition utilities
- pinia: State management
- tailwindcss: Utility-first CSS
- fuse.js: Fuzzy search
- lucide-vue-next: Icons
- Various Tauri plugins for OS integration

## Architecture
- **Frontend** (`src/`): Vue components, composables, stores, views, router
- **Backend** (`src-tauri/`): Rust code with Tauri configuration
- **Components**: Organized by feature (layout, player, settings, ui)
- **State**: Pinia stores for global state management
- **Composables**: Reusable Vue composition functions
- **Lib**: Utility functions, API clients, platform helpers

## Development Environment
- **OS**: Windows (with PowerShell)
- **Prerequisites**: Bun v1.0+, Rust stable, Node.js v20+, Visual Studio C++ Build Tools
- **IDE**: VS Code recommended with Vue/TypeScript extensions

## Build Process
- Development: Hot-reloaded Vite dev server + Tauri window
- Production: TypeScript checking, optimized Vite build, Rust release compilation
- Output: Platform-specific installers/packages

## Key Features
- Music library browsing (albums, artists, playlists, songs)
- Playback controls with queue management
- Audio processing (equalizer, crossfade, gapless)
- Lyrics display with synchronization
- Discord Rich Presence integration
- Fullscreen immersive player
- Smart fuzzy search
- System tray controls
- Multiple themes and accent colors

## Development Workflow
- Install dependencies with `bun install`
- Run development with `bun run tauri dev`
- Build production with `bun run tauri build`
- Code quality: ESLint + TypeScript strict mode
- Modern Vue 3 patterns, no legacy code