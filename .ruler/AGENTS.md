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