# House S22 APK — review brief for Grok Bot

Justin 2026-10-26 asked grok to show this sitting on the new Hall
(`tv_hall` :8765) and push it to git so **Grok Bot** can review and
keep working the same tree.

This file is the durable brief. Hall speech is the wake.

## Git (house fork — do not push upstream)

| | |
|---|---|
| Upstream (leave alone) | `OutThisLife/azeroth-survivors-re` `main` @ `dd5e330` |
| House fork | `brownjustin88-eng/azeroth-survivors-re` |
| Branch | `house-s22-apk` |
| TheCore clone | `/home/justin/azeroth-survivors-re` |
| Remote name on TheCore | `house` |

`origin/main` is still a clean `dd5e330`. House work lives only on
`house/house-s22-apk`. Cloud and other seats keep their own branches.

Clone:

```
git clone -b house-s22-apk git@github.com:brownjustin88-eng/azeroth-survivors-re.git
```

Do **not** `git push origin`. Do **not** open a PR against OutThisLife
unless Justin says so.

Not in git (and must stay out): `target/`, `wow-data/`, MPQs, APK
binaries, `android/local.properties`, `android/app/src/main/jniLibs/`,
`web/dist/`, `benilla-config/`, connector tokens.

## What this is

OutThisLife clone of a 1.12.1-era WoW client on **Rust / Bevy 0.18**,
survivors mode (`benilla_app::run_survivors`). README: not an original
project. Aethus's Linux ELF at `~/Downloads/AzerothSurvivors` is a
different tree (wenilla-rogue-app); we copied the Options *idea* into
`crates/benilla-app/src/survivors/options.rs`, not their code.

Play path Justin locked: **native GameActivity APK** on the daily
S22 Ultra (SM-S908U, Adreno 730, 12 G RAM, LTPO 1440×3088). Package
`house.azeroth.survivors`. **Local only** — Manifest has
`MANAGE_EXTERNAL_STORAGE` + `WAKE_LOCK`, no `INTERNET`.
`GamePlugins { connect: false, start: InWorld }`. MPQs live on
`/sdcard/azeroth-survivors/Data` (~5 GB, 14 files including
`model.MPQ` ~182 M). Do not pack MPQs into the APK.

## What grok already did (this sitting)

Android packaging:

- `android/` GameActivity 4.4.0 (Java must match the crate; 2.0.2
  aborted `RegisterNatives`).
- `scripts/android-build.sh` (`cargo ndk -t arm64-v8a -P 31`),
  `android-install.sh`, `android-push-data.sh`.
- `warcraft-survivors` is a `lib` on Android (`src/lib.rs`); desktop
  `main.rs` still exists.

Touch / HUD:

- Overlay WASD / Space / F / C / Esc (`touch.rs`). II = pause.
- `TouchPointerPlugin` synthesizes a cursor from the stick.
- `apply_ui_scale = min(w,h)/768` (short side). Portrait 1440 still
  clips 1024-wide 1.12 frames (XP bar, menus).
- Pause gained an Options overlay (`options.rs`) — Interface / Action
  Bars / Combat / Key Bindings / Nameplates / Audio / Graphics.

S22 heat / frame pace (the portion Justin just asked for):

- Java **used to** `pinHighRefreshRate` → always mode 1 = 1440×3088@120.
  That is the heat. S22 has native **mode 5 = 1440@30**, mode 3 = @60,
  mode 1 = @120 at the same resolution.
- New CVar `maxFps`, Android default **30**. Options Graphics cycles
  30 / 60 / 120.
- Rust writes `$BENILLA_HOME/max-fps`. Java polls ~every 45 frames,
  picks the same-resolution `Display.Mode` closest to the target,
  `Surface.setFrameRate` FIXED_SOURCE, `setRequestedFrameRate` (API 35),
  ADPF period 33.3 / 16.7 / 8.33 ms. SharedPreferences default 30 so
  `onCreate` is 30 before native.
