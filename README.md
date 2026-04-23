# oadig

**OpenAPI dig** — a CLI to extract structured slices from large OpenAPI specs.

Designed for non-interactive use, including AI agents. When a full spec is too large to pass to an LLM, use `oadig` to pull out only what's needed.

---

## Install

**Cargo:**

```sh
cargo install oadig
```

**Pre-built binaries** — macOS arm64, Linux x86_64, Windows x86_64 on [GitHub Releases](https://github.com/misebox/oadig/releases).

**Homebrew:** coming soon.

---

## Quick Start

### AI agent workflow

```sh
# Orient: what's in this spec?
oadig overview openapi.yaml

# List operations the agent might call
oadig operations openapi.yaml --filter 'tag=Customers' -l

# Drill into one operation before generating code
oadig op getUser openapi.yaml

# Search for a keyword across the whole spec
oadig search "webhook" openapi.yaml
```

### Interactive exploration

```sh
# Read from stdin
curl -s https://example.com/openapi.json | oadig paths -

# Compact JSON, pipe to jq
oadig operations openapi.yaml -c | jq '.[] | select(.deprecated == true)'

# Filter to a subset, then inspect one
oadig operations stripe-api.json --filter 'method=POST' --filter 'path=/v1/charges*' -l
oadig op createCharge stripe-api.json
```

For testing with real-world large specs (GitHub REST API ~30 MB, Stripe ~10 MB, Amazon SP-API Swagger 2.0), see `tmp/realspecs/README.md`.

---

## Subcommands

### Meta / summary

| Subcommand | Description |
|---|---|
| `spec <file>` | Spec version string (`"3.1.0"`, `"2.0"`, or `null`) |
| `info <file>` | title, version, description, contact, license, servers |
| `stats <file>` | Path/operation/schema counts, by method and by tag |
| `overview <file>` | info + stats + operations in one call |

### Drill-down (single item)

| Subcommand | Alias | Description |
|---|---|---|
| `operation <id> <file>` | `op` | Full operation with `$ref` resolved; also accepts `-m METHOD -p PATH` |
| `request <id> <file>` | `req` | `requestBody` only |
| `response <id> <file>` | `res` | `responses` only; `--status 200` to narrow |
| `schema <name> <file>` | — | Component schema definition |

### Lists

| Subcommand | Alias | Description |
|---|---|---|
| `paths <file>` | — | Path strings; supports `--filter` |
| `operations <file>` | `ops` | `[{operationId, method, path, summary}, ...]`; supports `--filter`, `--include`, `--exclude` |
| `requests <file>` | — | Operations that have a `requestBody` |
| `responses <file>` | — | Each operation's responses; `--status` to narrow |
| `statuses <file>` | — | Deduplicated status codes used across all operations |
| `tags <file>` | — | Declared + referenced tags with operation counts |
| `components <file>` | — | Names in each components section |
| `schemas <file>` | — | Component schema names |

### Search

| Subcommand | Description |
|---|---|
| `search <keyword> <file>` | Full-spec string scan; results include `pointer`, `jsonPath`, `operationRef`, `at`, `value` |

`<file>` accepts a path or `-` for stdin. JSON and YAML are auto-detected.

---

## Options

| Flag | Default | Description |
|---|---|---|
| `--format json\|yaml` | `json` | Output format |
| `-c`, `--compact` | off | Compact JSON (no newlines) |
| `-l`, `--lines` | off | Array: top-level newline-separated, each element compact. Good for LLM streaming or `jq` line mode |
| `--resolve-refs` / `--no-resolve-refs` | resolve | Inline `$ref` expansion |
| `--max-depth <N>` | unlimited | Stop `$ref` expansion at depth N |
| `--show-null` | off | Emit expected-but-absent fields as `null` |

Applying a flag to a subcommand that ignores it prints a warning to stderr but does not error.

---

## `--filter` DSL

Supported by: `paths`, `operations`.
Multiple `--filter` flags are AND-combined.

| Key | Values | Notes |
|---|---|---|
| `method` | `GET`, `POST,PUT` (comma = OR) | `operations` only |
| `path` | `*foo*` / `foo*` / `*foo` / `foo` | glob-style |
| `tag` | `pets`, `users,admin` | `operations` only |
| `operationId` | glob | `operations` only |
| `summary`, `description` | glob | `operations` only |
| `deprecated` | `true` / `false` | `operations` only |

```sh
oadig operations spec.yaml --filter 'method=GET' --filter 'path=/v1/*'
oadig paths spec.yaml --filter 'path=*admin*'
```

---

## `--include` / `--exclude`

Control which fields appear in list output. Values are camelCase enums; typos are rejected and valid candidates are printed in `--help`.

```sh
oadig operations spec.yaml --include all --exclude description
oadig statuses spec.yaml --include schema,headers
oadig search "token" spec.yaml --include jsonPath --exclude pointer
```

**Default included fields:**

| Subcommand | Defaults |
|---|---|
| `operations` | `operationId`, `summary` |
| `statuses` | `description` |
| `search` | `pointer`, `value`, `operationRef`, `at` |

Output keys follow OpenAPI naming: `requestBody`, `responses`, `operationId`. Flag shorthand (`--include request`) maps to the spec key (`"requestBody"`).

---

## `$ref` Handling

- Circular references → `{"$circular_ref": "#/..."}` marker (no infinite loop).
- `--max-depth N` exceeded → `{"$truncated_depth": N}` marker.
- `--no-resolve-refs` preserves `$ref` strings as-is.

---

## Swagger 2.0 Support

Partial:

- `spec`, `info`, `paths`, `operations`, `tags`, `stats` work.
- `statuses --include schema` maps Swagger 2.0 `response.schema` under the `"*/*"` key.
- `content` (media type map) is OpenAPI 3.x only.
- Strict `requestBody` interop is out of scope for v0.2.

---

## Limitations

- General-purpose JSON querying (filtering, transformation, projection) is out of scope — use `jq`.
- No spec validation; invalid specs may produce partial output.
- External `$ref` (cross-file references) are not resolved.

---

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

## Contributing

Issues and pull requests welcome at [github.com/misebox/oadig](https://github.com/misebox/oadig).
