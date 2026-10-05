# specdev

Specification-driven development toolkit for humans and AI agents.

`specdev` is a CLI and an agent skill that keep project work organized around Markdown specs. The CLI scaffolds the spec files, owns the structured state of tasks (status, stage, scope, history), and checks that everything stays in order. The skill teaches any coding agent how to read and update specs, run tasks through the pipeline, and route each piece of content to its home.

## Why

To organize the workflow and keep everything in order. A project's knowledge is spread across what is being done now, what comes next, what was decided, and what is known to bite. specdev gives each of these one home and a fixed path between them, so humans and agents work from the same current picture:

- **Content routing** — every kind of note has one file it belongs in, and it moves forward as it matures: idea → roadmap → active work → changelog.
- **Checkbox plans** — active work carries an explicit plan, a checkable definition of "done".
- **Commands for state, checks for drift** — task status, scope, and history change only through `specdev task …` commands, and `specdev check` catches anything that slipped out of order.

## Install

```bash
cargo install specdev
```

## Quick start

```bash
# Scaffold specs/, specdev.toml, and the root files in your project
specdev init

# Install the agent skill globally
specdev skill install
```

This creates:

```
specs/
  overview.md    # durable project overview (+ Gotchas/Knowledge home)
  ctx.md         # working context: current focus, pointer to the active task, context shared across tasks (header-only when idle)
  roadmap.md     # committed future direction
  ideas.md       # uncommitted backlog
  cleanup.md     # code-smell notes (read on demand)
  tasks/         # one file per task, managed by `specdev task …`; finished tasks move to tasks/done/
CHANGELOG.md     # completed work (archive destination)
AGENTS.md        # minimal pointer to the specdev skill (auto-added)
specdev.toml     # project config: task contract, scope allowlist, quality limits
```

`init` never overwrites: every file is write-if-missing, and an existing `AGENTS.md` gets a small specdev section appended (wrapped in `<!-- BEGIN/END specdev -->` markers, so re-running is idempotent). Re-run it any time to fill in missing pieces.

