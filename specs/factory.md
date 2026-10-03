# Dark Factory — multi-agent specdev

Design for driving a repo with agents through a **multi-stage pipeline per task**: plan → implement → verify → review → fix, with the human at the edges. Status: **design draft** — nothing here is implemented yet.

## Direction

- **specdev manages specs and keeps them sorted.** It helps agents and humans manage human-readable, human-editable specification and context files: every kind of content in its home, every task in a valid state.
- **A tool for agents, not an orchestrator.** specdev never launches agents, never calls an LLM, and never writes to git (no branches, commits, or PRs). It only reads changed-file lists by calling the `git` CLI. It stores state, validates it, and tells the agent where a task stands. Running stages is the agent's job.
- **Agent-agnostic.** No dependency on any specific agent or CI. The skill describes each stage; whichever agent you use follows it.
- **Output for humans and agents.** Every command supports `--format text|json|toon`: text for humans, JSON for tools, TOON (token-oriented object notation) as a compact format for LLM agents.
- **The unit of work is the task, not the agent.** One agent may walk every stage of a task, or different agents may take different stages. One agent may also take several small tasks in sequence.
- **Depth first, breadth later.** Current focus is a flexible multi-stage pipeline for one task at a time. Parallel tasks come later; the task contract already carries what they need (`scope`, `depends`), so no format change then.
- **A tool, not weak contracts.** specdev moves from "CLI inspects, skill asks nicely" to commands that make valid state changes and a check that rejects invalid ones. This reverses the current `overview.md` rule that the CLI never mutates spec content — for *structured state* only; prose stays agent-edited. See "State vs prose".
- **specdev formats specs, not the agent.** Spec files use soft wraps: one line per paragraph or bullet, the editor wraps it. `specdev fmt` normalizes files (unwraps hard-wrapped paragraphs, list markers, blank lines around headings), and `specdev check` flags unformatted files, so agents don't spend effort on Markdown width.

## Why the current model isn't enough

- `ctx.md` holds one task. Issue-sized tasks get their own files in `specs/tasks/`, listed in a generated `specs/tasks/_index.md`; `ctx.md` becomes the working context (see "Role of ctx.md").
- Nothing declares which files a task may change. The task contract adds `scope`, agreed before work starts (see "Scope contract").
- Every PR wants to edit the same shared files (`CHANGELOG.md`, `roadmap.md`, `overview.md`). This only matters for parallel work — see "Future: parallel execution".

## Operating limits

- **One active task** (`in-progress` or `approval`) at a time. `specdev task advance` refuses to start a second; `specdev check` flags it.
- Other tasks wait in a **queue**: status `ready`, taken in id order.
- Human picks issues, approves plans, does the e2e check, handles branches, opens the PR and merges. Agents do the stages in between.
- **Dropped: automation levels.** An earlier draft had levels L0–L3 (L0 today, L1 one task through a pipeline, L2 parallel pipelines, L3 an autonomous planner drafting from the roadmap). Only L1 is in scope; L2 lives in "Future: parallel execution"; L3 is not planned.

## Workflow

Happy path: draft → ready → implement → verify → review → approval → done.
Loop: verify fail or review changes → fix → verify again.

1. **draft** — agent, on a human request.
   - Human points the agent at one issue or roadmap item. No bulk intake.
   - Agent runs `specdev task new <slug>`, which creates `specs/tasks/<id>.md` from the task definition in `specdev.toml`, with valid frontmatter and empty required sections.
   - Agent fills `source` itself with whatever the origin is (issue URL, roadmap item, chat request), and records Plan, `scope`, Acceptance. The "planner" is just the agent you asked — or you, writing by hand.
   - A roadmap item bigger than one commit is split into several draft tasks, each one commit, ordered by `depends`.
   - `specdev check` validates the file: required fields and sections, `scope` paths exist, Acceptance not empty, and the quality gate (see "Spec quality gate").
   - Human reviews the file with `^^^`; agent answers with `&&&`.
   - Next: human approves → agent compresses the resolved `^^^`/`&&&` dialogue into the plan → `ready` (joins the queue). Approval is the explicit compress intent for that task file, so the implementer reads a clean plan, not a transcript. `specdev task advance` refuses `ready` while open `^^^` remain.
