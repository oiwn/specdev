# Roadmap

Committed future direction, in rough priority order. The task pipeline ("Dark Factory", design in `specs/factory.md`) is implemented through `task done`, `check`, the quality gate, and `fmt` — see CHANGELOG.md 2026-10-03 to 2026-10-05.

## Next — 0.4.0 release: trust + papercuts

From real-session friction (#15–#21). Six tasks, then the release; issues #3–#8 and #10 closed as shipped or superseded.

- **fmt hardening** (#20 item 2, #21 item 6): `fmt` rejects explicit paths outside `specs/` + open task files; `fmt --check` exits 1 without writing, for CI/hooks.
- **ctx header invariant** (#15): `check` flags a `ctx.md` that doesn't start with `# Current Task Context` (warning, like `unformatted`).
- **CHANGELOG placement** (#20 item 4): `task done` inserts above the newest dated entry — the file header already promises "newest at the top"; rule documented in overview.
- **Scope ergonomics** (#20 items 3+6, #21 item 7, #16): `scope rm` takes an optional `--reason`; out-of-scope errors list every candidate task; skill documents `scope rm`.
- **task approve** (#17, #20 item 5): human-run `specdev task approve <id> --note "..."` recorded in `## Log`; `advance` refuses draft → ready without it. Strict by default, no config knob.
- **Explicit-only task creation** (#19): skill default "no task or record unless the user asks"; `[task] explicit_only` in config (false restores old behavior); out-of-scope downgrades to warning while true.
- **Release 0.4.0**: version bump, CHANGELOG, `skill install --local` to re-dogfood, close #15–#17, #19, #20 (#21 stays open).

## Later

- **Parallel tasks** (#21 item 5): multiple in-progress tasks with non-overlapping scopes, `specdev plan` computing waves for an external orchestrator (see "Future: parallel execution" in `factory.md`). Needs factory.md design first.
- **Task sizing overrides** (#18, #21 item 8): per-task limit overrides or a `kind: design` task shape, so large working-manual tasks comply without weakening global defaults.
- **Done-task scope attribution** (#20 item 1): files of a `done` task become out-of-scope errors before the tree is committed; track a pending-commit scope or warn for unclaimed changes.
- **Spec hygiene suite** (#21 items 1–4): `spec rm`, internal link checker, `specdev audit`, spec frontmatter schema validation.
- **`--format json|toon` for `init` and `skill`** (then remove `output::require_text`), run as real tasks through the pipeline.
- **Examples expansion**: a full task lifecycle plus more routing examples in `references/examples.md`, drawn from real projects.
- **Log line numbers**: `log-*` diagnostics point at the offending `## Log` line (`LogEntry` keeps its source line).
- **Replay checks `source`/`depends`**: compare them with the logged `set` values.
- **Replay-side draft-approval gate**: once no pre-0.4.0 logs remain, have `replay` enforce `draft approved` for draft → ready too, not just `advance`.
- **Scan improvements**: surface ctx checkbox progress inside `scan` too, and group `^^^` remarks by file with a one-line health summary.
- **Freshness thresholds**: warn when `ctx.md` hasn't been touched in N days while a task is marked in-progress (possible stale task).
- **Tune `[quality]` defaults** from what real tasks hit.
