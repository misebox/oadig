# oadig

**OpenAPI dig** — a CLI to extract structured slices from large OpenAPI specs (3.1 primary, 3.0 compatible).

Designed for non-interactive use, including AI agents. When a full spec is too large to pass to an LLM, use `oadig` to extract only what's needed.

---

## Install

**Cargo** (available after crates.io release):

```sh
cargo install oadig
```

**Homebrew** (coming soon):

```sh
brew install misebox/tap/oadig
```

**Pre-built binaries** — macOS arm64, Linux x86_64, Windows x86_64 available on [GitHub Releases](https://github.com/misebox/oadig/releases).

---

## Quick Start

```sh
# Summarize spec before passing to an LLM
oadig overview openapi.yaml

# Extract a schema with $ref resolved
oadig schema User openapi.yaml

# Read from stdin
curl -s https://example.com/openapi.json | oadig paths -

# Compact JSON, pipe to jq
oadig paths openapi.yaml -c | jq '.[] | select(.method == "GET")'
```

---

## Subcommands

| Subcommand | Description |
|---|---|
| `info <file>` | title, version, description, contact, license, servers |
| `overview <file>` | info + stats + paths combined |
| `stats <file>` | path/operation/schema counts, by method and tag |
| `paths <file>` | list of method + path pairs |
| `schemas <file>` | list of component schema names |
| `schema <name> <file>` | full detail of a named schema, with `$ref` resolved |

`<file>` accepts a file path or `-` for stdin. JSON and YAML are detected automatically.

---

## Options

| Flag | Default | Description |
|---|---|---|
| `--format json\|yaml` | `json` | Output format |
| `-c`, `--compact` | off | Compact JSON (no newlines) |
| `--resolve-refs` / `--no-resolve-refs` | resolve | Inline `$ref` expansion |
| `--max-depth <N>` | unlimited | Max `$ref` resolution depth |

---

## $ref Handling

By default, `$ref` values are resolved and inlined into the output.

- **Circular references** are replaced with `{"$circular_ref": "#/..."}` markers instead of looping.
- `--no-resolve-refs` preserves `$ref` strings as-is from the source.
- `--max-depth N` stops resolution at depth `N`, leaving remaining `$ref` strings intact.

---

## Limitations

- **Swagger 2.0 is not supported.** OpenAPI 3.0 and 3.1 only.
- General-purpose JSON querying (filtering, transformation, projection) is out of scope — use `jq` for that.
- No validation of spec correctness; invalid specs may produce partial or unexpected output.
- Phase 1 (v0.1.0) covers the subcommands listed above. Path details, tag filtering, search, and validation are planned for future phases.

---

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.

---

## Contributing

Issues and pull requests are welcome at [github.com/misebox/oadig](https://github.com/misebox/oadig).
