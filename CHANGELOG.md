# Changelog

Completed tasks, moved here from `specs/ctx.md` once every checkbox in their
plan is done. Newest entry at the top, dated. Freeform — one block per finished
task. This coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->

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
