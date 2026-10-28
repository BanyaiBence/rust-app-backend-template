# 0002. Error responses carry a stable machine-readable code

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

Every error goes through `AppError`, which rendered as `{"error": "<message>"}`. Clients that
need to react differently to different failures (show "username taken" next to a form field,
redirect to login on an expired token) could only match on the HTTP status or on the message
text. Several errors share a status (e.g. all 422s), and messages are prose that we want to be
free to reword. No client depends on the format yet: the sibling frontend is still empty, so
this is the cheapest moment to settle it.

## Considered options

1. **Keep `{"error": "..."}`.**
   - Pro: nothing to change.
   - Con: clients must parse prose; rewording a message is a breaking change.
2. **Add a `code` field: `{"code": "not_found", "error": "not found"}`.**
   - Pro: additive and flat; the existing `error` field keeps its meaning; trivial for clients
     to branch on; one `code()` method on `AppError` keeps codes next to statuses.
   - Con: a home-grown format; no standard slot for per-field details.
3. **RFC 9457 `application/problem+json`** (`type`, `title`, `status`, `detail`, ...).
   - Pro: a standard, with an extension mechanism and some client tooling.
   - Con: heavier for a small API; `type` is meant to be a URI we would have to host or invent;
     different content type for errors.

## Decision

We chose **option 2**. Every error body is `ErrorBody { code, error }` (`src/error.rs`), where
`code` comes from `AppError::code()`: `not_found`, `unauthorized`, `conflict`,
`validation_failed`, `payload_too_large`, `internal`. Codes are part of the API contract: an
existing code never changes meaning or spelling; new failure kinds get new codes. Server-side
failures keep hiding their details (`internal` / `internal server error`).

## Consequences

- **Positive:** clients branch on `code` and can show `error` to users or logs; messages can be
  reworded freely; the OpenAPI document describes one error schema for every endpoint.
- **Negative:** codes are coarse (one per `AppError` variant): two different 409s share
  `conflict`. Validation errors carry one message, not per-field details.
- **Follow-up:** if clients need finer distinctions, add variants or a code parameter to
  `AppError` rather than reusing an existing code. Moving to RFC 9457 later would need a new
  ADR and a versioning plan.
