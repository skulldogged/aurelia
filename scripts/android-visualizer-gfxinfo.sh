#!/usr/bin/env bash
set -euo pipefail

PKG="${1:-com.aurelia.app}"
DURATION_SECONDS="${2:-30}"
OUT_DIR="${3:-/tmp}"

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found in PATH"
  exit 1
fi

if [[ -d "$OUT_DIR" ]]; then
  :
elif [[ -e "$OUT_DIR" ]]; then
  echo "Output path exists and is not a directory: $OUT_DIR"
  exit 1
else
  mkdir -p "$OUT_DIR"
fi
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
OUT_FILE="${OUT_DIR}/${PKG//./_}-gfxinfo-${TIMESTAMP}.txt"

echo "Resetting gfxinfo stats for ${PKG}..."
adb shell dumpsys gfxinfo "${PKG}" reset >/dev/null

cat <<EOF
Capture window started.
For the next ${DURATION_SECONDS}s, reproduce this flow on device:
1) Start playback and open mini player.
2) Expand fullscreen player.
3) Toggle visualizer style(s) and return to mini player.
EOF

sleep "${DURATION_SECONDS}"

echo "Collecting gfxinfo output..."
adb shell dumpsys gfxinfo "${PKG}" > "${OUT_FILE}"

echo
echo "Summary (from dumpsys gfxinfo):"
sed -n '/Janky frames:/,/Number Slow issue draw commands/p' "${OUT_FILE}" | sed '/^$/d' || true
echo
echo "Saved full report: ${OUT_FILE}"
