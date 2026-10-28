# 0001. Compile-time checked SQL with sqlx, offline cache for builds without a database

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

The repository layer talks to PostgreSQL through sqlx's `query!` / `query_as!` macros, which
check every SQL statement against a real schema while the code compiles. The schema comes
from one of two places:

- **Online:** a live database at `DATABASE_URL`, which must have the migrations applied.
- **Offline:** query metadata saved in `.sqlx/` by `cargo sqlx prepare`, used when
  `SQLX_OFFLINE=true`.

Some builds have no database: the Docker image build, `cargo doc`, clippy in CI and the unit
tests. The repo and API tests need the `db-test` Postgres anyway, because they run queries.

Originally the Makefile exported `SQLX_OFFLINE=true` for every target, but `.sqlx/` had never
been generated, so `make test` and every other build failed with "no cached data for this
query". Offline mode everywhere also means a forgotten `cargo sqlx prepare` is only discovered
at build time, as a confusing error.

## Considered options

1. **Compile-time checked macros, offline mode for every target.**
   - Pro: no target needs a database to compile.
   - Con: every query or migration change must be followed by `cargo sqlx prepare` before
     anything builds, including tests; a stale cache can mask schema drift.
2. **Compile-time checked macros, online for database-backed tests, offline elsewhere, cache
   committed and verified in CI.**
   - Pro: tests always check against the real migrated schema; builds without a database still
     work; CI catches a stale cache.
   - Con: two modes to understand; contributors must remember `make sqlx-prepare`.
3. **Runtime-checked queries (`sqlx::query_as::<_, T>()` without macros).**
   - Pro: no build-time database or cache at all.
   - Con: SQL typos and type mismatches surface only when a test or request runs the query;
     loses the main benefit of sqlx for a template meant to teach good practice.

## Decision

We chose **option 2**.

- `.sqlx/` is committed to the repository.
- The Makefile applies `SQLX_OFFLINE=true` only to targets that need no database: `release`,
  `check`, `clippy`, `doc`, `test-unit` and `test-e2e`. The Dockerfile builds offline too.
- `test`, `test-repo` and `test-api` depend on `migrate` and compile online against `db-test`.
- `make sqlx-check` (`cargo sqlx prepare --check`) is part of `make ci`, so a cache that does
  not match the queries fails CI.
- `make setup` bootstraps a fresh copy: start `db-test`, migrate, generate the cache.

## Consequences

- **Positive:** SQL errors are compile errors; the Docker image and docs build without a
  database; tests never run against a stale cache.
- **Negative:** after changing a query or migration, `make sqlx-prepare` must be run and
  `.sqlx/` committed; `make test` requires `make db-up` first.
- **Follow-up:** the CI pipeline must start `db-test` before running `make ci`. Revisit if
  sqlx changes its offline workflow or the project moves to another query layer.

## References

- sqlx offline mode: <https://docs.rs/sqlx/latest/sqlx/macro.query.html#offline-mode>
- `Makefile` targets `sqlx-prepare`, `sqlx-check`, `setup`
