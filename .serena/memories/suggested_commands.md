# Suggested Commands for Jellyfin Music Player Development

## Package Management
- `bun install` - Install all dependencies (use Bun exclusively, no npm/pnpm/yarn)
- `bun add <package>` - Add a new dependency
- `bun remove <package>` - Remove a dependency

## Development
- `bun run tauri dev` - Start development mode (Vite dev server + Tauri window with hot reload)
- `cargo tauri dev` - Alternative way to start dev mode
- `bun run dev` - Start Vite dev server only (without Tauri)

## Building
- `bun run tauri build` - Build production app for current platform
- `cargo tauri build` - Alternative build command
- `bun run build` - Build frontend only (Vite)

## Code Quality
- `bunx eslint --fix` - Run ESLint and auto-fix issues
- `vue-tsc --noEmit` - TypeScript type checking
- `bun run build` - Includes type checking as part of build process

## Testing
- No specific test commands configured yet (add as needed)

## Environment Setup (Windows PowerShell)
- `$env:VITE_DISCORD_APP_ID = "your-discord-app-id"` - Set custom Discord app ID for development

## Utility Commands (Windows)
- `git status` - Check git status
- `git add .` - Stage all changes
- `git commit -m "message"` - Commit changes
- `ls` - List directory contents (PowerShell alias for Get-ChildItem)
- `cd <path>` - Change directory
- `mkdir <name>` - Create directory