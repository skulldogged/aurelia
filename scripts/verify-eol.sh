#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ALLOWLIST=(
  "apps/mobile/android/gradlew.bat"
)

has_allowlist_entry() {
  local candidate="$1"
  for allowed in "${ALLOWLIST[@]}"; do
    if [[ "$candidate" == "$allowed" ]]; then
      return 0
    fi
  done
  return 1
}

mapfile -t files_with_crlf < <(git grep -Il $'\r' -- . || true)

violations=()
for path in "${files_with_crlf[@]}"; do
  if has_allowlist_entry "$path"; then
    continue
  fi
  violations+=("$path")
done

if [[ "${#violations[@]}" -gt 0 ]]; then
  echo "ERROR: Found CRLF line endings in tracked text files (outside allowlist)."
  printf '%s\n' "${violations[@]}"
  echo
  echo "Use LF line endings. Allowed CRLF files:"
  printf '%s\n' "${ALLOWLIST[@]}"
  exit 1
fi

echo "Line ending checks passed."
