# specdev.toml Reference

Read this before editing `specdev.toml`. The file is the project's task contract; every key changes agent behavior, and unknown keys are rejected (a typo fails loudly instead of silently doing nothing). Current values for this project live in the file itself; `specdev init` writes a fully commented template.

## Rules that apply everywhere

- **Missing means default.** A missing `specdev.toml`, section, or key falls back to the built-in default. Copy only what you change.
- **Omitted keys inherit.** Inside a limits table, a key you don't set keeps its default. Set a key to `0` to turn that limit off entirely.
- **Unknown keys are rejected.** If `specdev` errors on the config, the key doesn't exist — check the spelling against this file.
- **Nothing here is agent-secret.** The agent reads this file the same way you do; the contract works because both sides see it.

## `[task]` — the task file contract

```toml
[task]
required_sections = ["Plan", "Acceptance"]
optional_sections = ["Summary", "Manual checks", "Context", "Findings", "Review", "Spec updates"]
extra_fields = []
explicit_only = true
```

- **`required_sections`** — every task file must have these `##` sections; `check` errors on a missing one. `task new` creates them.
- **`optional_sections`** — sections `check` knows but doesn't require. Anything else in a task body is plain prose (fine, just unvalidated).
- **`extra_fields`** — project-specific frontmatter fields the agent may set with `task set` (e.g. `owner`, `labels`). Built-ins (`source`, `grade`, `depends`, …) need no listing.
- **`explicit_only`** — while `true` (the default), tasks are created only when the user asks, and working-tree out-of-scope findings are **warnings**, so a small direct fix doesn't pressure the agent into creating a task. Set `false` for strict task-per-change projects; that also restores out-of-scope **errors**. `check --staged` (the pre-commit hook) warns either way.

## `[pipeline]` — repair budget

```toml
[pipeline]
max_attempts = 3
```

**`max_attempts`** caps failed verifies: each `verify → fix` move counts, and past the cap the task blocks itself instead of looping. Revisions (review notes, user feedback) never count — new or changed work is not a failure.

## `[acceptance]` — pre-filled checks

```toml
[acceptance]
default = ["cargo test"]
```

**`default`** items are written into every new task's `## Acceptance` by `task new`. Put here whatever the repo's hooks and CI run. specdev never executes them; the agent does, and ticks only what it ran.

## `[scope]` — what never counts as out of scope

```toml
[scope]
always_allowed = ["Cargo.lock", "Cargo.toml"]
```

**`always_allowed`** globs are exempt from the scope check, on top of everything under `specs/` and specdev's own files (`CHANGELOG.md`). Use it for files any task may touch incidentally; prefer exact paths over broad globs.

## `[quality]` — size limits and content rules

```toml
[quality]
errors = false
forbid_tables = false

[quality.task]
max_lines = 150
max_words = 1000
max_plan_steps = 8
max_scope = 6

[quality.grade.tiny]
max_lines = 80
max_words = 500
max_plan_steps = 5
max_scope = 3

[quality.grade.huge]
max_lines = 300
max_words = 2500
max_plan_steps = 12
max_scope = 12

[quality.ctx]
max_lines = 120
max_words = 600

[quality.overview]
max_lines = 200
max_words = 1500

[quality.changelog]
max_lines = 400
max_words = 3000
```

- **`errors`** — `false` (default): quality violations are warnings. `true`: they become errors — and the state commands' safety net then also refuses changes that would add one (e.g. a `scope add` past `max_scope`).
- **`forbid_tables`** — flag Markdown tables and ASCII box diagrams in `specs/` and task files; they read badly in terminal editors.
- **Kind tables** — `task` (open task files), `ctx`, `overview`, `changelog`; other `specs/*.md` get only the table rule. Metrics per file: lines, words, plan steps (tasks), scope entries (tasks).
- **`[quality.task]` is the default grade** — limits for tasks that declare no grade.

## Grades — sizing tasks honestly

A task declares its size with `specdev task set <id> grade <name>`; the name must exist as a `[quality.grade.<name>]` table or `check` errors. `tiny` and `huge` ship by default; defining a table with the same name replaces it, and other built-ins stay. Keys set in a grade win; the rest inherit from `[quality.task]`. A task file's grade sits in its frontmatter (`grade: huge`), visible at the draft review.

The philosophy, in one line: **prefer splitting into smaller tasks ordered by `depends` over raising limits.** A grade you reach for every time is a sign the tasks are the wrong shape; `huge` is for user-mandated umbrella/working-manual tasks, and the human sees the grade at the draft gate. Over-limit warnings say both options — split, or raise the grade.

## Recipes

- **Strict scope discipline:** `[task] explicit_only = false` — every change outside the active task's scope is an error again.
- **A project grade set:** add `[quality.grade.manual]` with the limits a working manual really needs; agents then say `grade: manual` instead of living with warnings.
- **Hard-fail quality:** `[quality] errors = true` — for teams where warnings rot. The safety net enforces it on every state command too.
- **Minimal repo:** no file at all — every default applies; add the file only when a default is wrong for this project.
