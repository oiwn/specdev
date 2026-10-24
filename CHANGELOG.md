# Changelog

Completed tasks, moved here from `specs/ctx.md` once every checkbox in their
plan is done. Newest entry at the top, dated. Freeform — one block per finished
task. This coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->

## 2026-10-25 — skill stale

- `check` and `status` now warn `skill-stale` when the project's local skill install (`.agents/skills/specdev/`) drifts from the bundled skill — stale, modified, or missing files — with the `specdev skill install --local` remedy.
- Task `0010-skill-stale`; source: session 2026-10-25: silent skill/version drift until skill check is run manually

## 2026-10-25 — release 040

- Released 0.4.0: README synced with the batch, version bumped, skill reinstalled and verified current, #15–#20 closed and #21 updated.
- Task `0007-release-040`; source: https://github.com/oiwn/specdev/issues/21

## 2026-10-25 — Release 0.4.0: trust + papercuts

Batch of eight tasks from real-session friction (#15–#21), plus the 0.4.0 release itself. Human draft approval is now a recorded, gated step; tasks are explicit by default; task size grades make the quality gate honest for large working manuals; the skill ships a full config reference.

- `task approve <id> --note` — draft → ready requires the human's recorded approval; a bounce back to draft voids it (#17, #20).
- Explicit-only tasks — `[task] explicit_only = true` default; small fixes are direct edits; working-tree out-of-scope warns instead of erroring (#19).
- Task grades — `[quality.grade.<name>]` + `task set <id> grade`, `tiny`/`huge` defaults, split-or-raise warning wording (#18, #21).
- fmt hardening — paths restricted to `specs/` + open tasks; `fmt --check` for CI/hooks (#20, #21).
- `check` — ctx.md header invariant warning (#15); out-of-scope errors name the covering task (#20); `task done` CHANGELOG entries land above the newest dated heading (#20); `scope rm` takes a logged `--reason` (#20, #16).
- Skill — `references/config.md` (specdev.toml semantics) shipped and verified by a generalized installer/checker; docs cover `scope rm`, `task approve`, grades, explicit-only.
- Issues closed with the release: #15–#20; #21 stays open for the hygiene suite and parallel tasks.

## 2026-10-25 — config skill

- The skill ships `references/config.md` (full `specdev.toml` semantics, rules, recipes, sizing philosophy), and the installer/checker now ship and verify every bundled reference file, not just examples.
- Task `0009-config-skill`; source: https://github.com/oiwn/specdev/issues/21

## 2026-10-25 — task grades

- Tasks declare a size grade (`task set <id> grade <name>`, from `[quality.grade.<name>]` tables; default `tiny`/`huge`), the quality gate measures against it, and over-limit warnings say whether to split or raise the grade.
- Task `0008-task-grades`; source: https://github.com/oiwn/specdev/issues/18

## 2026-10-25 — explicit only

- Tasks are explicit by default: the skill says never create a task unless asked, and working-tree `out-of-scope` is a warning unless a project sets `[task] explicit_only = false`.
- Task `0006-explicit-only`; source: https://github.com/oiwn/specdev/issues/19

## 2026-10-25 — task approve

- `specdev task approve <id> --note "<their words>"` records the human's draft approval in the log, and `advance` refuses draft → ready without it.
- Task `0005-task-approve`; source: https://github.com/oiwn/specdev/issues/17

## 2026-10-25 — scope ergonomics

- `task scope rm` accepts an optional logged `--reason`, and out-of-scope errors name the other open task whose scope already covers the file.
- Task `0004-scope-ergonomics`; source: https://github.com/oiwn/specdev/issues/20

## 2026-10-25 — changelog placement

- `task done` CHANGELOG entries now land above the newest dated entry instead of above whatever level-2 heading comes first.
- Task `0003-changelog-placement`; source: https://github.com/oiwn/specdev/issues/20

## 2026-10-25 — ctx header invariant

- `check` and `status` now warn `ctx-header` when `ctx.md` doesn't start with `# Current Task Context` (optionally `: <task>`), with the offending line number.
- Task `0002-ctx-header-invariant`; source: https://github.com/oiwn/specdev/issues/15

## 2026-10-25 — fmt hardening

- `specdev fmt` refuses paths outside `specs/` and open task files, and gains `--check` (report-only, exit 1) for CI and hooks.
- Task `0001-fmt-hardening`; source: https://github.com/oiwn/specdev/issues/20

## 2026-10-05 — New specdev version: Phases 6–7

Closes the ctx.md task "New specdev version and updated workflow" (started 2026-10-02). Phase 8 — skill, docs, dogfood, 0.3.0 release — moved to `specs/roadmap.md` → Next. Durable rules and gotchas from its Findings now live in `specs/overview.md`.

- **Phase 6 — `task done`** (`src/task/done.rs`): approval → done, file moved to `specs/tasks/done/`, a dated CHANGELOG entry built from the task's one-line `## Summary`, index regenerated. Refuses without a Summary or while `## Manual checks` has unticked boxes; everything is checked before the first write. `check` adds `summary-missing` (warning, approval tasks) and `not-archived` (error). `state.rs` split its load/guard, log-append, and safety-net steps into shared helpers.
- **Phase 7a — quality gate** (`src/quality.rs`): per-file `Metrics` (also behind `list --stats`, which gained `lines`/`code`/`tbl` columns) and `[quality.*]` thresholds per kind — task 150 lines / 8 plan steps / 6 scope, ctx 120, overview 200, CHANGELOG 400. Warnings by default, errors with `[quality] errors = true` (then the state commands' safety net also refuses, e.g. a `scope add` past `max_scope`). `forbid_tables` flags Markdown tables and ASCII box diagrams. Omitted keys in a kind's table keep their defaults; `0` turns a limit off.
- **Phase 7b — `specdev fmt`** (`src/fmt.rs`): unwraps hard-wrapped paragraphs (not across hard breaks or `^^^`/`&&&` lines), `*`/`+` bullets → `-`, blank lines around `#` headings — by position from `md::fmt_facts`, every other byte kept. Covers `specs/*.md` and open task files; `check` warns `unformatted` on the same set. `init`'s templates are soft-wrapped now, and this repo's specs were reformatted with it.
- This repo got its own `specdev.toml` (`forbid_tables = true`). 156 unit + 38 e2e tests; clippy `-D warnings` clean; `prek` passes.

## 2026-10-04 — New specdev version: Phases 3–5

Part of the ongoing task "New specdev version and updated workflow" (Phases 6–8 remain in `specs/ctx.md`).

- **E2e coverage first**: `tests/cli.rs` gained `task index`, `skill install`/`skill check` (with `HOME` pointed at a tempdir), and behavioral `scan`/`status` tests, so the Phase 3 refactor had a net.
- **Phase 3 — `specdev check`** (`src/check.rs`, `src/task/transition.rs`): one transition table plus a `## Log` replay; per-task contract (id/filename, `stage`/`blocked_reason` iff, required sections, `depends`, completeness — warnings in draft, errors after), scope lint, log consistency (frontmatter must equal the replayed state), one active task, `_index.md` freshness, ctx warnings folded in from `status`. Errors exit 1. `scan` and `status` now emit reports, so `--format json|toon` works on them; `status` warnings read `<file>: <message>`.
- **Phase 4 — state commands** (`src/task/state.rs`): `task advance [--to]`, `task block --reason`, `task set`, `task scope add|rm`. Gates: no open `^^^` for draft → ready, Plan ticked for implement → verify, Acceptance ticked for verify → review, one active task; entering fix past `max_attempts` blocks the task. Every command refuses a change that would add `check` errors and refuses outright on a broken log.
- **Phase 5 — scope check** (`src/vcs.rs`): `check` compares changed files from the `git` CLI (read-only) with the active task's scope; `specs/` and `[scope] always_allowed` are exempt, plain entries cover their subtree. `check --staged` warns and runs as a `prek.toml` hook. No git → one `scope-skipped` warning. `task new` pre-fills `## Acceptance` from `acceptance.default`; `Manual checks` joined the default optional sections.
- Fixed: a freshly initialized project failed `check` on the missing `_index.md`; a missing index is now fine until the first task exists.
- Dependencies: `glob` 0.3 (+1 crate, no deps of its own). 135 unit + 34 e2e tests; clippy `-D warnings` clean.

## 2026-10-03 — New specdev version: design + Phases 1–2

Part of the ongoing task "New specdev version and updated workflow" (Phases 3–8 remain in `specs/ctx.md`).

- **Design ("Dark Factory")** in `specs/factory.md`: a multi-stage pipeline per task (draft → ready → implement → verify → review → fix → approval → done), one active task with a queue, a `scope` file contract approved at `draft → ready`, a per-task `## Log` so state is validated without git history, the task contract defined in `specdev.toml`, hardcoded `Status`/`Stage` enums. specdev is an agent-agnostic tool, not an orchestrator; it never writes to git and reads it only via the `git` CLI.
- **Specs aligned**: roadmap milestones mapped to the phases, overview states the new direction, skill formatting rule switched to soft wraps (plus the practical moshack additions: edit-scope boundary, archival-gate wording, remark/prose consistency, compression boundary).
- **`AGENTS.md`** rewritten with design principles, Rust conventions (no `mod.rs`), spec-writing rules, gotchas, and the full verification command; `_typos.toml` added.
- **Phase 1 — foundation**: `src/md.rs` (comrak outline; `list`/`status`/`scan` no longer count headings, checkboxes, or `^^^` inside code blocks), `src/output.rs` (global `--format text|json|toon`), `src/config.rs` (`specdev.toml` with defaults), `src/diag.rs`, `src/task.rs` + `src/task/{id,frontmatter,log}.rs` (hand-written flat-YAML frontmatter, log grammar, body kept byte-for-byte).
- **Phase 2 — commands**: `init` also writes `specdev.toml` and `specs/tasks/done/`; `task new|list|show|index`; generated `specs/tasks/_index.md`.
- Fixed: `list --stats` `[ ]` column printed the total checkbox count instead of unchecked.
- Dependencies: runtime tree 23 → 53 crates (comrak, serde, toml, serde_json, toon-format, chrono). 109 unit + 15 e2e tests; clippy `-D warnings` clean.

## 2026-07-15 — `specdev list` + `list --stats` command

- New `specdev list` command: lists `specs/*.md` with the first Markdown header
  as the description and a line count.
- `specdev list --stats` adds a structural breakdown per file: heading counts by
  level (H1/H2/H3/H4+), checkboxes ([x]/[ ]), open `^^^` remarks, and word count.
- Core files list in canonical order (overview, ctx, roadmap, ideas, cleanup),
  then extras alphabetically.
- ctx.md H1 is now `# Current Task Context: <task>` — the header doubles as the
  `list` description. The no-task state is header-only, not blank.
- New `src/list.rs` module with `first_header`/`count_headings`/`truncate`
  helpers (unit-tested); `init` ctx template updated to header-only.
- Header ripple across SKILL.md, AGENTS.md (root + init template), README,
  overview. 45 tests pass; clippy clean.

## 2026-07-15 — specdev time-axis redesign + content routing

- Established the **time-axis content-routing model**: `CHANGELOG.md` (root) ←
  `specs/ctx.md` → `specs/roadmap.md` → `specs/ideas.md`; `overview.md` holds
  Gotchas/Knowledge; `cleanup.md` is read-on-demand.
- `ctx.md` is now a **narrow checkbox plan** (Task / State / Plan / Context /
  Next) with a forbidden-content norm. Agents tick boxes while implementing.
- **Definition of done**: all `[x]` → archive to `CHANGELOG.md`, reset ctx.
  Archival is agent-driven on user request; no destructive CLI command, by
  design.
- Added `AGENTS.md` (root) so third-party agents are aware of specdev.
- `init` now scaffolds `roadmap.md` + root `CHANGELOG.md`/`AGENTS.md`.
- `status` reports task progress (`N/M` steps), flags forbidden content in
  ctx, nudges archival when all steps are checked, and warns on missing root
  files.
- Rewrote `skills/SKILL.md` with the routing table + soft template; added a
  before/after routing example to `references/examples.md`.
- 35 tests pass; `cargo clippy -D warnings` clean.
