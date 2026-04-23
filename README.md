# oadig

**OpenAPI dig** — extract structured slices from large OpenAPI specs.

Built for non-interactive use, including AI agents. When a full spec is too large to pass to an LLM, use `oadig` to pull out only what's needed.

Run `oadig --help` and `oadig <subcommand> --help` for the authoritative command and flag reference. This README covers the non-obvious bits.

---

## Install

```sh
cargo install oadig
```

Pre-built binaries for macOS arm64, Linux x86_64, and Windows x86_64 are on [GitHub Releases](https://github.com/misebox/oadig/releases).

Homebrew tap: coming soon.

---

## Quick Start

### AI agent workflow

```sh
oadig overview openapi.yaml
oadig operations openapi.yaml --filter 'tag=Customers' -l
oadig op getUser openapi.yaml
oadig search webhook openapi.yaml
```

### Shell exploration

```sh
curl -s https://example.com/openapi.json | oadig paths -
oadig operations openapi.yaml -c | jq '.[] | select(.deprecated == true)'
oadig operations stripe-api.json --filter 'method=POST' --filter 'path=/v1/charges*' -l
```

For trying it against real large specs (GitHub REST, Stripe, Amazon SP-API), see `tmp/realspecs/README.md`.

---

## `--filter` DSL

Supported by `paths` and `operations`. Multiple `--filter` flags compose AND.

| Key | Values | Notes |
|---|---|---|
| `method` | `GET`, `POST,PUT` (comma = OR) | `operations` only |
| `path` | `*foo*` / `foo*` / `*foo` / `foo` | glob, **quote the value** to protect from the shell |
| `tag` | `pets`, `users,admin` (comma = OR) | `operations` only |
| `operationId`, `summary`, `description` | glob | `operations` only |
| `deprecated` | `true` / `false` | `operations` only |

```sh
oadig operations openapi.yaml --filter 'method=GET' --filter 'path=/v1/*'
oadig paths openapi.yaml --filter 'path=*admin*'
```

Regex is intentionally not supported here — use `search` for that.

---

## `$ref` Handling

- Resolved inline by default.
- Circular references become `{"$circular_ref": "#/..."}` (no infinite loop).
- `--max-depth N` exceeded becomes `{"$truncated_depth": N}`.
- `--no-resolve-refs` preserves `$ref` strings as-is.
- External (cross-file) `$ref` are not resolved.

---

## Swagger 2.0 Support

Partial:

- `spec`, `info`, `paths`, `operations`, `tags`, `stats` work on Swagger 2.0 specs.
- `statuses --include schema` folds Swagger 2.0's bare `response.schema` under the `"*/*"` key.
- `content` (media type map) is OpenAPI 3.x only.
- Full `requestBody` interop (Swagger 2.0 uses `parameters: [{ in: body }]`) is out of scope for v0.2.

---

## Output Conventions

- JSON is pretty by default. `-c` compacts. `-l` puts one array element per line (friendly for `jq` and LLM streaming).
- Keys preserve insertion order (spec order), not alphabetical.
- Output keys match OpenAPI naming (`operationId`, `requestBody`, `responses`). The corresponding `--include` flag names are shorthand (`request`, `response`).
- Applying a flag that the chosen subcommand ignores prints a warning on stderr but does not fail the run.

---

## Limitations

- General-purpose JSON querying (filtering, transformation, projection) is out of scope — use `jq`.
- No spec validation; invalid specs may produce partial output.
- External `$ref` not supported.

---

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

## Contributing

Issues and pull requests welcome at [github.com/misebox/oadig](https://github.com/misebox/oadig).
