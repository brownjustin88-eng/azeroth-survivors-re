#!/usr/bin/env bash
# Cross-compile warcraft-survivors for arm64 Android and wrap a debug-signed APK.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-/home/justin/Android/Sdk}"
export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_SDK_ROOT/ndk/30.0.14904198}"
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v cargo-ndk >/dev/null; then
  echo "cargo-ndk is missing — install with: cargo install cargo-ndk --locked" >&2
  exit 1
fi

# TheCore is 16 GiB; two rustc jobs and fewer CGUs keep the box usable.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-4}"

# Gradle reads sdk.dir from here; ANDROID_HOME alone is not always enough.
printf 'sdk.dir=%s\n' "$ANDROID_SDK_ROOT" > "$ROOT/android/local.properties"

# cargo-ndk defaults to platform 21. AAudio (cpal on Android) exists from API 26;
# this APK's minSdk is 31, and API 21's aarch64 sysroot has no libaaudio.so.
PLATFORM="${CARGO_NDK_PLATFORM:-31}"
AAUDIO="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/${PLATFORM}/libaaudio.so"
test -f "$AAUDIO" || { echo "missing $AAUDIO (need cargo-ndk -P ${PLATFORM})" >&2; exit 1; }
test -d "$ROOT/crates/benilla-app/src/target" || { echo "missing crates/benilla-app/src/target (do not rsync-exclude this)" >&2; exit 1; }

echo "==> cargo ndk -P ${PLATFORM} --release -p warcraft-survivors --lib --no-default-features"
cargo ndk -t arm64-v8a -P "$PLATFORM" -o android/app/src/main/jniLibs \
  build --release -p warcraft-survivors --lib --no-default-features

SO=android/app/src/main/jniLibs/arm64-v8a/libwarcraft_survivors.so
test -f "$SO" || { echo "missing $SO" >&2; exit 1; }
echo "==> $(du -h "$SO" | awk '{print $1}') $SO"

# oboe (kira's Android backend) needs the NDK C++ shared runtime next to the game .so
CXX_SHARED="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
test -f "$CXX_SHARED" || { echo "missing $CXX_SHARED" >&2; exit 1; }
cp -f "$CXX_SHARED" android/app/src/main/jniLibs/arm64-v8a/libc++_shared.so
echo "==> copied libc++_shared.so"

echo "==> gradle assembleDebug"
"$ROOT/android/gradlew" -p "$ROOT/android" assembleDebug --no-daemon

APK="$ROOT/android/app/build/outputs/apk/debug/app-debug.apk"
test -f "$APK" || { echo "missing $APK" >&2; exit 1; }
echo "==> $APK ($(du -h "$APK" | awk '{print $1}'))"
