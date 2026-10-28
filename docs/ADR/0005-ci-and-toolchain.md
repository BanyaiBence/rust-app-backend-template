# 0005. CI on GitHub Actions runs the Makefile targets; pinned toolchain and dependency policy

- **Status:** Accepted
- **Date:** 2026-10-28
- **Deciders:** Bence Bányai

## Context

The Makefile's `ci` target claimed to mirror a CI job that did not exist, so nothing enforced
formatting, lints, the sqlx cache check (ADR 0001) or tests on changes. Local builds used
whichever Rust happened to be installed while the Docker image pinned 1.95, so lints and
behaviour could differ between machines. Dependencies were not checked for known
vulnerabilities or license compatibility.

## Considered options

1. **No CI; rely on `make ci` before committing.**
   - Pro: nothing to maintain. Con: unenforced; easy to skip.
2. **GitHub Actions calling the same Makefile targets used locally.**
   - Pro: one definition of "the checks"; a failing CI step reproduces locally with the same
     command; Postgres runs as a service container with db-test's credentials and port.
   - Con: GitHub-specific workflow file; another platform needs its own (thin) equivalent.
3. **GitHub Actions with steps that call cargo directly.**
   - Pro: idiomatic, uses marketplace actions for each tool.
   - Con: duplicates the Makefile's flags (`SQLX_OFFLINE`, `-D warnings`, `--locked`) and drifts.

## Decision

We chose **option 2**, plus:

- `rust-toolchain.toml` pins the toolchain (channel 1.95, rustfmt, clippy) for local and CI
  builds; the Dockerfile's `RUST_VERSION` must match and `.dockerignore` keeps the file out of
  the image build, which uses the image's toolchain.
- Lint policy lives in `Cargo.toml` `[lints]` (`unsafe_code` forbidden; `dbg!`, `todo!`,
  `unimplemented!` warn), and `make clippy` denies warnings.
- `deny.toml` + `make deny` (cargo-deny) fail on RustSec advisories, licenses outside a
  permissive allow-list, and unknown registries or git sources.
- `.github/workflows/ci.yml` has three jobs: `make ci` against a Postgres service, `make deny`,
  and `make up` + `make test-e2e` against the built Docker image.

## Consequences

- **Positive:** every change is checked the same way locally and in CI; the Docker image and
  e2e tests are exercised before merge; vulnerable or incompatibly licensed dependencies are
  caught.
- **Negative:** bumping Rust means editing two files; a new advisory can fail CI on an
  unchanged branch until the dependency is updated or the advisory is consciously ignored in
  `deny.toml`; the workflow is GitHub-specific.
- **Accepted advisories:** at the time of writing, RUSTSEC-2023-0071 (`rsa`) and
  RUSTSEC-2026-0387 (`crypto-bigint`) arrive through jsonwebtoken's `rust_crypto` backend.
  Only HS256 is used, so the affected code never runs; both are ignored in `deny.toml` with
  that reason, to be removed once jsonwebtoken ships fixed versions.
- **Follow-up:** add a release/publish workflow when the template gains a deployment target.
