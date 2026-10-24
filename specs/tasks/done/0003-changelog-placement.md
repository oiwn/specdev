---
id: 0003-changelog-placement
status: done
scope: [src/task/done.rs]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/20
---
# Task: changelog placement

## Plan

- [x] `insert_entry` (src/task/done.rs): insert above the first **dated** H2 (`## YYYY-MM-DD …`, `[YYYY-MM-DD]` tolerated); fall back to the first H2 of any kind when no dated entry exists; append when there are no H2s (#20 item 4)
- [x] Unit tests: dated entry below an undated H2 (`## [Unreleased]`, `## Notes`) targets the dated one; undated-only keeps today's behavior
- [x] Document the rule in overview (`task done` bullet)

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #20 item 4 — with an undated H2 high in the file (Unreleased/Notes sections), the entry landed above it or far from the dated entries. The file header already promises "newest at the top", so the dated region is the entry list. Draft approved in session 2026-10-24 with the batch plan.

## Review

- Change is one `find` with a predicate plus a fallback `or_else`; the splice code is untouched, so the byte-for-byte guarantee and write order stand.
- Date detection is `NaiveDate::parse_from_str` on the first 10 chars after stripping `[` — no regex crate, no fuzzy matching (`## 1.2.3` stays undated).
- All four insertion shapes covered by unit tests; e2e placement test unchanged because this repo's CHANGELOG starts with dated entries.

## Summary

`task done` CHANGELOG entries now land above the newest dated entry instead of above whatever level-2 heading comes first.

## Log

- 2026-10-24 created: changelog placement
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/20
- 2026-10-24 scope add src/task/done.rs — insert_entry and its tests live here; overview.md is specs/ (always allowed)
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/task/done.rs
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 baseline: skills/SKILL.md@785280029cea, src/fmt.rs@53e7052a3944, src/main.rs@756076dfe14d, src/status.rs@bf68a020638f, tests/cli.rs@fc285b1a0f61
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
