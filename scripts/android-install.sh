#!/usr/bin/env bash
# Sideload the debug APK and grant all-files access for /sdcard/azeroth-survivors/Data.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APK="$ROOT/android/app/build/outputs/apk/debug/app-debug.apk"
PKG=house.azeroth.survivors
SERIAL="${ANDROID_SERIAL:-}"
ADB=(adb)
if [[ -n "$SERIAL" ]]; then
  ADB=(adb -s "$SERIAL")
fi

test -f "$APK" || { echo "no $APK — run scripts/android-build.sh first" >&2; exit 1; }

"${ADB[@]}" install -r -d "$APK"
"${ADB[@]}" shell appops set "$PKG" MANAGE_EXTERNAL_STORAGE allow || true
"${ADB[@]}" shell mkdir -p /sdcard/azeroth-survivors/Data
echo "==> installed $PKG"
echo "    data: /sdcard/azeroth-survivors/Data  (scripts/android-push-data.sh if empty)"
echo "    launch: ${ADB[*]} shell am start -n $PKG/.MainActivity"
