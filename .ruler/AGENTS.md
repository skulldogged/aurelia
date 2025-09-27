# AGENTS.md

## Project Overview

This is a Tauri application built with Vue 3, TypeScript, and Tailwind CSS. The project combines a Rust backend with a modern web frontend for cross-platform desktop application development.

## Tech Stack

- **Frontend**: Vue 3 with Composition API, TypeScript
- **Styling**: Tailwind CSS v4, Custom UI components (shadcn-vue)
- **Build Tool**: Vite
- **Desktop Framework**: Tauri v2
- **Package Manager**: Bun
- **State Management**: Pinia stores with Vue composables (stores live in `src/stores`, shared logic via `src/composables`)
- **Routing**: Vue Router 4

## Code Style Guidelines

### Vue Components

- Use Composition API with `<script setup>` syntax
- Prefer composables over mixins for reusable logic
- Use TypeScript for all components and composables
- Follow single-file component structure: `<script setup>`, `<template>`, `<style>`
- Use PascalCase for component names
- Prefer reactive variables using `ref()` and `reactive()`

### TypeScript

- Use strict type checking
- Define proper interfaces for props and emits
- Prefer type annotations for better IDE support
- Use generic types where appropriate

### File Structure

- Components go in `src/components/`
- Views go in `src/views/`
- Composables go in `src/composables/`
- Utilities go in `src/lib/`
- UI components follow shadcn-vue pattern in `src/components/ui/`

### Styling

- Use Tailwind CSS classes
- Prefer utility classes over custom CSS
- Use the configured theme colors and spacing
- Follow responsive design principles
- Leverage Catppuccin color scheme when appropriate

### Tauri Integration

- Use `@tauri-apps/api` for Rust backend communication
- Handle async operations properly with try-catch blocks
- Implement proper error handling for Tauri commands
- Follow Tauri security best practices

### General Principles

- AVOID overuse of comments. Follow code self-documentation.
- Write clean, readable, and maintainable code
- Use descriptive variable and function names
- Implement proper error handling
- Follow Vue 3 best practices and conventions
- Ensure accessibility (a11y) compliance
- Optimize for performance where necessary

## IMPORTANT

- DO NOT run dev servers or builds unless explicitly asked or instructed to do so.
- Resolve linting errors by running `bunx eslint --fix` instead of attempting manual fixes.

## Dependencies Management

- Use Bun for package management
- Sync dependencies with `bun install`; add new packages with `bun add <package>` (do not use npm, pnpm, or yarn)
- Run package scripts with `bun run <script>` and one-off tooling through `bunx`
- Keep dependencies up to date
- Prefer well-maintained packages
- Document any custom configurations