2. **ready** — queued; first in id order is taken when no task is active.
   - `specdev task advance <id>` → status `in-progress`, stage `implement`; records the approved scope in `## Log`.
   - Agent works on whatever branch is checked out. Branches, commits for the PR, and the PR itself are the human's; specdev never writes to git.
3. **implement** — agent.
   - Codes inside `scope`, ticks Plan boxes.
   - If a needed file is missing from `scope`, see "Scope contract".
   - Next: all boxes ticked → `verify`.
4. **verify** — deterministic, no judgement. The agent runs it locally.
   - `specdev task verify <id>` runs the Acceptance commands — tests, lint, build, format check; defaults from `specdev.toml` plus the task's own — and checks changed files against `scope` (changed files come from the `git` CLI; see "Scope contract").
   - Manual e2e steps listed in Acceptance are not run here; the human runs them at `approval`.
   - Next: pass → `review`; fail → `fix`.
5. **review** — agent, ideally a fresh session or a different model.
   - Judges the diff against Plan and `scope`; accepts or rejects each scope expansion; writes `## Review`.
   - Next: approve → status `approval`; changes requested → `fix`.
6. **fix** — agent.
   - Addresses `## Review` notes or verify failures. `attempts` +1.
   - Next: → `verify`. If `attempts` > `max_attempts` → `blocked`.
7. **approval** — human. The agent stops here.
   - Runs the manual e2e steps; problems go in as `^^^` → `fix`.
   - Opens the PR, CI runs, human merges.
   - Next: merge → `done`.
8. **done** — human, or the agent when asked, after merge.
   - `specdev task done <id>`: moves the file to `specs/tasks/done/`, adds a changelog entry, regenerates `_index.md`. Next queued task can start.
9. **blocked** — set by anyone, or automatically at the attempts cap.
   - `blocked_reason` is required. Human decides: back to a stage, or back to `draft`.

Notes:

- **Verify is not judgement.** It is the deterministic gate; agents only act on its output, so "tests pass" is never just an agent's claim.
- Status `approval` (waiting for the human) is deliberately not named `review`, to avoid clashing with the agent `review` stage.
- Statuses and stages are both hardcoded enums in specdev, not configurable. Each stage has defined semantics (verify is the deterministic gate, fix bumps `attempts`).

## Scope contract

`scope` is the list of files (globs) a task may change. It is a contract agreed before work starts, not a guess.

- **Proposed in `draft`.** The agent writes `scope` together with the Plan. `specdev check` verifies the paths exist (or are clearly new files) and the quality gate checks the size.
- **Agreed at `draft → ready`.** The human approves scope and plan together. `specdev task advance` records the approved scope in the task's `## Log`, so later expansions are measured against it without needing git history.
- **Locked from `implement` on.** The agent changes only files inside `scope`, plus the task file itself.
- **Expansion is declared, never silent.** Planning can't foresee every file. The agent runs `specdev task scope <id> add <path> --reason "..."`, which updates `scope` and logs the reason. Review accepts or rejects each expansion. A large expansion (many files, a new module) means the plan was wrong → `blocked`, back to `draft`.
- **Checked continuously.** A pre-commit hook (`specdev check --staged`) warns when staged files fall outside the active task's `scope`. `specdev task verify` does the same check for the whole working tree. Changed files come from the `git` CLI, read-only: `git diff --name-only -z HEAD`, `git diff --cached --name-only -z`, `git ls-files --others --exclude-standard -z`. Without git, or outside a repo, the scope check is skipped with a warning; everything else still runs.

## How it runs

- One agent in one session can walk all stages in order. "Multi-stage" means recorded checkpoints with checks between them, not necessarily many agents.
- A separate agent per stage (e.g. a fresh reviewer) is optional and is the agent harness's business, not specdev's.
- `specdev task show <id>` prints the current status/stage and what the skill says to do next — inspection only, it runs nothing.
- Every stage change goes through `specdev task advance`, which refuses illegal moves, so any agent resuming the task sees a consistent state.

