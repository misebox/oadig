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
run "operations --include tags,operationId"  operations tests/fixtures/petstore.yaml --include tags,operationId
run "operations --include all"               operations tests/fixtures/petstore.yaml --include all
run "operations --exclude summary"           operations tests/fixtures/petstore.yaml --exclude summary
run "operations --method GET"                operations tests/fixtures/petstore.yaml --method GET
run "operations --filter petId"              operations tests/fixtures/petstore.yaml --filter petId
run "operations --prefix /pets/"             operations tests/fixtures/petstore.yaml --prefix /pets/
run "operations --tag pets"                  operations tests/fixtures/petstore.yaml --tag pets
run "operation listPets"        operation tests/fixtures/petstore.yaml listPets
run "operation -m GET -p /pets" operation tests/fixtures/petstore.yaml -m GET -p /pets
run "request createPet"         request tests/fixtures/petstore.yaml createPet
run "response listPets --status 200"  response tests/fixtures/petstore.yaml listPets --status 200
run "requests"                  requests tests/fixtures/petstore.yaml
run "responses"                 responses tests/fixtures/petstore.yaml
run "responses --status 200"    responses tests/fixtures/petstore.yaml --status 200
run "search Pet --lines"        search Pet tests/fixtures/petstore.yaml --lines
run "tags"                      tags tests/fixtures/petstore.yaml
run "components"                components tests/fixtures/petstore.yaml
run "schemas (yaml)"            schemas tests/fixtures/petstore.yaml
run "schema Pets resolved"      schema Pets tests/fixtures/petstore.yaml
run "schema Pets no-resolve"    schema Pets tests/fixtures/petstore.yaml --no-resolve-refs
run "schema Node circular"      schema Node tests/fixtures/circular.yaml

echo "=== info via stdin ==="
cat tests/fixtures/petstore.json | cargo run --quiet -- info -
