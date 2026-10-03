# Examples: Spec Editing and Compression

## Example 1: Addressing a remark

**Before** (user added a remark to `ctx.md`):

```md
## Next Steps

- [ ] Implement user authentication
- [ ] Add session management

^^^ Should we use JWT or session cookies here?
```

**After** (agent addresses the remark):

```md
## Next Steps

- [ ] Implement user authentication
- [ ] Add session management

^^^ Should we use JWT or session cookies here?
&&& Addressed: session cookies. JWT adds complexity for rotation and
revocation that we don't need. Cookies with `httpOnly` + `sameSite` are
simpler and sufficient for this project.
```

## Example 2: Addressing with a clarifying question

**Before:**

```md
## Architecture

The API gateway routes requests to microservices.

^^^ Not sure about microservices — is this project big enough?
```

**While waiting:** Leave the spec unchanged and ask the user in conversation
what scale they expect. The remark remains unresolved, so do not add an
`&&&` answer yet.

## Example 3: Compression

**Before compression** (all remarks resolved):

```md
## Data Pipeline

We need to import CSV files from the legacy system.

^^^ What format are the CSV files in?
&&& Addressed: standard RFC 4180 CSV with UTF-8 encoding, semicolon
delimiters, and a header row.

The import runs nightly.

^^^ Should we validate before or after import?
&&& Addressed: validate before import. Reject the entire batch if any row
fails validation, and log the failing rows for manual review.

^^^ How do we handle duplicates?
&&& Addressed: upsert on a composite key of (source_id, timestamp). Existing
records get updated; new records get inserted.
```

**After compression** (user requested "compress this spec"):

```md
## Data Pipeline

Import CSV files from the legacy system. Files use standard RFC 4180 format
with UTF-8 encoding, semicolon delimiters, and a header row.

The import runs nightly. Validation happens before import: reject the entire
batch if any row fails, and log failing rows for manual review. Handle
duplicates via upsert on composite key (source_id, timestamp). Existing
records are updated; new records are inserted.
```

The dialogue is gone. The decisions remain as clear prose.

## Example 4: Partial compression

**Before compression** (one remark still open):

```md
## API Design

^^^ REST or GraphQL?
&&& Addressed: REST. The API surface is small and well-defined. GraphQL
adds complexity we don't need.

^^^ Should the API be versioned?
&&& Addressed: yes, URL-based versioning (/v1/, /v2/).

^^^ What about rate limiting?
```

**After compression:**

```md
## API Design

REST API with URL-based versioning (/v1/, /v2/). The API surface is small
and well-defined, so GraphQL is not needed.

^^^ What about rate limiting?
```

The resolved dialogue is compressed. The open question stays as `^^^`.

## Example 5: Routing a cluttered ctx.md

The most common failure mode: an agent dumps scope, deferred items, roadmap
pointers, and gotchas into `ctx.md`. The fix is **not** to rewrite the prose —
it is to route each piece to its right home.

**Before** (cluttered `ctx.md`):

```md
Active task: **portal overview page (v1)** — implementation not started.
Roadmap pointer: this supersedes/absorbs the dashboard part of v1.8 in
`specs/roadmap.md`.

Scope:
1. PriceTicker on / — reuse `latest_price_snapshots("usd")`.
2. Trending-entities table — top entities in a 7-day window anchored to the
   latest record's `publication_date`, not wall-clock "now".

Deferred, own follow-up discussions:

- LLM-generated daily content — how to form the LLM's context needs a
  dedicated discussion before any subcommand is designed.
- Charts — deferred; v1 uses tables instead.

v1.9 (calendar/daily-digest page) remains separately unstarted.

## SurrealDB quirks

- `FETCH in` silently fails on relation tables — use `in.field AS alias`
- Case-insensitive search: `string::lowercase(field) CONTAINS $query`
```

**After** — content routed to its home:

`specs/ctx.md` (only the current task, as a checkbox plan):

```md
# Current Task Context: portal overview page (v1)
State: in progress

## Plan
- [ ] PriceTicker on / — reuse `latest_price_snapshots("usd")`
- [ ] Trending-entities table — 7-day window anchored to latest publication_date

## Context
Need a date-bounded variant of top_entity_summaries (repository.rs). Window
anchors to the latest record's publication_date, not wall-clock now.

## Next
Implement the trending-entities query.
```

`specs/roadmap.md` (committed future direction):

```md
- v1.9 calendar/daily-digest page (separate, not yet started)
```

`specs/ideas.md` (uncommitted possibilities):

```md
- LLM-generated daily content: needs a discussion on how to form the LLM's
  context before designing a wisdom subcommand.
- Charts: deferred for v1 (tables instead); revisit rendering approach if back
  in scope.
```

`specs/overview.md` → Gotchas / Knowledge:

```md
## Gotchas / Knowledge

### SurrealDB
- `FETCH in` silently fails on relation tables — use `in.field AS alias`.
- Case-insensitive search: `string::lowercase(field) CONTAINS $query`.
```

Nothing was lost. Everything went where it belongs, and `ctx.md` is now narrow
enough that any agent can resume the task in one read.
