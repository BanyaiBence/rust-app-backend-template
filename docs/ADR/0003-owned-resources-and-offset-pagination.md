# 0003. Owned resources are scoped in every query; lists use limit/offset in an envelope

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

The template gains its first resource, `notes`, as the worked example users copy for their own
resources. Two choices made there will be repeated in every copy:

- **Authorization of user-owned rows.** A handler could load a row and then compare its owner
  with the caller, or the owner could be part of every query.
- **List endpoints.** Unbounded lists grow without limit; some form of paging is needed, and
  the response shape decides whether paging metadata can be added later without breaking
  clients.

## Considered options

Ownership:

1. **Check after loading** (`find(id)`, then `if note.user_id != caller { 403 }`).
   - Pro: explicit in the handler. Con: easy to forget on one endpoint; a 403 reveals that the
     id exists.
2. **Filter by owner in SQL** (`WHERE id = $1 AND user_id = $2` in every repo function).
   - Pro: impossible to reach another user's row through the repo; foreign rows look exactly
     like missing ones (404). Con: every repo function takes the owner id.

Paging:

1. **Bare array, no paging.** Simple; unbounded responses.
2. **`?limit=&offset=` with a `{items, limit, offset}` envelope.** Simple to implement and
   use, supports jumping to a page; deep offsets get slower and rows can shift between pages
   under concurrent inserts.
3. **Keyset (cursor) pagination.** Stable and fast at any depth; more complex for clients
   and for the template, opaque cursors.

## Decision

**Ownership:** option 2. Repository functions for owned resources take the owner's id and
filter on it; a row owned by someone else is `NotFound`. `ON DELETE CASCADE` removes a user's
rows with the user.

**Paging:** option 2. `PageParams` (`src/models/page.rs`) parses `limit` (1-100, default 50)
and `offset` (≥ 0, default 0); out-of-range values are `validation_failed`. Responses are
`Page<T> { items, limit, offset }`, ordered by a stable key (`created_at DESC, id`).

## Consequences

- **Positive:** cross-user access is ruled out structurally and tested in both repo and API
  tests; the envelope can gain `total` or `next` fields without breaking clients.
- **Negative:** offset paging degrades on very large tables and can skip or repeat rows when
  data changes between requests.
- **Follow-up:** a resource with large or fast-changing lists should switch to keyset
  pagination behind the same envelope (add a `next_cursor` field) under a new ADR. Shared
  resources (not owned by one user) need a separate authorization design.
