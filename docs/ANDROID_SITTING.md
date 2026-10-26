# S22 Ultra APK sitting log — 2026-10-25

Living picture: `docs/ANDROID_DOSSIER.md`. This file is the timestamp log.

Play path: native GameActivity APK on SM-S908U, **local** (no LAN, no
`INTERNET`, MPQs on `/sdcard/azeroth-survivors/Data`). Browser on the
phone is a preview only (menu visible, taps miss, choppy, audio flicker).

## Tooling actually used (TheCore, not a workaround)

| Piece | Where |
|---|---|
| Android SDK | `/home/justin/Android/Sdk` |
| NDK | `30.0.14904198` |
| Platforms | android-34, **35** (installed this sitting), 36 |
| Build-tools | 30.0.3, 34.0.0, 36.1.0 |
| CMake | missing — `libc++_shared.so` copied from NDK sysroot instead |
| Java | OpenJDK 21.0.10 |
| rustc | 1.98.1 + target `aarch64-linux-android` |
| cargo-ndk | 4.1.2 |
| Package | `house.azeroth.survivors` / lib `warcraft_survivors` |

Gradle `local.properties` writes `sdk.dir=/home/justin/Android/Sdk`.

## Timeline

| UTC | What |
|---|---|
| 21:48 | `adb push` 14 MPQs → `/sdcard/azeroth-survivors/Data` (5.4 GB, 91 s, 56.7 MB/s). Still there. |
| 21:48 | First `cargo ndk --release` on TheCore, `CARGO_BUILD_JOBS=2`. |
| 22:13 | First compile died: `WindowMode` / `PresentMode` not in scope in the Android-only boot block. |
| 22:14 | Restart with the import fix. `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4` (first run had been 16). Cache mostly rebuilt. |
| 22:29 | Live: `bevy_pbr` / `winit`. No `.so` yet, no APK yet. |
| 22:40 | House crates: `benilla-world`, `avian3d`, `kira`, `benilla-ui`. `.so` still linking. Not stalled. |
| 22:45 | Rented Vast **57731778** (`grok-compile`, Sweden, Xeon E5-2695 v3 56 threads, RTX 3060 Ti, 80 G disk, $0.118/hr) for wasm + later LTO. Current aarch64 `.so` stays on TheCore (cache already here). |
| 22:44 | Second cargo-ndk **exit 101**: `BorderRadius` in `touch.rs` spawn tuples is not a Bevy 0.18 `Component`. House UI already puts it on `Node.border_radius` (`hud.rs` / `cards.rs`). |
| 22:53 | 57731778 **running** (`ssh1.vast.ai:26116`). rustc 1.98.1, wasm32 installed, 251 GiB RAM, 78 G free overlay, ONSTART_DONE. |
| 22:54 | Moved `BorderRadius` onto `Node` in `touch.rs`. Relaunch cargo-ndk on TheCore. Rsync tree (no wow-data/target) to 57731778 for wasm rebuild. |
| 23:06 | Third cargo-ndk: `benilla-app` **compiled**. Link failed: `ld.lld: unable to find library -laaudio`. cargo-ndk default platform **21**; `libaaudio.so` starts at API 26. Gradle minSdk is 31. Fix: `cargo ndk -P 31`. |
| 23:04 | Wasm on 57731778: wrapper exit 0 was the trailing `echo`. Real rc=101. `--exclude target` also dropped `crates/benilla-app/src/target/`. E0689 `discount.to_bits()` was a cascade. Fix: rsync `--exclude /target`, then `cargo check` before `web-build`. |
| 23:20 | While rustc runs: ADPF hint session + sustained performance in `MainActivity`; touchscreen/leanback/gamepad `required=false` + `LEANBACK_LAUNCHER` so kirkwood can accept the same APK later. kirkwood ADB `:5555` refused (debugging off); ping 192.168.0.215 ok. |
| 23:30 | Justin: keep 57731778 until finalized; perf tests after first launch; preferred pad is gamepad-on-Wiimote = Classic / Classic Pro. Glance-over `docs/ANDROID_REVIEW.md`. kirkwood Cast 8008/8443 open, ADB still off. |
| — | Phone ADB: USB `R5CT62YWN6V` + wifi `192.168.0.68:5555`. |
| ~23:40 | First APK installed. Launch aborted: `RegisterNatives failed for 'com/google/androidgamesdk/GameActivity'`. Java `games-activity` **2.0.2** (copied from Bevy's stale example) vs native `android-activity` **0.6.1** GameActivity **4.4.0** C++ glue already in the `.so`. SPEG also parked the first pid on displayId=6 @ 60 Hz. |
| — | Justin: Wii U Pro is the pad; ethernet Streamer is the Hall TV; unplug the other Google box if it helps; APK "closed because of a bug." |
| 2026-10-25 | Bump `gamesActivity` to **4.4.0**, drop Gradle `prefab` (crate ships its own glue). Gradle-only rebuild — no cargo-ndk. Hall `?hallway=grok` deep-link. |
| 23:58Z | `assembleDebug` 25s, 107M APK. `adb -s R5CT62YWN6V install -r` Success. monkey launch (display 0). |
| 23:59Z | pid **31390** stays up. `AdapterInfo { name: "Adreno (TM) 730", backend: Vulkan }`. `modeId 1`, `renderFrameRate 120.00001`, MainActivity `requestedFrameRate 120 Hz Exact`. Goldshire HUD (Level 3, 10 kills) + stick/JUMP. Audio: `SCHED_FIFO` denied, user-interactive QoS. Pipeline warm timed out 25 pipelines on first cover. PSS ~3.5G during Azeroth stream. |
| 00:00Z | kirkwood Cast `catt -d 192.168.0.215 cast_site http://192.168.0.10:8765/?hallway=grok` (DashCast). Android TV remote `InvalidAuth` (need re-pair). ADB `:5555` still closed. |
| 00:18Z | Pad-shrink + `fullUser` cargo-ndk **EXIT:0**. Overlay `STICK_R` 32 / `BTN` 28×24. Manifest `screenOrientation=fullUser`. |
| 00:18Z | Backup `/home/justin/backups/azeroth-survivors-re-20261026/` (HEAD.bundle 11M, worktree.tar.gz 317M, no MPQs/target). |
| 00:33Z | Sideload pad-shrink APK USB `R5CT62YWN6V` Success, monkey launch, pid **11477**. aapt `screenOrientation` 0xd = fullUser. |
| 00:34Z | Read-only review MCP on grok-compile 57731778 `:9830`, TheCore local-forward + trycloudflare funnel. Card `~/.config/pko/azeroth-survivors.connector.txt` (do not paste bearer into Hall). Units `azeroth-survivors-mcp-tunnel` + `azeroth-survivors-mcp-funnel`. |
| 00:36Z | Hall `@claude-chat` whole-tree review ask. Brief `docs/REVIEW_ASK.md`. Claude-chat Hall seat was stale; torus absorb on :8791 + grok :8788. |
| 00:50Z | Read repo `AGENTS.md` (fork map: cfg-gated carries, METHOD/MAP/CONTRIBUTING). Portrait HUD: `apply_ui_scale` uses short side / 768. Stick `STICK_R` 64. Android `pipe_warm` slice 4 / timeout 45s (M2 bodies vs spell FX). cargo-ndk in flight. |
| 01:00Z | Portrait/stick cargo-ndk **EXIT:0**. Sideload USB `R5CT62YWN6V`, monkey launch, pid **23681**. |
| 01:15Z | Justin: Aethus Linux build at `Downloads/AzerothSurvivors` **has** Options with full listings. Compared ELF strings: pause `ResumeOptionsQuit game`, pages Interface/Action Bars/Combat/Key Bindings/Nameplates/Audio/Graphics, crate `wenilla-rogue-app`. House overlay `survivors/options.rs` (own file). Pause rungs Resume / Options / Abandon / Quit. |
| 01:50Z | Options APK sideload USB `R5CT62YWN6V` Success, monkey launch, pid **2220**. APK 201M. Pause → Options. |
| 02:07Z | Justin: APK is local-only (confirmed: no INTERNET, `connect: false`, 14 MPQs on device). Still no 3D sprites. Smooth, heats, UI chrome off-screen after short-side scale. Dossier `docs/ANDROID_DOSSIER.md`. Game closed; thermal idle AP 40.2 / skin 35.8. |

## What the APK already pins

- Vulkan only, wgpu Functionality limits, MemoryHints::Performance
- Borderless fullscreen + Fifo (forced, config cannot drop it)
- MainActivity: highest refresh at current resolution, Android 15 `setRequestedFrameRate`, `preferMinimalPostProcessing`, `FLAG_KEEP_SCREEN_ON`, Game Mode Performance/Battery
- arm64-v8a only, `extractNativeLibs=false`, `largeHeap`, `isGame`
- MSAA off at 1440×3088@120 (`WOW_MSAA=off`)
- Touch: last finger held as cursor (GlueClicks Pressed→Hovered) + analog stick + JUMP/F/C/II; pad hidden on the menu (`Display::None`)
- Data: `$WOW_DATA` → `/storage/emulated/0/azeroth-survivors/Data`
- lua-src: Android is POSIX+dlopen, not `LUA_USE_LINUX` (that pulls readline)
- arboard gated off Android

## Measured so far

- TheCore 16 GiB: two rustc jobs, ~6–9 GiB available during compile. No OOM.
- First compile ~25 min to `benilla-world`, then import error.
- Restart still compiling Bevy 0.18.1 for aarch64; house crates not linked yet.
- Phone panel **right now** is mode 3 = 1440×3088 **@ 60 Hz** (`renderFrameRate 60`). Mode 1 (120 Hz) exists. Chrome is not holding 120. Native pins mode 1.
- Phone storage: ~20 G free after MPQs.

## Sideload wait

`adb install` needs `android/app/build/outputs/apk/debug/app-debug.apk`.
Gradle needs `android/app/src/main/jniLibs/arm64-v8a/libwarcraft_survivors.so`.
That `.so` is the Bevy game, produced by `cargo ndk`. Until rustc finishes
`warcraft-survivors --lib --release` for `aarch64-linux-android`, there is
nothing to pack or install. After the `.so`: copy `libc++_shared.so`,
`gradle assembleDebug` (~1–2 min), `adb install -r` over USB, `appops`
`MANAGE_EXTERNAL_STORAGE allow`, `am start`.

## WASM / “too slow”

TheCore cannot run a second Bevy release (wasm32, ~86 MB `wenilla_bg.wasm`)
beside this aarch64 job on 16 GiB. That is a **this-box, this-moment**
constraint, not “wasm is slow”. TouchPointerPlugin is already in `GluePlugin`;
a later wasm rebuild picks it up.

Workshop 47993302: 192 cores, ~1 TiB RAM, overlay **4.2 G free of 40 G**,
**no rustc**. A Bevy target dir will not fit. 55473579 is the Evolution
room (150 G overlay historically). A compile-only rent (CPU, 64 GB+, 100 G+
disk, no GPU) would take both jobs at once.

## 2026-10-26 — lag journal + git check

Justin asked: identify what causes lag in the code (not just Options
knobs); did house edits break original git?

Git: HEAD == origin/main `dd5e330`. Zero house commits. Dirty working
tree only. Original history intact. Some survivors HUD/Options/touch
changes also run on desktop/wasm; Android window/present/pipe_warm are
`cfg(target_os = "android")`.

Lag: `FpsJournalPlugin` already shipped in every build. Survivors has
no `/console`, so it never started. Next APK:

- Android `bind_android_paths` sets `WOW_FPS_JOURNAL` to
  `$BENILLA_HOME/Diagnostics/fps-journal.csv` for the whole run.
- On-screen cost pill (mean_ms / p95, 4 Hz, player build).
- Options Graphics note on the phone; checkbox still on desktop/wasm.

Pull after a Goldshire run (cards + glow on/off) and read `gpu_glow` vs
`gpu_ui` vs `main_ms`.

## 2026-10-26 — 30 / 60 / 120 FPS

Justin asked for frame-rate options, default 30, and how to use the
S22 Ultra chip.

S22 Ultra (SM-S908U) Display.Modes at 1440×3088: **5 = 30 Hz**, **3 =
60 Hz** (system defaultModeId), **1 = 120 Hz**. Same resolution, real
panel modes, plus LTPO alts 10/24/48/96. 1080×2316 starts at mode 8.

Next APK:

- CVar `maxFps` default **30** on Android. Options → Graphics → Frame
  Rate cycles 30 / 60 / 120. Writes `benilla-config/max-fps`.
- Java `pinTargetFps` picks the same-resolution mode closest to that
  rate, `setFrameRate` / `setRequestedFrameRate`, ADPF period matches.
- Default glow **off**, grade **off**, swapchain latency 1, compute
  pool 4 (X2+A710). Fifo stays on.

Still later: pipeline cache, ADPF on render tid, cards cheap, 1080p
resolution knob, `+fp16` (not `+dotprod` until kirkwood is a separate
APK — MT8696 A73 may lack it).

## Room to improve after first launch

1. Measure: `dumpsys gfxinfo`, `adb shell dumpsys SurfaceFlinger --latency`, in-game FPS journal.
2. If fill-rate bound: quality step to mode 8 (1080×2316 @ 120) as a cvar, keep 1440 as default.
3. ADPF `PerformanceHintManager` at 8.33 ms; pin render thread to CPU 7 (X2).
4. Second compile: `+neon,+fp16,+dotprod`, thin LTO / `profile.ship`.
5. Oboe exclusive low-latency callback; drop any residual mixer spin.
6. Touch: stick is already its own pointer id. Next: 120 Hz sampling, deadzone curve, haptic on JUMP, menu buttons on raw `TouchInput` as well as the cursor hold.
7. MPQ: mmap from UFS, BLP decode off the render thread (already IO pool). MPQs stay beside the APK.
