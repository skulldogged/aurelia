# What to Do When a Task is Completed

## Code Quality Checks
1. **Type Check**: Run `vue-tsc --noEmit` to ensure no TypeScript errors
2. **Lint**: Run `bunx eslint --fix` to check and auto-fix code style issues
3. **Build Test**: Run `bun run build` to ensure the frontend builds successfully
4. **Tauri Build**: Run `bun run tauri build` to ensure the full app builds (optional for quick checks)

## Validation Steps
- Test the changes in development mode: `bun run tauri dev`
- Verify functionality works as expected
- Check for any console errors or warnings
- Ensure UI looks correct and is accessible

## Before Committing
- Stage changes: `git add .`
- Commit with descriptive message: `git commit -m "Brief description of changes"`
- Push to branch if working collaboratively

## Quality Gates Status
- Report pass/fail status for type checking, linting, and building
- Include key output or error messages if issues occur
- If build fails, iterate up to 3 targeted fixes before asking for help

## Follow-ups
- Suggest 1-2 bite-sized improvements only if they clearly add value
- Note any technical debt or deprecations that should be addressed later
- Keep the repo in better state than found (modern code, no legacy patterns)

## Reporting
- List files created or touched and why
- Provide quick "how to run" commands for user verification
- Mark deferred checks with reasons (e.g., full integration tests)