#!/usr/bin/env bash
# Copy the 14 MPQs onto the phone. The APK does not contain them.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/wow-data/Data"
DEST="${DEST:-/sdcard/azeroth-survivors/Data}"
SERIAL="${ANDROID_SERIAL:-}"

if [[ ! -d "$SRC" ]]; then
  echo "no $SRC — run make fetch-data first" >&2
  exit 1
fi

ADB=(adb)
if [[ -n "$SERIAL" ]]; then
  ADB=(adb -s "$SERIAL")
fi

"${ADB[@]}" shell mkdir -p "$DEST"
echo "==> pushing $SRC -> $DEST"
"${ADB[@]}" push "$SRC/." "$DEST/"
echo "==> $(${ADB[@]} shell ls "$DEST" | wc -l) files on the phone"
