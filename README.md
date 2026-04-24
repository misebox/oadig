# oadig

**OpenAPI dig** — extract structured slices from large OpenAPI specs.

Non-interactive, composable with `jq` and LLM pipelines.

Run `oadig --help` and `oadig <subcommand> --help` for the command and flag reference. This README covers what `--help` cannot.

---

## Install

```sh
cargo install oadig
```

Pre-built binaries for macOS arm64, Linux x86_64, and Windows x86_64 are on [GitHub Releases](https://github.com/misebox/oadig/releases).

Homebrew:

```sh
brew install misebox/tap/oadig
```

## Uninstall

```sh
cargo uninstall oadig

# or, if installed via brew
brew uninstall oadig
brew untap misebox/tap   # optional: drop the tap entirely
```

---

## Quick Start

```sh
# Orient yourself
oadig overview openapi.yaml

# Narrow to a subset of operations, one entry per line
oadig operations --filter 'tag=Customers' -l openapi.yaml

# Drill into a single operation
oadig op getUser openapi.yaml

# Search string values anywhere in the spec
oadig search webhook openapi.yaml

# Read from stdin
curl -s https://example.com/openapi.json | oadig paths -

# Pipe to jq
oadig operations -c openapi.yaml | jq '.[] | select(.deprecated == true)'
```

---

## Usage

```
Extract specific info from OpenAPI specs

Usage: oadig [OPTIONS] <COMMAND>

Commands:
  spec        Emit the spec version string (openapi 3.x or swagger 2.0)
  info        Show title, version, description, contact, license, servers
  stats       Show counts: paths, operations, schemas, tags, methods
  overview    Combined info + stats + operations
  operation   Show a single operation with every field, $refs resolved.
  request     Show the requestBody of a single operation, $refs resolved.
  response    Show the responses of a single operation, $refs resolved.
  schema      Show a single component schema definition
  paths       List path strings (keys of the `paths` object)
  operations  List operations (method + path, with configurable extras)
  requests    List requestBodies of operations that have one
  responses   List responses of every operation. Optionally narrow to one status
  statuses    List unique status codes used across the spec with a description
  tags        List declared and referenced tags with operation counts
  components  Show component sections and the names defined in each
  schemas     List component schema names
  validate    Validate a spec against the OpenAPI 3.x schema
  convert     Convert a spec to a target version. Supports Swagger 2.0 → 3.0 / 3.1 and OpenAPI 3.0 → 3.1
  search      Search string values in the spec for a keyword
  help        Print this message or the help of the given subcommand(s)

Options:
      --format <FORMAT>        [default: json] [possible values: json, yaml]
  -c, --compact                Compact JSON output (no-op for YAML). JSON is pretty by default
  -l, --lines                  JSON: top-level array on multiple lines, each element on one line. Falls back to pretty for non-array values. No-op for YAML
      --resolve-refs           Resolve $ref inline (default). Use --no-resolve-refs to disable
      --no-resolve-refs
      --max-depth <MAX_DEPTH>
      --show-null              Emit `null` for expected-but-absent fields instead of omitting the key
  -h, --help                   Print help
  -V, --version                Print version
```

Each subcommand has its own `--help` with arg details.

---

## `--filter` DSL

Supported by `paths` and `operations`. Repeat `--filter` to narrow the result further.

| Key | Accepted values | Example |
|---|---|---|
| `method` | HTTP method | `method=GET,POST,PUT` |
| `path` | Glob: `foo` exact, `foo*` prefix, `*foo` suffix, `*foo*` contains. Quote the value. | `path=/v1/*` |
| `tag` | Tag name | `tag=users,admin` |
| `operationId`, `summary`, `description` | Same glob as `path` | `summary=*deprecated*` |
| `deprecated` | `true` or `false` | `deprecated=true` |

`paths` only accepts the `path` key; `operations` accepts all keys.

```sh
oadig operations --filter 'method=GET' --filter 'path=/v1/*' openapi.yaml
oadig paths --filter 'path=*admin*' openapi.yaml
```

Regex is not supported here — use `search` for that.

---

## `$ref` Handling

- Resolved inline by default.
- Circular references become `{"$circular_ref": "#/..."}` (no infinite loop).
- `--max-depth N` exceeded becomes `{"$truncated_depth": N}`.
- `--no-resolve-refs` preserves `$ref` strings as-is.
- External (cross-file) `$ref` are not resolved.

---

## Swagger 2.0 Support

Swagger 2.0 specs are auto-converted to OpenAPI 3.0 internally so every data command behaves the same. Commands that depend on the source shape keep seeing the raw spec:

- `spec`, `search`, `validate`, `convert` operate on the original document.
- `overview` reports the original version while running stats and operations on the converted form.
- All other commands see the converted OpenAPI 3.0 shape.

Use `oadig convert 3.0 swagger2.yaml` to emit the converted document explicitly.

---

## Output Conventions

- JSON is pretty by default. `-c` compacts. `-l` puts one array element per line.
- Keys preserve spec order, not alphabetical.
- Output keys follow OpenAPI names (`requestBody`, `responses`); `--include` flag names are shorthand (`request`, `response`).
- Applying a flag the subcommand ignores prints a stderr warning but does not fail.

---

## Limitations

- General-purpose JSON querying (filtering, transformation, projection) is out of scope — use `jq`.

---

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

## Contributing

Issues and pull requests welcome at [github.com/misebox/oadig](https://github.com/misebox/oadig).
