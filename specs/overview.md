# specdev — overview

specdev is a small Rust CLI plus a bundled agent skill. The CLI scaffolds, inspects, and (for tasks) manages structured spec state; the skill teaches the spec workflow to any coding agent. This file overviews **developing specdev itself** — the workflow the skill teaches is documented in `skills/SKILL.md`, not here.

## Architecture

- `src/main.rs` — clap CLI: `init`, `scan`, `status`, `list`, `skill`, `task`; global `--format text|json|toon`; crate-level `Error`.
- `src/md.rs` — the shared Markdown parser (comrak): headings, checkboxes, sections, code-block awareness. All Markdown analysis goes through it.
- `src/diag.rs` — `Diagnostic` (file, line, severity, stable code, message).
- `src/output.rs` — `Format` and the `Report` trait; renders every migrated command as text, JSON, or TOON.
- `src/config.rs` — `specdev.toml` (task contract, pipeline, acceptance, quality) with built-in defaults.
- `src/task.rs` — `Task`, `Status`, `Stage`; parse and `render()` (frontmatter + `## Log` regenerated, body byte-for-byte).
  - `src/task/id.rs` — `TaskId` (`0007-slug`).
  - `src/task/frontmatter.rs` — hand-written flat-YAML subset parser.
  - `src/task/log.rs` — `## Log` line format.
  - `src/task/store.rs` — `TaskStore`: loads `specs/tasks/` and `done/`, lookup, queue order, `_index.md`.
  - `src/task/cmd.rs` — `task new|list|show|index`.
- `src/init.rs` — scaffolds `specs/` + root `CHANGELOG.md`/`AGENTS.md` (AGENTS.md is append-aware, idempotent via `<!-- BEGIN/END specdev -->`).
- `src/scan.rs` — parses `^^^`/`&&&` remark markers.
- `src/status.rs` — health: markers, ctx checkbox progress, forbidden content, archive nudge, missing-file warnings.
- `src/list.rs` — `list` and `list --stats`.
- `src/skill.rs` — installs `skills/SKILL.md` + `references/examples.md`.
- `skills/SKILL.md` — the shipped agent skill (workflow documentation).
- `references/examples.md` — before/after examples, bundled with the skill.
- `tests/cli.rs` — end-to-end binary tests.

Direction ("Dark Factory", design in `specs/factory.md`): the CLI owns *structured* spec state — task frontmatter, status/stage transitions, the per-task `## Log`, generated indexes — through commands, and `specdev check` enforces it. Prose stays agent-edited. specdev never writes to git and runs no project commands; it only reads changed-file lists by calling the `git` CLI (no bindings). Build order: `specs/ctx.md`.

## Build & test

`cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings`

## Public surfaces

- `specdev` binary (via `cargo install`).
- Installable skill (`specdev skill install`) — global at `~/.agents/skills/specdev/`, or local at `.agents/skills/specdev/`.

## Gotchas / Knowledge

- `skills/SKILL.md` and `references/examples.md` are embedded via `include_str!` (`src/skill.rs`); editing them changes the installed skill on reinstall.
- `AGENTS.md` injection is the only append-aware path in `init`; all other files are write-if-missing.
- `init`, `scan`, `status`, and `skill` are still text-only; `--format json|toon` on them errors (`output::require_text`). `scan` and `status` migrate to `Report` in Phase 3.
- This repo dogfoods its own `specs/`.
