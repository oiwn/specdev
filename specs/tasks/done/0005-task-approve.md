---
id: 0005-task-approve
status: done
scope: [src/task/log.rs, src/task/transition.rs, src/task/state.rs, src/main.rs, skills/SKILL.md, tests/cli.rs, src/check.rs]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/17
---
# Task: task approve

## Plan

- [x] `LogEvent::DraftApproved { note }` (`draft approved: <note>`, bare `draft approved` when no note); legal only while the task is in draft; re-entering draft clears it (#17, #20 item 5)
- [x] `specdev task approve <id> [--note "<their words>"]` records the human's approval in `## Log`
- [x] `advance` draft → ready refuses without a recorded approval (alongside the open-`^^^` gate); replay does not gate old logs, which stay valid
- [x] Skill: draft stage names `task approve` as the human's step; CLI list gains the command; overview documents the gate and the replay caveat
- [x] Unit + e2e: round-trip, refuse without approval, approve-then-advance works, non-draft approve refused, ready→draft needs fresh approval, old-style logs still check green

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #17, #20 item 5 — an agent advanced a draft to ready twice on a session-level "let's go" without ever showing the draft. The log event makes the skip auditable instead of silent; strictly human-run in spirit (quote their words, don't paraphrase). This task's own draft→ready runs before the gate exists; its approval evidence is the session quote: "the rest looks good. let's go". Draft approved with the batch plan 2026-10-24.

## Review

- Deliberate non-goal: replay does not enforce the draft→ready approval gate — pre-0.4.0 logs would all fail `check` at upgrade. `advance` enforces it live from replayed state; replay-enforcement is a roadmap item once old logs are gone.
- `Replayed.draft_approved` resets on every entry into draft (advance or unblock), so a bounced draft needs fresh approval; the flag rides the same replay path as scope.
- Five e2e setups (`drafted`, `ready_to_go`, three inline) needed an approve step — expected: the gate is on by default.
- README's gate list still says `^^^` only; deferred to the release task with the other doc sync.

## Summary

`specdev task approve <id> --note "<their words>"` records the human's draft approval in the log, and `advance` refuses draft → ready without it.

## Log

- 2026-10-24 created: task approve
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/17
- 2026-10-24 scope add src/task/log.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add src/task/transition.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add src/task/state.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add src/main.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add skills/SKILL.md — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add tests/cli.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 scope add src/check.rs — log event, replay rules, approve command + gate, CLI, skill docs, tests
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/task/log.rs, src/task/transition.rs, src/task/state.rs, src/main.rs, skills/SKILL.md, tests/cli.rs, src/check.rs
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 baseline: src/fmt.rs@53e7052a3944, src/status.rs@bf68a020638f, src/task/done.rs@01fe4b216230
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
