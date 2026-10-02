# Roadmap

Dark Factory track — design in `specs/factory.md`.

## v0.3 — Task protocol

- `specs/tasks/<id>.md` format (frontmatter: status, touches, reads, depends,
  branch, claimed_by) + `specs/tasks/done/`.
- Skill section for the planner and worker roles, incl. hot-file discipline.
- `specdev task new|list|show`; task board summary in `specdev status`.

## v0.4 — Planner check

- `specdev plan --check`: pairwise `touches` glob overlap, dependency cycles,
  ready tasks with unmet deps; prints proposed waves.

## v0.5 — Scope guard

- `specdev task verify <id> --diff <base>`: PR diff ⊆ `touches` + own task file.
- Reusable CI step that runs it plus the task's acceptance commands.

## v0.6 — opencode dispatch

- Dispatcher workflow (`workflow_dispatch` + `schedule`, single concurrency
  group): claim up to N non-overlapping ready tasks via commit to main, hand
  each to opencode.

## Later

- **Autonomous planner loop (L3)**: planner picks roadmap items and drafts
  tasks without prompting; humans gate only promotion and merge.
- **Merge queue / auto-rebase** for task PRs that go stale against main.

- **Scan improvements**: surface ctx checkbox progress inside `scan` too, and
  group `^^^` remarks by file with a one-line health summary.
- **Freshness thresholds**: warn when `ctx.md` hasn't been touched in N days
  while a task is marked in-progress (possible stale task).
- **`specdev skill` updates**: detect an out-of-date installed skill vs the
  bundled one and offer to reinstall.
- **Templates / examples expansion**: more before/after routing examples in
  `references/examples.md`, drawn from real projects.
