---
id: 0009-config-skill
status: done
scope: [src/skill.rs, skills/SKILL.md, tests/cli.rs, references/config.md]
created: 2026-10-25
source: https://github.com/oiwn/specdev/issues/21
---
# Task: config skill

## Plan

- [x] Write `skills/references/config.md`: per-section semantics of `specdev.toml` (`[task]` incl. `explicit_only`, `[pipeline]`, `[acceptance]`, `[scope]`, `[quality]` incl. per-kind tables and `[quality.grade.<name>]` + the `task set <id> grade` workflow), the cross-cutting rules (omitted keys inherit, `0` turns a limit off, unknown keys rejected, grades validated against frontmatter, `[quality] errors = true` arms the state-command safety net), recipes, and the sizing philosophy (prefer splitting over raising)
- [x] `src/skill.rs`: replace the hardcoded `examples.md` pair with every file under `skills/references/` for both `install` and `check` (same version stamp, same states)
- [x] `skills/SKILL.md`: one pointer line — editing `specdev.toml` → read `references/config.md` first
- [x] Unit + e2e: install writes every reference file, `skill check` flags a missing/modified one, old installs without the new file report as stale on reinstall

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings

## Context

Agreed design (session 2026-10-25): one skill, references grow inside it — no second top-level skill (fragmented triggering/install for content only reachable via specdev anyway). The reference documents semantics; current values live in the project's own file and `specdev init`'s template, so the doc stays drift-free.

## Review

- Path slip at draft time: references live at the repo root (`references/`, per the `include_str!` paths), not `skills/references/` — scope corrected with `scope rm`/`add` before implementation, which is exactly what those commands are for.
- The installer list (`REFERENCES`) is the single point a future reference joins; install and check both iterate it, and `installs_every_bundled_reference` fails if the list and bundle disagree. A genuinely directory-driven include isn't possible with `include_str!`; one-line additions are the intended ergonomics.
- The reference deliberately states semantics over values (values live in each project's file and the init template), so it can't drift as defaults evolve; the sizing philosophy mirrors the skill and #18.

## Summary

The skill ships `references/config.md` (full `specdev.toml` semantics, rules, recipes, sizing philosophy), and the installer/checker now ship and verify every bundled reference file, not just examples.

## Log

- 2026-10-25 created: config skill
- 2026-10-25 set source https://github.com/oiwn/specdev/issues/21
- 2026-10-25 scope add skills/references/config.md — the new reference; generalized installer/check; pointer line; e2e coverage
- 2026-10-25 scope add src/skill.rs — the new reference; generalized installer/check; pointer line; e2e coverage
- 2026-10-25 scope add skills/SKILL.md — the new reference; generalized installer/check; pointer line; e2e coverage
- 2026-10-25 scope add tests/cli.rs — the new reference; generalized installer/check; pointer line; e2e coverage
- 2026-10-25 draft approved: done
- 2026-10-25 advance draft → ready
- 2026-10-25 scope approved: skills/references/config.md, src/skill.rs, skills/SKILL.md, tests/cli.rs
- 2026-10-25 advance ready → in-progress/implement
- 2026-10-25 baseline: src/check.rs@c6307e8d083c, src/config.rs@e87d573212db, src/quality.rs@ab84112c4561, src/task/cmd.rs@490a8d1b4815, src/task/frontmatter.rs@6952e8942777, src/task/state.rs@fd17f61e79cf
- 2026-10-25 scope rm skills/references/config.md — wrong path; references live at the repo root
- 2026-10-25 scope add references/config.md — the new reference; the repo keeps references at the root (include_str paths)
- 2026-10-25 advance in-progress/implement → in-progress/verify
- 2026-10-25 advance in-progress/verify → in-progress/review
- 2026-10-25 advance in-progress/review → approval
- 2026-10-25 approved: yes, looks good.
- 2026-10-25 advance approval → done
