# Roadmap

Committed future direction, in rough priority order. The task pipeline ("Dark Factory", design in `specs/factory.md`) is implemented through `task done`, `check`, the quality gate, and `fmt` — see CHANGELOG.md 2026-10-03 to 2026-10-05.

## Next — 0.3.0 release

- **Skill rewrite** (`skills/SKILL.md`): the task workflow per stage, which commands own which state (frontmatter and `## Log` belong to commands; Plan, Findings, Review, Summary, and ticks are the agent's), the gates, the scope contract (`task scope add --reason` before touching a file), `check` at every transition and how to read its codes and refusals, `fmt`, `--format`. Keep the ctx/marker/routing parts; no tables.
- **README**: CLI sections for `check`, `task …`, `fmt`, `--format`; a short task-pipeline section; `specdev.toml` keys; soft wraps.
- **Examples**: a full task lifecycle in `references/examples.md` (file at draft, approved, done; the CHANGELOG entry).
- **Dogfood here**: `--format json|toon` for `init` and `skill`, run as two real tasks through the pipeline (then remove `output::require_text`); collect friction, including from use in other projects.
- **Tune `[quality]` defaults** from what real tasks hit.
- **Release 0.3.0**: version bump, CHANGELOG entry.

## Later

- **Log line numbers**: `log-*` diagnostics point at the offending `## Log` line (`LogEntry` keeps its source line).
- **Replay checks `source`/`depends`**: compare them with the logged `set` values.
- **Parallel tasks**: non-overlapping `scope`, `specdev plan` computing waves for an external orchestrator (see "Future: parallel execution" in `factory.md`).
- **Scan improvements**: surface ctx checkbox progress inside `scan` too, and group `^^^` remarks by file with a one-line health summary.
- **Freshness thresholds**: warn when `ctx.md` hasn't been touched in N days while a task is marked in-progress (possible stale task).
- **Templates / examples expansion**: more before/after routing examples in `references/examples.md`, drawn from real projects.
