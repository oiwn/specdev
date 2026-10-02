# Dark Factory — multi-agent specdev

Design for running several agents **in parallel** on one repo, fed by a planner
that cuts roadmap items into non-overlapping tasks. Status: **design draft** —
nothing here is implemented yet; see `roadmap.md` v0.3–v0.6.

^^^ design should be kinda mixes, there can be 1 agent can be tasks for many.
&&& Agreed — the unit of work is the **task**, not the agent. Two axes:
&&& *depth* = a multi-stage pipeline per PR (implement → verify against
&&& `## Acceptance` → review against `## Plan` → fix), where one task passes
&&& through several stage agents; *breadth* = several such PRs in parallel, only
&&& for tasks proven non-overlapping. One agent may also take several small
&&& sequential tasks in one PR. Build depth first (works with opencode today,
&&& needs no overlap machinery), then add breadth on top — every parallel PR
&&& runs the same pipeline.
^^^ we need to keep in mind that we'll try to parallel tasks eventually, but current focust is multi stage flexible working pipeline

## Why the current model doesn't scale

- `ctx.md` is a singleton — two workers would fight over it.
^^^ but it can contain links to the current tasks and descriptions, so new agent can easily figure out what's going on
&&& Yes — better than "factory never touches ctx.md". In factory mode `ctx.md`
&&& becomes the **index**: a short list of active tasks (id, one-line title,
&&& status, branch/PR link) so any agent orients in one read. To keep it from
&&& being a merge hotspot, only the dispatcher/integrator edits it, on main;
&&& workers read it but never write it. The detailed state stays in each
&&& `specs/tasks/<id>.md`.
- Every PR wants to touch the same hot files (`CHANGELOG.md`, `roadmap.md`,
  `overview.md`), so parallel PRs conflict even when their code doesn't.
  ^^^ need to keep in mind parallel, but for it this is some distant future, maybe with to put into the ideas.
- Nothing declares what a task may touch, so overlap can't be checked.
^^^ here we need proper file contract per task.

## Workflow

^^^ here need to visualize workflow, step by step

## Roles

- **Planner** (agent) — turns one roadmap item into task files with `touches`,
  `depends`, and acceptance commands; proposes waves. Never writes code.
- **Worker** (agent) — executes exactly one task file on its own branch.
- **Integrator** (agent or human) — reviews/merges the PR, moves the task to
  `done/`, applies the task's `## Spec updates` to `overview.md`, folds
  changelog entries.
- **Human gates** — `draft→ready` (approve the plan) and PR merge.

## Task file protocol

One file per task: `specs/tasks/<id>.md`. Shaped like `ctx.md` plus YAML
frontmatter:

```md
---
id: 0007-status-freshness
status: ready            # draft | ready | claimed | review | done | blocked
touches: [src/status.rs, tests/cli.rs]   # globs the worker MAY modify
reads: [src/scan.rs]                     # informational, no lock
depends: [0005-scan-groups]
roadmap: "Freshness thresholds"
branch: task/0007-status-freshness       # set on claim
claimed_by: opencode#123                 # or worktree:<name>; set on claim
---
# Task: warn when ctx.md is stale while in progress

## Plan
- [ ] parse `State:` line in ctx.md
- [ ] add freshness warning to `collect_warnings`
- [ ] tests

## Context
<what the worker needs; links to overview sections>

## Acceptance
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`

## Spec updates
<proposed overview.md / gotcha changes — applied by the integrator, not the worker>
```

- **Ids**: zero-padded sequence + slug (`0007-status-freshness`). Sequence keeps
  ordering obvious; slug keeps ids unique even if two planners race on a number.
- `touches` is a **write lock**, not a hint. It must be specific — prefer files
  over directories, never `**`.
- The task file is the worker's ctx: same rules as `ctx.md` (surgical edits,
  tick boxes, decision-relevant findings).

## Lifecycle

```
roadmap item ─planner─▶ draft ─human─▶ ready ─dispatcher─▶ claimed ─worker─▶ review ─integrator─▶ done
                                         ▲                    │
                                         └──── blocked ◀──────┘   (any state may go to blocked)
