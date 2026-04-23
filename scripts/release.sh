#!/usr/bin/env bash
# Tag and push a release.
#
# Usage:
#   scripts/release.sh patch | minor | major    bump Cargo.toml, commit, tag, push
#   scripts/release.sh --current                tag HEAD at the current version, push
#   scripts/release.sh --dry-run <arg>          preview only, no changes
#   scripts/release.sh -y       <arg>           skip confirmation
#   scripts/release.sh --no-push <arg>          stop after tagging, skip push
#
# `--current` is for bootstrap (first release) or unusual versions set by
# hand (e.g. rc). Use patch/minor/major for routine bumps.
set -euo pipefail

cd "$(dirname "$0")/.."

die() { echo "error: $*" >&2; exit 1; }

usage() {
  sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//' >&2
  exit 2
}

current_version() {
  awk '
    /^\[/ && !/^\[package\]/ { in_pkg = 0 }
    /^\[package\]/            { in_pkg = 1; next }
    in_pkg && /^version[[:space:]]*=/ {
      match($0, /"[^"]*"/)
      print substr($0, RSTART + 1, RLENGTH - 2)
      exit
    }
  ' Cargo.toml
}

next_version() {
  local current="$1" bump="$2" base maj min pat
  base="${current%%-*}"
  IFS='.' read -r maj min pat <<<"$base"
  case "$bump" in
    patch) echo "${maj}.${min}.$((pat + 1))" ;;
    minor) echo "${maj}.$((min + 1)).0" ;;
    major) echo "$((maj + 1)).0.0" ;;
    *)     die "bump must be patch, minor, or major (got: ${bump})" ;;
  esac
}

write_version() {
  local v="$1" tmp
  tmp=$(mktemp)
  awk -v v="$v" '
    BEGIN { in_pkg = 0; done = 0 }
    /^\[/ && !/^\[package\]/ { in_pkg = 0 }
    /^\[package\]/            { in_pkg = 1; print; next }
    in_pkg && !done && /^version[[:space:]]*=/ {
      print "version = \"" v "\""
      done = 1
      next
    }
    { print }
  ' Cargo.toml > "$tmp"
  mv "$tmp" Cargo.toml
}

require_release_state() {
  local branch
  branch=$(git rev-parse --abbrev-ref HEAD)
  [ "$branch" = "main" ] || die "must run on main (current: ${branch})"

  git diff --quiet && git diff --cached --quiet || die "working tree is dirty"

  git fetch --quiet origin main
  [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
    || die "local main is not in sync with origin/main"
}

confirm() {
  [ "$YES" = 1 ] && return 0
  local reply
  read -r -p "$1 [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]]
}

# ---- args ----
DRY_RUN=0
YES=0
PUSH=1
MODE=""   # patch|minor|major|current

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)   DRY_RUN=1 ;;
    -y|--yes)    YES=1 ;;
    --no-push)   PUSH=0 ;;
    --current)   [ -z "$MODE" ] || die "too many args"; MODE="current" ;;
    -h|--help)   usage ;;
    -*)          die "unknown flag: $1" ;;
    *)           [ -z "$MODE" ] || die "too many args"; MODE="$1" ;;
  esac
  shift
done
[ -n "$MODE" ] || usage

# ---- plan ----
current=$(current_version)
[ -n "$current" ] || die "could not read current version from Cargo.toml"
if [ "$MODE" = "current" ]; then
  next="$current"
else
  next=$(next_version "$current" "$MODE")
fi
tag="v${next}"

git rev-parse -q --verify "refs/tags/${tag}" >/dev/null \
  && die "tag ${tag} already exists" || true

echo "current: ${current}"
echo "next:    ${next}  (tag ${tag})"

if [ "$DRY_RUN" = 1 ]; then
  echo "(dry-run; no changes made)"
  exit 0
fi

# ---- execute ----
require_release_state

echo "==> cargo fmt --check"
cargo fmt --all -- --check
echo "==> cargo test"
cargo test --quiet

confirm "Proceed with release?" || die "aborted"

if [ "$MODE" != "current" ]; then
  write_version "$next"
  echo "==> cargo build (updates Cargo.lock)"
  cargo build --quiet
  git add Cargo.toml Cargo.lock
  git commit -m "Release ${tag}"
fi
git tag -a "${tag}" -m "Release ${tag}"

repo_url() {
  local url
  url=$(git config --get remote.origin.url || echo "")
  url="${url#git@github.com:}"
  url="${url#https://github.com/}"
  url="${url%.git}"
  echo "https://github.com/${url}"
}

print_crates_reminder() {
  cat <<EOF

---
Actions: $(repo_url)/actions

crates.io publish is NOT automated.
crates.io is IMMUTABLE: a published version cannot be changed or removed.

  cargo publish --dry-run    # verify
  cargo publish              # irreversible
EOF
}

if [ "$PUSH" = 0 ]; then
  echo
  echo "Release ${tag} prepared locally (push skipped by --no-push)."
  echo "To publish later: git push origin main \"${tag}\""
  print_crates_reminder
  exit 0
fi

echo "==> git push origin main ${tag}"
git push origin main "${tag}"

echo
echo "Release ${tag} published. CI will build binaries and attach them."
print_crates_reminder
