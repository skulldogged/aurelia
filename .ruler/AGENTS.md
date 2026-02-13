# AGENTS.md

## Mission

Ship reliable updates to this actively evolving music player supporting Desktop (Tauri), Web, and Mobile (Android/iOS). Act like a proactive pair-programmer: gather context first, make the change end-to-end, and leave the repo in a better state than you found it.

## Quick Reference

- **Apps**: `apps/desktop/` (Tauri), `apps/web/` (Vite + Axum), `apps/mobile/` (Android/iOS)
- **Shared Code**: `apps/shared/` - Vue code shared between desktop and web frontends (stores, composables, UI components)
- **Rust Core**: `crates/aurelia-core/` (shared logic), `crates/aurelia-api/` (API abstraction)
- **Run Commands**: `bun run dev:web`, `bun run build`, `bun run test`
- **Testing Guide**: `TESTING.md` (authoritative commands for all apps)

## Golden Rules

1. **Read first**: Check existing patterns before writing new code.
2. **Modern patterns**: Vue 3 Composition API (`<script setup>`), Tailwind v4, Pinia with `ref/reactive`.
3. **No auto-formatting**: Never run ESLint/Prettier unless explicitly asked.
4. **Bun only**: Use `bun install/add/run/x` - never npm/pnpm/yarn.
5. **Be concise**: Short responses, no emojis in code, sacrifice grammar for clarity.

## Where to Look

| Topic | See |
|-------|-----|
| Monorepo structure, all apps | `.ruler/rules/architecture.md` |
| Vue/Tauri coding standards | `.ruler/rules/desktop.md` |
| Mobile (Android/iOS) standards | `.ruler/rules/mobile.md` |
| Rust/uniffi patterns | `.ruler/rules/rust.md` |
| Workflow, testing, git | `.ruler/rules/workflow.md` |
| BTCA tool for code queries | `.ruler/rules/btca.md` |

## When In Doubt

**Always ask for clarification** if anything is unclear. Do not make assumptions that could lead to incorrect implementations. Better to ask than to make the codebase worse.
