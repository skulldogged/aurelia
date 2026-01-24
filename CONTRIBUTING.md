# Contributing to Aurelia

Thank you for your interest in contributing! This guide will help you get started.

## Getting Started

1. **Fork the repository** and clone your fork
2. **Follow the build instructions** in [BUILDING.md](./BUILDING.md)
3. **Create a branch** for your changes: `git checkout -b feature/your-feature-name`

## Development Workflow

### Prerequisites

- **Bun** v1.0+ (not npm, pnpm, or yarn)
- **Rust** stable toolchain
- Platform-specific dependencies (see [BUILDING.md](./BUILDING.md))

### Running the App

```bash
bun install              # Install dependencies
cargo tauri dev          # Run in development mode
```

### Making Changes

1. **Write your code** following the project's patterns and conventions
2. **Test your changes** thoroughly
3. **Run linting** before committing:
   ```bash
   bunx eslint --fix .
   ```
4. **Commit your changes** with clear, descriptive messages
5. **Push to your fork** and open a pull request

## Code Style

### TypeScript / Vue

- Use **TypeScript** for all new code
- Follow **Vue 3 Composition API** patterns with `<script setup>`
- Use **strict typing** — avoid `any` when possible
- Component file structure:
  ```vue
  <script setup lang="ts">
  // Imports
  // Component logic
  </script>

  <template>
    <!-- Template -->
  </template>

  <style scoped>
  /* Styles if needed */
  </style>
  ```

### Naming Conventions

- **Components**: PascalCase (e.g., `MusicPlayer.vue`)
- **Composables**: camelCase with `use` prefix (e.g., `usePlayerControls.ts`)
- **Stores**: camelCase (e.g., `player.ts`)
- **Files**: Match the export name

### Code Organization

- **Composables** (`src/composables/`) — Reusable logic and state
- **Stores** (`src/stores/`) — Global state with Pinia
- **Components** (`src/components/`) — Organized by feature:
    - `layout/` — App-level layout components
    - `player/` — Music player UI
    - `settings/` — Settings panels
    - `shared/` — Reusable components
    - `ui/` — Base UI components (shadcn-vue style)
- **Views** (`src/views/`) — Route-level components
- **Lib** (`src/lib/`) — Utilities and helpers

### Styling

- Use **Tailwind CSS** utility classes
- Follow the configured theme system (available via CSS custom properties)
- Keep styles scoped and minimal
- Use the configured design tokens for spacing, colors, etc.

### Rust Code

- Follow **standard Rust conventions**
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Keep handlers in `src-tauri/src/handlers/`
- Use proper error handling with `Result` types

## Linting and Formatting

This project uses ESLint with strict rules:

```bash
bunx eslint .           # Check for issues
bunx eslint --fix .     # Auto-fix issues
```

Key style rules:
- 2-space indentation
- Single quotes for strings
- Trailing commas in multiline structures
- 120 character line limit
- Aligned object values

**Your PR must pass linting** before it can be merged.

## Type Checking

TypeScript is enforced throughout the project:

```bash
bun run build          # Runs vue-tsc --noEmit
```

Fix any type errors before submitting your PR.

## Testing

While we don't have formal tests yet, please:
- **Manually test** all functionality you touch
- **Test edge cases** and error conditions
- **Verify** the app works on your platform
- **Check** that existing features still work

## Pull Request Guidelines

### Before Submitting

- Code follows the project's style conventions
- ESLint passes without errors
- TypeScript compiles without errors
- App runs and your changes work as expected
- No console errors or warnings
- Commit messages are clear and descriptive

### PR Description

Include:
- **What** the PR does (new feature, bug fix, refactor, etc.)
- **Why** the change is needed
- **How** you implemented it (if not obvious)
- **Screenshots** for UI changes
- **Testing** steps to verify the changes

### Review Process

- Maintainers will review your PR and may request changes
- Address feedback and push new commits
- Once approved, your PR will be merged

## What to Contribute

### Good First Issues

Look for issues labeled `good first issue` — these are great for getting started.

### Ideas for Contributions

- **Bug fixes** — Check open issues
- **UI improvements** — Better layouts, animations, accessibility
- **New features** — Discuss in an issue first for larger changes
- **Documentation** — Improve README, BUILDING.md, or code comments
- **Performance** — Optimize slow operations
- **Accessibility** — ARIA labels, keyboard navigation, screen reader support

### Before Starting Large Features

**Open an issue first** to discuss:
- Whether the feature fits the project's goals
- Implementation approach
- Potential conflicts with other work

This saves time and ensures your work can be merged.

## Code of Conduct

- Be respectful and professional
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intent

## Questions?

- **Open an issue** for bugs or feature requests
- **Start a discussion** for questions or ideas
- **Check existing issues** before creating duplicates

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to Aurelia!
