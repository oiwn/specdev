---
name: specdev
description: >
  Use when working in a project that uses specification-driven development.
  Start from specs/overview.md and specs/ctx.md, treat specs as the active
  task context, address user remarks marked with ^^^, answer with &&&, and
  keep specs current while planning or implementing. Trigger when the user
  asks to work on specs, continue from specs, address remarks, compress
  specs, or when entering a project with a specs/ directory.
---

Teaches the agent to work with project specs as the primary planning,
coordination, and memory surface. Specs are Markdown files in a `specs/`
directory. The agent reads them, updates them, and uses them as context
before writing code.

## Core spec files

Every project using this workflow has three stable files:

- **`specs/overview.md`** — durable project overview: architecture, data flow,
  public surfaces, documentation map. Should be reasonably current across
  sessions.
- **`specs/ctx.md`** — current task context: active task, state, decisions,
  blockers, next steps. Updated during development. Empty when no task is
  active.
- **`specs/ideas.md`** — loose backlog for possible future work, doubts,
  experiments. Not a committed plan.

Projects may have additional spec files for features, architecture decisions,
etc. The three files above are the stable workflow; other specs can use
whatever structure fits.

## Session startup

When entering a project or when the user says "continue from specs":

1. Read `specs/overview.md`.
2. Read `specs/ctx.md`.
3. If `ctx.md` is empty, ask the user what to work on.
4. If `ctx.md` has an active task, use it as immediate context.
5. Read `specs/ideas.md` when the user asks for brainstorming or roadmap work,
   or when `ctx.md` points there.
6. Identify any `^^^` remarks in relevant specs (see below).

## Marker protocol

`^^^` and `&&&` are inline markers used **only inside spec files** to
distinguish user remarks from agent answers during spec editing.

- `^^^` at the start of a line = user remark, question, correction, or TODO.
- `&&&` at the start of a line = agent answer, resolution, or clarification
  addressing a nearby `^^^` remark.

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

Compression is only by explicit user request. Never auto-compress.

See `references/examples.md` for before/after examples of compression.

## Behavior

**Do:**

- Treat specs as the source of task context when the user points to them.
- Start with `overview.md` and `ctx.md` when entering a project.
- Update specs before implementation when the task is ambiguous or strategic.
- Proceed to implementation when the spec makes the next step clear.
- Preserve user wording when it carries useful intent.
- Preserve addressed `^^^`/`&&&` pairs until the user asks to compress.
- Keep unresolved issues visible.
- Compact stale text when it no longer reflects reality.
- Update `ctx.md` after meaningful progress so the next session can resume.

**Do not:**

- Silently delete `^^^` remarks.
- Delete `^^^` remarks after adding `&&&` answers (the dialogue stays until
  compression).
- Mark a remark as addressed unless it has actually been addressed with `&&&`.
- Bury open questions in prose.
- Overwrite user-authored nuance with a generic plan.
- Start coding while active, relevant `^^^` remarks remain unresolved.

## Specification style

Specs should be actionable and current. Useful sections vary by file:

- `overview.md`: broad, durable — architecture, data flow, public surfaces.
- `ctx.md`: short, current, task-focused — state, decisions, blockers, next
  steps. Can be empty.
- `ideas.md`: loose, backlog-like — no commitment implied.
- Feature specs: whatever structure best explains the feature.

Common useful sections: current context, verified facts, decisions,
assumptions, open questions, planned steps, verification commands, risks,
next task. This is guidance, not a mandatory template.

## Gotchas

- `^^^`/`&&&` markers are for spec editing only — never use them in code,
  comments, or conversation.
- `ctx.md` should represent reality. If it is stale, update it before
  starting work.
- An empty `ctx.md` is valid — it means no active task, not an error.
- Do not force every spec into the same structure. The three-file workflow is
  stable; other specs are flexible.
