# specdev — overview

specdev is a small Rust CLI plus a bundled agent skill. The CLI scaffolds, inspects, and (for tasks) manages structured spec state; the skill teaches the spec workflow to any coding agent. This file overviews **developing specdev itself** — the workflow the skill teaches is documented in `skills/SKILL.md`, not here. Design: `specs/factory.md`. What's next: `specs/roadmap.md`.

## Architecture

- `src/main.rs` — clap CLI: `init`, `scan`, `status`, `check`, `fmt`, `list`, `skill`, `task`; global `--format text|json|toon`; crate-level `Error`.
- `src/md.rs` — the shared Markdown parser (comrak): headings, checkboxes, sections, code-block awareness, tables/diagrams, and `fmt_facts`. All Markdown analysis goes through it.
- `src/diag.rs` — `Diagnostic` (file, line, severity, stable code, message).
- `src/output.rs` — `Format` and the `Report` trait; renders every migrated command as text, JSON, or TOON.
- `src/config.rs` — `specdev.toml` (task contract, pipeline, acceptance, scope, quality) with built-in defaults.
- `src/task.rs` — `Task`, `Status`, `Stage`; parse and `render()` (frontmatter + `## Log` regenerated, body byte-for-byte).
  - `src/task/id.rs` — `TaskId` (`0007-slug`).
  - `src/task/frontmatter.rs` — hand-written flat-YAML subset parser.
  - `src/task/log.rs` — `## Log` line format.
  - `src/task/store.rs` — `TaskStore`: loads `specs/tasks/` and `done/`, lookup, queue order, `_index.md`.
  - `src/task/transition.rs` — the legal status/stage moves (`is_legal`, `targets`, `next`) and `## Log` replay.
  - `src/task/cmd.rs` — `task new|list|show|index`.
  - `src/task/state.rs` — `task advance|block|set|scope`: frontmatter + `## Log` only, stage gates, and a before/after `check` safety net.
  - `src/task/done.rs` — `task done`: approval → done, archive to `done/`, CHANGELOG entry from `## Summary`, index.
- `src/check.rs` — `specdev check [--staged]`: task contract, log consistency, one active task, index freshness, scope vs changed files, quality gate, `unformatted`, plus the spec warnings from `status`. Errors exit 1.
- `src/vcs.rs` — changed files from the `git` CLI (read-only, `-z`, `--relative`).
- `src/quality.rs` — `Metrics` per file (also behind `list --stats`) and the `[quality.*]` thresholds.
- `src/fmt.rs` — `specdev fmt`: soft-wrap paragraphs, `-` bullets, blank lines around headings, by position from `md::fmt_facts` (no re-render).
- `src/init.rs` — scaffolds `specs/`, `specs/tasks/done/`, `specdev.toml`, root `CHANGELOG.md`/`AGENTS.md` (AGENTS.md is append-aware, idempotent via `<!-- BEGIN/END specdev -->`).
- `src/scan.rs` — parses `^^^`/`&&&` remark markers.
- `src/status.rs` — health summary; `spec_diagnostics` (forbidden ctx content, archive nudge, missing root files) is shared with `check`.
- `src/list.rs` — `list` and `list --stats`.
- `src/skill.rs` — installs `skills/SKILL.md` + `references/examples.md`.
- `skills/SKILL.md` — the shipped agent skill; `references/examples.md` — before/after examples bundled with it.
- `tests/cli.rs` — end-to-end binary tests.

## Build & test

`cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings`. `prek.toml` runs the same plus `specdev check --staged`, `typos`, `gitleaks`.

## Public surfaces

- `specdev` binary (via `cargo install`).
- Installable skill (`specdev skill install`) — global at `~/.agents/skills/specdev/`, or local at `.agents/skills/specdev/`.

## Task pipeline rules (as implemented)

