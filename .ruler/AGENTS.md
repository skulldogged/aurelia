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

1. **Digest the full prompt and repo guidance before touching code.** If context is missing, fetch it to verify before proceeding.
2. **Work like an engineer, not a note-taker.** Propose a short plan, execute it immediately (using the tools).
3. **Keep the repo healthy.** Leave files better organized, follow the patterns already in place, and note follow-ups only when work is safely scoped.
4. **NEVER run ESLint or other code formatting/linting tools automatically.** Only run them when explicitly requested by the user. Style/formatting issues that don't affect functionality should be ignored.
5. **Respect tooling guardrails.** Use Bun for package management (`bun install`, `bun add <pkg>` only—no npm/pnpm/yarn). Run scripts with `bun run <script>` or one-off tooling with `bunx`. Spin up long-running dev servers or perform full builds ONLY when the user explicitly requests it.
6. **Keep the code modern.** This project is actively in development; retire deprecated patterns instead of preserving them.
7. **Communicate clearly and concisely.** Use professional language without emojis in code, comments, documentation, and commit messages. In responses, ALWAYS sacrifice grammar for conciseness, even with incomplete sentences or informal grammar, as long as meaning is clear.

## Workflow Expectations

- Start responses with a concise acknowledgement + plan; update progress without repeating unchanged items.
- Pull enough context: prefer `read_file`, searches, or repo docs over assumptions. When details are missing, make up to two explicit, reasonable assumptions.
- Break complex tasks into actionable steps. Prefer implementing tests (happy path + 1-2 edge cases) before or alongside code where practical.
- If a command fails, iterate up to three targeted fixes. Still failing? Summarize the root cause and next options instead of looping indefinitely.
- Never run ESLint, Prettier, or formatting tools automatically - only when explicitly asked.

## Coding Standards

### Vue & Components

- Use `<script setup>` with Composition API.
- Enforce TypeScript everywhere; define interfaces for props/emits.
- Keep single-file component order: `<script setup>`, `<template>`, `<style>`.
- Prefer Tailwind utility classes over `<style>` elements; include `<style>` only for scoped overrides when necessary.
- Favor composables (`src/composables`) for reusable logic over mixins.
- Use PascalCase component names in both code and templates (enforced by ESLint) and `ref/ reactive` for state.

### TypeScript & Architecture

- Maintain strict typing and descriptive names.
- Keep utilities in `src/lib`; shared UI in `src/components/ui` per shadcn-vue style.
- Prefer pure functions and small modules; surface errors with typed `Result` helpers when relevant.

### Styling & UX

- Leverage Tailwind utility classes and the configured theme system with color tokens; avoid ad-hoc CSS unless necessary.
- Keep components accessible (ARIA, keyboard focus, reduced motion awareness).

### Tauri & Rust Bridge

- Call Rust via `@tauri-apps/api` commands with `try/catch` + user-facing error handling.
- Follow Tauri security best practices: validate payloads to ensure secure command execution.
- Rust modules live under `src-tauri/src`; mirror frontend models when needed.

## Deliverables & Reporting

- Summaries must list the files you created or touched and why.
- Provide “quality gates” status (tests, lint, build) for any runnable change; mark deferred checks with reasons.
- Offer quick “how to run” commands when the user needs to reproduce results; keep them copyable and minimal.
- Suggest one or two bite-sized follow-ups only if they clearly add value.

## When In Doubt

- Ask a clarifying question only when you truly cannot proceed.
- Otherwise, state the assumption you’re making and keep moving.
