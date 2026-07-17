# Plan: specdev time-axis redesign + content routing

Status: **agreed, ready to execute** (blocked by plan mode). Exit plan mode to
run it.

## Problem

Third-party agents (codex, claude, glm, deepseek) leave `specs/ctx.md` cluttered
with stale scope, deferred items, roadmap pointers, and gotchas. Root cause:
nothing tells agents *where* each kind of content belongs, and there is no
mechanical definition of "done" to force cleanup.

## Fix (three non-destructive pieces)

1. **Skill + AGENTS.md** teach a content-routing table + a hard archive rule.
2. **`specdev status`** surfaces drift: ctx checkbox progress, forbidden
   content present, all-checked-but-not-archived.
3. **ctx.md becomes a checkbox plan** — a shared, visible definition of done.

CLI only inspects + scaffolds. Content mutations stay agent-driven (consistent
with the existing "no compress CLI command" decision). **No destructive
`specdev done` command** (user rejected it).

## Mental model: time axis

```
CHANGELOG.md (root)  <-  specs/ctx.md  ->  specs/roadmap.md  ->  specs/ideas.md
   past done            present task      future committed     future uncommitted
```
`specs/overview.md` = stable backdrop + Gotchas/Knowledge home.
`specs/cleanup.md` = code-smell notes (unchanged), read on demand.

## Locked decisions

- `CHANGELOG.md` at **project root**, freeform dated entries, coexists with any
  existing changelog.
- `ctx.md`: soft template (Task / State / **Plan (checkboxes)** / Context /
  Next) + strong norm against forbidden content. Agent writes it; no
  scaffolder command.
- No `specdev done` — archival is agent-driven, on user request.
- Drop v0.1 history from ctx; CHANGELOG starts fresh.
- Scope this session: **all four** (skill + CLI + dogfood + docs).

---

## Execution steps

### C. Dogfood specdev's own repo

- **Create `CHANGELOG.md`** (root): header + format note + commented entry
  template. Fresh, no v0.1 entry.
- **Create `AGENTS.md`** (root): declares specdev usage; session startup;
  content-routing table; ctx.md soft template; definition-of-done + archive
  rule; marker protocol; verification commands. This file is also the template
  `init` writes.
- **Create `specs/roadmap.md`**: v0.2 milestone (this redesign, checkboxes),
  a "Later" section, pointer to `ideas.md`.
- **Rewrite `specs/ctx.md`**: narrow checkbox-plan task for THIS redesign
  (Task / State / Plan / Context / Next). At the very end, after all boxes are
  ticked, archive it into `CHANGELOG.md` and leave `ctx.md` empty to model the
  workflow.
- **Rewrite `specs/overview.md`**: clean project overview (no details) + a
  Gotchas/Knowledge section. Removes the old resolved `^^^`/`&&&` skill-draft
  content (decisions folded into new SKILL.md / overview).

### A. Skill rewrite — `skills/SKILL.md`

- Update core-files list: add `roadmap.md`; note `CHANGELOG.md` + `AGENTS.md`
  live at project root.
- Add **content-routing table** (time axis).
- Add **ctx.md soft template** + checkbox plan + forbidden-content list.
- Add **archive rule**: all `[x]` -> agent moves task to CHANGELOG, wipes ctx;
  CLI does not do this.
- Distinguish `roadmap.md` (committed) vs `ideas.md` (uncommitted).
- State `overview.md` holds Gotchas/Knowledge.
- Keep `^^^`/`&&&` protocol and compression sections.

### B. CLI changes — `src/`

**`init.rs`**
- Add `ROADMAP_TEMPLATE` ("# Roadmap\n\nCommitted future direction...\n").
- Scaffold `specs/roadmap.md` (write_if_missing).
- After scaffolding specs, also create root `CHANGELOG.md` (no overwrite) and
  root `AGENTS.md` (no overwrite) via the same write_if_missing helper, paths
  relative to the specs parent.
- Update the "already initialized" completeness check to include roadmap.md.
- Update tests: assert roadmap.md created; assert CHANGELOG.md / AGENTS.md
  created at parent root; update does_not_overwrite and fills_missing tests.

**`status.rs`**
- Add `core_files` entry `roadmap.md`.
- New helpers:
  - `count_checkboxes(content) -> (done, total)` parse `- [x]` / `- [ ]`.
  - `find_forbidden(content) -> Vec<&str>` detect forbidden headings in ctx
    (`Deferred`, `Roadmap pointer`, `Gotchas`, `Quirks`) — case-insensitive
    `## ` heading match.
- In ctx.md row: when checkboxes exist, print `N/M steps`.
- After file table, if ctx.md has checkboxes and all are `[x]` and ctx is
  non-empty -> warn "task looks done, consider archiving to CHANGELOG.md".
- If forbidden content found in ctx -> warn listing the headings.
- Warn if `CHANGELOG.md` / `AGENTS.md` missing at root.
- Tests: checkbox parsing, forbidden detection, all-done warning, missing
  root files warning.

**`main.rs`**: no new command (per decision).

**`scan.rs`**: unchanged.

### D. Docs

- **`README.md`**: update core-files list (add roadmap.md, mention root
  CHANGELOG.md + AGENTS.md); document `status`'s new checks (step progress,
  forbidden content, archive nudge, missing root files). Keep `scan`, `skill
  install` sections.
- **`references/examples.md`**: add "Example 5: routing a cluttered ctx.md"
  showing the user's bad ctx (portal overview page) before -> correctly routed
  after (task stays in ctx, deferred->ideas/roadmap, quirks->overview). Keep
  existing examples (tests require "Example 1" and "Compression" tokens — keep
  those headings).

### Verify

- `cargo build`
- `cargo test`
- `cargo clippy` (must be clean)

---

## Dogfood ctx.md content (to write first, archive last)

```md
# Task: specdev time-axis redesign + content routing

State: in progress (started 2026-07-14)

## Plan
- [ ] Create root CHANGELOG.md + AGENTS.md; add specs/roadmap.md
- [ ] Rewrite skills/SKILL.md with routing table + ctx soft template
- [ ] init.rs: scaffold roadmap.md + root CHANGELOG.md/AGENTS.md
- [ ] status.rs: ctx checkbox progress + forbidden-content warnings
- [ ] Update README.md + references/examples.md
- [ ] Verify: cargo build/test/clippy

## Context
- CLI inspects/scaffolds only; no destructive `done` command (mutations are
  agent-driven, like compression).
- ctx.md soft template = Task / State / Plan (checkboxes) / Context / Next.
- Forbidden in ctx.md: Deferred sections, roadmap pointers, gotchas,
  architecture essays.

## Next
Start with the dogfood files (CHANGELOG/AGENTS/roadmap), then the skill.
```

After all boxes are `[x]`: move this block to `CHANGELOG.md` under
`## 2026-07-14 — specdev time-axis redesign + content routing`, then wipe
`ctx.md` to empty.