```

| Transition         | Who          | Where it happens                              |
|--------------------|--------------|-----------------------------------------------|
| → draft            | Planner      | Commit/PR adding `specs/tasks/<id>.md`        |
| draft → ready      | Human        | Edit `status`, commit to main                 |
| ready → claimed    | Dispatcher   | Commit to main setting `branch`/`claimed_by`  |
| claimed → review   | Worker       | On its branch, when all boxes ticked + PR open|
| review → done      | Integrator   | Merge; move file to `specs/tasks/done/`       |
| * → blocked        | Anyone       | With a one-line reason in `## Context`        |

## Non-overlap rules

Two tasks may be **claimed at the same time** only if:

1. their `touches` globs are disjoint, and
2. neither (transitively) depends on the other, and
3. every `depends` entry is already `done`.

The planner groups `ready` tasks into **waves** — sets that satisfy the rules
pairwise and can all run at once. Waves are advisory; the dispatcher re-checks
the rules at claim time against what is actually claimed.

**Scope guard**: a task's PR diff must be a subset of its `touches` plus its own
task file. Anything else fails CI. This is what turns declared globs into a
guarantee instead of a promise.

## Hot-file discipline

This is what lets parallel PRs merge cleanly.

- Workers edit **only** their own task file and paths in `touches`.
- Workers **never** edit `ctx.md`, `CHANGELOG.md`, `roadmap.md`, or
  `overview.md`.
- "Done" = the task file moves to `specs/tasks/done/`. Those files are the
  changelog fragments; the integrator folds them into `CHANGELOG.md` (per merge
  or per release).
- Durable knowledge (gotchas, architecture notes) goes in the task's
  `## Spec updates`; the integrator applies it to `overview.md`.

## Relation to ctx.md

`ctx.md` stays the human's interactive focus (L0) and is untouched by factory
workers. A factory worker's ctx is its task file. Both can be active at once —
e.g. the human designs the next roadmap item in `ctx.md` while workers run the
current wave.

## Claiming without races

- Exactly **one** dispatcher claims at a time: a GitHub Actions `concurrency`
  group in CI, a lock file locally.
- A claim is a **commit to main** (`ready → claimed`, sets `branch`,
  `claimed_by`). Git is the coordination surface; every worker and planner sees
  claims by pulling.
- Workers branch from the claim commit.

## opencode dispatch (L2)

opencode's GitHub action is triggered by `/opencode` comments, issue/PR events,
`schedule`, or `workflow_dispatch` (the last two require a `prompt:` input).

Dispatcher workflow (`workflow_dispatch` + `schedule`, `concurrency: dispatch`):

1. Pull main; select up to N `ready` tasks claimable under the non-overlap rules.
2. Commit the claims to main.
3. Per task, either:
   - open an issue whose body is the task file and comment
     `/opencode implement specs/tasks/<id>.md following the specdev skill`, or
   - invoke the opencode action directly with that `prompt:` and the task branch.
4. The worker's PR runs the normal CI plus the task's `## Acceptance` commands
   and the scope guard.

## Future CLI surface (design only)

The CLI stays an inspector/scaffolder; it never moves task state on its own.

- `specdev task new <slug>` — scaffold a task file with the next id.
- `specdev task list` / `task show <id>` — board view by status.
- `specdev plan --check` — pairwise glob overlap among `ready`+`claimed`,
  dependency cycles, `ready` tasks with unmet deps; prints proposed waves.
- `specdev task verify <id> --diff <base>` — scope guard for CI.
- `specdev status` — adds a task board summary (counts per status, stale claims).

## Open questions

- Task granularity: cap on files per task? Cap on plan steps?
- Should `reads` ever act as a soft lock (e.g. warn when a read file is in
  another task's `touches`)?
- Who triggers the integrator — on PR merge (CI), or a human command?
- Stuck `claimed` tasks: TTL after which the dispatcher reclaims? Retry limit
  before `blocked`?
- Shared files like `src/main.rs` (CLI wiring) that many tasks need: serialize
  those tasks, carve a dedicated "wiring" task, or allow append-only overlap?
- Is a planner output reviewed as one PR per roadmap item (all drafts together)?

## My Additions

1) also will need to figure out changelog protocol, file grow too much we'll need to compress old records periodically or archive it somewhere.

