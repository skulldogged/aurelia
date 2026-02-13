# Workflow & Process

## Golden Rules
1.  **Modernization First** (within scope): Prioritize modern patterns over consistency with legacy code. If you encounter deprecated patterns (e.g., Vue Options API) **in code you're already modifying**, refactor them. Don't refactor unrelated code.
2.  **Read before writing**: Gather context (files/search) before touching code. Make max 2 assumptions.
3.  **Execute**: Propose a short plan, then implement. Don't narrate excessively.

## Workflow Cycle
1.  **Plan**: Concise acknowledgment + plan.
2.  **Context**: Pull context (read files).
3.  **Action**: Break complex tasks. Implement tests alongside code if practical.
4.  **Fix**: If a command fails, try 3 targeted fixes before asking/summarizing.
5.  **Update**: Report progress without repeating unchanged items.

## Deliverables Checklist
- [ ] List files created/modified and reasoning.
- [ ] Note quality gate status (tests, lint, build).
- [ ] Provide copyable "how to run" commands.
- [ ] Suggest 1-2 valuable follow-ups (optional).

## When Stuck
- State your assumption and keep moving.
- Ask only if you truly cannot proceed.

## Building & Running

**Authoritative guide**: `BUILDING.md` in repo root.

### Development

| Platform | Command |
|----------|---------|
| Web | `bun run dev:web` |
| Desktop | `bun run dev:desktop` |
| Android | `bun run dev:android` |
| iOS | `bun run dev:ios` (macOS only) |

### Building

| Platform | Command |
|----------|---------|
| All | `bun run build` |
| Web | `bun run build:web` |
| Desktop | `bun run build:desktop` |
| Android | `bun run build:android` |
| iOS | `bun run build:ios` (macOS only) |

### Testing

| Scope | Command |
|-------|---------|
| All | `bun run test` |
| Web frontend | `bun run test:web` |
| Desktop frontend | `bun run test:desktop` |
| JavaScript tests | `bun run test:js` |
| Rust tests | `bun run test:rust` |
| Android | `bun run test:android` |
| iOS | `bun run test:ios` |

### Code Quality

| Check | Command |
|-------|---------|
| Lint | `bun run lint` |
| TypeScript | `bun run typecheck` |

## Notes
1. Use `bun run test` for the standard combined test suite.
2. When adding tests, co-locate them with the code being tested.
3. iOS commands require macOS - they'll be skipped automatically in `build`/`test` on non-macOS.
