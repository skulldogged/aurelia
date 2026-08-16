# Building Aurelia

This guide covers development and production builds for the monorepo.

## Prerequisites

### Required tools

1. Bun (v1+)
2. Rust stable toolchain
3. Node.js 20+

### Platform dependencies

#### Windows (Electron)

- Visual Studio C++ Build Tools (for the Rust backend)
- Electron is installed via Bun/Nix

#### macOS

```bash
xcode-select --install
```

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  pkg-config \
  libssl-dev \
  libasound2-dev \
  libgtk-3-dev
```

#### Linux (Fedora)

```bash
sudo dnf install -y \
  gcc \
  gcc-c++ \
  make \
  pkgconf-pkg-config \
  openssl-devel \
  alsa-lib-devel \
  gtk3-devel \
  curl \
  wget \
  file
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
| `bun run dev:desktop` | Desktop (Electron + local Rust backend) |
| `bun run dev:android` | Android app |
| `bun run dev:ios` | iOS app (macOS only) |
| `bun run dev:all` | All platforms |

## Building

| Command | Description |
|---------|-------------|
| `bun run build:web` | Web (frontend + Rust backend) |
| `bun run build:desktop` | Desktop (Electron + local Rust backend) |
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
| `bun run build:android:release` | Release APK |
| `bun run build:ios:ipa` | iOS IPA (requires macOS) |

### Build options

Build commands powered by `scripts/aurelia.ts` support these flags:

- `--fast` - Use faster local-release profile (default for dev builds)
- `--skip-bindings` - Skip bindings generation
- `--force-bindings` - Force regenerate bindings

## Testing

| Command | Description |
|---------|-------------|
| `bun run test` | All tests (JS + Rust + Android) |
| `bun run test:web` | Web frontend tests |
| `bun run test:desktop` | Desktop frontend tests |
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
bun run bindings:full   # + iOS Rust bindings
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
```