## Roles

Roles are stages any agent can play, not separate products.

- **Planner** — writes the task file from an issue or roadmap item. Agent or human. Never writes code.
- **Implementer** — `implement` and `fix` stages.
- **Verifier** — `specdev task verify`; no judgement.
- **Reviewer** — `review` stage; judges the diff against Plan and contract.
- **Human** — picks the issue, approves the plan, e2e check, branches, PR, merge.

## Task contract

One file per task: `specs/tasks/<id>.md` — flat YAML frontmatter plus a Markdown body. The contract is defined in `specdev.toml` (see "Project config"): `task new` generates new files from it and `check` validates against it, so the template and the validation can't drift. Without a config, specdev uses a built-in default.

```md
---
id: 0007-status-freshness    # required, matches filename
status: in-progress          # required, see statuses below
stage: verify                # required iff status = in-progress
scope: [src/status.rs, tests/cli.rs]   # required, non-empty globs
created: 2026-10-02          # required

source: https://github.com/oiwn/specdev/issues/12   # optional, free-form
depends: [0005-scan-groups]  # optional
attempts: 1                  # optional, entries into fix
blocked_reason: "..."        # required iff status = blocked
---
# Task: warn when ctx.md is stale while in progress

## Plan          (required, checkboxes)
## Acceptance    (required, verify commands + manual e2e steps)
## Context       (optional)
## Findings      (optional)
## Review        (optional, written by review, consumed by fix)
## Spec updates  (optional, proposed overview/gotcha changes)
## Log           (written by specdev only: transitions, approved scope, expansions)
```

Statuses: `draft`, `ready`, `in-progress`, `approval`, `done`, `blocked`. Stages: `implement`, `verify`, `review`, `fix`.

Rules:

- **Two fields, two meanings**: `status` = lifecycle (fixed set); `stage` = pipeline position (fixed set).
- **Title lives only in the `# Task:` heading** — no duplicate in frontmatter.
- **Ids**: zero-padded sequence + slug (`0007-status-freshness`).
- **`scope` is the file contract**: see "Scope contract". Prefer files over directories; never bare `**`.
- **Flat YAML only** — scalars and inline lists, no nesting. Unknown fields are allowed and ignored (forward compatibility).
- **The task file is the agent's ctx**: same rules as `ctx.md` (surgical edits, tick boxes, decision-relevant findings).

Layout:

- `specs/tasks/<id>.md` — draft, queued, and active tasks.
- `specs/tasks/_index.md` — generated list: id, title, status/stage, source. Never hand-edited.
- `specs/tasks/done/` — finished tasks.

## Spec quality gate

specdev measures spec and task files and checks them against thresholds, so "is this spec any good" has a mechanical first answer before a human reads it.

