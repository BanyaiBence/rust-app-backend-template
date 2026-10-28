# backend

A template for Rust JSON REST backends: [axum](https://docs.rs/axum) on tokio,
PostgreSQL through compile-time checked [sqlx](https://docs.rs/sqlx) queries, argon2id
password hashing and JWT bearer authentication. It comes with registration, login and an
example `notes` resource to copy, plus tests at every layer, OpenAPI docs, Docker packaging
and CI.

## Quickstart

Prerequisites: Rust (via [rustup](https://rustup.rs); the version is pinned in
`rust-toolchain.toml`), Docker with Compose, and sqlx-cli:

```sh
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

Then:

```sh
make setup   # create .env, start the test database, run migrations, cache queries
make run     # start the server on http://localhost:3000
```

Open <http://localhost:3000/docs> for the interactive API reference, or try it from the shell:

```sh
curl -s localhost:3000/auth/register -H 'content-type: application/json' \
  -d '{"username":"alice","password":"password123"}'
TOKEN=$(curl -s localhost:3000/auth/login -H 'content-type: application/json' \
  -d '{"username":"alice","password":"password123"}' | sed 's/.*"token":"\([^"]*\)".*/\1/')
curl -s localhost:3000/notes -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -d '{"title":"hello"}'
```

`make doctor` checks which tools are installed; `make help` lists every task.

## Project layout

```text
src/
  main.rs         startup: config, logging, pool, migrations, serve, graceful shutdown
  lib.rs          build_app(): routes + middleware (request id, tracing, CORS, body limit)
  config.rs       Config::from_env(); every environment variable is parsed here
  state.rs        AppState shared by handlers (pool + config)
  error.rs        AppError -> HTTP status + {"code", "error"} body
  extract.rs      AppJson / AppPath / AppQuery: axum extractors that fail with AppError
  openapi.rs      OpenAPI metadata; paths come from the handler annotations
  db.rs           connection pool
  telemetry.rs    tracing subscriber (pretty or JSON logs)
  auth/           password hashing, JWT issue/verify, AuthUser extractor
  models/         request/response types and their validation
  repo/           SQL, one module per table; the only code that talks to the database
  routes/         HTTP handlers, one module per resource, each with its own router()
  bin/openapi.rs  prints the OpenAPI document (make openapi)
migrations/       forward-only SQL migrations, applied by `make migrate` and at startup
tests/            unit/ (no DB), repo/ and api/ (db-test), e2e/ (running stack), common/
docs/             ADRs, guides, API walkthrough (api.hurl), architecture diagrams
```

Dependencies point one way: `routes` → `models`, `repo`, `auth` → `error`. Handlers stay thin:
validate input, call the repository, shape the response. SQL lives only in `repo`.

A request passes through: request id → tracing span → CORS → body limit → extractors
(`AuthUser` checks the bearer token, `AppJson` parses the body) → handler → `AppError` or
JSON response.

## Everyday development

| Task | Command |
| ---- | ------- |
| Run the server, restart on change | `make watch` (needs [bacon](https://dystroy.org/bacon/)) |
| All checks CI runs | `make ci` |
| Tests | `make test`, or `make test-unit` / `test-repo` / `test-api` |
| New migration | `make migration name=create_widgets`, then `make migrate` |
| After changing SQL or a migration | `make sqlx-prepare`, commit `.sqlx/` |
| Fresh test database | `make db-reset` |
| SQL shell on the test database | `make psql` |
| Full stack in Docker | `make up`, `make logs`, `make down` |
| API smoke test against a server | `make smoke` (needs [hurl](https://hurl.dev)) |
| Export the OpenAPI document | `make openapi` |

To add your own resource, follow [docs/adding-a-resource.md](docs/adding-a-resource.md).

### Databases

`make db-up` starts **db-test** on port 5433, with its data in tmpfs: it is what `.env` points
at, and what `cargo run`, the tests and sqlx's compile-time checks use. `#[sqlx::test]` gives
every test its own fresh database on that server. `make up` starts a separate **db** with a
persistent volume for the containerized app; it is not published to the host.

## Configuration

Read from the environment, and from `.env` when present (`make setup` copies
`.env.example`).

| Variable | Default | Meaning |
| -------- | ------- | ------- |
| `DATABASE_URL` | required | PostgreSQL connection string |
| `JWT_SECRET` | required | HS256 signing key; use a long random value outside development |
| `JWT_TTL_SECONDS` | `3600` | Token lifetime |
| `DB_MAX_CONNECTIONS` | `10` | Connection pool size |
| `BIND_ADDR` | `0.0.0.0:3000` | Listen address |
| `RUST_LOG` | `backend=info,tower_http=info` | Log filter ([syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)) |
| `LOG_FORMAT` | `pretty` | `pretty` or `json` |
| `CORS_ALLOWED_ORIGINS` | empty (none) | Comma-separated browser origins, or `*` |

The Compose stack also reads `POSTGRES_PASSWORD` and `APP_PORT` (host port, default 3000).

## API conventions

- JSON in and out. Errors are `{"code": "not_found", "error": "not found"}`; branch on `code`,
  which never changes, and show `error`, which may be reworded
  ([ADR 0002](docs/ADR/0002-error-responses-with-stable-codes.md)).
- Authenticate with `Authorization: Bearer <token>` from `POST /auth/login`.
- Users only ever see their own resources; someone else's is a 404. Lists take
  `?limit=&offset=` and return `{items, limit, offset}`
  ([ADR 0003](docs/ADR/0003-owned-resources-and-offset-pagination.md)).
- Every response carries an `x-request-id` that also appears in the server logs.
- `GET /health` is liveness, `GET /ready` checks the database. `GET /openapi.json` is the
  spec, `GET /docs` browses it.

## Documentation

- [docs/ADR/](docs/ADR/README.md): why things are the way they are. Read the relevant ADR
  before changing an area; record new decisions there.
- [docs/adding-a-resource.md](docs/adding-a-resource.md): the step-by-step for new endpoints.
- [docs/api.hurl](docs/api.hurl): a runnable walkthrough of the API.
- `make architecture`: renders `docs/architecture/*.puml` and builds an architecture PDF
  (`docs/architecture.tex`); `make doc-open` for the Rust API docs.

## Starting a project from this template

1. Rename the crate in `Cargo.toml` (`name`), the `backend::` imports in `tests/`, the
   `--bin backend` and `ENTRYPOINT` in the `Dockerfile`, and `backend=` in `RUST_LOG`
   (`.env.example`, `Dockerfile`, `src/telemetry.rs`).
2. Change the database name, user and password in `docker-compose.yml`, `.env.example`, the
   CI workflow and `make psql`.
3. Update the title in `src/openapi.rs` and `\projectname` in `docs/architecture.tex`.
4. Replace or delete the `notes` example once you have your own resource: its migration,
   model, repo, routes, tests, `docs/api.hurl` entries and the paths in
   `tests/api/openapi.rs`.
5. Review the ADRs: accept the ones you keep, supersede the ones you change.
