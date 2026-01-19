# Aurelia Rules

Multi-platform music player (Desktop: Tauri/Vue, Mobile: Android/Kotlin, shared Rust core).

## 🚨 Critical Mandates

- **Tooling**:
  - **Desktop**: `bun` ONLY. No `npm`/`yarn`.
  - **Android**: `gradle`.
- **Formatting**: **NO** auto-formatting (ESLint/Prettier/ktlint) unless requested.
- **Context**: Read before writing.

## 📚 Documentation

- [Architecture & Tech Stack](./rules/architecture.md)
- [Workflow & Process](./rules/workflow.md)
- [Desktop Guidelines (Vue/Tauri)](./rules/desktop.md)
- [Android Guidelines](./rules/android.md)

## btca

When you need up-to-date information about technologies used in this project, use btca to query source repositories directly.

**Available resources**: vue, vite, tailwind, tauri, pinia, vueRouter, vueUse, tanstackVirtual, typescript, tokio, rodio, media3, coil, okhttp, kotlinCoroutines

### Usage

```bash
btca ask -r <resource> -q "<question>"
```

Use multiple `-r` flags to query multiple resources at once:

```bash
btca ask -r vue -r tauri -q "How do I integrate Vue with Tauri v2?"
```
