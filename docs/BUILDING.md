# Building Aurelia

This guide covers development and production builds for the monorepo.

## Prerequisites

### Required tools

1. Bun (v1+)
2. Rust stable toolchain
3. Node.js 20+

### Platform dependencies

#### Windows (Tauri)

- Visual Studio C++ Build Tools (Desktop development with C++)
- WebView2 runtime

#### macOS

```bash
xcode-select --install
```

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### Linux (Fedora)

```bash
sudo dnf install -y \
  webkit2gtk4.0-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

## Initial setup

```bash
git clone https://github.com/pupbrained/aurelia.git
cd aurelia
bun install
```

## Quick Start

Build all platforms (web + desktop + Android + iOS on macOS):

```bash
bun run build
```

Run all tests:

```bash
bun run test
```

## Development

| Command | Description |
|---------|-------------|
| `bun run dev:web` | Web frontend + backend |
| `bun run dev:desktop` | Desktop (Tauri) |
| `bun run dev:gpui` | Desktop (GPUI) |
| `bun run dev:android` | Android app |
| `bun run dev:ios` | iOS app (macOS only) |
| `bun run dev:all` | All platforms |

## Building

| Command | Description |
|---------|-------------|
| `bun run build:web` | Web (frontend + Rust backend) |
| `bun run build:desktop` | Desktop (Tauri) |
| `bun run build:gpui` | Desktop (GPUI) |
| `bun run build:android` | Android APK |
| `bun run build:ios` | iOS app |
| `bun run build` | All platforms |

### Build variants

| Command | Description |
|---------|-------------|
| `bun run build:web:release` | Full release build |
| `bun run build:web:strict` | Typecheck + build |
| `bun run build:desktop:release` | Full release build |
| `bun run build:desktop:strict` | Typecheck + build |
| `bun run build:gpui:release` | Full GPUI release build |
| `bun run build:android:release` | Release APK |
| `bun run build:ios:ipa` | iOS IPA (requires macOS) |

### Build options

Build commands powered by `scripts/aurelia.ts` support these flags:

- `--fast` - Use faster local-release profile (default for dev builds)
- `--skip-bindings` - Skip bindings generation
- `--force-bindings` - Force regenerate bindings

GPUI commands use `CARGO_TARGET_DIR` when it is set. On Windows with a `D:` drive, they default to `D:\aurelia-cargo-target` to avoid filling the repo drive with GPUI build artifacts.

## Testing

| Command | Description |
|---------|-------------|
| `bun run test` | All tests (JS + Rust + Android) |
| `bun run test:web` | Web frontend tests |
| `bun run test:desktop` | Desktop frontend tests |
| `bun run test:gpui` | GPUI desktop Rust tests |
| `bun run test:android` | Android unit tests |
| `bun run test:ios` | iOS tests (macOS only) |
| `bun run test:js` | All JavaScript tests |
| `bun run test:rust` | Rust tests |

## Type Checking

| Command | Description |
|---------|-------------|
| `bun run typecheck` | Typecheck all |
| `bun run typecheck:web` | Typecheck web frontend |
| `bun run typecheck:desktop` | Typecheck desktop frontend |

## Bindings

Generate Rust FFI bindings for TypeScript:

```bash
bun run bindings        # TypeScript bindings + web backend
bun run bindings:full` # + iOS Rust bindings
```

The `build:*` commands auto-cache binding generation and skip it when Rust sources are unchanged. Use `--force-bindings` to bypass.

## Code Quality

```bash
bun run lint           # Lint all
bun run lint:fix       # Auto-fix lint issues
```

## Troubleshooting

### Clean build artifacts

```bash
# Rust
cargo clean

# Tauri only
cd apps/desktop/tauri/src-tauri && cargo clean
```

### Linux AppImage permissions

```bash
chmod +x apps/desktop/tauri/src-tauri/target/release/bundle/appimage/*.AppImage
```

### macOS quarantine issues

```bash
xattr -cr apps/desktop/tauri/src-tauri/target/release/bundle/macos/*.app
```
