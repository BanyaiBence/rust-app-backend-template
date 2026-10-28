# 0006. Request correlation IDs and separate liveness and readiness probes

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

Log lines from concurrent requests interleave, and nothing tied a log line or a client-side
error report to the request that produced it. Separately, the only health endpoint,
`/health`, says whether the process serves HTTP; load balancers and orchestrators also need to
know whether the app can actually serve requests, which depends on the database. Restarting
the app because the database is down does not help, so the two questions need different
endpoints.

## Considered options

Correlation:

1. **Nothing; rely on timestamps.** Con: ambiguous under load.
2. **`x-request-id` via tower-http's `SetRequestIdLayer`/`PropagateRequestIdLayer`, recorded
   on the request's tracing span.** Pro: a few lines, no new dependency (a tower-http
   feature); clients and proxies can send their own id. Con: a client can set any id, so ids
   are for correlation only, never trusted.
3. **Full OpenTelemetry tracing.** Pro: distributed traces. Con: collector infrastructure; too
   much for the template's default.

Probes:

1. **Only `/health`, checking the database.** Con: a database outage makes the orchestrator
   restart healthy app containers.
2. **`/health` (process only) and `/ready` (runs `SELECT 1`, 503 on failure).** Pro: standard
   liveness/readiness split. Con: one more endpoint.

## Decision

We chose **correlation option 2** and **probe option 2**. Every request gets an `x-request-id`
(a UUID unless the client sent one); it appears as `request_id` on the `request` span, so on
every log line of that request, and is returned in the response and exposed to browsers via
CORS. The Docker healthcheck keeps probing `/health`; `/ready` is for whatever routes traffic.

## Consequences

- **Positive:** a user-reported request id finds the matching logs; probes tell "restart me"
  apart from "don't send me traffic".
- **Negative:** client-supplied ids are not unique or trustworthy; `/ready` costs a database
  round-trip per probe.
- **Follow-up:** adopt OpenTelemetry under a new ADR if the system grows to several services.