- **Transitions** (`transition::is_legal`): draft ⇄ ready; ready → implement; implement → verify; verify → review | fix; review → fix | approval; fix → verify; approval → fix | done; blocked → draft | ready | any stage. Blocking is the `Blocked` log event, legal from any status but done/blocked.
- **Replay**: `created` comes first and its title and date must match the file; `scope approved` only while ready and required before ready → in-progress; `ready → draft` drops the approval; scope add/rm count only after approval (draft scope is free); `set` events are informational (`source`/`depends` aren't compared).
- **`advance`** without `--to` takes the happy-path successor (draft → ready → implement → verify → review → approval; fix → verify); from approval and blocked `--to` is required. `--to` takes a status or a bare stage name.
- **Gates**: no open `^^^` for draft → ready; all Plan boxes ticked for implement → verify; all Acceptance boxes for verify → review; one active task (in-progress or approval). Entering fix past `max_attempts` blocks the task instead, exit 0 with a note.
- **Safety net**: every state command runs `check::task_diagnostics` before and after on a copy and refuses if the change adds errors (exact diagnostic match). Any `log-*` error beforehand refuses outright.
- **`set`**: empty value clears and logs `set <field> ""`; `depends` values resolve through `TaskStore::find`. Scope paths can't contain `, ` or ` — ` (log separators).
- **`task done`**: only from approval; refuses while `## Manual checks` has unticked boxes or `## Summary` is missing (first prose line becomes the CHANGELOG entry). Write order: archived file → remove old → CHANGELOG → index, all computed and checked first. The entry goes above the first level-2 heading outside code/HTML blocks.
- **Completeness** (`scope-empty`, `plan-empty`, `acceptance-empty`) is a warning in draft and blocked, an error otherwise. A missing `_index.md` is fine while there are no tasks.
- **Scope check**: only with exactly one active task. Exempt: anything under `specs/`, `[scope] always_allowed` globs; a plain entry covers its subtree. Working tree → `out-of-scope` errors, `--staged` → warnings, no git → one `scope-skipped` warning. Changed files = `git diff HEAD` + untracked, so earlier commits of a multi-commit task aren't re-checked; before the first commit every file counts.
- **Quality gate**: kinds task (open task files), ctx, overview, changelog; other `specs/*.md` get only the table rule. Defaults: task 150 lines / 8 plan steps / 6 scope, ctx 120, overview 200, changelog 400. Warnings unless `[quality] errors = true`. Omitted keys in a kind's table keep their defaults; `0` turns a limit off. Diagrams = plain/`text` code blocks with box-drawing characters or `+--`/`--+`; arrows alone don't count.
- **`fmt`**: joins a paragraph's lines (not across hard breaks or `^^^`/`&&&` lines; blockquotes untouched), `*`/`+` bullets → `-` (task items found from the parent `List`, since comrak emits `TaskItem`), one blank line around ATX headings except right after frontmatter. Covers `specs/*.md` + open task files, same set as the `unformatted` warning.

## Dependencies

Keep the tree small; justify any addition in the change that adds it. Runtime: ~54 unique crates.

- `comrak` (defaults off) — Markdown AST with source positions; never used to re-render.
- `serde` + `toml` (config), `serde_json` (JSON), `toon-format` (TOON, defaults off — its `cli` feature pulls clap, ratatui, syntect).
- `chrono` (`clock`, `std`) — local dates.
- `glob` 0.3 (zero deps) — scope matching with `require_literal_separator`; `globset` rejected (regex + aho-corasick).
- `clap`, `thiserror`. Frontmatter is a hand-written flat-YAML parser; no YAML crate.
- Git: the `git` CLI, read-only; no `git2`/`gix` (C build, large tree, can diverge from the user's git). Revisit only for bulk history access or no-git environments; then prefer `gix`.

## Gotchas / Knowledge

- `skills/SKILL.md` and `references/examples.md` are embedded via `include_str!` (`src/skill.rs`); editing them changes the installed skill on reinstall. `skill check` compares versions, so without a version bump an updated bundle shows as "LOCALLY MODIFIED".
- `AGENTS.md` injection is the only append-aware path in `init`; all other files are write-if-missing.
- `init` and `skill` are still text-only; `--format json|toon` on them errors (`output::require_text`).
- `check::task_diagnostics` decides "archived" from the file's location, so the safety net checks `task done`'s moved file as archived.
- The git environment passes through unchanged so `--staged` inside a hook reads the hook's `GIT_INDEX_FILE`. E2e tests run every spawned command through `isolated()` (hook variables stripped, `GIT_CEILING_DIRECTORIES` set) and point `HOME` at a tempdir for skill tests.
- Test fixtures needing trailing spaces (Markdown hard breaks) use `concat!` with escapes — editors strip them from multi-line literals.
- comrak counts `1. [x]` as a task item (GFM); `md` counts only bullet-list task items.
- Log diagnostics have no line numbers (`LogEntry` doesn't keep its source line).
- Task lookup accepts the full id, the sequence number (`1`, `0001`), or the slug. Broken task files are warnings, and their number stays reserved.
- `TaskId` and `Status` serialize as strings; TOON quotes ids that start with digits.
- This repo dogfoods its own `specs/` and `specdev.toml` (`forbid_tables = true`).
