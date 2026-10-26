# Glance-over — S22 APK sitting (2026-10-25)

**Current picture: `docs/ANDROID_DOSSIER.md`.** This glance-over is the
first-sitting compile/launch notes; the dossier has Options, short-side
scale, sprite/heat/UI-clip, and the local-only play path.

For any seat reading this after grok: what shipped, why, what was wrong,
what to measure, what the TV/controller story actually is.

Play path is the **S22 Ultra native GameActivity APK**. It does not talk
to the LAN. kirkwood is a later 1080p60 surface. Browser wasm is a
tap-preview, not the phone play path.

## What we did, and why

| Change | Why |
|---|---|
| `warcraft-survivors` `cdylib` + Bevy 0.18 GameActivity Gradle (`house.azeroth.survivors`) | Same Rust client. rustc `aarch64-linux-android` emits a `.so` talking Vulkan. |
| MPQs on `/sdcard/azeroth-survivors/Data`, not in the APK | 5.1 GB. mmap from UFS. |
| `TouchPointerPlugin` + survivors stick/JUMP/F/C | GlueClicks only fire `Pressed → Hovered`. Phone lift used to drop the cursor. |
| `BorderRadius` as `Node.border_radius` | Bevy 0.18: it is a `Node` field, not a `Component`. `hud.rs` already did this. |
| `cargo ndk -P 31` | Default platform 21 has no `libaaudio.so`. cpal on Android links AAudio (API 26+). minSdk is 31. |
| rsync `--exclude /target` | Bare `target` also deleted `crates/benilla-app/src/target/`. |
| `cargo check` wasm before `web-build` | Fail on types before a 7-minute release. |
| MainActivity: 120 Hz mode pin, ADPF session at panel period, sustained performance, GameState | Public 120 Hz recipe on 8 Gen 1. Java, so it does not bounce rustc. |
| Manifest: touchscreen/leanback/gamepad `required=false`, `LEANBACK_LAUNCHER` | `touchscreen required=true` would refuse kirkwood. Phone still has a panel. |

Compiles this sitting (TheCore aarch64, grok-compile 57731778 wasm). Justin
said keep 57731778 until this work is finalized.

## What went wrong (do not repeat)

1. **`WindowMode` / `PresentMode` import** — android-only block used names
   that were not in scope. First cargo-ndk death.
2. **`BorderRadius` in spawn tuples** — second death, E0277, `benilla-app`.
3. **`--exclude target`** — wasm death, missing module, cascade `{float}.to_bits()`.
4. **cargo-ndk default API 21** — third death, `ld.lld: -laaudio`. The rlib
   was fine; the sysroot was the wrong year.
5. **Wrapper last-line `echo`** — task exit 0 while `.exit` was 101. Watch
   the `.exit` file, not the outer bash status.
6. **SoC guesses** — Tensor G3 and Amlogic S905X5 were both wrong. kirkwood
   firmware is **MediaTek MT8696**, GPU **PowerVR GE9215**, 4 GB, HDMI 4K60,
   Bluetooth 5.1. Android Authority (build props) + Google issue tracker
   `488961391` (EGL impl: powervr).
7. **`games-activity` 2.0.2** — Bevy 0.18.1's mobile example still pins this.
   Native `android-activity` 0.6.1 compiles GameActivity **4.4.0** C++ into
   the `.so`. Java 2.0.2 `onCreate` then `RegisterNatives` aborts. Fix:
   `androidx.games:games-activity:4.4.0`, and do **not** enable Gradle prefab.

## Where this can be improved

- **Level-up cards hitch on the S22.** `cards.rs` authors 236×330 Px
  desktop cards with LinearGradient, BoxShadow, RadialGradient glow,
  a shine band, and per-frame tilt toward `RelativeCursorPosition`.
  `hud.rs` LevelUpRoot also spins a 1400×1400 Conic+Radial `Rays` node
  every real-time frame while the sim is paused. Cheapen or skip that
  FX on Android; scale cards from the window; stack them in portrait.
- **HUD/menu are a 1024×768 landscape canvas.** First APKs used
  `height/768` (~4.0) and clumped. Current APK uses `min(w,h)/768`
  (1.875). Logical width is 768; 1024-wide XP/menu is 1920 px on a 1440
  panel, so chrome clips. Next: `Percent` / stack / punch-hole pad.
  Manifest is `fullUser`.
- **Options is a Bevy overlay, not 1.12 GameMenu.** Aethus's Linux
  ELF (`/home/justin/Downloads/AzerothSurvivors`) pause is Resume /
  Options / Quit and opens `wenilla-rogue-app::screens::options` with
  Interface / Action Bars / Combat / Key Bindings / Nameplates / Audio /
  Graphics. This clone never vendored that crate. Pause now has Options
  (`survivors/options.rs`) writing the same CVars (`MasterVolume`,
  `gxVSync`, `renderScale`, `ffxGlow`, …). FrameXML `BenillaOptionsFrame`
  stays unloaded because survivors sets `CaptureMode`.
- **ADPF session is on the Java UI tid.** The 120 Hz budget belongs on the
  Bevy render thread (CPU 7 / X2 on 8 Gen 1). That is JNI after the first
  frame, not this Java stub.
- **`-P 31` rebuilt android-activity / winit / bevy_asset.** A
  `RUSTFLAGS=-L …/31` relink would have kept more cache. Platform 31 is
  still the right minSdk match; do it on the first compile next time.
- **`GilrsPlugin` is disabled** in `benilla-world` boot (`1.12` is
  keyboard/mouse). Survivors already reads `TouchPad` + WASD. Gamepad is a
  mapper onto that resource, not a second move system.
