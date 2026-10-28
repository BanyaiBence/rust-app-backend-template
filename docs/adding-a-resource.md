# Adding a resource

This walks through adding a new user-owned resource, using `notes` as the model to copy. Every
step has a counterpart in the existing code; open it side by side.

Running example: a `widgets` resource with a `name`.

## 1. Migration

```sh
make migration name=create_widgets      # creates migrations/NNNN_create_widgets.sql
```

Write the table, following `migrations/0002_create_notes.sql`:

- `id UUID PRIMARY KEY` (ids are generated in Rust with `Uuid::new_v4()`).
- `user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE` for owned rows.
- `created_at` / `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
- An index matching the list query, e.g. `(user_id, created_at DESC)`.

Migrations are forward-only and never edited once merged: to change a schema, add a new
migration. Apply it with `make migrate` (the server also applies pending migrations at startup).

## 2. Model — `src/models/widget.rs`

Copy `src/models/note.rs`:

- The row/response struct (`Widget`) derives `Serialize` and `ToSchema`. Keep secrets out of
  it, or add a separate response type as `models/user.rs` does for `UserResponse`.
- Request bodies (`CreateWidget`, `UpdateWidget`) derive `Deserialize` and `ToSchema`, with a
  `validate()` that returns `AppError::Validation`. Partial updates use `Option` fields.
- Register the module in `src/models/mod.rs`.

## 3. Repository — `src/repo/widget_repo.rs`

Copy `src/repo/note_repo.rs`. The rules (ADR 0003):

- Every function takes the owner's `user_id` and filters on it, so another user's widget is
  `NotFound`.
- Use the `sqlx::query_as!` / `query!` macros, so SQL is checked at compile time.
- Return `Result<_, AppError>`; `?` converts sqlx errors (no row → `NotFound`, unique
  violation → `Conflict`). Use `fetch_one` when absence should be a 404.

Then refresh the offline query cache and commit it (ADR 0001):

```sh
make sqlx-prepare
```

## 4. Handlers — `src/routes/widgets.rs`

Copy `src/routes/notes.rs`:

- Take `AuthUser(user_id)` to require a bearer token.
- Use the `AppJson`, `AppPath` and `AppQuery` extractors (`src/extract.rs`), not axum's
  directly, so bad input produces the standard error body (ADR 0002).
- Annotate each handler with `#[utoipa::path(...)]`: method, path, `tag`,
  `security(("bearer" = []))`, request body and every response status (ADR 0004).
- Lists take `PageParams` and return `Page<Widget>`.
- Expose `pub fn router() -> OpenApiRouter<AppState>`; handlers that share a path go in one
  `routes!(...)` call.

Register it in `src/routes/mod.rs` (`.merge(widgets::router())`) and add a tag in
`src/openapi.rs`.

## 5. Tests

| Suite | File | What to cover |
| ----- | ---- | ------------- |
| unit | `tests/unit/models/widget.rs` | validation rules |
| repo | `tests/repo/widget_repo.rs` | CRUD, ordering, paging, another user's rows are `NotFound` |
| api  | `tests/api/widgets.rs` | status codes and bodies, auth required, cross-user isolation |

Add the module to the suite's `main.rs` / `mod.rs`. `tests/common/mod.rs` has `app()`,
`send()` and `register_and_login()`. Then add the new paths to the list in
`tests/api/openapi.rs`.

## 6. Check and document

```sh
make ci          # fmt, clippy, sqlx cache, all tests
make watch       # try it: http://localhost:3000/docs
```

Optionally add requests to `docs/api.hurl` and a diagram to `docs/architecture/`. If the
resource needed a decision the ADRs do not cover (shared ownership, a different pagination
scheme, soft deletes), write an ADR (`.claude/rules/adr.md`).
