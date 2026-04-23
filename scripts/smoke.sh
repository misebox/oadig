#!/usr/bin/env bash
# Smoke test: run each Phase 1 subcommand against fixtures and print output.
# Usage: scripts/smoke.sh
set -euo pipefail

cd "$(dirname "$0")/.."

run() {
  local label="$1"
  shift
  echo "=== $label ==="
  cargo run --quiet -- "$@"
  echo
}

run "info (yaml)"               info tests/fixtures/petstore.yaml --pretty
run "stats (yaml)"              stats tests/fixtures/petstore.yaml --pretty
run "paths (json)"              paths tests/fixtures/petstore.json --pretty
run "schemas (yaml)"            schemas tests/fixtures/petstore.yaml --pretty
run "schema Pets resolved"      schema Pets tests/fixtures/petstore.yaml --pretty
run "schema Pets no-resolve"    schema Pets tests/fixtures/petstore.yaml --no-resolve-refs --pretty
run "schema Node circular"      schema Node tests/fixtures/circular.yaml --pretty

echo "=== info via stdin ==="
cat tests/fixtures/petstore.json | cargo run --quiet -- info - --pretty
