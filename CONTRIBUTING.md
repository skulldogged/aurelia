# Contributing to Aurelia

## Getting started

1. Fork and clone the repository.
2. Install prerequisites from `/Users/marshall/Projects/aurelia/BUILDING.md`.
3. Create a branch for your work.

## Development workflow

Run from repository root unless noted.

```bash
bun install
bun run verify:structure
```

Common dev commands:

```bash
bun run dev:web
bun run dev:desktop:tauri
bun run dev:desktop:macos
```

## Repository structure

- Shared UI/package: `/Users/marshall/Projects/aurelia/apps/shared`
- Web app:
  - frontend: `/Users/marshall/Projects/aurelia/apps/web/frontend`
  - backend: `/Users/marshall/Projects/aurelia/apps/web/backend`
- Desktop app:
  - Tauri frontend: `/Users/marshall/Projects/aurelia/apps/desktop/tauri`
  - Tauri Rust backend: `/Users/marshall/Projects/aurelia/apps/desktop/tauri/src-tauri`
  - Native macOS SwiftUI app: `/Users/marshall/Projects/aurelia/apps/desktop/macos/AureliaMac`
- Mobile:
  - Android: `/Users/marshall/Projects/aurelia/apps/mobile/android`
  - iOS: `/Users/marshall/Projects/aurelia/apps/mobile/ios`
- Core Rust crates: `/Users/marshall/Projects/aurelia/crates`

## Code style

### TypeScript / Vue

- Use TypeScript for new code.
- Prefer Vue 3 Composition API with `<script setup>`.
- Keep strict typing; avoid `any` unless justified.

### Rust

- Follow standard Rust style.
- Run:

```bash
cargo fmt --check
cargo clippy --workspace
```

### Swift (native macOS/iOS)

- Keep native macOS code under `apps/desktop/macos/AureliaMac`.
- Do not reintroduce a parallel SwiftPM `Sources/` app tree.

## Tests

Run relevant suites before opening a PR:

```bash
bun run test:js
bun run test:rust
bun run test:desktop
bun run test:desktop:macos
bun run test:web
bun run test:android
bun run test:ios
```

See `/Users/marshall/Projects/aurelia/TESTING.md` for details.

## Pull request checklist

- Linting passes.
- Tests relevant to the change pass.
- Docs and paths are updated if structure or commands changed.
- No generated cache artifacts are committed (`*.tsbuildinfo`, nested `apps/**/bun.lock`).

## Questions

Open an issue or discussion for large changes before implementation.
