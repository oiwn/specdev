---
id: 0008-task-grades
status: done
scope: [src/config.rs, src/task.rs, src/task/frontmatter.rs, src/task/state.rs, src/quality.rs, src/check.rs, skills/SKILL.md, tests/cli.rs, src/task/cmd.rs]
created: 2026-10-25
source: https://github.com/oiwn/specdev/issues/18
---
# Task: task grades

## Plan

- [x] Config: `[quality.grade.<name>]` tables (`BTreeMap<String, Limits>` in `Quality`), defaults `tiny` and `huge` shipped beside `[quality.task]` (which stays the unnamed default grade); keys inherit from `[quality.task]`, `0` still turns a limit off; template updated (#18, #21 item 8)
- [x] Frontmatter: plain `grade` field (like `source`: scalar, optional, rendered between them); unset = default grade
- [x] `task set <id> grade <name>`: built-in set arm, value validated against the configured grades, logged as `set grade <name>`; empty clears
- [x] `check`: `grade` naming an undefined grade is an error; the quality gate measures each task against its grade's effective limits
- [x] Quality warnings name the remedies: split into smaller tasks ordered by `depends`, or raise the grade (`task set <id> grade <name>`)
- [x] Skill: `grade:` in the frontmatter sketch, one `task set` example, and the sizing rule — match the grade to real size, prefer splitting, `huge` only for user-mandated umbrellas
- [x] Unit + e2e: defaults/inheritance parse, set flow + validation, over-default task passes clean with a grade, unknown grade errors, remedy wording

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #18 (plus #21 item 8) — large working-manual tasks can't comply with one-size limits; permanent warnings train people to ignore them. Agreed design (session 2026-10-25): grades are config metadata, not task-body sections; `[quality.task]` = default grade; split suggestion lives in the warning wording and skill guidance, no new machinery.

## Review

- Grade tables merge over the built-ins by name, so old configs without any `[quality.grade.*]` keep `tiny`/`huge` for free; unused names are inert. Inheritance reuses `Limits::or` — a grade replaces only the keys it sets.
- The quality gate now resolves task limits through `Quality::task_limits(grade)`; every other kind is untouched, and the `scope_len` special case survives.
- Process slip to not repeat: the multi-site test-signature update went through a python one-liner instead of edit tools (AGENTS.md forbids it); and a stray `#` commented out `[quality.task]` in the template mid-edit — caught immediately by `default_toml_matches_default_config`, which is exactly what that test is for.
- Task file is 7 scope entries (one over default max) — the batch tasks 0004/0005 hit the same ripple and it's documented there.

## Summary

Tasks declare a size grade (`task set <id> grade <name>`, from `[quality.grade.<name>]` tables; default `tiny`/`huge`), the quality gate measures against it, and over-limit warnings say whether to split or raise the grade.

## Log

- 2026-10-25 created: task grades
- 2026-10-25 set source https://github.com/oiwn/specdev/issues/18
- 2026-10-25 scope add src/config.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add src/task.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add src/task/frontmatter.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add src/task/state.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add src/quality.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add src/check.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add skills/SKILL.md — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 scope add tests/cli.rs — grades config + template; Frontmatter field; set arm + validation; effective limits; unknown-grade error; skill anatomy; e2e
- 2026-10-25 draft approved: ok
- 2026-10-25 advance draft → ready
- 2026-10-25 scope approved: src/config.rs, src/task.rs, src/task/frontmatter.rs, src/task/state.rs, src/quality.rs, src/check.rs, skills/SKILL.md, tests/cli.rs
- 2026-10-25 advance ready → in-progress/implement
- 2026-10-25 advance in-progress/implement → in-progress/verify
- 2026-10-25 advance in-progress/verify → in-progress/review
- 2026-10-25 advance in-progress/review → approval
- 2026-10-25 scope add src/task/cmd.rs — Frontmatter construction needed the new grade field
- 2026-10-25 approved: done, implement 0009
- 2026-10-25 advance approval → done
