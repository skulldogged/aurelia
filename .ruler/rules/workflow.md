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

| Platform | Framework | Location | Run Command |
|----------|-----------|----------|-------------|
| Rust | `cargo test` | `crates/aurelia-core/src/**` (inline `#[cfg(test)]`) | `cargo test -p aurelia-core` |
| Desktop | Vitest (if configured) | `apps/desktop/src/**/*.test.ts` | `cd apps/desktop && bun test` |
| Android | JUnit + Compose | `apps/mobile/android/app/src/test/` | `cd apps/mobile/android && ./gradlew test` |

**Note**: Test coverage is currently minimal. When adding tests, co-locate them with the code being tested.
