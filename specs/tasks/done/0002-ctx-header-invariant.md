---
id: 0002-ctx-header-invariant
status: done
scope: [src/status.rs, tests/cli.rs]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/15
---
# Task: ctx header invariant

## Plan

- [x] `ctx_diagnostics` (src/status.rs, shared by `status` and `check`): the first non-empty line must be `# Current Task Context` or `# Current Task Context: <task>` with a non-empty task; anything else (or an empty file) is a `ctx-header` warning carrying the line number (#15)
- [x] Unit tests: canonical header, header with task, drift (`# Current Task:`), empty file
- [x] E2e: `check` flags a drifted ctx.md header

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #15 — oiwn/tarts revived with `# Current Task:` and `check` stayed green, though the skill calls the header the task identity. Warning first (like `unformatted`), not an error. Draft approved in session 2026-10-24 with the batch plan.

## Review

- One rule in one place (`ctx_diagnostics`), so `status` and `check` agree by construction; line number rides the existing `Diagnostic` field.
- Two existing unit fixtures used `# Task:` headers — updated to the canonical form; nothing else in the repo drifts (this repo's ctx.md is canonical).
- `strip_prefix` + non-empty suffix keeps `: ` alone from passing as a title.

## Summary

`check` and `status` now warn `ctx-header` when `ctx.md` doesn't start with `# Current Task Context` (optionally `: <task>`), with the offending line number.

## Log

- 2026-10-24 created: ctx header invariant
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/15
- 2026-10-24 scope add src/status.rs — ctx_diagnostics lives here and is shared with check; e2e assertion on the drifted header
- 2026-10-24 scope add tests/cli.rs — ctx_diagnostics lives here and is shared with check; e2e assertion on the drifted header
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/status.rs, tests/cli.rs
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 baseline: skills/SKILL.md@785280029cea, src/fmt.rs@53e7052a3944, src/main.rs@756076dfe14d
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
