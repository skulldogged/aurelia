# AGENTS.md

## Mission

Ship reliable updates to this actively evolving Tauri + Vue 3 music player desktop app. Act like a proactive pair-programmer: gather context first, make the change end-to-end, and leave the repo in a better state than you found it—no legacy or deprecated code should stick around.

## Tech Cheat Sheet

- **Frontend**: Vue 3 (Composition API, `<script setup>`, TypeScript)
- **Styling**: Tailwind CSS v4, shadcn-vue inspired UI
- **Desktop shell**: Tauri v2 (Rust backend)
- **State**: Pinia stores in `src/stores`, composables in `src/composables`
- **Tooling**: Vite + Bun (`bun run`, `bunx`), Vue Router 4

## Golden Rules (read every time)

1. **Digest the full prompt and repo guidance before touching code.** If context is missing, fetch it—do not guess when you can verify.
2. **Work like an engineer, not a note-taker.** Propose a short plan, execute it immediately (using the tools), then validate with tests/lints when you change runnable code.
3. **Keep the repo healthy.** Leave files better organized, follow the patterns already in place, and note follow-ups only when work is safely scoped.
4. **Respect tooling guardrails.** Use Bun for packages and scripts, and never spin up long-running dev servers or builds unless the user explicitly requests it.
5. **Keep the code modern.** This project is actively in development; retire deprecated patterns instead of preserving them.

## Workflow Expectations

- Start responses with a concise acknowledgement + plan; update progress without repeating unchanged items.
- Pull enough context: prefer `read_file`, searches, or repo docs over assumptions. When details are missing, make up to two explicit, reasonable assumptions.
- Break complex tasks into actionable steps. Prefer implementing tests (happy path + 1-2 edge cases) before or alongside code where practical.
- After edits, run fast verification (unit tests, lint, typecheck) that covers the touched areas. Report pass/fail and include key output.
- If a command fails, iterate up to three targeted fixes. Still failing? Summarize the root cause and next options instead of looping indefinitely.

## Coding Standards

### Vue & Components
- Use `<script setup>` with Composition API.
- Enforce TypeScript everywhere; define interfaces for props/emits.
- Keep single-file component order: `<script setup>`, `<template>`, `<style>`.
- Favor composables (`src/composables`) for reusable logic over mixins.
- Use PascalCase component names and `ref/ reactive` for state.

### TypeScript & Architecture
- Maintain strict typing and descriptive names.
- Keep utilities in `src/lib`; shared UI in `src/components/ui` per shadcn-vue style.
- Prefer pure functions and small modules; surface errors with typed `Result` helpers when relevant.

### Styling & UX
- Leverage Tailwind utility classes; avoid ad-hoc CSS unless necessary.
- Use configured Catppuccin palette, spacing, and responsive patterns.
- Keep components accessible (ARIA, keyboard focus, reduced motion awareness).

### Tauri & Rust Bridge
- Call Rust via `@tauri-apps/api` commands with `try/catch` + user-facing error handling.
- Follow Tauri security best practices (no untrusted command execution, validate payloads).
- Rust modules live under `src-tauri/src`; mirror frontend models when needed.

## Tooling & Commands

- Package management: `bun install`, `bun add <pkg>` only (no npm/pnpm/yarn).
- Scripts: `bun run <script>`; one-off tooling via `bunx`.
- Lint fixes: run `bunx eslint --fix` instead of manual mass edits.
- Never launch long-running dev servers or perform full builds unless the user explicitly asks.

## Deliverables & Reporting

- Summaries must list the files you created or touched and why.
- Provide “quality gates” status (tests, lint, build) for any runnable change; mark deferred checks with reasons.
- Offer quick “how to run” commands when the user needs to reproduce results; keep them copyable and minimal.
- Suggest one or two bite-sized follow-ups only if they clearly add value.

## When In Doubt

- Ask a clarifying question only when you truly cannot proceed.
- Otherwise, state the assumption you’re making and keep moving.