- **Portrait + Vulkan-required** is the phone APK. TV wants landscape,
  1920×1080 @ 60, Vulkan-optional with GLES fallback. That is a product
  flavor or a runtime backend pick, not a second engine.
- **No frame-time journal on device yet.** Do not tune quality cvars until
  `dumpsys gfxinfo` / in-game FPS exist.
- **Oboe exclusive** is still cpal/AAudio. Mixer work after a real run.
- **tv_limb :9811** is hung waiting on kirkwood ADB. Do not start a second
  daemon. Wireless debugging is off (`192.168.0.215:5555` refused; Cast
  8008/8443 open).

## Performance tests (after the APK launches)

Industry-standard, phone first. No Magisk.

With `house.azeroth.survivors` focused:

```text
adb shell dumpsys display          # renderFrameRate, mode id (want 120)
adb shell dumpsys gfxinfo house.azeroth.survivors framestats
adb shell dumpsys SurfaceFlinger --latency
adb shell dumpsys meminfo house.azeroth.survivors
adb shell dumpsys thermalservice
```

In-game: existing FPS journal (`benilla-app` `perf`). 30 s combat, 30 s
idle camera, 30 s menu. If `renderFrameRate` is 60, fix Game Booster /
SurfaceFlinger before blaming Rust. If GPU bound (high frametime, cool
CPU): quality step 1080×2316 @ 120. If CPU bound: pin render tid, then
look at BLP/ADT on the IO pool.

Perfetto / Android GPU Inspector is the studio-grade capture if those
numbers are ugly. Snapdragon Profiler is S22-only.

## Controllers (names)

Justin prefers the **gamepad that plugs into the Wiimote**. That piece is
the **Wii Classic Controller** or **Classic Controller Pro** (extension
port on the bottom of the remote). It is not the Wii U GamePad.

| Plastic | How it talks | On Android TV / this APK |
|---|---|---|
| **Wii Remote** | Bluetooth, Nintendo report format (L2CAP), not HID gamepad | Radio may pair (`Nintendo RVL-CNT-01`). OS does not emit `KEYCODE_BUTTON_*` / analog axes. |
| **Classic Controller / Classic Controller Pro** | I2C extension **through the Wiimote** | Same stack as the remote. Sticks live in extension reports. Android gilrs will not see them. |
| **Wii U Pro Controller** | Standalone Bluetooth (no plug into a Wiimote) | Often HID. Try this before writing a decoder. |
| **Wii U GamePad** (the tablet) | Proprietary 5 GHz link to a Wii U | Will not pair to kirkwood or the S22. Linux `hid-wiimote` says so explicitly. |
| **8BitDo GBros / Mayflash** | Classic Controller plug **or** Wiimote in, Xbox/D-input HID out | This is the clean path for the preferred plastic. Streamer reviewers got **D-input**, Switch protocol flaky. |
| Xbox / 8BitDo D-input / many Switch Pro in HID | Standard Android gamepad | gilrs, once we turn the plugin on. |

House choice: **Wii U Pro Controller**. Pair in Settings → remotes on the
ethernet Streamer (and on the S22 if playing there). `GilrsPlugin` stays
off until the APK stays up.

Classic / Classic Pro on a Wiimote is still the original preferred plastic;
that path is 8BitDo GBros / Mayflash (Xbox/D-input) later.

House S22 is **SM-S908U** (US Snapdragon). Magisk needs an unlockable
bootloader; that toggle does not exist on U/U1. International S908B/E can
unlock. kirkwood (Google TV Streamer) has no OEM unlock either — published
"root" on that puck is a lab EM-glitch, not Magisk. These two boxes are
not Magisk-able in the house.

## Vulkan on kirkwood vs GLES

Manifest has `android.hardware.vulkan.level` **required**. That is correct
for the S22 (Adreno 730). It is a **guess** on kirkwood until ADB is on:

```text
adb -s 192.168.0.215:5555 shell pm list features | grep vulkan
adb -s 192.168.0.215:5555 shell ls /vendor/lib64/hw/vulkan* /vendor/lib/hw/vulkan*
```

PowerVR GE9215 is a tiny Rogue. Some MT8696 devices expose Vulkan 1.0/1.1;
Google's own kirkwood GPU-leak bug is on the **OpenGL ES / EGL** path
(`EGL impl: powervr`). Reviews of the Streamer say RetroArch Vulkan works.
Treat that as unverified on *this* unit.

Alternative if `pm list features` has no Vulkan (stay in Rust/Bevy):

- Same GameActivity APK, `vulkan.level` **not** required.
- wgpu already has a GLES backend (`WGPU_BACKEND=gl`). Pick at boot:
  Vulkan if `Backends::VULKAN` instantiates, else GL.
- Do **not** port to Unity/Godot. Do **not** ship ANGLE/Zink ourselves
  (Zink is GL-on-Vulkan; useless if Vulkan is the missing piece).
- Cap TV swapchain at 1920×1080 @ 60. 4K on GE9215 is a fill-rate trap.
  4 GB RAM: mmap MPQs, do not decode the whole chain.

Phone APK stays Vulkan-required until that probe.

## After `.so` lands

1. `gradle assembleDebug` (Java ADPF already in MainActivity).
2. `adb install -r` USB `R5CT62YWN6V`, `appops MANAGE_EXTERNAL_STORAGE allow`,
   `am start -n house.azeroth.survivors/.MainActivity`.
3. Run the perf block above.
4. Then gilrs mapper + TV flavor.
