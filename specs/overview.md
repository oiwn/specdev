# Draft Skill: specdev

## Purpose

Create a global skill for specification-driven development with an Agent.

This skill should teach any coding Agent how to work with project specs as the
main planning, coordination, and memory surface. It is not only about editing
Markdown. It is about the general workflow between user and Agent:

- start from project specs
- understand current context before coding
- resolve user remarks in specs
- update specs as decisions change
- implement only after the current task is clear
- leave specs accurate for the next session

^^^ And general workflow with agent!
&&& Addressed: this is broader than the `^^^` / `&&&` marker protocol. The
skill should describe the general Agent workflow around specs, planning,
implementation, and session continuity.

^^^ need to highlight that this used only when editing specs
&&& Addressed: the `^^^`/`&&&` marker protocol is used only when editing spec
files. It does not apply to general conversation, code comments, or any other
context. When the agent and user are discussing specs, markers go in the spec
file itself.

## Candidate skill metadata

```yaml
name: specdev
description: Use when working in a project that uses specification-driven development. The Agent should start from specs/overview.md and specs/ctx.md, treat specs as the active task context, address user remarks marked with ^^^, answer with &&&, and keep specs current while planning or implementing.
```

## Installation target

The skill should be global, not repository-specific.

Target directory:

```text
~/.agents/skills/specdev/
  SKILL.md
```

^^^ i'll use "~/.agents/skills" for it. name it "specdev"
&&& Addressed: use `~/.agents/skills` and name the skill `specdev`.

## Core spec files

Each project may have many specs, but the stable starting point is:

- `specs/overview.md`
- `specs/ctx.md`
- `specs/ideas.md`

Expected meaning:

- `overview.md` is the durable project overview. It should explain the project,
  architecture, data flow, public surfaces, and documentation map. It should be
  reasonably current and useful across sessions.
- `ctx.md` is the current task context. It should describe the active task,
  current state, decisions, blockers, and next steps. If there is no active
  task, it can be short or explicitly empty.
- `ideas.md` is a loose backlog for possible future work, doubts, experiments,
  and non-urgent ideas. It should not be treated as committed plan.

When starting a session in a project that uses this workflow, the Agent should
read `specs/overview.md` and `specs/ctx.md` first. It should read
`specs/ideas.md` when the user asks for brainstorming, roadmap work, or when the
current task points there.

^^^ need to mention that starting point of each session should be
("specs/overview.md" and "specs/ctx.md"), where overview is general project
overview, and ctx is current task context. Which we should update during the
process of development, and it should represent current state of things or be
empty.
&&& Addressed: the session starting point should be `overview.md` and `ctx.md`.
`ctx.md` is updated during development and should represent the current state of
things or be empty when there is no active context.

^^^ not sure about it, do even look current context here? to have and idea look
at "../../exoplanets-catalog/specs" for examples of another project.
&&& Addressed: I looked at `../exoplanets-catalog/specs` for examples. That
project has the expected shape: a durable `overview.md`, an empty
`ctx.md` when no task is active, and an `ideas.md` backlog. The skill should not
force one rigid structure for every spec, but it should recognize this stable
three-file workflow.
^^^ agreed

## Marker protocol

User remarks are marked with `^^^` at the start of a line.

Agent answers are marked with `&&&` at the start of a line.

Meaning:

- `^^^` means user remark, objection, question, correction, proposed change, or
  TODO.
- `&&&` means Agent answer, resolution, clarification, or implementation note
  that addresses a nearby `^^^` remark.

The markers distinguish speakers:

- `^^^` is user-authored intent.
- `&&&` is Agent-authored response.

The Agent must not delete `^^^` remarks when addressing them. The dialogue
should remain visible in the spec. When a remark is addressed, keep the original
`^^^` text and add an adjacent `&&&` answer.

Example:

```md
<user remark> ^^^ Should this feature be part of the first milestone?
<agent answer> &&& Addressed: no. It depends on the import pipeline being
stable first, so it belongs in the follow-up milestone.
```

If a remark is still unresolved, keep it marked with `^^^`.

^^^ Don't be project specific.
&&& Addressed: the example is no longer project-specific.

^^^ need to distinct one from another so use "&&&" for Agent answer
&&& Addressed: use `&&&` specifically for Agent answers so user remarks and
Agent responses remain visually distinct.

## General Agent Workflow

When starting work in a project:

1. Read `specs/overview.md`.
2. Read `specs/ctx.md`.
3. If relevant, read `specs/ideas.md` or any spec linked by `ctx.md`.
4. Identify active `^^^` remarks in relevant specs.
5. Decide whether the task is specification work, implementation work, or both.
6. If remarks affect the requested work, address them before coding.
7. Update specs as decisions are made or reality changes.
8. Implement only when the next step is clear.
9. Verify the implementation using project-appropriate commands.
10. Leave `ctx.md` accurate for the next session.

