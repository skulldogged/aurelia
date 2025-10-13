# Aurelia

A modern desktop music player for Jellyfin, built with Tauri and Vue 3.

## Features

- **Full music library management** — Browse albums, artists, playlists, and songs
- **Beautiful UI** — Modern interface with multiple themes and customizable accent colors
- **Playback controls** — Queue management, shuffle, repeat, and more
- **Audio features** — Equalizer, gapless playback, and crossfade
- **Lyrics support** — View synchronized lyrics while listening
- **Discord Rich Presence** — Show what you're listening to on Discord
- **Fullscreen player** — Immersive listening experience
- **Smart search** — Quickly find music with fuzzy search
- **Native desktop app** — Built with Tauri for performance and small binary size
- **System tray integration** — Control playback from your taskbar

## Tech Stack

- **Frontend**: Vue 3 (Composition API, `<script setup>`, TypeScript)
- **UI Components**: shadcn-vue
- **Styling**: Tailwind CSS v4
- **Desktop**: Tauri v2 (Rust backend)
- **State Management**: Pinia
- **Build Tool**: Vite
- **Package Manager**: Bun

## Quick Start

### Prerequisites

- [Bun](https://bun.sh) v1.0+
- [Rust](https://rustup.rs/) (stable toolchain)
- Platform-specific dependencies (see [BUILDING.md](./BUILDING.md))

### Installation

```bash
# Clone the repository
git clone https://github.com/pupbrained/aurelia.git
cd jellyfin-music-player

# Install dependencies
bun install

# Run in development mode
bun run tauri dev
```

For detailed build instructions, see [BUILDING.md](./BUILDING.md).

## Development

### Project Structure

```
src/
├── components/      # Vue components
│   ├── layout/     # App layout components
│   ├── player/     # Music player components
│   ├── settings/   # Settings panels
│   ├── shared/     # Shared/reusable components
│   └── ui/         # Base UI components (shadcn-vue style)
├── composables/    # Vue composables for shared logic
├── stores/         # Pinia stores
├── views/          # Route views
├── lib/            # Utilities and helpers
└── router/         # Vue Router configuration

src-tauri/
├── src/            # Rust source code
│   ├── handlers/   # Tauri command handlers
│   ├── services/   # Backend services
│   └── models/     # Data models
└── tauri.conf.json # Tauri configuration
```

### Scripts

```bash
bun run dev          # Vite dev server (frontend only)
bun run build        # Production build
bun run preview      # Preview production build
bun run tauri dev    # Tauri development mode
bun run tauri build  # Build desktop app
```

### Code Style

This project uses:
- **ESLint** for linting
- **TypeScript** for type safety
- **Prettier** (via ESLint Stylistic) for formatting

Run linting:
```bash
bunx eslint .           # Check for issues
bunx eslint --fix .     # Auto-fix issues
```

## Discord Rich Presence

Discord Rich Presence is enabled by default. To use a custom Discord application for development:

**Windows:**
```powershell
$env:VITE_DISCORD_APP_ID = "your-discord-app-id"
```

**macOS/Linux:**
```bash
export VITE_DISCORD_APP_ID="your-discord-app-id"
```

Create your application at the [Discord Developer Portal](https://discord.com/developers/applications).

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on code style, development workflow, and how to submit pull requests.

## Recommended IDE Setup

- **VS Code** with extensions:
    - [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
    - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
    - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
    - [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)

## License

See [LICENSE](./LICENSE) for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI inspired by [shadcn-vue](https://www.shadcn-vue.com/)
