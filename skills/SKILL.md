---
name: specdev
description: >
  Use when working in a project that uses specification-driven development.
  Start from specs/overview.md and specs/ctx.md, treat specs as the active
  task context, keep ctx.md concise and decision-relevant (a checkbox plan for
  the current task), route content to its right home
  (changelog/roadmap/ideas/overview), address user remarks marked with ^^^,
  answer with &&&, and keep specs current while planning or implementing. Trigger when the user asks to work
  on specs, continue from specs, address remarks, compress specs, or when
  entering a project with a specs/ directory.
---

Teaches the agent to work with project specs as the primary planning,
coordination, and memory surface. Specs are Markdown files in a `specs/`
directory. The agent reads them, updates them, and uses them as context
before writing code.

## Invariants

Hold these at all times. They exist because agents silently drift on exactly
these points.

- **Task identity is immutable by default.** The `ctx.md` title is the active
  task. Don't rename, narrow, replace, or archive it without explicit user
  intent.
- **Never bulk-replace an active `ctx.md`.** Make the smallest surgical edit
  that achieves the change. Wholesale overwrite or delete requires explicit
  reset/archive/compress intent.
- **Archival requires explicit intent.** All steps done is necessary, not
  sufficient — the titled task must be finished AND the user must explicitly
  ask to archive/close/reset. When uncertain, leave `ctx.md` active and ask.
- **Preserve decision-relevant state.** "Concise" means concise and
  decision-relevant, not minimal. Keep findings, decisions, open questions, and
  constraints that remain relevant to the next unchecked step.

## Session startup

When entering a project, or when the user says "continue from specs":

1. Read `specs/overview.md` (durable project backdrop + gotchas).
2. Read `specs/ctx.md` (the current task — concise, with a checkbox plan).
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

`ctx.md` is **concise and decision-relevant, not minimal** — enough for the
agent to execute the current task autonomously, without stale scope, deferred
items, or gotchas (route those to their home files). Recommended shape:

```md
# Current Task Context: <one line>
State: <not started | in progress | blocked>
## Plan
- [ ] step one
- [ ] step two
## Findings      # optional — concise research/results when the next step is evaluate/discuss
## Context
<what the current unchecked step needs to act on>
## Next
<the immediate next action>
```

- The file always starts with `# Current Task Context`; append `: <task>` when
  a task is active. The header **is the task identity** — don't rename it
  without explicit user intent. The no-task state is header-only.
- `## Plan` is the ordered checklist of the task's steps. A step is a **child**,
  not the task — completing one step is never permission to reset or archive.
- `## Findings` (optional): compact accumulated research, decisions, results.
  Use it when the next step is evaluate/discuss/select. Keep it concise and
  decision-relevant; never delete it without explicit reset intent.
- `## Context` holds what the *current unchecked step* needs to act on.
- Tick boxes (`[ ]` -> `[x]`) as you implement. Prefer **surgical edits** —
  re-read the target section, then patch it; never rewrite or bulk-delete an
  active `ctx.md` to change one thing.

## Task identity + archival

**Task identity is immutable by default.** The `ctx.md` title is the active
task. Don't rename, narrow, replace, or archive it because a step completed or
because the user praised one piece of work. "X works; let's decide what's next"
confirms X, not the parent task.

**Archive only when ALL three hold:**

1. Every checkbox in the active task's plan is `[x]`; **and**
2. The titled task itself — not merely one child step — is finished; **and**
3. The user explicitly asks to archive/close/reset, or explicitly states the
   named task is finished.

All three are required. "All steps checked" is necessary, not sufficient, and
`specdev status`'s archive nudge means *confirm with the user* — it is **not**
permission to auto-archive. When uncertain, leave `ctx.md` active and ask.

To archive once the gate holds: move the task into `CHANGELOG.md` as a dated
block at the top (newest first), then reset `ctx.md` to header-only or to the
next single task. Archival is agent-driven; there is no destructive CLI command
for it, by design.

## Core spec files

- **`specs/overview.md`** — durable project overview (architecture, data flow,
  public surfaces) **and the Gotchas/Knowledge home**. Broad; no task detail.
- **`specs/ctx.md`** — current task (checkbox plan, above). Header-only
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

- The agent never writes `^^^`. That marker belongs exclusively to the human.
  The agent writes `&&&` answers.
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

Compress only on **explicit user request** — e.g. "compress / fold / condense /
clean up the `^^^`/`&&&` dialogue." A general "rewrite this section for clarity"
is **not** a compress request. Never auto-compress.

To compress:

1. Read the spec file.
2. Identify the resolved `^^^`/`&&&` pairs.
3. Fold each resolved decision into the surrounding prose, removing the markers
   and the back-and-forth. **Keep each pair adjacent to the assertion it
   concerns — never relocate resolved markers into a detached Q&A transcript**
   (e.g. at the end of the file).
4. Preserve unresolved `^^^` remarks as-is.
5. The result reads as clean prose reflecting final decisions, not a transcript.

See `references/examples.md` for before/after examples.

## Before mutating a spec

Before editing a spec, classify the intended change as exactly one of:

- **tick a child step** — `[ ]` -> `[x]`; nothing else.
- **update findings/context** — add or refine decision-relevant state for the
  current task.
- **add a future idea** — to `ideas.md` or `roadmap.md`.
- **compress** — fold resolved `^^^`/`&&&` dialogue into prose.
- **archive** — move a finished task to `CHANGELOG.md` (requires the archive
  gate above).

State that classification in one line before editing. Some operations always
require **explicit user intent** (never infer them): changing the task title,
resetting or archiving `ctx.md`, editing `CHANGELOG.md`, deleting findings, or
compressing. When the change type is ambiguous, ask before editing.

Make the **smallest edit** that achieves the classified change: re-read the
target section, then surgical-patch it. Never rewrite a whole section to change
one line.

## Behavior

**Do:**

- Treat specs as the source of task context when the user points to them.
- Start with `overview.md` and `ctx.md` when entering a project.
- Keep `ctx.md` concise and decision-relevant — route other content to its home
  per the table above.
- Update specs before implementation when the task is ambiguous or strategic.
- Proceed to implementation when the spec makes the next step clear.
- Preserve user wording when it carries useful intent.
- Preserve addressed `^^^`/`&&&` pairs until the user asks to compress.
- Keep unresolved issues visible.
- Classify the edit and make the smallest surgical patch; ask before destructive
  ops (title change, archive/reset, CHANGELOG edit, delete findings, compress).
- Tick `ctx.md` checkboxes as you implement; archive only at the explicit user
  request, per the archive gate.

**Do not:**

- Silently rename, narrow, replace, or archive the task title.
- Bulk-replace or wholesale-delete an active `ctx.md` to change one thing.
- Archive because a child step finished or because `status` nudged you — confirm
  with the user that the whole titled task is done.
- Delete findings/research without explicit reset intent.
- Silently delete `^^^` remarks, or delete them after adding `&&&` answers (the
  dialogue stays until compression).
- Relocate resolved `^^^`/`&&&` pairs into a detached Q&A transcript.
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
