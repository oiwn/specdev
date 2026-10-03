# specdev — overview

specdev is a small Rust CLI plus a bundled agent skill. The CLI scaffolds and
inspects project specs; the skill teaches the spec workflow to any coding agent.
This file overviews **developing specdev itself** — the workflow the skill
teaches is documented in `skills/SKILL.md`, not here.

## Architecture

- `src/main.rs` — clap CLI: `init`, `scan`, `status`, `list`, `skill`.
- `src/init.rs` — scaffolds `specs/` + root `CHANGELOG.md`/`AGENTS.md`
  (AGENTS.md is append-aware, idempotent via `<!-- BEGIN/END specdev -->`).
- `src/scan.rs` — parses `^^^`/`&&&` remark markers.
- `src/status.rs` — health: markers, ctx checkbox progress, forbidden content,
  archive nudge, missing-file warnings.
- `src/list.rs` — `list` and `list --stats`.
- `src/skill.rs` — installs `skills/SKILL.md` + `references/examples.md`.
- `skills/SKILL.md` — the shipped agent skill (workflow documentation).
- `references/examples.md` — before/after examples, bundled with the skill.
- `tests/cli.rs` — end-to-end binary tests.

Today the CLI only inspects and scaffolds; archival, compression, and edits are agent-driven (taught by the skill).

Direction ("Dark Factory", design in `specs/factory.md`): the CLI takes ownership of *structured* spec state — task frontmatter, status/stage transitions, the per-task `## Log`, generated indexes — through commands, and `specdev check` enforces it. Prose stays agent-edited. specdev never writes to git; it only reads changed-file lists by calling the `git` CLI (no bindings).

## Build & test

`cargo build && cargo test && cargo clippy -- -D warnings`.

## Public surfaces

- `specdev` binary (via `cargo install`).
- Installable skill (`specdev skill install`) — global at
  `~/.agents/skills/specdev/`, or local at `.agents/skills/specdev/`.

## Gotchas / Knowledge

- `skills/SKILL.md` and `references/examples.md` are embedded via `include_str!`
  (`src/skill.rs`); editing them changes the installed skill on reinstall.
- `AGENTS.md` injection is the only append-aware path in `init`; all other
  files are write-if-missing.
- This repo dogfoods its own `specs/`.
