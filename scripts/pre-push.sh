#!/usr/bin/env bash
# Pre-push hook: block commits containing likely secrets from being pushed.
# Scans the range remote_oid..local_oid for each ref being pushed.
# Override with `git push --no-verify` if a match is a false positive.
set -euo pipefail

zero=0000000000000000000000000000000000000000

# Regex of secret content patterns (awk/grep ERE).
secret_re='(AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|gh[pousr]_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{40,}|sk-[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]+|-----BEGIN (RSA|OPENSSH|EC|DSA|PRIVATE) KEY-----|eyJ[A-Za-z0-9_-]{20,}\.eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]+)'

# Filenames that should never be committed.
file_re='(^|/)(\.env(\..+)?|.*\.(pem|key|p12|pfx)|id_rsa|id_ed25519|credentials)$'

# Pathspecs to exclude from scans (these files legitimately contain the
# patterns as literals — the hook implementation itself).
exclude_paths=(
  ':(exclude)scripts/pre-push.sh'
  ':(exclude).rusty-hook.toml'
)

fail=0
workflows_changed=0

while read -r local_ref local_oid remote_ref remote_oid; do
  # Deleting a remote branch: nothing to scan.
  if [ "$local_oid" = "$zero" ]; then
    continue
  fi

  if [ "$remote_oid" = "$zero" ]; then
    # New remote ref: compare against origin/main merge-base when possible.
    if base=$(git merge-base "$local_oid" origin/main 2>/dev/null); then
      range="$base..$local_oid"
    else
      # No origin/main yet — scan everything reachable from local_oid.
      range="$local_oid"
    fi
  else
    range="${remote_oid}..${local_oid}"
  fi

  # Filename check: any newly added or modified file with a sensitive name.
  bad_files=$(git diff --name-only --diff-filter=AM "$range" 2>/dev/null | grep -iE "$file_re" || true)
  if [ -n "$bad_files" ]; then
    echo "pre-push: blocked — secret-like filenames in ${range}:" >&2
    echo "$bad_files" | sed 's/^/  /' >&2
    fail=1
  fi

  # Content check: scan only added lines in the diff.
  added=$(git diff "$range" -U0 --no-color -- "${exclude_paths[@]}" 2>/dev/null \
    | grep -E '^\+' | grep -vE '^\+\+\+' || true)
  hits=$(printf '%s\n' "$added" | grep -nE "$secret_re" || true)
  if [ -n "$hits" ]; then
    echo "pre-push: blocked — secret patterns in ${range}:" >&2
    echo "$hits" | sed 's/^/  /' >&2
    fail=1
  fi

  # Flag if any GitHub workflow file changed — linted once after the loop.
  if git diff --name-only "$range" -- \
       '.github/workflows/*.yml' '.github/workflows/*.yaml' 2>/dev/null \
     | grep -q .; then
    workflows_changed=1
  fi
done

if [ "$workflows_changed" -eq 1 ]; then
  if command -v actionlint >/dev/null 2>&1; then
    echo "pre-push: running actionlint on .github/workflows/..."
    if ! actionlint; then
      echo "pre-push: blocked — actionlint found issues" >&2
      fail=1
    fi
  else
    echo "pre-push: warning — actionlint not installed; skipping workflow lint" >&2
    echo "pre-push: install with 'brew install actionlint' (macOS) or see https://github.com/rhysd/actionlint" >&2
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo >&2
  echo "To override after review, re-run with: git push --no-verify" >&2
  exit 1
fi
