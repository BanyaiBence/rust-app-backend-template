# Common development tasks. Run `make` or `make help` for the list.

CARGO ?= cargo

# sqlx checks queries at compile time, against DATABASE_URL (online) or the committed .sqlx
# cache (offline). Targets that need no database build offline; test targets need db-test
# anyway, so they check online against the migrated schema and never see a stale cache.
OFFLINE := SQLX_OFFLINE=true

# Where `make smoke` and `make test-e2e` send requests.
BASE_URL ?= http://localhost:$(or $(APP_PORT),3000)

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*## "} \
		/^##@/ {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)} \
		/^[a-zA-Z0-9_.-]+:.*## / {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

##@ Getting started

.PHONY: setup
setup: .env db-up migrate sqlx-prepare ## Bootstrap a fresh checkout: .env, db-test, migrate, cache queries

.env: ## Create .env from .env.example (never overwrites an existing one)
	cp .env.example .env
	@echo "Created .env from .env.example; adjust it as needed."

.PHONY: doctor
doctor: ## Check that the required and optional tools are installed
	@scripts/doctor.sh

##@ Build and run (the server needs db-test: `make db-up` first)

.PHONY: run
run: .env ## Run the server with a debug build (reads .env)
	$(CARGO) run

.PHONY: watch
watch: .env ## Run the server and restart it on every change (needs bacon)
	bacon run-long

.PHONY: run-release
run-release: .env ## Run the server with a release build
	$(CARGO) run --release

.PHONY: build
build: ## Debug build
	$(CARGO) build

.PHONY: release
release: ## Optimized release build
	$(OFFLINE) $(CARGO) build --release --locked

.PHONY: check
check: ## Type-check all targets without building
	$(OFFLINE) $(CARGO) check --all-targets

##@ Quality

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Fail if the code is not formatted
	$(CARGO) fmt --all --check

.PHONY: clippy
clippy: ## Lint all targets, treating warnings as errors (as CI does)
	$(OFFLINE) $(CARGO) clippy --all-targets --locked -- -D warnings

.PHONY: lint
lint: fmt-check clippy ## fmt-check + clippy

.PHONY: deny
deny: ## Check dependencies for advisories, licenses and sources (needs cargo-deny)
	$(CARGO) deny check

.PHONY: ci
ci: lint sqlx-check test ## Run the same checks as the CI "Lint and test" job (.github/workflows)

##@ Tests (repo/api compile and run against db-test: `make db-up` first)

.PHONY: test
test: migrate ## Run unit, repo and api tests
	$(CARGO) test --locked

.PHONY: test-fast
test-fast: migrate ## Same tests with cargo-nextest: parallel binaries, clearer output
	$(CARGO) nextest run --locked

.PHONY: test-unit
test-unit: ## Pure logic tests; no database needed (builds from the .sqlx cache)
	$(OFFLINE) $(CARGO) test --test unit

.PHONY: test-repo
test-repo: migrate ## Repository tests against db-test
	$(CARGO) test --test repo

.PHONY: test-api
test-api: migrate ## HTTP tests through the full router against db-test
	$(CARGO) test --test api

.PHONY: test-e2e
test-e2e: ## Smoke tests against the running Compose stack (`make up` first)
	E2E_BASE_URL=$(BASE_URL) $(OFFLINE) $(CARGO) test --test e2e -- --ignored

.PHONY: smoke
smoke: ## Run docs/api.hurl against a running server (needs hurl; BASE_URL=...)
	hurl --test --variable base_url=$(BASE_URL) --variable username=smoke_$$(date +%s) docs/api.hurl

##@ Docs (no database needed; architecture needs plantuml, rsvg-convert and latexmk)

.PHONY: doc
doc: ## Generate API docs for this crate
	$(OFFLINE) $(CARGO) doc --no-deps

.PHONY: doc-open
doc-open: ## Generate API docs and open them in a browser
	$(OFFLINE) $(CARGO) doc --no-deps --open

.PHONY: openapi
openapi: ## Export the OpenAPI document to docs/build/openapi.json
	@mkdir -p docs/build
	$(OFFLINE) $(CARGO) run --quiet --bin openapi > docs/build/openapi.json
	@echo "Wrote docs/build/openapi.json (also served at /openapi.json, browsable at /docs)"

.PHONY: diagrams
diagrams: ## Render docs/architecture/*.puml to docs/rendered/*.svg
	docs/build.sh --svg-only

.PHONY: architecture
architecture: ## Render the diagrams and build docs/build/architecture.pdf
	docs/build.sh

##@ Database and sqlx

.PHONY: db-up
db-up: ## Start the throwaway db-test Postgres (port 5433)
	docker compose --profile test up -d --wait db-test

.PHONY: db-down
db-down: ## Stop and remove db-test
	docker compose --profile test rm -sf db-test

.PHONY: db-reset
db-reset: db-down db-up migrate ## Recreate db-test empty and migrate it (its data lives in tmpfs)

.PHONY: psql
psql: ## Open a psql shell on db-test
	docker compose --profile test exec db-test psql -U app -d app

.PHONY: migrate
migrate: .env ## Apply migrations to DATABASE_URL (from .env)
	sqlx migrate run

.PHONY: migration
migration: ## Create the next migration file: make migration name=create_widgets
	@test -n "$(name)" || { echo "usage: make migration name=<snake_case_description>"; exit 2; }
	sqlx migrate add --sequential $(name)

.PHONY: sqlx-prepare
sqlx-prepare: migrate ## Refresh the offline .sqlx query cache (commit the result)
	$(CARGO) sqlx prepare -- --all-targets

.PHONY: sqlx-check
sqlx-check: migrate ## Fail if the .sqlx cache is out of date with the queries
	$(CARGO) sqlx prepare --check -- --all-targets

##@ Docker Compose stack

.PHONY: up
up: .env ## Build and start the app + database stack (APP_PORT=... to change the port)
	docker compose up -d --build --wait

.PHONY: down
down: ## Stop the stack (data volume is kept)
	docker compose down

.PHONY: down-volumes
down-volumes: ## Stop the stack and delete the database volume
	docker compose down -v

.PHONY: logs
logs: ## Follow the app logs
	docker compose logs -f app

##@ Housekeeping

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf docs/build
