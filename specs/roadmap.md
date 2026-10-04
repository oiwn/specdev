# Roadmap

Dark Factory track — design in `specs/factory.md`; build phases and their steps in `specs/ctx.md`.

## v0.3 — Task files (Phases 1–2)

- Foundation: comrak-backed Markdown outline, `specdev.toml` config, task types with flat-YAML frontmatter and per-task `## Log`, `--format text|json|toon`.
- `init` scaffolds `specdev.toml` and `specs/tasks/`; `task new|list|show|index`.

## v0.4 — Contract enforcement (Phases 3–4) — done 2026-10-04

- `specdev check`: task contract, log consistency, one active task, generated index.
- State commands: `task advance|block|set|scope` with one transition table.

## v0.5 — Verify, archive, quality (Phases 5–7)

- Scope check via the `git` CLI, read-only (`check`, and `check --staged` for pre-commit); `verify → review` gated on ticked Acceptance boxes. specdev runs no project commands. — done 2026-10-04 (Phase 5)
- `task done`: archive to `done/`, changelog entry, index.
- Quality gate metrics and thresholds; `specdev fmt` for soft-wrapped specs.

## v0.6 — Skill and docs (Phase 8)

- Skill rewritten around the staged task workflow; README, overview, examples; dogfood on real specdev tasks.

## Later

- **Parallel tasks**: non-overlapping `scope`, `specdev plan` computing waves for an external orchestrator (see "Future: parallel execution" in `factory.md`).

- **Scan improvements**: surface ctx checkbox progress inside `scan` too, and
  group `^^^` remarks by file with a one-line health summary.
- **Freshness thresholds**: warn when `ctx.md` hasn't been touched in N days
  while a task is marked in-progress (possible stale task).
- **`specdev skill` updates**: detect an out-of-date installed skill vs the
  bundled one and offer to reinstall.
- **Templates / examples expansion**: more before/after routing examples in
  `references/examples.md`, drawn from real projects.
