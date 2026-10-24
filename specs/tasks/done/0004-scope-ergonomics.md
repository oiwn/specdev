---
id: 0004-scope-ergonomics
status: done
scope: [src/task/log.rs, src/task/state.rs, src/main.rs, src/check.rs, skills/SKILL.md, tests/cli.rs, src/task/transition.rs]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/20
---
# Task: scope ergonomics

## Plan

- [x] `LogEvent::ScopeRemove` gains an optional `reason` (`scope rm <path> — <reason>`, mirroring add; old lines still parse), `task scope rm` accepts `--reason` (#20 item 3)
- [x] Out-of-scope diagnostics name the other open tasks whose scope covers the file (#20 item 6)
- [x] Skill documents `task scope rm` alongside `add` (#16)
- [x] Unit + e2e tests: reason round-trip and logging, candidate-task message

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #20 items 3+6, #16. In a real multi-task session a mistyped `--reason` on `rm` errored, and the out-of-scope error blamed the only task it checked while the file belonged to another open one. Draft approved in session 2026-10-24 with the batch plan.

## Review

- Scope grew to 7 entries (one over `max_scope`): the `ScopeRemove` field rippled into `src/task/transition.rs`'s replay pattern (`..` only). Deliberate, not scope creep.
- Old `scope rm <path>` log lines still parse (reason optional on both sides of the round trip); replay ignores the reason, so no transition semantics changed.
- The e2e `scope rm` step reverts the file first — the safety net would rightly refuse a scope removal that orphans a changed file. `step` wasn't usable at that point in the test (standing errors), so those calls assert their own exit codes.

## Summary

`task scope rm` accepts an optional logged `--reason`, and out-of-scope errors name the other open task whose scope already covers the file.

## Log

- 2026-10-24 created: scope ergonomics
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/20
- 2026-10-24 scope add src/task/log.rs — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 scope add src/task/state.rs — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 scope add src/main.rs — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 scope add src/check.rs — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 scope add skills/SKILL.md — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 scope add tests/cli.rs — scope rm event + command; out-of-scope message; skill docs; tests
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/task/log.rs, src/task/state.rs, src/main.rs, src/check.rs, skills/SKILL.md, tests/cli.rs
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 baseline: src/fmt.rs@53e7052a3944, src/status.rs@bf68a020638f, src/task/done.rs@01fe4b216230
- 2026-10-24 scope add src/task/transition.rs — ScopeRemove gained a reason field; the replay pattern needs
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
