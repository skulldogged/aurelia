# Building Aurelia

This guide covers building the Aurelia from source for development and production.

## Prerequisites

### Required Tools

1. **Bun** (v1.0+)
    - Install from [bun.sh](https://bun.sh)
    - Used for package management and running scripts
    - **Do not use npm, pnpm, or yarn**
      - this project exclusively uses Bun. All other package managers are UNTESTED.

2. **Rust** (stable toolchain)
    - Install via [rustup](https://rustup.rs/)
    - Tauri requires Rust 1.70 or later
    - Verify: `rustc --version`

3. **Node.js** (v20+)
    - Required by Bun and Vite
    - Download from [nodejs.org](https://nodejs.org/)

### Platform-Specific Dependencies

#### Windows
- **Visual Studio C++ Build Tools**
  - Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
  - Select "Desktop development with C++" workload
  - Includes MSVC and Windows SDK

- **WebView2**
  - Pre-installed on Windows 10/11 (1809+)
  - Tauri uses it for rendering

#### macOS
```bash
xcode-select --install
```

Required for Rust compilation and Tauri builds.

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

#### Linux (Arch)
```bash
sudo pacman -Syu
sudo pacman -S --needed \
  webkit2gtk \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  libvips
```

## Initial Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/pupbrained/aurelia.git
   cd jellyfin-music-player
   ```

2. **Install dependencies**
   ```bash
   bun install
   ```

3. **(Optional) Custom Discord App ID**
   
   Discord Rich Presence works out of the box with the official app ID. If you want to use your own Discord application for testing:
   
   **Windows (PowerShell):**
   ```powershell
   $env:VITE_DISCORD_APP_ID = "your-discord-app-id"
   ```
   
   **macOS/Linux:**
   ```bash
   export VITE_DISCORD_APP_ID="your-discord-app-id"
   ```
   
   Create your own application at the [Discord Developer Portal](https://discord.com/developers/applications).

## Development Build

### Run in Development Mode

```bash
cargo tauri dev
```

This starts:
- Vite dev server with hot module replacement (HMR)
- Tauri development window
- Rust backend with hot reload

The app will open automatically. Frontend changes reload instantly; Rust changes trigger a rebuild.

## Production Build

### Build for Your Platform

```bash
cargo tauri build
```

This will:
1. Run TypeScript type checking (`vue-tsc --noEmit`)
2. Build optimized frontend bundle with Vite
3. Compile Rust backend in release mode
4. Package the application

**Build artifacts** are located in:
- **Windows:** `src-tauri/target/release/bundle/msi/` or `src-tauri/target/release/bundle/nsis/`
- **macOS:** `src-tauri/target/release/bundle/dmg/` and `.app`
- **Linux:** `src-tauri/target/release/bundle/deb/`, `appimage/`, or `rpm/`

### Build Configuration

Tauri build settings are in `src-tauri/tauri.conf.json`:
- App name, version, and identifiers
- Bundle targets (MSI, NSIS, DMG, AppImage, etc.)
- Window configuration
- Security and capability settings

## Code Quality Checks

### Linting

Run ESLint to check code style:

```bash
bunx eslint .
```

Auto-fix issues:

```bash
bunx eslint --fix .
```

### Type Checking

Verify TypeScript types:

```bash
bun run build
```

This runs `vue-tsc --noEmit` before building.

### Rust Formatting & Linting

```bash
cd src-tauri
cargo fmt --check  # Check formatting
cargo fmt          # Apply formatting
cargo clippy       # Run linter
```

## Troubleshooting

### "Command not found: tauri"

Ensure `@tauri-apps/cli` is installed:

```bash
bun install
```

Run via the script:

```bash
cargo tauri dev
```

### Build Fails with Rust Errors

1. Update Rust: `rustup update stable`
2. Clean build cache: `cd src-tauri && cargo clean`
3. Rebuild: `cargo tauri build`

### Frontend Build Errors

1. Remove `node_modules`: `rm -rf node_modules`
2. Clear Bun cache: `bun pm cache rm`
3. Reinstall: `bun install`

### WebView2 Missing (Windows)

Download and install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).

### Permission Errors (macOS)

If the built `.app` won't open due to security:

```bash
xattr -cr src-tauri/target/release/bundle/macos/Jellyfin\ Music\ Player.app
```

### Linux AppImage Won't Run

Make it executable:

```bash
chmod +x src-tauri/target/release/bundle/appimage/*.AppImage
```

## Advanced Build Options

### Custom Build Profiles

Create different Cargo profiles in `src-tauri/Cargo.toml`:

```toml
[profile.release-small]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
```

Build with:

```bash
cargo build --profile release-small
```

### Cross-Compilation

Tauri supports cross-compilation with additional setup. See the [Tauri cross-compilation guide](https://tauri.app/v2/guides/building/cross-platform/).

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `VITE_DISCORD_APP_ID` | Override Discord app ID (optional) | Official app ID |
| `TAURI_PRIVATE_KEY` | Code signing (production) | Not set |
| `TAURI_KEY_PASSWORD` | Key password (production) | Not set |

## CI/CD Integration

For automated builds, see the Tauri [GitHub Actions guide](https://tauri.app/v2/guides/distribution/github-actions/).

Example workflow structure:
1. Install Bun
2. Install Rust via `actions-rs/toolchain`
3. Install platform dependencies
4. Run `bun install`
5. Run `bun run tauri build`
6. Upload artifacts

## Additional Resources

- [Tauri v2 Documentation](https://tauri.app/v2/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Vite Documentation](https://vitejs.dev/)
- [Bun Documentation](https://bun.sh/docs)
- [Project README](./README.md)
- [Agent Instructions](./AGENTS.md)
