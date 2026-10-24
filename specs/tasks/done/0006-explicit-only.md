---
id: 0006-explicit-only
status: done
scope: [src/config.rs, src/check.rs, src/init.rs, skills/SKILL.md, specdev.toml, tests/cli.rs]
created: 2026-10-24
source: https://github.com/oiwn/specdev/issues/19
---
# Task: explicit only

## Plan

- [x] Config: `[task] explicit_only` (default `true`) in `config.rs` defaults, the `init` template, and this repo's `specdev.toml` (#19)
- [x] `check`: working-tree `out-of-scope` downgrades error → warning while `explicit_only = true` (`--staged` already warns; `false` restores errors)
- [x] Skill: tasks are explicit — never create a task or any specdev record unless the user asks; small fixes are direct edits plus verification; `[task] explicit_only = false` restores the old routing; the `out-of-scope` bullet explains the warning mode
- [x] Unit + e2e: default config warns, `explicit_only = false` errors; template carries the key

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Source: #19 — agents spun up full tasks for clippy-level fixes; the out-of-scope error then pressured more structure. Skill-first fix with a config knob. Batch plan approved in session 2026-10-24 ("the rest looks good. let's go"); draft approval via `task approve` is the user's step under the new gate (task 0005).

## Review

- The severity switch is one `||` in `scope_check`; the default lives in `TaskDef::default`, so `init`'s template (`DEFAULT_TOML`), the equality unit test, and old configs missing the key all agree by construction.
- Behavior note: strict projects must now set `explicit_only = false` to keep hard scope enforcement — the e2e strict fixtures do, and README/config docs say so (README sync is task 0007's).
- The strict-mode e2e commits its config rewrite; `specdev.toml` is not `specdev_owned`, and leaving it uncommitted would have made the config file itself out-of-scope.

## Summary

Tasks are explicit by default: the skill says never create a task unless asked, and working-tree `out-of-scope` is a warning unless a project sets `[task] explicit_only = false`.

## Log

- 2026-10-24 created: explicit only
- 2026-10-24 set source https://github.com/oiwn/specdev/issues/19
- 2026-10-24 scope add src/config.rs — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 scope add src/check.rs — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 scope add src/init.rs — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 scope add skills/SKILL.md — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 scope add specdev.toml — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 scope add tests/cli.rs — config default + template, severity switch, skill text, repo config, tests
- 2026-10-24 draft approved: your words
- 2026-10-24 draft approved: ggwp
- 2026-10-24 advance draft → ready
- 2026-10-24 scope approved: src/config.rs, src/check.rs, src/init.rs, skills/SKILL.md, specdev.toml, tests/cli.rs
- 2026-10-24 advance ready → in-progress/implement
- 2026-10-24 baseline: src/fmt.rs@53e7052a3944, src/main.rs@09ec7f71ee06, src/status.rs@bf68a020638f, src/task/done.rs@01fe4b216230, src/task/log.rs@395b1a082bc1, src/task/state.rs@722daa4dc110, src/task/transition.rs@a79440d5c008
- 2026-10-24 advance in-progress/implement → in-progress/verify
- 2026-10-24 advance in-progress/verify → in-progress/review
- 2026-10-24 advance in-progress/review → approval
- 2026-10-25 approved: lots go!
- 2026-10-25 advance approval → done
