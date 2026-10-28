#!/usr/bin/env bash
# Checks that the tools this template uses are installed; `make doctor`.
# Exits non-zero only if a required tool is missing.
set -uo pipefail

cd "$(dirname "$0")/.."

missing=0
ok()   { printf '  \033[32m✓\033[0m %-15s %s\n' "$1" "$2"; }
bad()  { printf '  \033[31m✗\033[0m %-15s %s\n' "$1" "$2"; }
warn() { printf '  \033[33m-\033[0m %-15s %s\n' "$1" "$2"; }

# check <name> <required|optional> <purpose> <install hint> <command...>
check() {
  local name=$1 level=$2 purpose=$3 hint=$4
  shift 4
  if "$@" >/dev/null 2>&1; then
    ok "$name" "$purpose"
  elif [[ $level == required ]]; then
    bad "$name" "$purpose — install: $hint"
    missing=1
  else
    warn "$name" "$purpose (optional) — install: $hint"
  fi
}

echo "Required:"
check cargo     required "build and test"         "https://rustup.rs"                         cargo --version
check docker    required "databases, Compose stack" "https://docs.docker.com/engine/install/" docker compose version
check sqlx-cli  required "migrations, query cache" "cargo install sqlx-cli --no-default-features --features rustls,postgres" sqlx --version

echo "Optional:"
check bacon     optional "make watch"             "cargo install --locked bacon"              bacon --version
check nextest   optional "make test-fast"         "cargo install --locked cargo-nextest"      cargo nextest --version
check cargo-deny optional "make deny"             "cargo install --locked cargo-deny"         cargo deny --version
check hurl      optional "make smoke"             "https://hurl.dev/docs/installation.html"   hurl --version
check plantuml  optional "make diagrams"          "package manager, or Docker is used"        bash -c 'command -v plantuml || command -v docker'
check latexmk   optional "make architecture"      "TeX Live (latexmk + pdflatex)"             latexmk --version
check rsvg-convert optional "make architecture"   "librsvg2-tools, or inkscape"               bash -c 'command -v rsvg-convert || command -v inkscape'

echo "Configuration:"
if [[ -f .env ]]; then
  ok ".env" "present"
  if grep -q '^JWT_SECRET=change-me$' .env; then
    warn "JWT_SECRET" "still the example value; fine locally, never in production"
  fi
else
  bad ".env" "missing — run: make setup (or cp .env.example .env)"
  missing=1
fi

exit $missing
