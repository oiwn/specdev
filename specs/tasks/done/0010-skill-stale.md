---
id: 0010-skill-stale
status: done
scope: [src/skill.rs, src/status.rs, README.md, skills/SKILL.md, tests/cli.rs]
created: 2026-10-25
source: session 2026-10-25: silent skill/version drift until skill check is run manually
---
# Task: skill stale

## Plan

- [x] `src/skill.rs`: `local_drift(root)` — when `.agents/skills/specdev/` exists, classify every installed file (reuse `check_at`) and report each non-up-to-date one (stale, locally modified, missing, unknown origin, newer) as a `skill-stale` warning naming the installed path and the `specdev skill install --local` remedy; no local install → silent
- [x] Surface it through `status::spec_diagnostics` so both `check` and `status` carry it; warnings only, never errors — an unreinstalled skill doesn't block work
- [x] Docs: one clause in README's check bullet and the skill's `check` section
- [x] Unit + e2e: stale stamp warns, tampered content warns, missing file warns, fresh install and absent install stay silent

## Acceptance

- [x] cargo fmt --check
- [x] cargo build
- [x] cargo test
- [x] cargo clippy --all-targets -- -D warnings
- [x] specdev check

## Context

Source: session 2026-10-25 (user question: "what happen if local skill will be different from specdev version?"). `specdev skill check` already classifies drift per file, but nothing surfaces it automatically — after a `cargo install` upgrade the local skill feeds agents old instructions silently. Global installs stay with `skill check`; `check` is project-scoped, so it watches the local one only.

## Review

- Drift is content-first, matching `check_at`: an old version stamp with identical files is silent (nothing to reinstall — agents already have the right bytes); the stamp only classifies *how* differing content drifted.
- Dogfooded end to end in this repo: editing `SKILL.md` for the docs clause immediately turned `check` orange with `skill-stale`, and `skill install --local` (from the working-tree build) cleared it. First reinstall used the older global binary and stayed drifted — correct behavior, wrong operator; reran with `cargo run`.
- One `Vec<Diagnostic>` from `skill::local_drift`, folded into the `spec_diagnostics` both `check` and `status` already share; no new public surface beyond the helper.

## Summary

`check` and `status` now warn `skill-stale` when the project's local skill install (`.agents/skills/specdev/`) drifts from the bundled skill — stale, modified, or missing files — with the `specdev skill install --local` remedy.

## Log

- 2026-10-25 created: skill stale
- 2026-10-25 set source session 2026-10-25: silent skill/version drift until skill check is run manually
- 2026-10-25 scope add src/skill.rs — drift helper + shared spec diagnostics; docs clauses; e2e
- 2026-10-25 scope add src/status.rs — drift helper + shared spec diagnostics; docs clauses; e2e
- 2026-10-25 scope add README.md — drift helper + shared spec diagnostics; docs clauses; e2e
- 2026-10-25 scope add skills/SKILL.md — drift helper + shared spec diagnostics; docs clauses; e2e
- 2026-10-25 scope add tests/cli.rs — drift helper + shared spec diagnostics; docs clauses; e2e
- 2026-10-25 draft approved: looks good to me
- 2026-10-25 advance draft → ready
- 2026-10-25 scope approved: src/skill.rs, src/status.rs, README.md, skills/SKILL.md, tests/cli.rs
- 2026-10-25 advance ready → in-progress/implement
- 2026-10-25 baseline: Cargo.lock@dfc89197e253, Cargo.toml@f86660575405, references/config.md@33f7670d82ea, src/check.rs@c6307e8d083c, src/config.rs@e87d573212db, src/quality.rs@ab84112c4561, src/task/cmd.rs@490a8d1b4815, src/task/frontmatter.rs@6952e8942777, src/task/state.rs@fd17f61e79cf
- 2026-10-25 advance in-progress/implement → in-progress/verify
- 2026-10-25 advance in-progress/verify → in-progress/review
- 2026-10-25 advance in-progress/review → approval
- 2026-10-25 approved: lgtm
- 2026-10-25 advance approval → done
