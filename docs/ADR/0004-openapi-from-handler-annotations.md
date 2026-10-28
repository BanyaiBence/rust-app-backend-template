# 0004. OpenAPI document generated from handler annotations with utoipa

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

Users of the template, and the frontend in particular, need an accurate description of the
API: paths, request and response bodies, auth requirements, error shapes. Hand-written docs
drift from the code. An OpenAPI document also enables generated typed clients and interactive
"try it" pages. The solution must not need a database or network access to build, because the
Docker image builds offline (ADR 0001).

## Considered options

1. **Hand-written `openapi.yaml`.**
   - Pro: no dependency; full control.
   - Con: drifts from the code; nothing checks it.
2. **utoipa + utoipa-axum: `#[utoipa::path]` on handlers, `ToSchema` on models, routes
   registered through `OpenApiRouter` + `routes!`.**
   - Pro: the router and the document are built from the same registration, so a routed
     handler cannot be missing from the spec; schemas follow the Rust types; the most used
     option for axum.
   - Con: annotation boilerplate on each handler; response statuses in the annotation are
     not checked against what the handler returns; proc-macro compile time.
3. **aide.**
   - Pro: infers more from handler types.
   - Con: smaller ecosystem; ties routing to its own API more deeply.

## Decision

We chose **option 2** (utoipa 6, utoipa-axum 0.3, utoipa-scalar 0.4).

- Each `routes::<resource>::router()` returns an `OpenApiRouter<AppState>` and registers
  handlers with `routes!`; handlers sharing a path go in one `routes!` call.
- `src/openapi.rs` holds document metadata, tags and the `bearer` security scheme.
- The spec is served at `GET /openapi.json`, with a Scalar UI at `GET /docs`.
  `make openapi` exports it to `docs/build/openapi.json` via the `openapi` binary, which needs
  no database.
- An API test asserts the set of documented paths, so adding a route updates a test.

## Consequences

- **Positive:** interactive docs and a client-generation source that stay in sync with the
  routes; frontend developers can explore the API without reading Rust.
- **Negative:** three more dependencies; handlers carry annotation blocks; documented error
  statuses can still drift from reality (the code does not check them). The Scalar page loads
  its JavaScript from a CDN at view time, and `/docs` and `/openapi.json` are public.
- **Follow-up:** gate `/docs` behind configuration if a deployment must not expose its API
  description; consider committing the exported spec and checking it in CI once a generated
  client depends on it.
