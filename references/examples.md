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

**After** (agent needs more input):

```md
## Architecture

The API gateway routes requests to microservices.

^^^ Not sure about microservices — is this project big enough?
&&& Clarification needed: what is the expected scale? If this serves fewer
than 10k requests/day, a monolith with clear module boundaries may be
simpler. Should I draft both options?
```

The `^^^` stays because the question is not fully resolved yet.

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
