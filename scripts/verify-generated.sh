#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

scripts/generate-bindings.sh "$@"

GENERATED_PATHS=(
  "apps/shared/src/generated"
  "apps/mobile/android/app/src/main/java/uniffi"
  "apps/mobile/ios/AureliaCore/Sources"
)

CHANGED=false
for path in "${GENERATED_PATHS[@]}"; do
  if ! git diff --quiet -- "$path" || ! git diff --cached --quiet -- "$path"; then
    CHANGED=true
    break
  fi
done

if $CHANGED; then
  echo "Generated files are out of date. Commit regenerated artifacts." >&2
  echo "Changed files:" >&2
  git status --short -- "${GENERATED_PATHS[@]}" >&2
  exit 1
fi

echo "Generated artifacts are up to date."
