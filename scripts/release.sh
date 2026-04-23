#!/usr/bin/env bash
# Bump Cargo.toml version, commit, and tag. Does NOT push (user pushes manually).
# Usage: scripts/release.sh <version>     e.g. scripts/release.sh 0.1.0
set -euo pipefail

cd "$(dirname "$0")/.."

if [ $# -ne 1 ]; then
  echo "usage: scripts/release.sh <version>" >&2
  exit 2
fi
version="$1"

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "error: version must be semver (e.g. 0.1.0 or 0.1.0-rc.1), got: $version" >&2
  exit 2
fi

tag="v${version}"
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "error: tag ${tag} already exists" >&2
  exit 1
fi

branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" != "main" ]; then
  echo "error: releases must be cut from main (current: ${branch})" >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "error: working tree is dirty" >&2
  exit 1
fi

git fetch --quiet origin main
if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
  echo "error: local main is not in sync with origin/main" >&2
  exit 1
fi

echo "==> cargo fmt --check"
cargo fmt --all -- --check
echo "==> cargo test"
cargo test --quiet

# Bump version. Match only the [package] block by stopping at the next [section].
tmp=$(mktemp)
awk -v v="$version" '
  BEGIN { in_pkg = 0; done = 0 }
  /^\[package\]/ { in_pkg = 1; print; next }
  /^\[/ && !/^\[package\]/ { in_pkg = 0 }
  in_pkg && !done && /^version[[:space:]]*=/ {
    print "version = \"" v "\""
    done = 1
    next
  }
  { print }
' Cargo.toml > "$tmp"
mv "$tmp" Cargo.toml

echo "==> cargo build (updates Cargo.lock)"
cargo build --quiet

git add Cargo.toml Cargo.lock
git commit -m "Release ${tag}"
git tag -a "${tag}" -m "Release ${tag}"

cat <<EOF

Release ${tag} prepared locally.

To publish, run:
  git push origin main "${tag}"

CI will build binaries and attach them to the GitHub Release.
EOF
