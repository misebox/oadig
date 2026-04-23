# Changelog

All notable changes to oadig are recorded here. The format roughly follows
[Keep a Changelog](https://keepachangelog.com/), and the project uses
semantic versioning (pre-1.0: each phase bumps minor).

## [Unreleased]

## [0.2.0] — pending

### Added

- **New subcommands**
  - `spec` — emit just the spec version string.
  - `operation <id> <file>` (alias `op`) — drill into one operation, full
    fields with `$ref` resolved. Also accepts `-m METHOD -p PATH`.
  - `request <id> <file>` (alias `req`) — just the `requestBody`.
  - `response <id> <file>` (alias `res`) — the `responses` map;
    `--status` narrows to one code.
  - `operations <file>` (alias `ops`) — list of operations with
    `--filter`, `--include`, `--exclude`.
  - `requests`, `responses`, `statuses`, `tags`, `components` — plural
    list views.
  - `search <keyword> <file>` — full-spec string search with
    `operationRef`, `jsonPath`, and JSON Pointer context.
- **`--filter` DSL** (`key=value`, repeatable, AND) for `paths` and
  `operations`. Keys: method, path, tag, operationId, summary,
  description, deprecated. Path-like keys accept `*` glob.
- **`--include` / `--exclude` field selection** on `operations`,
  `statuses`, `search`, driven by clap ValueEnum so `--help` lists
  every candidate and typos fail fast.
- **`-l, --lines`** output mode: top-level array on multiple lines,
  each element compact — suitable for LLM streaming and `jq` line use.
- **`--show-null`** surfaces expected-but-absent fields as `null`
  instead of omitting the key.
- **Stderr warnings** when a global flag (`--max-depth`,
  `--no-resolve-refs`, `--show-null`) is ignored by the chosen
  subcommand.
- **Partial Swagger 2.0 support**: `spec`/`info`/`paths`/`operations`/
  `tags`/`stats` work; `statuses --include schema` folds legacy
  `response.schema` under `"*/*"`.
- **OAI example fixtures** added under `tests/fixtures/oai/` for
  petstore-expanded, callbacks, links, USPTO, webhooks.

### Changed

- **`paths` now returns only path strings**; the old method+path list
  moved to the new `operations` command. (Breaking)
- **Output keys match the OpenAPI spec**: `requestBody` and `responses`
  rather than the earlier short forms `request` / `response`. CLI flag
  names stay short. (Breaking)
- **`operations` default fields** now lead with `operationId` so the
  identifier is the visual anchor for each entry.
- **Subcommand help order**: meta → drill-down → lists → search, with
  `spec` first so a quick sanity check is always at the top.
- **Preserve insertion order** in JSON output (via `serde_json`
  `preserve_order`) so keys follow the spec rather than appearing
  alphabetically.

### Fixed

- `info` no longer emits `openapi: null` for specs that lack the top-
  level version field. Swagger 2.0 specs now show `swagger: "2.0"`.

## [0.1.0] — 2026-04

Initial release (Phase 1 MVP).

- Commands: `info`, `stats`, `overview`, `paths`, `schemas`,
  `schema <name>`.
- `$ref` resolution with circular detection.
- JSON/YAML input from file or stdin.
- JSON (pretty by default) and YAML output; `-c/--compact`.
