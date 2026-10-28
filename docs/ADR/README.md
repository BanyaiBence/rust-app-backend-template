# Architectural Decision Records

Each file records one significant architectural decision: its context, the options considered,
the choice made and its consequences. New ADRs start from [template.md](template.md) and are
numbered sequentially (`NNNN-short-kebab-title.md`). Accepted ADRs are not rewritten; a changed
decision gets a new ADR that supersedes the old one.

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](0001-compile-time-checked-sql-with-sqlx.md) | Compile-time checked SQL with sqlx, offline cache for builds without a database | Accepted |
| [0002](0002-error-responses-with-stable-codes.md) | Error responses carry a stable machine-readable code | Accepted |
| [0003](0003-owned-resources-and-offset-pagination.md) | Owned resources are scoped in every query; lists use limit/offset in an envelope | Accepted |
| [0004](0004-openapi-from-handler-annotations.md) | OpenAPI document generated from handler annotations with utoipa | Accepted |
| [0005](0005-ci-and-toolchain.md) | CI on GitHub Actions runs the Makefile targets; pinned toolchain and dependency policy | Accepted |
| [0006](0006-request-ids-and-health-probes.md) | Request correlation IDs and separate liveness and readiness probes | Accepted |
