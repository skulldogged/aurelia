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

## Testing

**Authoritative guide**: `TESTING.md` in repo root.

| Scope | Framework | Location | Run Command |
|-------|-----------|----------|-------------|
| All JS/TS | Vitest | `apps/**/tests` + `apps/**/src` | `bun run test:js` |
| Shared UI | Vitest + Vue Test Utils | `apps/shared/tests/` | `cd apps/shared && bun test` |
| Web Frontend | Vitest + Testing Library | `apps/web/frontend/tests/` | `cd apps/web/frontend && bun test` |
| Desktop | Vitest + Testing Library | `apps/desktop/tests/` | `cd apps/desktop && bun test` |
| Rust Core | `cargo test` | `crates/aurelia-core/src/**` | `cargo test -p aurelia-core` |
| Rust API | `cargo test` | `crates/aurelia-api/src/**` | `cargo test -p aurelia-api` |
| Rust Macros | `cargo test` | `crates/aurelia-api-macros/src/**` | `cargo test -p aurelia-api-macros` |
| Web Backend | `cargo test` | `apps/web/backend/tests/` | `cargo test -p aurelia-web-backend` |
| Android (unit) | JUnit | `apps/mobile/android/app/src/test/` | `bun run test:android` |
| Android (UI) | Compose UI | `apps/mobile/android/app/src/androidTest/` | `bun run test:android:ui` |
| iOS | Swift Package | `apps/mobile/ios/AureliaCore/Tests/` | `bun run test:ios` |

**Notes**
1. Use `bun run test` for the standard combined test suite.
2. When adding tests, co-locate them with the code being tested.
