#!/usr/bin/env bash
# Smoke test: run each subcommand against fixtures and print output.
# Usage: scripts/smoke.sh
set -euo pipefail

cd "$(dirname "$0")/.."

F=tests/fixtures/petstore.yaml
FJ=tests/fixtures/petstore.json
FC=tests/fixtures/circular.yaml
FS2=tests/fixtures/swagger2.yaml

run() {
  local label="$1"
  shift
  echo "=== $label ==="
  cargo run --quiet -- "$@"
  echo
}

run "info"                                    info "$F"
run "spec"                                    spec "$F"
run "overview"                                overview "$F"
run "stats"                                   stats "$F"
run "paths (json)"                            paths "$FJ"
run "operations"                              operations "$F"
run "operations --lines"                      operations --lines "$F"
run "operations --include tags,operationId"   operations --include tags,operationId "$F"
run "operations --include all"                operations --include all "$F"
run "operations --exclude summary"            operations --exclude summary "$F"
run "operations --filter method=GET"          operations --filter 'method=GET' "$F"
run "operations --filter path=*petId*"        operations --filter 'path=*petId*' "$F"
run "operations --filter path=/pets/*"        operations --filter 'path=/pets/*' "$F"
run "operations --filter tag=pets"            operations --filter 'tag=pets' "$F"
run "operation listPets"                      operation listPets "$F"
run "operation -m GET -p /pets"               operation -m GET -p /pets "$F"
run "request createPet"                       request createPet "$F"
run "response listPets --status 200"          response --status 200 listPets "$F"
run "requests"                                requests "$F"
run "responses"                               responses "$F"
run "statuses"                                statuses "$F"
run "responses --status 200"                  responses --status 200 "$F"
run "search Pet --lines"                      search --lines Pet "$F"
run "tags"                                    tags "$F"
run "components"                              components "$F"
run "schemas"                                 schemas "$F"
run "schema Pets"                             schema Pets "$F"
run "schema Pets --no-resolve-refs"           schema --no-resolve-refs Pets "$F"
run "schema Node (circular)"                  schema Node "$FC"
run "validate (petstore)"                     validate "$F"
run "validate (swagger2)"                     validate "$FS2"
run "convert 3.0 (swagger2)"                  convert 3.0 "$FS2"
run "convert 3.1 (swagger2)"                  convert 3.1 "$FS2"

echo "=== info via stdin ==="
cat "$FJ" | cargo run --quiet -- info -
