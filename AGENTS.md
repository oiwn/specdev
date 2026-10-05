# AGENTS.md

<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding.
<!-- END specdev -->

## Project

- specdev is a Rust CLI plus a bundled agent skill for managing human-readable, human-editable spec and context files and keeping them sorted.
- Architecture, pipeline rules, dependencies, and gotchas: `specs/overview.md`. Current task: `specs/ctx.md`. What's next: `specs/roadmap.md`. Multi-stage task pipeline design ("Dark Factory"): `specs/factory.md`.
- This repo dogfoods its own `specs/`; follow the specdev skill when editing them.

## Design principles

- **A tool for agents, not an orchestrator.** specdev never launches agents, never calls an LLM, never creates branches, commits, or PRs. Git is only ever read, by calling the `git` CLI — no `git2`/`gix` bindings.
- **Agent-agnostic.** No code or docs may assume a specific agent, harness, or CI.
- **Runs no project commands.** specdev never executes tests, lints, or builds; the agent does and records the result (ticked Acceptance boxes).
- **Platforms: macOS first, then Linux.** No Windows support; don't add Windows-specific code paths.
- **Structured state goes through commands; prose stays agent-edited.** Commands rewrite only what they own (frontmatter, `## Log`, generated indexes) and must keep the rest of a file byte-for-byte.
- **Hardcoded where semantics matter.** Task statuses and stages are fixed enums, not config.

## Rust conventions

- Edition 2024. Let-chains (`if let ... && ...`) are fine and already used.
- **No `mod.rs`.** Use the modern module layout: `src/task.rs` declares `mod frontmatter;` and the child lives in `src/task/frontmatter.rs`.
- Errors: the crate-level `Error` enum (`thiserror`) in `src/main.rs`; add variants there rather than per-module error types unless a module clearly needs its own.
- Unit tests inline in each module (`#[cfg(test)] mod tests`); end-to-end binary tests in `tests/cli.rs` using `tempfile`.
- Keep dependencies minimal; justify each new crate in the task's Findings. The approved list and the reasons live in `specs/overview.md` (Dependencies).
- Reuse shared parsers instead of re-scanning Markdown per command (see the `md` module in the build plan).

## Writing specs and docs

- No Markdown tables and no ASCII diagrams — they read badly in terminal editors. Use bullets, numbered steps, and short sections.
- Soft wraps: one line per paragraph or bullet; don't hard-wrap.
- Code blocks only for literal content: file formats, commands, config, type sketches.
- Never write `^^^` (the human's marker); answer with `&&&`. Compress only when asked.

## Working rules

- Edit files with patch/edit tools, not ad-hoc scripts (python, sed) — edits must be reviewable.
- Don't rename the `specs/ctx.md` task title, archive it, or edit `CHANGELOG.md` without explicit user intent.
- If an edit is limited to named files, record follow-ups in `specs/ctx.md` instead of touching other files.

## Gotchas

- `skills/SKILL.md` and `references/examples.md` are embedded via `include_str!` (`src/skill.rs`). Changes reach users only after `cargo install --path .` and `specdev skill install`; until then `specdev skill check` reports the installed skill as stale.
- The `<!-- BEGIN/END specdev -->` block above is managed by `specdev init`; don't edit it by hand.
- All Markdown analysis goes through `src/md.rs` (comrak, code-block aware). Don't add line-based heading or checkbox scanning elsewhere.
- Task files are rewritten only through `Task::render()`: frontmatter and `## Log` are regenerated, the body between them is kept byte-for-byte.

## Verification

Before declaring a task done, run:

```
cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings
```

The pre-commit hooks in `prek.toml` run the same plus `typos` and `gitleaks`.