The skill installs to `~/.agents/skills/specdev/SKILL.md`, where any [Agent Skills](https://agentskills.io)-compatible agent will find it.

## Two ways to track work

- **`ctx.md`** — the working context. Design work and big, fuzzy work run here directly, with a checkbox plan, through the ideas → roadmap → ctx → changelog loop. When a task file is active, `ctx.md` just points at it.
- **Task files** (`specs/tasks/<id>.md`) — one well-defined piece of work, sized to land as one commit. specdev owns its frontmatter and `## Log`; the agent and the human own the prose. It moves through a fixed pipeline, one active task at a time.

## The task pipeline

Happy path: draft → ready → implement → verify → review → approval → done. Failures loop through fix.

1. **draft** — `specdev task new <slug>`; the agent declares `scope` and writes Plan and Acceptance; the human reviews with `^^^` remarks. Approved → `specdev task advance` → ready (records the approved scope; refused while remarks are open).
2. **ready** — queued; `advance` starts it when no other task is active.
3. **implement** — code inside `scope` only; new files are declared first with `specdev task scope <id> add <path> --reason "…"`. All Plan boxes ticked → verify.
4. **verify** — the agent runs the Acceptance commands and ticks what passed (specdev runs nothing itself). All ticked → review; failures → `--to fix`.
5. **review** — the diff is judged against Plan and scope; `## Review` is written. Approve → approval; changes → fix.
6. **fix** — address the review; back to verify. Past `max_attempts` the task is blocked automatically.
7. **approval** — the agent stops. The human runs `## Manual checks`, opens the PR, merges.
8. **done** — `specdev task done <id>`: archives the file to `tasks/done/` and writes a CHANGELOG entry from the task's one-line `## Summary`.

Any open task can be parked with `specdev task block <id> --reason "…"`; the human decides where it goes next. specdev never writes to git and never runs project commands — branches, commits, PRs, and tests stay with you and your agent.

## CLI commands

All commands run from the project root. The global `--format json|toon` flag gives machine-readable output (for `check`, `task`, `fmt`, `scan`, `status`, `list`); task ids accept the full id, the number (`7`), or the slug.

### Setup

- `specdev init` — scaffold the files above; safe to re-run.
- `specdev skill install [--local]` — install the agent skill globally (`~/.agents/skills/specdev/`) or into the project (`.agents/skills/specdev/`).
- `specdev skill check` — report whether installed skills are up to date, stale, or locally modified.

### Tasks

- `specdev task new <slug>` — create a draft task file (next free number) with the sections from `specdev.toml`.
- `specdev task list` — open tasks in queue order: active, ready, draft, blocked.
- `specdev task show <id>` — state, scope, plan progress, log, and the next step.
- `specdev task advance <id> [--to <target>]` — move to the next stage, or to `draft`, `ready`, `approval`, or a stage (`implement`, `verify`, `review`, `fix`). Gates: no open `^^^` for draft → ready, Plan ticked for implement → verify, Acceptance ticked for verify → review, one active task.
- `specdev task block <id> --reason "…"` — park a task.
- `specdev task set <id> <field> <value>` — plain fields: `source`, `depends` (comma-separated ids), and extra fields from `specdev.toml`; `""` clears.
- `specdev task scope <id> add <path> --reason "…"` / `specdev task scope <id> rm <path>` — change the scope; every change is logged.
- `specdev task done <id>` — finish an approved task (needs `## Summary`; `## Manual checks` ticked).
- `specdev task index` — regenerate `specs/tasks/_index.md`.

Every state command rewrites only the frontmatter and appends to `## Log`, and refuses a change that would make `specdev check` fail:

```
$ specdev task advance 1
0001-status-freshness — ready → in-progress/implement
  log: 2026-10-05 advance ready → in-progress/implement
Updated specs/tasks/_index.md

$ specdev task advance 1
error: `0001-status-freshness`: 1 of 1 `## Plan` items unticked; tick them before moving on
```

### Checking

- `specdev check` — validate everything; errors exit 1, warnings don't:
  - task contract: required sections, fields, ids, `depends`;
  - log consistency: each task's frontmatter must match what its own `## Log` replays to, so hand edits are caught without git;
  - one active task; `_index.md` up to date;
  - scope: files changed in git outside the active task's `scope` (anything under `specs/` and `[scope] always_allowed` are exempt);
  - quality gate: size and plan limits per file kind, optional no-tables rule;
  - formatting, and the `ctx.md` warnings from `status`.
- `specdev check --staged` — the same, checking only staged files against the scope, as warnings; meant for a pre-commit hook.

```
$ specdev check
specs/tasks/0001-status-freshness.md: error[log-mismatch]: frontmatter status is `in-progress/review` but the log replays to `in-progress/implement`; change state with specdev commands, not by hand
specs/tasks/_index.md: error[index-stale]: out of date; run `specdev task index`
2 errors, 0 warnings
```

- `specdev fmt [<file>...]` — normalize Markdown in `specs/` and open task files: unwrap hard-wrapped paragraphs into soft wraps, `-` bullets, blank lines around headings. Code blocks, tables, links, and `^^^`/`&&&` lines stay as written.

### Inspecting

- `specdev scan` — `^^^` remarks across `specs/`, resolved (with an adjacent `&&&` answer) or open.
- `specdev status` — spec file health: line counts, last modified, marker counts, `ctx.md` plan progress, and warnings (archive nudge, forbidden content in `ctx.md`, missing root files).
- `specdev list [--stats]` — spec files with their first header and line count; `--stats` adds headings by level, checkboxes, open remarks, words, lines, code blocks, and tables.

```
$ specdev status
Core spec files:
  [OK] overview.md      79 lines  1h ago     11 markers (1 open)
  [OK] ctx.md           22 lines  2h ago
       task progress: 4/4 steps done
  [OK] roadmap.md       18 lines  2h ago
  [OK] ideas.md          9 lines  2h ago

Markers: 1 open, 10 resolved

Warnings:
  - ctx.md: all 4 plan steps are checked. Confirm with the user that the whole task is done before archiving.
```

## Configuration

`specdev.toml` (written by `init`; every key is optional and falls back to the default shown):

```toml
[task]
required_sections = ["Plan", "Acceptance"]   # `task new` creates them; `check` requires them
optional_sections = ["Summary", "Manual checks", "Context", "Findings", "Review", "Spec updates"]
extra_fields = []                            # project-specific frontmatter fields for `task set`

[pipeline]
max_attempts = 3                             # entries into fix before the task is blocked

[acceptance]
default = []                                 # pre-filled Acceptance items, e.g. ["cargo test"]

[scope]
always_allowed = []                          # globs never counted against a task's scope, e.g. ["Cargo.lock"]

[quality]
errors = false                               # true: quality violations fail `check`
forbid_tables = false                        # flag Markdown tables and ASCII diagrams

[quality.task]                               # omitted keys keep their defaults; 0 turns a limit off
max_lines = 150
max_plan_steps = 8
max_scope = 6

[quality.ctx]
max_lines = 120

[quality.overview]
max_lines = 200

[quality.changelog]
max_lines = 400
```

## The spec workflow

The agent skill (`skills/SKILL.md`) is the full reference; this is the short version.

- **The loop:** `ideas.md` (uncommitted) → `roadmap.md` (committed) → `ctx.md` or `specs/tasks/` (active) → `CHANGELOG.md` (done).
- **Routing:** durable knowledge and gotchas go to `overview.md`, future work to `roadmap.md` or `ideas.md`, finished work to `CHANGELOG.md`. `ctx.md` holds no deferred items, roadmap pointers, or gotchas.
- **Formatting:** specs are read in terminal editors — bullets and short sections, soft wraps, no Markdown tables or ASCII diagrams, code blocks only for literal content.

### ctx.md — the working context

```md
# Current Task Context: <one line>
State: <not started | in progress | blocked>
## Plan
- [ ] step one
- [ ] step two
## Findings      # optional — research/results when the next step is evaluate/discuss
## Context
<what the current unchecked step needs; or a pointer to the active task file>
## Next
<the immediate next action>
```

Concise and **decision-relevant, not minimal**. The header is the task identity; the no-task state is header-only. Tick boxes as you go; archive only on explicit request, once the whole titled task is done (the `status` nudge means confirm, not auto-archive).

### Markers

- `^^^` — a human remark, question, correction, or TODO in a spec or task file.
- `&&&` — the agent's answer, next to the remark it addresses.

Both stay until the human asks to compress them into the prose. Approving a draft task counts as that request for that task.

## Agent skill

The bundled skill (`skills/SKILL.md`) follows the [Agent Skills specification](https://agentskills.io). It teaches agents to:

1. Start each session with `specs/overview.md`, `specs/ctx.md`, and `specdev check`.
2. Route content to its home and keep `ctx.md` narrow.
3. Run task files through the pipeline with `specdev task …` commands — never by hand-editing frontmatter or the log.
4. Declare scope before touching files, run the Acceptance commands themselves, and stop at approval.
5. Address `^^^` remarks before writing code, and compress them only on request.

`references/examples.md` holds before/after examples, loaded on demand. After upgrading specdev, re-run `specdev skill install` (`specdev skill check` tells you when it's stale).

## License

MIT
