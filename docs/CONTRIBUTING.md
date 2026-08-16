# Contributing to Aurelia

## Getting started

1. Fork and clone the repository.
2. Install prerequisites from `BUILDING.md`.
3. Create a branch for your work.

## Development workflow

Run from repository root unless noted.

```bash
bun install
```

Common dev commands:

```bash
bun run dev:web
bun run dev:desktop
```

## Repository structure

- Shared UI/package: `apps/shared`
- Web app:
  - frontend: `apps/web/frontend`
  - backend: `apps/web/backend`
- Desktop app:
  - Electron shell: `apps/desktop/electron`
  - Local Rust backend: `apps/web/backend`
- Mobile:
  - Android: `apps/mobile/android`
  - iOS: `apps/mobile/ios`
- Core Rust crates: `crates`

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

### Swift (iOS)

## Tests

Run relevant suites before opening a PR:

```bash
bun run test:js
bun run test:rust
bun run test:desktop
bun run test:web
bun run test:android
bun run test:ios
```

See `TESTING.md` for details.

## Pull request checklist

- Linting passes.
- Tests relevant to the change pass.
- Docs and paths are updated if structure or commands changed.
- No generated cache artifacts are committed (`*.tsbuildinfo`, nested `apps/**/bun.lock`).

## Questions

Open an issue or discussion for large changes before implementation.
