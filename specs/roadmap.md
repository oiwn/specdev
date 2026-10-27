# Roadmap

Committed future direction, in rough priority order. The task pipeline ("Dark Factory", design in `specs/factory.md`) is implemented through `task done`, `check`, the quality gate, and `fmt` — see CHANGELOG.md 2026-10-03 to 2026-10-05.

## Next — distribution, then parallel work

0.4.0 shipped (trust + papercuts: approve gate, explicit-only tasks, grades, skill-stale, config reference — CHANGELOG 2026-10-25). Next up, install without a Rust toolchain, then the parallel-tasks design.

- **Nix flake** (task 0011): `flake.nix` + lock at the repo root; `nix run github:oiwn/specdev`.
- **Homebrew tap** (task 0012): release workflow builds and attaches per-target tarballs + checksums; formula drafted here for the user's `oiwn/homebrew-tap` repo; `brew install oiwn/tap/specdev`. Consumes the "Release binaries" idea.
- **Parallel-tasks design** (#21 item 5): a design task in `ctx.md` extending `factory.md` — entity/component registry (`[entity.cli]`, future `[entity.server]`), non-overlapping in-progress tasks, `plan`/graph waves for an external orchestrator. Entity shape (first-class scope unit vs path alias) decided inside the design.

## Later

- **Parallel tasks** (#21 item 5): multiple in-progress tasks with non-overlapping scopes, `specdev plan` computing waves for an external orchestrator (see "Future: parallel execution" in `factory.md`). Design is queued in Next; implementation follows it.
- **Entity registry**: `[entity.<name>]` components (cli today, server someday) as scope units and per-entity defaults — shaped by the parallel-tasks design above.
- **Tap bump automation**: the release workflow updates `oiwn/homebrew-tap` after publishing, once the tap exists and the manual bump has proven the formula.
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
