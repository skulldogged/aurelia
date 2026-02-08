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

## Development builds (repo root)

### Web

```bash
bun run dev:web
```

Runs:
- Axum backend (`apps/web/backend`)
- Vite frontend (`apps/web/frontend`)

### Desktop (Tauri)

```bash
bun run dev:desktop:tauri
```

Runs the Tauri desktop app from:
- frontend: `apps/desktop/tauri`
- Rust backend: `apps/desktop/tauri/src-tauri`

## Build commands (repo root)

### Web

```bash
bun run build:web
bun run build:web -- --skip-bindings
bun run build:web -- --force-bindings
bun run build:web:strict
bun run build:web:release
```

Outputs:
- Frontend bundle: `apps/web/frontend/dist`
- Backend binary: Cargo `local-release` output for `apps/web/backend` (use `build:web:release` for full release profile)

### Desktop (Tauri)

```bash
bun run build:desktop:tauri
bun run build:desktop:strict
bun run build:desktop:release
```

Tauri bundles are under:
- `apps/desktop/tauri/src-tauri/target/release/bundle`

## Code quality and checks

### Linting

```bash
bunx eslint .
bunx eslint --fix .
```

### Type checking and builds

```bash
bun run typecheck
bun run build
bun run build:web:strict
bun run build:desktop:strict
```

CI runs `bun run typecheck` via `.github/workflows/typecheck.yml`.

### Rust checks

```bash
cargo fmt --check
cargo clippy --workspace
cargo test --workspace
```

## Generated code and bindings

```bash
bun run bindings:generate
bun run bindings:generate:full
bun run bindings:verify
bun run bindings:verify:full
```

`bun run build:*` commands auto-cache binding generation and skip it when relevant Rust sources/manifests are unchanged. Use `--force-bindings` to bypass the cache.

## Troubleshooting

### Dependency or lockfile drift

Use root lockfile workflow:

```bash
bun install
bun run verify:structure
```

### Clean Rust build artifacts

```bash
cargo clean
```

For Tauri-only cleaning:

```bash
cd apps/desktop/tauri/src-tauri
cargo clean
```

### Linux AppImage permissions

```bash
chmod +x apps/desktop/tauri/src-tauri/target/release/bundle/appimage/*.AppImage
```

### macOS quarantine issues

```bash
xattr -cr apps/desktop/tauri/src-tauri/target/release/bundle/macos/*.app
```