- Fifo present + `desired_maximum_frame_latency = 1`.
- Android compute pool capped at **4** threads (X2 + 3×A710). Little
  A510s stay free.
- `ffxGlow` default **off** on Android. `WOW_GRADE=0` at bind.
- Engine `FpsJournalPlugin` auto-on via `WOW_FPS_JOURNAL` on Android.
  CSV: `$BENILLA_HOME/Diagnostics/fps-journal.csv`. On-screen CostPill
  (mean_ms / p95, 4 Hz, z=440).

Still compiling on TheCore when this brief was written: cargo-ndk
release of `benilla_app` for aarch64-android (30/60/120 APK). Sideload
only after `android_apk_fps.log.exit` is 0 and the APK timestamp is
newer than the journal APK (201 M, 2026-10-25 22:53 ET).

## Standing constraints

- Stay native Rust/Bevy. No Unity / Unreal / Godot. No browser play.
- SM-S908U and kirkwood **cannot Magisk** (no OEM unlock). Do not
  recommend it.
- Do not destroy Vast 47993302 / 55473579. Keep grok-compile **57731778**
  until Justin says the APK sitting is finalized.
- Do not put wenilla-host `/data` or tv_hall `:8765` on the public
  internet.
- grok ≠ Grok Bot. Speak as yourself. Do not mint Hall citizens.
- Do not `adb usb`. Phone serials stay in sitting notes, not in Hall.
- Do not print Hall `say_token`, connector bearers, Vast API keys.

## Please review and/or work

Read in this order: this file, `docs/ANDROID_DOSSIER.md`,
`docs/REVIEW_ASK.md`, then the code listed below.

Open problems Justin already felt on the phone:

1. **3D M2 bodies / “no sprites”** — files are on the phone; first
   Adreno launch timed out ~25 GPU pipelines. `pipe_warm` is now 4
   pipelines/frame, 45 s timeout. Unverified on device in this APK.
2. **Portrait HUD clips** — short-side scale is the right *size*;
   1024-wide 1.12 layout overflows 1440. Next: Percent(100) XP,
   stack upgrade cards, punch-hole pad.
3. **Level-up cards lag** — `cards.rs` + `hud.rs` 1400² `Rays` node
   every frame. Cheapen or replace on Android.
4. **Heat at 120 Hz** — root cause is Java always picking the max
   refresh. Default 30 should fix it; confirm with `dumpsys display`
   after the new APK (want mode 5 @ 30).
5. **Lag identity** — after Goldshire, `adb pull` the fps-journal CSV.
   Columns name GPU passes (`gpu_glow` / `gpu_ui` / `main_ms`).

Later, not this PR: gilrs (Wii U Pro), kirkwood sideload, JNI mode 8
(1080@120 “Balanced”), ADPF onto Bevy render tid / CPU 7, pipeline
cache persist, `+neon` (do **not** add `+dotprod` — kirkwood MT8696
A73 may lack it).

## How to sit with it

- Source on TheCore: `/home/justin/azeroth-survivors-re`.
- Read-only MCP of the tree on grok-compile, loopback
  `http://127.0.0.1:9830/mcp` (card on disk, **do not paste bearer
  into Hall**). Tools: `azeroth_summary` / `azeroth_tree` /
  `azeroth_read` / `azeroth_grep` / `azeroth_git_*`.
- Write notes into `docs/` on `house-s22-apk` and push `house`.
- If you sideload, USB serial is the daily S22; wifi ADB is already
  kept. Launch with monkey on **display 0** (SPEG 6 is the idle 24 Hz
  face).
- Justin watches the new Hall. Report findings there as **Grok Bot**,
  and drop a note in `docs/` so it survives compact.

Filed on this branch: `docs/GROKBOT_REVIEW_aeefe7b.md`,
`docs/GROKBOT_REVIEW_30hz_live.md`, `docs/GROKBOT_NEXTSTEP_SEARCH.md`
(code search of the live-pass next step; no rebuild).
