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

run "info (yaml)"               info tests/fixtures/petstore.yaml
run "overview (yaml)"           overview tests/fixtures/petstore.yaml
run "stats (yaml)"              stats tests/fixtures/petstore.yaml
run "paths (json)"              paths tests/fixtures/petstore.json
run "operations (yaml)"         operations tests/fixtures/petstore.yaml
run "operations --lines"        operations tests/fixtures/petstore.yaml --lines
run "endpoints alias"           endpoints tests/fixtures/petstore.yaml
run "schemas (yaml)"            schemas tests/fixtures/petstore.yaml
run "schema Pets resolved"      schema Pets tests/fixtures/petstore.yaml
run "schema Pets no-resolve"    schema Pets tests/fixtures/petstore.yaml --no-resolve-refs
run "schema Node circular"      schema Node tests/fixtures/circular.yaml

echo "=== info via stdin ==="
cat tests/fixtures/petstore.json | cargo run --quiet -- info -
