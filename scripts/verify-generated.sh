#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

scripts/generate-bindings.sh "$@"

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Generated files are out of date. Commit regenerated artifacts." >&2
  echo "Changed files:" >&2
  git status --short >&2
  exit 1
fi

echo "Generated artifacts are up to date."
