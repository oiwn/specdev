# Changelog

Completed tasks, moved here from `specs/ctx.md` once every checkbox in their
plan is done. Newest entry at the top, dated. Freeform — one block per finished
task. This coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->

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
