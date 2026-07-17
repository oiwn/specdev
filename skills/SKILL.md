---
name: specdev
description: >
  Use when working in a project that uses specification-driven development.
  Start from specs/overview.md and specs/ctx.md, treat specs as the active
  task context, keep ctx.md narrow (a checkbox plan for the current task
  only), route content to its right home (changelog/roadmap/ideas/overview),
  address user remarks marked with ^^^, answer with &&&, and keep specs
  current while planning or implementing. Trigger when the user asks to work
  on specs, continue from specs, address remarks, compress specs, or when
  entering a project with a specs/ directory.
---

Teaches the agent to work with project specs as the primary planning,
coordination, and memory surface. Specs are Markdown files in a `specs/`
directory. The agent reads them, updates them, and uses them as context
before writing code.

## Session startup

When entering a project, or when the user says "continue from specs":

1. Read `specs/overview.md` (durable project backdrop + gotchas).
2. Read `specs/ctx.md` (the current task — narrow, with a checkbox plan).
3. If `ctx.md` has no active task (just the `# Current Task Context` header),
   ask the user what to work on, or take the next item from `specs/roadmap.md`
   (see "The working loop"). The header-only state is valid, not an error.
4. Read `specs/ideas.md` when the user asks for brainstorming or roadmap work,
   or when `ctx.md` points there.
5. Do not read `specs/cleanup.md` unless the user asks — it is not session
   context.
6. Identify any `^^^` remarks in relevant specs (see below).

## Content routing — put each thing in its right home

Content is routed by *when* it lives. This is what keeps `ctx.md` clean and is
the main fix for agents that leave stale scope, deferred items, and gotchas
piling up in the current task.

| Content                                   | Home                          |
|-------------------------------------------|-------------------------------|
| The current task + its checkbox plan      | `specs/ctx.md`                |
| A task whose checkboxes are all done      | `CHANGELOG.md` (move it here) |
| Committed future direction                | `specs/roadmap.md`            |
| Noted but not-now ideas                   | `specs/ideas.md`              |
| Architecture, data flow, durable knowledge| `specs/overview.md`           |
| Gotchas, quirks, DB/engine knowledge      | `specs/overview.md` (Gotchas) |
| Code smells, refactors spotted            | `specs/cleanup.md`            |

Note: `CHANGELOG.md` and `AGENTS.md` live at the project **root**, not under
`specs/`. `CHANGELOG.md` coexists with any conventional changelog already there.

**Forbidden inside `ctx.md`:** `Deferred` sections, `Roadmap pointer` lines,
`Gotchas`/`Quirks` blocks, architecture essays, and "completed work is not
repeated here" meta-notes. Route them to their home above instead.

## The working loop

Content matures left-to-right, then archives:

```
ideas.md  ->  roadmap.md  ->  ctx.md  ->  CHANGELOG.md
uncommitted   committed      active      done
```

- An `ideas.md` note moves to `roadmap.md` when you decide to do it.
- A `roadmap.md` item moves to `ctx.md` when you pick it up.
- A `ctx.md` task moves to `CHANGELOG.md` when every checkbox is `[x]`.

When `ctx.md` has no active task, the next one comes from `roadmap.md`. This is
the intended loop — keep each file to its stage.

## ctx.md — the current task (soft template)

`ctx.md` is narrow: only what is needed to implement the **current** task. It is
not a broad scope document. Recommended shape:

```md
# Current Task Context: <one line>
State: <not started | in progress | blocked>
## Plan
- [ ] step one
- [ ] step two
## Context
<only what the current unchecked step needs>
## Next
<the immediate next action>
```

- The file always starts with the `# Current Task Context` header. When there
  is an active task, append `: <one-line task>` to it — the header doubles as
  the task title and as the description `specdev list` shows.
- The **no-task state is header-only** (`# Current Task Context`), not a blank
  file.
- `## Plan` is the core: an ordered checklist of the task's steps.
- Tick boxes (`[ ]` -> `[x]`) as you implement. Progress is visible across
  sessions this way.
- `## Context` holds only what the *current unchecked step* needs — not the
  whole task's scope, and never deferred/roadmap/gotcha content.

## Definition of done + archive rule

When **every** checkbox in `ctx.md` is `[x]`, the task is done:

1. Move the task into `CHANGELOG.md` as a dated block at the top (newest first).
2. Reset `ctx.md` to the header-only state (`# Current Task Context`), or to the
   next single task.

This archival is **agent-driven, on user request**. There is no destructive CLI
command for it, by design. `specdev status` will surface the signal that a task
looks done (all boxes checked) but has not been archived yet.

## Core spec files

- **`specs/overview.md`** — durable project overview (architecture, data flow,
  public surfaces) **and the Gotchas/Knowledge home**. Broad; no task detail.
- **`specs/ctx.md`** — current task (narrow checkbox plan, above). Header-only
  (`# Current Task Context`) when no task is active.
