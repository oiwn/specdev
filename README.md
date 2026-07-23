# specdev

Specification-driven development toolkit for humans and AI agents.

`specdev` provides a CLI and an agent skill that establish a structured workflow
around project specs. The CLI scaffolds and inspects specs. The skill teaches any
coding agent how to read specs, keep `ctx.md` narrow, move work through the
**ideas → roadmap → ctx → changelog** loop, and archive done work.

## Why

Third-party agents (codex, claude, glm, deepseek, …) tend to leave
`specs/ctx.md` cluttered with stale scope, deferred items, and gotchas.
specdev fixes this with a clear **content-routing** model and a **checkbox
plan** that gives a machine-checkable definition of "done".

## Install

```bash
cargo install specdev
```

## Quick start

```bash
# Scaffold specs/ and the root files in your project
specdev init

# Install the agent skill globally
specdev skill install
```

This creates:

```
specs/
  overview.md    # durable project overview (+ Gotchas/Knowledge home)
  ctx.md         # current task context — checkbox plan (header-only until a task starts)
  roadmap.md     # committed future direction
  ideas.md       # uncommitted backlog
  cleanup.md     # code-smell notes (read on demand)
CHANGELOG.md     # completed tasks (archive destination)
AGENTS.md        # minimal pointer to the specdev skill (auto-added)
```

`init` is append-aware for `AGENTS.md`: if you already have one, it appends a
small specdev section (wrapped in `<!-- BEGIN/END specdev -->` markers, so
re-running is idempotent and never touches the rest of your file). The full
workflow lives in the agent skill.

The skill installs to `~/.agents/skills/specdev/SKILL.md`, where any
[Agent Skills](https://agentskills.io)-compatible agent will find it.

## CLI commands

### `specdev init`

Scaffold `specs/` with the five core spec files, plus `CHANGELOG.md` at the
project root (write-if-missing) and `AGENTS.md` at the root (a minimal specdev
pointer — created if absent, **appended idempotently** if the file already
exists). Re-run any time to fill in missing files.

### `specdev scan`

Scan `specs/` for `^^^` user remarks and report which are resolved (have an
adjacent `&&&` answer) and which are still open.

```
overview.md
  L 18  [resolved]  And general workflow with agent!
  L 23  [open]      need to highlight that this used only when editing specs

Total: 1 open, 10 resolved across 2 files
```

### `specdev status`

Show spec file health: which core files exist, line counts, last modified
times, marker counts, **current task progress**, and warnings.

```
Core spec files:
  [OK] overview.md     294 lines  3h ago     11 markers (1 open)
  [OK] ctx.md           22 lines  2h ago
       task progress: 1/4 steps done
  [OK] roadmap.md       18 lines  2h ago
  [OK] ideas.md          9 lines  2h ago

Markers: 1 open, 10 resolved

Warnings:
  - ctx.md: all 4 plan steps are checked. Confirm with the user that the whole task is done before archiving.
  - CHANGELOG.md missing at project root — run `specdev init` to add it.
```

Warnings surface exactly the drift agents fail to self-police:

- **task progress** — `N/M steps done`, parsed from the `## Plan` checkboxes.
- **archive nudge** — all steps checked but the task is still in `ctx.md`.
- **forbidden content** — `Deferred`/`Gotchas`/`Quirks` sections or `Roadmap
  pointer` lines inside `ctx.md` (they belong elsewhere).
- **missing root files** — `CHANGELOG.md` / `AGENTS.md` absent.

### `specdev list`

List spec files with the first header as the description:

```
File            Description                                Lines
overview.md     specdev — overview                           48
ctx.md          Current Task Context: portal overview v1    21
roadmap.md      Roadmap                                     30
ideas.md        Ideas                                       12
cleanup.md      Cleanup                                      5
```

With `--stats`, get a structural breakdown per file — heading counts by level,
checkboxes (checked/open), open `^^^` remarks, and word count:

```
File            H1  H2  H3  H4+   [x]  [ ]   ^^^   words
overview.md      1   3   0   0      0    0     0      312
ctx.md           1   4   0   0      2    4     0       88
roadmap.md       1   2   0   0      5    1     0      120
ideas.md         1   0   0   0      0    0     0       40
cleanup.md       1   0   0   0      0    0     0       24
```

Core files are listed in canonical order (`overview`, `ctx`, `roadmap`,
`ideas`, `cleanup`), then any extras alphabetically.

### `specdev skill install`

Install the specdev agent skill. Defaults to `~/.agents/skills/specdev/`
(global). Use `--local` to install to `.agents/skills/specdev/` within the
current project.

## The spec workflow

The agent skill (`skills/SKILL.md`) is the full reference; this is the short
version.

### The working loop

Content matures left-to-right, then archives:

```
ideas.md  ->  roadmap.md  ->  ctx.md  ->  CHANGELOG.md
uncommitted   committed      active      done
```

An idea becomes a roadmap item when decided; a roadmap item becomes the ctx
task when picked up; a ctx task becomes a CHANGELOG entry when every checkbox
is `[x]`.

### ctx.md — the current task

```md
# Current Task Context: <one line>
State: <not started | in progress | blocked>
## Plan
- [ ] step one
- [ ] step two
## Findings      # optional — research/results when the next step is evaluate/discuss
## Context
<what the current unchecked step needs to act on>
## Next
<the immediate next action>
```

Concise and **decision-relevant, not minimal**. The header is the task identity
— don't rename it without explicit user intent; no-task state is header-only.
A plan step is a child, not the task. Tick boxes as you implement; archive only
at the explicit user request, once the **whole** titled task is done (the
`status` nudge means confirm, not auto-archive).

### Markers (spec files only)

- `^^^` — user remark, question, correction, or TODO
- `&&&` — agent answer addressing a nearby `^^^`

Both stay until the user asks to compress. See the agent skill for content
routing, forbidden content, and compression.

## Agent skill

The bundled skill (`skills/SKILL.md`) follows the
[Agent Skills specification](https://agentskills.io). It teaches agents to:

1. Start each session by reading `specs/overview.md` and `specs/ctx.md`.
2. Move work through the **ideas → roadmap → ctx → changelog** loop.
3. Keep `ctx.md` narrow and route content to its right home.
4. Address `^^^` remarks before writing code.
5. Tick checkboxes while implementing; archive the task when all are `[x]`.
6. Compress specs on user request.

The skill is intentionally concise with a separate `references/examples.md`
for before/after examples, loaded on demand via progressive disclosure.

## License

MIT
