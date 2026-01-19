# Desktop Guidelines (Vue + Tauri)

## Coding Standards

### Vue & TypeScript
- **Structure**: `<script setup>` (Composition API) -> `<template>` -> `<style>`.
- **State**: Use `ref`/`reactive`. PascalCase components.
- **Styling**: Prefer Tailwind utilities (v4). Use `<style scoped>` only when necessary.
- **Logic**: Extract reusable logic to Composables (`src/composables`).
- **Locations**:
  - Utilities: `src/lib`
  - Shared UI: `src/components/ui`

### Tauri & Rust
- **Communication**: Call Rust via `@tauri-apps/api`. Always handle errors (try/catch) with user-facing messages.
- **Security**: Validate payloads. Follow Tauri security practices.
- **Structure**: Rust modules in `src-tauri/src`. Mirror frontend models in Rust where needed.

### UX/Accessibility
- Maintain ARIA accessibility.
- Respect reduced motion preferences.