- **`specs/roadmap.md`** — committed future direction (what we are doing next).
  Promote items here from `ideas.md` when decided.
- **`specs/ideas.md`** — uncommitted backlog of possibilities. Not a plan.
- **`specs/cleanup.md`** — code-smell notes (`##` heading per entry, 1-3
  sentences). Read on demand only.
- **`CHANGELOG.md`** (root) — completed tasks (the archive destination above).
- **`AGENTS.md`** (root) — agent entry point: a **minimal pointer** to this
  skill, auto-added by `specdev init` (created if absent, appended idempotently
  to an existing file). This skill is the full reference.

Projects may have additional spec files for features, architecture decisions,
etc. The files above are the stable workflow; other specs use whatever
structure fits.

## CLI tools

The `specdev` binary scaffolds and inspects specs. It never mutates spec content
— archival, compression, and edits are agent-driven.

- `specdev init` — scaffold `specs/` (overview, ctx, roadmap, ideas, cleanup) +
  root `CHANGELOG.md`/`AGENTS.md`. `AGENTS.md` is append-aware (created if
  absent, else the specdev pointer is appended idempotently).
- `specdev scan` — list `^^^`/`&&&` remarks across `specs/`, open vs resolved.
- `specdev status` — health: marker counts, ctx task progress (checked/total),
  forbidden content in ctx, an archive nudge when all steps are checked, and
  missing-root-file warnings.
- `specdev list` — first header + line count per spec. `--stats` adds heading
  counts by level, checkboxes, open `^^^`, and word count.
- `specdev skill install [--local]` — install this skill, globally
  (`~/.agents/skills/specdev/`) or locally (`.agents/skills/specdev/`).

## Marker protocol

`^^^` and `&&&` are inline markers used **only inside spec files** to
distinguish user remarks from agent answers during spec editing.

- `^^^` at line start = user remark, question, correction, or TODO.
- `&&&` at line start = agent answer, resolution, or clarification addressing a
  nearby `^^^` remark.

Rules:

- When addressing a `^^^` remark, add an adjacent `&&&` answer. Do not delete
  the original `^^^` line.
- If a remark is unresolved, keep it marked with `^^^` — no `&&&`.
- Do not use these markers outside spec files (not in code, not in
  conversation).

## Addressing remarks

When the user asks to work on a spec file with remarks:

1. Read the spec file.
2. Find all `^^^` remarks.
3. For each remark, either:
   - Address it with an adjacent `&&&` answer, or
   - Leave it unresolved with `^^^` if you need user input.
4. Do not proceed to code edits while relevant `^^^` remarks remain
   unresolved. Address the remarks first or ask for clarification.
5. Reply with:
   - what was addressed
   - what remains open
   - what the next task is

## Context compression

When the user asks to compress a spec:

1. Read the spec file.
2. Identify all `^^^`/`&&&` pairs where the issue is resolved.
3. Rewrite the spec sections according to the dialogue — fold the decisions
   into the surrounding prose, removing both markers and the back-and-forth.
4. Preserve any unresolved `^^^` remarks as-is.
5. The rewritten spec should read as a clean document reflecting the final
   state of decisions, not a transcript of the discussion.

Compression is only by explicit user request. Never auto-compress. See
`references/examples.md` for before/after examples.

## Behavior

**Do:**

- Treat specs as the source of task context when the user points to them.
- Start with `overview.md` and `ctx.md` when entering a project.
- Keep `ctx.md` narrow — route content to its right home per the table above.
- Update specs before implementation when the task is ambiguous or strategic.
- Proceed to implementation when the spec makes the next step clear.
- Preserve user wording when it carries useful intent.
- Preserve addressed `^^^`/`&&&` pairs until the user asks to compress.
- Keep unresolved issues visible.
- Compact stale text when it no longer reflects reality.
- Tick `ctx.md` checkboxes as you implement, and archive the task when all are
  `[x]`.

**Do not:**

- Silently delete `^^^` remarks.
- Delete `^^^` remarks after adding `&&&` answers (the dialogue stays until
  compression).
- Mark a remark as addressed unless it has actually been addressed with `&&&`.
- Bury open questions in prose.
- Overwrite user-authored nuance with a generic plan.
- Start coding while active, relevant `^^^` remarks remain unresolved.
- Put deferred items, roadmap pointers, gotchas, or architecture essays in
  `ctx.md`.

## Gotchas

- `^^^`/`&&&` markers are for spec editing only — never in code, comments, or
  conversation.
- `ctx.md` should represent reality. If it is stale, update it before starting
  work.
- The no-task state is `ctx.md` with only the `# Current Task Context` header —
  that is valid, not an error.
- Do not read `cleanup.md` unless the user explicitly asks — it is not session
  context.
- Do not force every spec into the same structure. The core workflow is stable;
  other specs are flexible.
- `roadmap.md` is committed direction; `ideas.md` is uncommitted. Don't blur
  them.
