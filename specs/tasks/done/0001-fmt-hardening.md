---
id: 0001-fmt-hardening
status: done
scope: [src/fmt.rs, src/main.rs, tests/cli.rs, skills/SKILL.md]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/20
---
# Task: fmt hardening

## Plan

- [x] Validate explicit `fmt` paths against the default set (`specs/*.md` + open `specs/tasks/*.md`, no `_`-prefixed files, nothing in `done/`); anything else fails with a usage error before any write (#20 item 2)
- [x] Add `fmt --check`: report files that would change, write nothing, exit 1 when any would — emit the report first, then `Error::CheckFailed` like `check` does (#21 item 6)
- [x] Unit tests for target validation and the check-mode report; e2e tests for the rejection and both `--check` exit codes
- [x] Skill CLI reference: `fmt` path rule + `--check`

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #20 (item 2 — `fmt README.md` unwrapped unrelated files), #21 (item 6 — CI flag). A target is allowed iff it is one of `default_files`; explicit paths only narrow that set. Draft approved in session 2026-10-24 with the batch plan: "the rest looks good. let's go".

## Review

- Diff matches the plan: `resolve_targets` guards every read/write; check mode mirrors `check`'s emit-then-`CheckFailed` exit pattern; both reports carry the same fields under honest names (`changed` vs `would_change`).
- `Vec<String>::new()` in expression position needs the turbofish (`Vec::<String>::new()`) — cost one build cycle, fixed.
- Skill line updated in place; no other docs mention `fmt` flags.

## Summary

`specdev fmt` refuses paths outside `specs/` and open task files, and gains `--check` (report-only, exit 1) for CI and hooks.

## Log

- 2026-10-24 created: fmt hardening
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/20
- 2026-10-24 scope add src/fmt.rs — fmt command + run(); e2e coverage; skill CLI reference documents --check and the path rule
- 2026-10-24 scope add src/main.rs — fmt command + run(); e2e coverage; skill CLI reference documents --check and the path rule
- 2026-10-24 scope add tests/cli.rs — fmt command + run(); e2e coverage; skill CLI reference documents --check and the path rule
- 2026-10-24 scope add skills/SKILL.md — fmt command + run(); e2e coverage; skill CLI reference documents --check and the path rule
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/fmt.rs, src/main.rs, tests/cli.rs, skills/SKILL.md
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
