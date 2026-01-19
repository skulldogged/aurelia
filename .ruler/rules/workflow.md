# Workflow & Process

## Golden Rules
1.  **Modernization First**: Prioritize modern patterns over consistency with legacy code. If you see deprecated patterns (e.g., Vue Options API), refactor them.
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
