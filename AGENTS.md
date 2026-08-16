# AGENTS.md

## Purpose

Aurelia is a multi-platform music client for Jellyfin. It ships as:

- a Vue web app backed by Axum,
- an Electron desktop app backed by the same local Rust server,
- native Android and iOS apps,
- shared Rust crates and generated cross-language bindings.

Make changes that preserve these platform boundaries and avoid silent drift between clients.

## Sources of truth

Use the current repository, not prose that may have aged:

1. The user's request and acceptance criteria.
2. The nearest `AGENTS.md` for the files being changed.
3. Executable configuration: `package.json`, `Cargo.toml`, Gradle files, Xcode project files, and CI workflows.
4. Existing implementation and tests next to the affected code.
5. Documentation only after checking it against the items above.

If a documented command disagrees with a manifest or script, follow the executable configuration and mention the discrepancy.

## Repository map

- `apps/shared/` — Vue components, pages, stores, composables, audio abstractions, and API integration shared by web and desktop.
- `apps/web/frontend/` — thin Vite web shell around `@aurelia/shared`.
- `apps/web/backend/` — Axum backend used by web and desktop.
- `apps/desktop/electron/` — Electron main/preload code and thin Vue desktop shell.
- `apps/mobile/android/` — native Kotlin/Jetpack Compose app.
- `apps/mobile/ios/` — native Swift app and Swift package.
- `crates/aurelia-core/` — domain logic, persistence, services, UniFFI exports, and desktop audio.
- `crates/aurelia-api/` — API abstraction and web-facing implementation support.
- `crates/aurelia-api-macros/` — procedural macros for API generation.
- `crates/aurelia-lyrics/` — lyrics functionality.
- `crates/aurelia-sidecar-daemon/` — sidecar daemon.
- `crates/uniffi-bindgen/` — TypeScript/Kotlin binding and client generation.
- `scripts/aurelia.ts` — top-level build, development, test, typecheck, and binding orchestration.

## Architecture boundaries

- Put web/desktop UI and business behavior in `apps/shared/` unless it truly depends on a host platform.
- Keep web and Electron entry points thin. Import shared functionality through `@aurelia/shared` rather than duplicating it.
- Electron playback uses the local Rust backend over HTTP/WebSocket. Do not introduce a second Electron-only Web Audio path.
- Keep Electron-only concerns such as windows, preload bridges, tray behavior, and OS integration in `apps/desktop/electron/`.
- Android and iOS are native clients. Share domain behavior through Rust/UniFFI where appropriate; do not assume Vue code is shared with mobile.
- Put reusable domain and service logic in Rust rather than reproducing it independently in multiple clients when the binding boundary supports it.

## Working method

1. Inspect `git status` and the relevant manifests, neighboring source, and tests before editing.
2. State a short plan for non-trivial work.
3. Make the smallest coherent end-to-end change. Avoid unrelated cleanup.
4. Add or update tests with behavior changes when practical.
5. Run the narrowest relevant checks first, then broader checks when justified.
6. Review the final diff for accidental generated output, formatting churn, secrets, and unrelated edits.
7. Report changed files, validation performed, and any remaining risks.

Preserve user changes already present in the worktree. Never discard, reset, or rewrite unrelated work without explicit permission.

## Package and tooling rules

- Use Bun for JavaScript/TypeScript dependency management and scripts. Do not use npm, pnpm, or Yarn.
- Add dependencies at the narrowest workspace that owns them.
- Do not run repository-wide auto-formatting or auto-fix commands unless requested.
- Do not edit lockfiles manually.
- Do not claim a command passed unless it was actually run successfully.
- Do not start long-lived development servers unless the task requires runtime verification.

## Generated code

Do not hand-edit generated bindings or clients, including:

- `apps/shared/src/generated/`
- `apps/shared/src/api/apiClient.ts`
- `apps/shared/src/lib/api/types.ts`
- `apps/mobile/android/app/src/main/java/uniffi/`
- generated Swift UniFFI sources

Change the Rust/API source or generator, then regenerate with the current binding command in `package.json`. Inspect generated diffs before keeping them. Do not regenerate bindings for unrelated changes.

## Frontend conventions

For `apps/shared/`, web, and Electron renderer code:

- Use Vue 3 Composition API with `<script setup lang="ts">`.
- Keep TypeScript types explicit at public boundaries; avoid `any` unless required by generated or external interfaces.
- Use Pinia for shared application state and composables for reusable behavior.
- Follow the established Effect patterns in `apps/shared/src/effect/` when working in that subsystem.
- Prefer existing shared components and Tailwind utilities over new one-off styling systems.
- Preserve accessibility: semantic controls, keyboard behavior, labels, focus handling, and reduced-motion behavior.
- Keep platform checks at adapter boundaries instead of scattering them through shared components.
- Co-locate focused tests or place cross-component tests in the package's existing `tests/` directory.

## Rust conventions

- Keep public and FFI-facing errors structured; use the existing application error types at boundaries.
- Use `tracing` rather than ad hoc printing for runtime diagnostics.
- Preserve feature gates for desktop-only audio, media controls, and integrations.
- UniFFI exports must use FFI-compatible owned types and the repository's existing async runtime annotations.
- Keep HTTP/API transport concerns separate from reusable domain logic.
- Add unit tests near modules and backend route tests under `apps/web/backend/tests/` as appropriate.

## Mobile conventions

### Android

- Use Kotlin, Jetpack Compose, Material 3, ViewModels, StateFlow, and coroutines following neighboring code.
- Keep playback behavior behind `PlayerController`/Media3 abstractions.
- Put credentials and preferences through the existing storage abstractions.
- Include `contentDescription` and other accessibility semantics where applicable.

### iOS

- Follow the existing SwiftUI and `AureliaCore` package structure.
- Keep platform UI in the app target and reusable Rust-facing behavior in the core/binding layer.
- iOS builds and tests require macOS; state when they could not be run.

## Validation

Choose checks based on the touched area. Commands below are defined by the current root `package.json`:

| Scope | Primary checks |
| --- | --- |
| Shared/web UI | `bun run typecheck:web`, `bun run test:web` |
| Shared/desktop UI or Electron | `bun run typecheck:desktop`, `bun run test:desktop` |
| All JS/TS tests | `bun run test:js` |
| Rust workspace | `bun run test:rust` |
| Android | `bun run test:android` |
| iOS on macOS | `bun run test:ios` |
| All TS typechecks | `bun run typecheck` |
| Lint when relevant | `bun run lint` |
| Broad suite | `bun run test` |

Important current behavior:

- `bun run test:web` and `bun run test:desktop` each run their typecheck plus the shared Vitest suite.
- `bun run test` runs web and desktop typechecks, JavaScript tests, Rust workspace tests, and iOS tests on macOS.
- Android tests are separate; run `bun run test:android` when Android is affected.
- Prefer a targeted `cargo test -p <crate>` or focused Vitest invocation while iterating, then run the appropriate command above before finishing.
- Build only affected deliverables unless the change crosses platform or binding boundaries.

## Completion standard

A change is complete when:

- behavior matches the request,
- architecture boundaries remain intact,
- relevant tests or checks pass,
- generated artifacts are current only when their sources changed,
- no unrelated files were modified,
- the final response lists changed paths, checks run, and unresolved platform limitations or risks.
