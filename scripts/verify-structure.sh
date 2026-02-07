#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

failed=0

echo "Checking repository structure..."

nested_bun_locks="$(find apps -type f -name 'bun.lock' | sort || true)"
if [[ -n "$nested_bun_locks" ]]; then
  echo "ERROR: Found nested Bun lockfiles under apps/."
  echo "$nested_bun_locks"
  echo "Keep only the root lockfile: bun.lock"
  failed=1
fi

tracked_tsbuildinfo="$(
  git ls-files '*.tsbuildinfo' | while IFS= read -r path; do
    if [[ -e "$path" ]]; then
      echo "$path"
    fi
  done
)"
if [[ -n "$tracked_tsbuildinfo" ]]; then
  echo "ERROR: Found tracked TypeScript build info files."
  echo "$tracked_tsbuildinfo"
  echo "These files are generated and must not be committed."
  failed=1
fi

doc_files=(README.md BUILDING.md CONTRIBUTING.md)
stale_src_tauri_refs="$(grep -nH 'src-tauri/' "${doc_files[@]}" | grep -v 'apps/desktop/tauri/src-tauri/' || true)"
if [[ -n "$stale_src_tauri_refs" ]]; then
  echo "ERROR: Found stale root-level src-tauri references in docs."
  echo "$stale_src_tauri_refs"
  failed=1
fi

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "Repository structure checks passed."
