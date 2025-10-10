# Code Style and Conventions

## Vue Components
- Use `<script setup>` syntax with Composition API
- Enforce TypeScript everywhere with strict typing
- Define interfaces for props/emits when needed
- Component order: `<script setup>`, `<template>`, `<style>`
- Use PascalCase for component names
- Use `ref` and `reactive` for state management
- Favor composables (`src/composables`) for reusable logic

## TypeScript
- Strict mode enabled (`"strict": true` in tsconfig.json)
- Use descriptive names and maintain type safety
- Base URL: "." with paths: `"@/*": ["src/*"]` for clean imports

## Styling
- Tailwind CSS v4 with utility classes
- shadcn-vue inspired UI components in `src/components/ui`
- Catppuccin color palette configured
- Avoid ad-hoc CSS; use Tailwind utilities
- Ensure accessibility (ARIA, keyboard focus, reduced motion)

## Architecture
- State management with Pinia stores in `src/stores`
- Utilities in `src/lib`
- API calls via `@tauri-apps/api` with try/catch error handling
- Rust backend in `src-tauri/src` with Tauri security best practices

## ESLint Rules
- Uses @stylistic/eslint-plugin for consistent formatting
- Perfectionist plugin for natural sorting
- Arrow function preferences
- 2-space indentation
- Comma dangle on multiline
- Single quotes preferred

## File Organization
- Components: `src/components/` (layout, player, settings, shared, ui)
- Views: `src/views/` (page-level components)
- Composables: `src/composables/` (reusable logic)
- Stores: `src/stores/` (Pinia state)
- Lib: `src/lib/` (utilities, API, etc.)
- Router: `src/router/`

## Tauri Integration
- Call Rust commands via `@tauri-apps/api`
- Validate payloads and handle errors user-facing
- No untrusted command execution
- Mirror frontend models in Rust when needed

## Git and Development
- Keep repo healthy: modern code, no deprecated patterns
- Follow existing patterns and conventions
- Test changes before committing
- Use Bun exclusively for package management