When asked to work with a specific spec file:

1. Read that spec file.
2. Find all `^^^` remarks in that file.
3. Interpret each remark explicitly.
4. Update the spec so each remark is either:
   - addressed with adjacent `&&&`, or
   - preserved as unresolved with `^^^`.
5. Do not remove user intent while cleaning up the document. Preserve original
   `^^^` remarks and add `&&&` answers next to them.
6. If the spec is stale, reorganize or replace stale sections, but make clear
   what superseded them.
7. Give the user a concise report of what was addressed and what remains open.

## Specification Style

Specs should be actionable and current. Avoid long essays unless the topic
requires it.

Useful sections, depending on the spec:

- current context
- facts already verified
- decisions
- assumptions
- open questions
- planned implementation steps
- verification commands
- known risks or blockers
- next task

This is guidance, not a mandatory template. Preserve the shape that fits the
project and file. For example:

- `overview.md` can be broad and durable.
- `ctx.md` should be short, current, and task-focused.
- `ideas.md` can be loose and backlog-like.
- feature specs can use whatever structure best explains the feature.

^^^ yes, we need to work on this currently i have stable workflow: "ctx.md",
"overview.md", "ideas.md"
&&& Addressed: do not force every spec into the same top-level structure. The
stable workflow is `ctx.md`, `overview.md`, and `ideas.md`, but individual specs
can have flexible structure.

## Behavior Rules

The Agent should:

- treat specs as the source of task context when the user points to them
- start with `overview.md` and `ctx.md` when entering a project
- update specs before implementation when the task is ambiguous or strategic
- proceed to implementation when the spec makes the next step clear
- preserve user wording when it carries useful intent
- preserve addressed `^^^` remarks so the user/Agent dialogue remains visible
- keep unresolved issues visible
- compact stale text when it no longer reflects reality
- avoid pretending a decision was made when it was only proposed
- update `ctx.md` after meaningful progress so the next session can resume

The Agent should not:

- silently delete `^^^` remarks
- delete addressed `^^^` remarks after adding `&&&` answers
- mark a remark as addressed unless it has actually been addressed with `&&&`
- bury open questions in prose
- overwrite user-authored nuance with a generic plan
- start coding while active, relevant `^^^` remarks remain unresolved

^^^ yes
&&& Addressed: yes, when relevant `^^^` remarks remain unresolved, the Agent
should not proceed into code edits. It should address the remarks first or ask
for clarification.

## Suggested Interaction Pattern

When the user says:

> look at `specs/ctx.md`, I added remarks

The Agent should:

1. Read `specs/overview.md` if project context is not already loaded.
2. Read `specs/ctx.md`.
3. Find all `^^^` remarks.
4. Update the file in place.
5. Add adjacent `&&&` answers for addressed remarks without deleting the
   original `^^^` remarks.
6. Keep unresolved `^^^` remarks visible.
7. Reply with:
   - what was addressed
   - what remains open
   - what the next task is

When the user says:

> continue from specs

The Agent should:

1. Read `specs/overview.md`.
2. Read `specs/ctx.md`.
3. Use `ctx.md` as the immediate task context.
4. If `ctx.md` is empty, ask or infer the next task from the user request.

## Context Compression

When the user judges that discussions in a spec have concluded, they can ask the
agent to compress it. This is agent-driven, not a CLI command.

The agent should:

1. Read the spec file containing resolved `^^^`/`&&&` dialogue.
2. Identify all `^^^`/`&&&` pairs where the issue is resolved.
3. Rewrite the spec sections according to the dialogue — fold the decisions into
   the surrounding prose, removing the markers and the back-and-forth.
4. Preserve any unresolved `^^^` remarks as-is.
5. The rewritten spec should read as a clean document reflecting the final
   state of decisions, not a transcript of the discussion.

Compression is only by user request. The agent should never auto-compress.

## Optional Tooling

No scripts are required for the first version.

Possible later additions:

- script to scan specs for unresolved `^^^` markers
- script to list `ctx.md` freshness or missing sections
- before/after examples of spec edits
- conventions for archiving stale `ctx.md` content

## Decisions Captured

- The skill should be global for many agents, not Woid-specific.
- The skill name should be `specdev`.
- The install target should be `~/.agents/skills/specdev/`.
- `^^^` is for user remarks.
- `&&&` is for Agent answers.
- `overview.md` and `ctx.md` are the session starting point.
- `ideas.md` is part of the stable workflow, but not always loaded.
- Relevant unresolved `^^^` remarks block code edits until addressed.
- Addressed `^^^` remarks should remain visible alongside the Agent's `&&&`
  answer.

^^^ global skill for any agents i use many
&&& Addressed: this is a global skill for any agents you use.

