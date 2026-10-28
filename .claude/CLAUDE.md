# Backend template

A copyable template for Rust CRUD REST backends: axum, sqlx (PostgreSQL), JWT auth, argon2.
The crate is named `backend`; the library lives in `src/lib.rs` and tests import it as
`backend::…`.

## Workflow

- `make help` lists every task. Use the Makefile targets rather than raw cargo commands, so
  sqlx runs in the right mode.
- Start the test database before testing: `make db-up`. Test targets run migrations first.
- After adding or changing a `sqlx::query*!` macro or a migration, run `make sqlx-prepare` and
  commit `.sqlx/`; `make ci` fails on a stale cache. See ADR 0001.
- Before finishing a change, run `make ci` (fmt-check, clippy with `-D warnings`, sqlx-check,
  tests). CI (`.github/workflows/ci.yml`) runs the same target, plus `make deny` and the e2e
  tests against the Docker image.
- Port 3000 may be taken by another local stack; use `APP_PORT=... make up` and
  `BIND_ADDR=127.0.0.1:... make run` instead of stopping it.

## Conventions

- New endpoints or resources: follow `docs/adding-a-resource.md`, copying the `notes` example
  (migration → model → repo → routes → tests). Keep `README.md` in step with what changes.
- Use `AppJson` / `AppPath` / `AppQuery` instead of axum's extractors, and return `AppError`,
  so every error has the `{"code", "error"}` body (ADR 0002). Never change an existing `code`.
- Repository functions for user-owned rows take the owner's id and filter on it in SQL
  (ADR 0003).
- Register handlers through each module's `router()` with `routes!` and annotate them with
  `#[utoipa::path]` (ADR 0004); update the path list in `tests/api/openapi.rs`.
- Migrations are forward-only: never edit a merged migration, add a new one
  (`make migration name=...`).

## Architectural decisions

Significant architectural decisions are recorded as ADRs in `docs/ADR/`, following
`docs/ADR/template.md`. The rules for when and how to write them are in
`.claude/rules/adr.md` — read the existing ADRs before changing an area they cover, and write a
new one whenever you make a significant decision.
