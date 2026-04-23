#!/usr/bin/env bash
# Pre-commit hook: format staged .rs files with rustfmt and re-stage them.
# Installed automatically by rusty-hook via `cargo build`.
set -euo pipefail

files=$(git diff --cached --name-only --diff-filter=ACM -- '*.rs')
if [ -z "$files" ]; then
  exit 0
fi

# Format each file; rustfmt picks up edition from Cargo.toml when run that way,
# but for standalone files we pass the edition explicitly.
printf '%s\n' "$files" | xargs rustfmt --edition 2024

# Re-stage the (possibly) reformatted files.
printf '%s\n' "$files" | xargs git add