- **Metrics** (extends today's `specdev list --stats`): line and word count, number of sections, heading depth, plan steps, `scope` size, code snippets present, tables/ASCII diagrams present, open `^^^`.
- **Thresholds** live in `specdev.toml`, per file kind (task, ctx, overview, changelog). Exceeding one is a warning by default, an error if configured.
- **Task size target: one commit.** A task should be small enough to land as a single commit whose changed files all sit inside `scope`. The gate flags tasks likely too big (many plan steps, large scope); thresholds start loose and get tuned from real use.
- **CHANGELOG size.** `CHANGELOG.md` is under git, so old entries can be compressed safely. The gate warns when it passes a size threshold; the agent then compresses older entries (a prose edit). History stays recoverable from git.

## Project config

`specdev.toml` at the repo root, created by `specdev init`:

```toml
[task]
required_sections = ["Plan", "Acceptance"]
optional_sections = ["Context", "Findings", "Review", "Spec updates"]
extra_fields = []         # project-specific optional frontmatter fields

[pipeline]
max_attempts = 3          # entries into `fix` before auto-blocked

[acceptance]
default = ["cargo test", "cargo clippy --all-targets -- -D warnings"]

[quality.task]
max_plan_steps = 8
max_scope = 6
max_lines = 150

[quality.changelog]
max_lines = 400
```

- The required frontmatter fields and the `## Log` section are fixed by specdev; the config adds to them, it can't remove them.
- Tasks inherit `acceptance.default` and may add their own commands.
- Quality numbers are placeholders, to be tuned.

## State vs prose

Spec content splits in two:

- **Structured state — specdev commands only.** Frontmatter, status/stage transitions, `attempts`, `scope`, ids and filenames, the `## Log` section, archive to `done/`, `specs/tasks/_index.md`.
- **Prose — agent edits directly.** Context, Findings, Review, overview / ideas / roadmap text, compression, checkbox ticks.

Frontmatter has dedicated commands: `task set` for plain fields, `task scope` for scope, `task advance` / `task block` for status and stage.

The agent always has a file-write tool, so "only via CLI" can't be prevented — it is **detected**, and detection doesn't depend on git history (it may be shallow, squashed, or uncommitted). Each task file carries its own audit trail:

- Every specdev command that changes state appends a line to the task's `## Log` (date, old → new, approved scope, expansion reasons).
- `specdev check` validates each file on its own: frontmatter matches the last `## Log` entry, every logged transition is legal, `scope` equals the approved scope plus logged expansions, required fields and sections are present, enum values are valid, at most one task is active, `_index.md` matches generated output.
- A hand edit of frontmatter leaves the file disagreeing with its own log, so it is caught without any git diff.
- The `created` log entry records the task title, so a renamed title is caught from the file alone. Deleted `^^^` lines are not detected; accepted.

Enforcement points:

- the agent runs `specdev check` at each transition;
- pre-commit (`prek.toml`): `specdev check --staged`;
- optionally the project's CI on the human's PR.

Humans may edit anything, but a change that breaks the rules fails the check regardless of author.

Commands (design):

```
specdev task new <slug>                   # from specdev.toml task def, next id
specdev task set <id> <field> <value>     # plain fields: source, depends
specdev task scope <id> add|rm <path> [--reason "..."]
specdev task advance <id> [--to <stage>]  # legal moves; fix bumps attempts
specdev task block <id> --reason "..."
specdev task verify <id>                  # Acceptance + scope check
specdev task done <id>                    # to done/, changelog, index
specdev task list
specdev task show <id>                    # status/stage + next-step hint
specdev task index                        # regenerate _index.md
specdev check [--staged]                  # lint, log consistency, quality gate
specdev fmt [<file>...]                   # normalize Markdown, soft wraps
```

## Role of ctx.md

- **`ctx.md` is the working context** — the first file any agent reads after `overview.md` to know what is going on right now.
- It holds: the current focus in one line, a pointer to the active task file (if any), and cross-task context implementation needs but that belongs to no single task — current constraints, decisions in flight, which other specs matter now.
- It does not hold: per-task plans (task files), durable architecture and gotchas (`overview.md`), the task list (`_index.md`).
- Design work and big, fuzzy work still run directly in `ctx.md` through the ideas → roadmap → ctx → changelog loop (like this spec).
- specdev never writes `ctx.md`; it is human/agent prose. `specdev check` can warn when it points at a task that isn't active.

## Future: parallel execution

Distant future; not current focus (one active task). Could move to `ideas.md`.

- **Non-overlap rules.** Two tasks may run at the same time only if their `scope` globs are disjoint, neither (transitively) depends on the other, and every `depends` entry is `done`. A future `specdev plan` command would group ready tasks into **waves** (machine-readable output for an orchestrator) and validate overlap, dependency cycles, and unmet deps. specdev still runs nothing.
- **Shared-file discipline.** Parallel tasks edit only their task file and `scope`; never `ctx.md`, `CHANGELOG.md`, `roadmap.md`, `overview.md`. Done task files in `specs/tasks/done/` act as changelog fragments; at the end a separate agent folds them into `CHANGELOG.md` and applies `## Spec updates`.
- **Claiming and dispatch.** Postponed until we have experience running the single-task pipeline. Starting several agents is orchestration and stays outside specdev either way.
- **Open questions.**
  - Should a `reads` field act as a soft lock?
  - Shared files like `src/main.rs`: serialize, a dedicated wiring task, or allow append-only overlap?
  - TTL for stuck claimed tasks?
