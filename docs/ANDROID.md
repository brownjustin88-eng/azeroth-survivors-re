# Native Android — S22 Ultra APK

Living picture of what the phone runs, what it talks to, sprites / heat /
UI: **`docs/ANDROID_DOSSIER.md`**. Timeline: `ANDROID_SITTING.md`. Glance-over:
`ANDROID_REVIEW.md`.

The browser build (`make start` / `wenilla-host`) is the LAN **preview**.
The installed APK is local: Vulkan GameActivity, MPQs **beside** the APK,
no `INTERNET` permission, `GamePlugins { connect: false }`. It does not
use TheCore, Vast, Hall, or `:8090` while you play.

## Why native on the S22 Ultra

Chrome on the phone already shows the menu. It cannot use the panel:

| | Chrome wasm | Native APK |
|---|---|---|
| GPU | WebGPU, compositor | Vulkan on Adreno 730 |
| Refresh | often 24 Hz | 1440×3088 mode pinned at **120 Hz** |
| Data | MPQs over ADB reverse / Wi‑Fi | local `/sdcard/azeroth-survivors/Data` |
| Audio | Web Audio, flickers | Oboe / kira, local files |
| Input | touch ends `Pressed → None` | finger held as a cursor + stick |
| Threads | one | pipelined render + IO pool |

The S22 Ultra (SM-S908U, Adreno 730, 8 cores, 1440×3088 @ 10–120 Hz) is the play phone.
The APK is **arm64-v8a only**. Do not pack the 5.1 GB MPQs into it.

## Layout

- `crates/warcraft-survivors` — `cdylib` `libwarcraft_survivors.so` + desktop bin
- `android/` — Bevy 0.18 GameActivity Gradle wrapper (not cargo-apk)
- Data: `/storage/emulated/0/azeroth-survivors/Data/*.MPQ`
- Config: app external files `benilla-config/`

## Build on TheCore

SDK `/home/justin/Android/Sdk`, NDK `30.0.14904198`, Java 21, rustc **1.98.1**
with target `aarch64-linux-android`. `cargo-ndk` on PATH.

```bash
export ANDROID_SDK_ROOT=/home/justin/Android/Sdk
export ANDROID_NDK_HOME=/home/justin/Android/Sdk/ndk/30.0.14904198
make apk                 # cargo ndk --release + gradle assembleDebug
ANDROID_SERIAL=192.168.0.68:5555 make apk-data
ANDROID_SERIAL=192.168.0.68:5555 make apk-install
adb -s 192.168.0.68:5555 shell am start -n house.azeroth.survivors/.MainActivity
```

Two rustc jobs (`CARGO_BUILD_JOBS=2`) so the 16 GiB box stays usable.
The `/usr/bin/xbuild` on PATH is .NET, not Bevy's mobile tool.

## S22 Ultra knobs (already in the wrapper)

- **Vulkan only** (`WgpuSettings.backends = VULKAN`, high-performance adapter)
- **Fifo present** vsyncs to the panel; MainActivity picks the highest refresh
  mode at the current resolution (mode 1 = 1440×3088 @ 120)
- **`preferMinimalPostProcessing`** so SurfaceFlinger does not cap the game
- **Pipelined rendering** (Bevy default off-wasm) + IO pool 2–8 threads
- **arm64-v8a**, uncompressed `.so` mmap'd from the APK, `largeHeap`
- **libc++_shared.so** copied from the NDK sysroot (oboe; SDK cmake is not installed)
- Portrait, immersive, GameActivity API 31+ (`androidx.games:games-activity:4.4.0` matching android-activity 0.6.1; Bevy's 2.0.2 example aborts `RegisterNatives`)
- **Game Mode** Performance/Battery advertised; `FLAG_KEEP_SCREEN_ON`; Android 15 `setRequestedFrameRate`
- **ADPF** `PerformanceHintManager` session at the panel period (8.33 ms @ 120 Hz) plus Choreographer `reportActualWorkDuration`; `setSustainedPerformanceMode`
- wgpu **Functionality** limits (every Adreno 730 feature the adapter offers)

Original WoW 1.12 is light. 1440p 120 Hz is the first setting; if the Adreno
heats, the OS already has 1080×2316 @ 120 as mode 8.

## After the first S22 launch (Rust, measured)

These wait on a frame-time journal. Do not change them in the dark:

1. Confirm `dumpsys display` `renderFrameRate` is 120 with the game focused.
2. If fill-rate bound (terrain overdraw, glow): quality cvar to 1080×2316 @ 120.
3. Pin the render thread to CPU 7 (X2) and move the ADPF session onto that tid.
4. Second binary: `+neon,+fp16,+dotprod`, then fat LTO (`profile.ship`).
5. Oboe exclusive / low-latency mixer callback.
6. 120 Hz stick sampling, deadzone curve, haptic on JUMP.
7. MPQ mmap from UFS; BLP decode stays on the IO pool.

## Google TV Streamer (kirkwood, later)

Same GameActivity APK family. kirkwood is MediaTek **MT8696** (4× A55 ~1.8 GHz,
PowerVR GE9215, **4 GB RAM**, HDMI **4K60**, Bluetooth 5.1, 32 GB). Sideload is
`adb install` once wireless debugging is on (`192.168.0.215:5555`). Touchscreen
is optional so the TV will accept the package; first play orientation stays
portrait for the S22.

TV defaults that belong in a later pass: 1920×1080 @ 60 Fifo, glow off, MSAA
off (already), landscape, `GilrsPlugin` on. 4K swapchain on that GPU is a
fill-rate trap. 4 GB means mmap the MPQs, do not decode the whole chain.

`GilrsPlugin` is currently **disabled** (1.12 is keyboard/mouse). Survivors
already reads `TouchPad` + WASD + Space/F/C/Esc. Gamepad maps onto that
resource: left stick = `pad.dir`, south = JUMP, east = F, north = C, start =
pause. Do that after the phone `.so` exists.

Controller on kirkwood (names + glance-over: `docs/ANDROID_REVIEW.md`):

The pad that **plugs into the bottom of a Wiimote** is the **Wii Classic
Controller** or **Classic Controller Pro**. That is not the Wii U GamePad
(the tablet; proprietary 5 GHz, will not pair) and not the Wii U Pro
Controller (standalone Bluetooth, no plug).

House pad is the **Wii U Pro Controller** (standalone Bluetooth HID). Pair
it in Settings → remotes on the Streamer (and on the S22 if playing there).
`GilrsPlugin` stays off until the APK stays up. 8BitDo GBros / Mayflash is
the fallback if we later want Classic-on-Wiimote. TV remote D-pad can alias
WASD.

## Touch

Glue clicks fire on `Pressed → Hovered`. A finger lift used to become `None`
because the cursor vanished. `touch_pointer` keeps the last finger on the
window cursor and synthesises left-mouse, so **Enter World** and the race
tower work with a tap.

In play:

- left thumb, bottom-left: analog stick (WASD)
- JUMP / F / C / II: hop, mobility, camera lock, pause
- pinch: zoom
- drag (camera unlocked with C): orbit

## 5570 on TheCore

No hardware Vulkan (only llvmpipe). Chrome WebGPU cannot use that card.
Native `WGPU_BACKEND=gl` is a later experiment, not the play path.
