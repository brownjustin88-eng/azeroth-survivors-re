# Azeroth Survivors — S22 Ultra dossier (2026-10-26)

Living note for anyone sitting this work. Timeline stays in
`ANDROID_SITTING.md`. Whole-tree review still in
`CODEBASE_REVIEW_20261026.md`. This file is the current picture:
what the phone actually runs, what it talks to, what Justin felt,
what the code does about it.

Package: `house.azeroth.survivors`  
Lib: `libwarcraft_survivors.so` (Bevy 0.18.1 / wgpu 27 Vulkan)  
Phone: SM-S908U (b0q), Adreno 730, Android 15, 1440×3088 density ~600  
ADB USB `R5CT62YWN6V`, wifi `192.168.0.68:5555`  
Latest sideload: Options overlay APK, 2026-10-25 21:50 ET, last pid **2220**
(game is closed as of 22:07 ET).

---

## 1. Play is on the phone. Nothing else is in the loop.

The installed APK does **not** use TheCore, Vast, Hall, `wenilla-host`,
Chrome, ADB reverse, or the LAN while you play.

| What | Where it lives |
|---|---|
| Game | `libwarcraft_survivors.so` inside the APK, Vulkan on Adreno 730 |
| World data | 14 MPQs at `/sdcard/azeroth-survivors/Data` (~5.1 GB, mmap from UFS) |
| Saves / CVars | app external files `benilla-config/` (`BENILLA_HOME`) |
| Audio | Oboe / kira, from `sound.MPQ` / `speech.MPQ` on the same folder |
| Input | on-screen stick + JUMP / F / C / II (WASD / Space / F / C / Esc) |

Evidence on the device:

- Manifest permissions: `MANAGE_EXTERNAL_STORAGE`, `WAKE_LOCK`. **No
  `INTERNET`.** `dumpsys package` requested list matches that.
- `warcraft-survivors` `bind_android_paths` sets `WOW_DATA` to
  `/storage/emulated/0/azeroth-survivors/Data` if that directory exists.
- `run_survivors` boots `GamePlugins { connect: false, start: InWorld }`.
  `NetPlugin` with `connect: false` inserts `NetOffline` and does **not**
  start the login/realm IO thread.
- Survivors also inserts `CaptureMode`, so FrameXML / Glue never loads
  and nothing tries `ToggleGameMenu` or a realm list.

What *is* on the LAN is **development** gear, used while we compile and
review. It is idle during a Goldshire run:

| Box | Role | Play? |
|---|---|---|
| TheCore | cargo-ndk, Gradle, ADB sideload | compile / install only |
| grok-compile Vast **57731778** | wasm check + read-only source MCP | keep until APK work is finalized; not in the play path |
| `wenilla-host` `:8090` | Chrome/wasm preview, `/data/*` MPQs | loopback; never the APK |
| Hall `:8765` | house chat | unrelated to the game |

USB ADB is how we push MPQs and install APKs. After that the phone can
unplug. Wifi ADB (`192.168.0.68:5555`) is the same door for logs, not a
game server.

---

## 2. Two games share the name

| | Aethus original | This APK (OutThisLife clone) |
|---|---|---|
| Justin's Linux ELF | `/home/justin/Downloads/AzerothSurvivors` (138M, 2026-10-09) | — |
| Game crate | `wenilla-rogue-app` | `warcraft-survivors` → `benilla_app::run_survivors` |
| Pause | Resume / **Options** / Quit | Resume / **Options** / Abandon / Quit (Options added this sitting) |
| Options | `screens/options.rs` + FrameXML `BenillaOptionsFrame` | `crates/benilla-app/src/survivors/options.rs` (own file). FrameXML stays dark under `CaptureMode`. |
| Data | ships `Data/patch.MPQ` only (1.3G) | full 14-MPQ 1.12.1-5875 chain beside the APK |

Options pages on both: Interface, Action Bars, Combat, Key Bindings,
Nameplates, Audio, Graphics. Audio/Graphics write live CVars
(`MasterVolume`, `gxVSync`, `renderScale`, `ffxGlow`, …). Action Bars /
Key Bindings on the phone list the overlay map (1.12 bars never load).

Pause → **Options**. Esc / II closes Options first, then unpauses.

---

## 3. What Justin felt on the latest APK (2026-10-25 evening)

Gameplay runs. Goldshire levels and kills. Then:

1. **Still no sprites.** Spell kits show (particles / billboards). Hero
   and creature **3D M2 bodies** stay missing.
2. **Smooth, not optimized, heats up.** Native pins **mode 1 =
   1440×3088 @ 120**. Glow (`ffxGlow` default on) and Color Grade
   (survivors default on) are extra fullscreen passes. First sitting PSS
   ~3.5 G. ADPF hint session is still on the Java UI thread, not the Bevy
   render tid (CPU 7 / X2).
3. **Whole UI not visible.** Short-side `UiScale` stopped the clump, then
   the 1024-wide 1.12 layout ran off the 1440-wide portrait panel.

Phone at 22:07 ET (game closed): thermal status 0, AP 40.2 °C, skin
35.8 °C. Those numbers are idle, not in-run. Display falls back to
mode 6 @ 24 Hz when the game is not focused; while focused, MainActivity
asks mode 1 @ 120.

---

## 4. Sprites (3D bodies)

**The files are on the phone.** `adb shell ls /sdcard/azeroth-survivors/Data`
this sitting:

| MPQ | Size |
|---|---|
| backup, base, dbc, fonts, misc, patch-2 | small |
| interface.MPQ | 66M (HUD BLPs) |
| model.MPQ | **182M** (M2 creature/hero bodies) |
| texture.MPQ | 634M |
| sound.MPQ | 675M |
| speech.MPQ | 219M |
| wmo.MPQ | 347M |
| terrain.MPQ | 1.0G |
| patch.MPQ | 1.9G |

Do not ask Justin for extra art. Do not pack MPQs into the APK.

Mechanism: Bevy compiles a Vulkan pipeline the first time a shader variant
draws. Spell kits are cheap billboards and often finish. Skinned M2
bodies are a larger set. `pipe_warm` hides a menagerie behind the loading
cover and reveals a slice per frame.

| | Desktop / first APK | Current APK (`cfg(target_os = "android")`) |
|---|---|---|
| Reveal / frame | 24 | **4** |
| Cover timeout | 10 s | **45 s** |

First launch (pid 31390): `pipeline warm: TIMED OUT with 25 pipelines
pending — cover released`. Bodies that had not compiled were skipped.
The 45 s / slice-4 change is in the Options APK; logcat from that run
already rotated (game closed). Next launch: watch for

```text
pipeline warm: TIMED OUT with N pipelines pending
pipeline warm: K rigs drained in T s
```

If it still times out: persist a Vulkan pipeline cache, and skip-draw
skinned meshes until `CachedPipelineState::Ok` (review §4). Do not lengthen
the cover forever.

UI chrome (XP bar, cards, overlay) is Bevy UI + `interface.MPQ` BLPs. That
is a **layout** problem (§5), not a missing `interface.MPQ`.

---

## 5. Why the HUD walked off the screen

1.12 glue is a **1024×768** landscape canvas. Survivors authors in those
units.

| Pass | `apply_ui_scale` | Portrait 1440×3088 | Felt |
|---|---|---|---|
| First APKs | `height / 768` ≈ **4.02** | logical width ≈ 358 | clump: tower, cards, HUD piled on top of each other |
| Current APK | `min(w,h) / 768` = **1.875** | logical width = 768 | elements keep landscape size; **1024-wide rows are 1920 px on a 1440 px panel** (~240 px clipped each side) |

Concrete clips:

- XP bar: `width: Px(1024)`, `left: 50%`, `margin-left: -512` (`hud.rs`).
- Menu: 1.12 create/select (`menu.rs` `TOWER_H=590`, zone `ROW_W=228`) on
  a landscape 768-tall frame.
- Punch-hole: `DisplayCutout` top inset **100 px**. Timer / level line
  can sit under it.
- Overlay stick: `STICK_R=64` (doubled this sitting) × 1.875 ≈ 120 px
  physical, bottom-left. Buttons stay 28×24 design. Pad nodes go through
  the same `UiScale`.

Fix (not in this APK): layout from window aspect. Portrait: XP
`Percent(100)`, stack the three cards, menu as a column, overlay in
`VMin`/`Percent` so the stick is a fraction of the short side. Landscape
can keep `min(w/1024, h/768)`. Manifest is already `fullUser`; Auto-rotate
on the phone is still required to *feel* landscape. `Window.resizable`
on Android belongs in `boot.rs`.

---

## 6. Heat and “smooth but not optimized”

The Adreno was filling **1440×3088 @ 120** (mode 1). The panel also has
**mode 5 = 1440×3088 @ 30 Hz** and **mode 3 = 1440×3088 @ 60 Hz** (the
phone's own defaultModeId). Next APK defaults to **30 FPS** (mode 5),
with Options → Graphics → Frame Rate cycling 30 / 60 / 120. Fifo stays
on; the compositor presents on that mode's vsync. ADPF's target duration
follows (33.3 / 16.7 / 8.33 ms).

Also now default-off on Android: `ffxGlow`, Color Grade (`WOW_GRADE=0`).
MSAA already off. Swapchain `desired_maximum_frame_latency = 1`. Compute
pool capped at 4 threads (X2 + 3× A710).

1080p quality-step is still open: mode 8 = 1080×2316 @ 120, a separate
resolution knob from frame rate.

Also still open:

- ADPF session on the Java UI tid → move onto Bevy render tid (CPU 7)
- Card FX: strip gradients/shadows/rays on Android (`SurvivorsUiQuality::Low`)
- MSAA already off (`WOW_MSAA=off`)
- FPS journal is now auto-on for every phone run (see §11). `dumpsys gfxinfo`
  still only sees the Java UI (2 Skia frames).

Industry checks (game focused):

```text
adb shell dumpsys display          # 30 FPS → mode 5 @ 30; 60 → mode 3; 120 → mode 1
adb shell dumpsys meminfo house.azeroth.survivors
adb shell dumpsys thermalservice
adb shell dumpsys gfxinfo house.azeroth.survivors framestats   # Java only
```

---

## 7. Controls on this APK

Overlay = remapped keys, not a second move system.

| Overlay | Key | What |
|---|---|---|
| left stick | WASD | move |
| JUMP | Space | hop |
| F | F | class mobility (Charge / Blink / Sprint), learned at 5 |
| C | C | camera Fixed / Orbit |
| II | Esc | pause; second press resumes; from Options, first press closes Options |

Level-up 1 / 2 / 3 are still keyboard / tap-the-card. Overlay is hidden
in `Phase::LevelUp`. Cards hitch because of desktop FX at phone
resolution (review §1).

`GilrsPlugin` is **off**. Wii U Pro (house pad) maps onto `TouchPad`
later. kirkwood (Google TV Streamer, `192.168.0.215`) is a later 1080p60
surface. Wireless debugging there is still off.

SM-S908U and kirkwood have **no OEM unlock**. Magisk is not a house step.

---

## 8. What shipped this sitting (newest last)

| When (ET) | APK | Why |
|---|---|---|
| 19:59 | first staying | GameActivity **4.4.0** (2.0.2 aborted `RegisterNatives`). monkey display 0 (SPEG 6 @ 60 Hz). pid 31390. Goldshire ran. |
| 20:33 | pad-shrink | overlay ~3× smaller (`STICK_R` 32). Manifest `fullUser`. pid 11477. |
| 21:14 | portrait + stick | `apply_ui_scale` short-side/768. `STICK_R` 64. android pipe_warm 4 / 45s. pid 23681. |
| 21:50 | **Options** | pause Options overlay, Aethus listings, CVars. pid 2220. APK 201M (`libwarcraft_survivors.so` 99M + `libc++_shared.so`). |

Build: TheCore, NDK `30.0.14904198`, `cargo ndk -t arm64-v8a -P 31`
(`libaaudio.so` is not in the API-21 sysroot). `CARGO_BUILD_JOBS=2`,
codegen-units 4. Gradle `assembleDebug`, `games-activity:4.4.0`, no prefab.

Backup: `/home/justin/backups/azeroth-survivors-re-20261026/` (no MPQs,
no `target/`).

---

## 9. Compile / review machines (not play)

- **TheCore** holds the warm `target/aarch64-linux-android` cache and the
  Android SDK. APKs are built here.
- **Vast 57731778** (`grok-compile`, $0.118/hr): Justin said keep it until
  this APK work is finalized. Wasm check already succeeded. Read-only
  source MCP on `:9830` (SSH local-forward + funnel). Card
  `~/.config/pko/azeroth-survivors.connector.txt` mode 600. **Do not paste
  the bearer into Hall. Do not serve `/data` or MPQs.**
- Do not destroy 47993302 / 55473579. Do not Magisk. Do not pack MPQs.
  Do not put wenilla-host `/data` on the public internet.

---

## 10. Next APK (priority)

1. Sideload the 30/60/120 APK; confirm `dumpsys display` mode 5 @ 30
   and the cost pill ~33 ms. Then try 60 / 120 from Options.
2. **Portrait layout** so the HUD, menu, XP bar and cards stay on the
   1440-wide panel (`Percent(100)`, stack cards, punch-hole padding).
3. **Confirm pipe_warm** on the next launch (logcat). Persist Vulkan
   pipeline cache; skip-draw M2 until `CachedPipelineState::Ok`.
4. Move ADPF hint session onto the Bevy render tid (CPU 7 / X2).
5. Cheapen level-up cards on Android; 1/2/3 overlay buttons during
   `Phase::LevelUp`. JNI mode 8 (1080×2316) as a resolution knob.
6. gilrs / Wii U Pro / kirkwood.

Stay Rust/Bevy. Overlay stays WASD/Space/F/C/Esc. MPQs stay beside the
APK.

---

## 11. How lag is identified (not guessed from Options knobs)

Changing Graphics settings can make a run feel better. It does not name
the pass. The engine already ships a player-build instrument for that:
`FpsJournalPlugin` in every APK (`crates/benilla-app/src/perf/journal.rs`).
Once a second it appends wall time, CPU, and **per-pass GPU** to a CSV.

Survivors has no chat `/console`, so `/console fpsJournal 1` never ran
on the phone. The next APK starts the journal by itself.

| Surface | What it is |
|---|---|
| On-screen pill (top-left, under the punch-hole) | Rolling 60-frame wall `mean_ms` and `p95`. Player build; the `feature = "dev"` `+P` HUD is **not** in this APK. |
| CSV | `benilla-config/Diagnostics/fps-journal.csv` under the app external dir |
| Env | Android sets `WOW_FPS_JOURNAL` in `bind_android_paths` |
| Options → Graphics | Desktop/wasm still has the `fpsJournal` checkbox. Phone note says the file is always written. |

Phone path:

```text
/storage/emulated/0/Android/data/house.azeroth.survivors/files/benilla-config/Diagnostics/fps-journal.csv
```

Pull after a run:

```text
adb pull /storage/emulated/0/Android/data/house.azeroth.survivors/files/benilla-config/Diagnostics/fps-journal.csv
```

Columns (header in `JOURNAL_HEADER`): `mean_ms`, `p95_ms`, `cpu_ms`,
`main_ms`, then GPU: `gpu_ms`, `gpu_opaque`, `gpu_static`, `gpu_transp`,
`gpu_glow`, `gpu_post`, `gpu_ui`, `gpu_other`. GPU cells fill on Vulkan
(`TIMESTAMP_QUERY_INSIDE_PASSES`); Adreno 730 has that. Empty cells mean
no timestamp that second, not zero cost.

How to read a hitch:

| What is large | Likely cause |
|---|---|
| `gpu_glow` | `ffxGlow` full-screen chain — turn Full-Screen Glow off |
| `gpu_post` | tonemap / upscale / color grade |
| `gpu_ui` during level-up | card FX (gradients, BoxShadow, 1400² rays) while sim is paused |
| `gpu_opaque` / `gpu_static` | 1440×3088 fill (world / WMO / M2) |
| `main_ms` high, GPU low | CPU: sim, live pipeline compiles, UI layout |
| logcat `pipeline warm: TIMED OUT` | skinned M2 shaders still compiling — missing bodies, hitch on first sight |

Budget: 8.33 ms at 120 Hz, 16.7 ms at 60. The pill is wall time (Fifo
present), so a 120 Hz grant can look like 8.3 ms even when the GPU
finished earlier. The CSV GPU columns are the actual pass cost.

---

## 12. Did the house edits break original git?

**History: no.** HEAD is still origin/main
`dd5e330d7dbdcd25e0989535d71c49198a31819b`. Zero house commits. Ahead 0,
behind 0. `git log origin/main..HEAD` is empty. Nothing was rewritten,
force-pushed, or committed.

**Working tree: dirty on purpose.** 22 tracked files modified (~170 / 47)
plus untracked `android/`, `survivors/options.rs`, `touch.rs`,
`touch_pointer.rs`, `warcraft-survivors/src/lib.rs`, docs, scripts.
`git checkout -- .` restores tracked files to origin. Untracked files
stay until `git clean`. Backup:
`/home/justin/backups/azeroth-survivors-re-20261026/`.

**Desktop / wasm behaviour** (what `make start` and `make native` run):

| Change | Android-only (`cfg`) | Also on desktop/wasm |
|---|---|---|
| `boot.rs` BorderlessFullscreen + Fifo + Vulkan `WgpuSettings` | yes | |
| `video.rs` force Fifo + borderless | yes | |
| `pipe_warm` slice 4 / timeout 45 s | yes | |
| lua-src POSIX (no readline) | yes | |
| `formats/install.rs` `/sdcard/.../Data` | yes | |
| arboard gated off | yes | |
| `WOW_FPS_JOURNAL` auto path | yes | |
| Options overlay + pause Options button | | yes (additive) |
| `apply_ui_scale` short-side / 768 | | yes (no-op on landscape: short == height) |
| OfferRow wrap | | yes |
| PauseRoot visibility moved to `options::gate_pause` | | yes |
| `TouchPad` plugin always registered | | yes (pad hidden on menu) |
| hero / mobility read the pad | | yes |
| Glue `TouchPointerPlugin` | | yes (helps wasm phone preview) |
| `warcraft-survivors` bin calls `warcraft_survivors::main()` | | yes (same `run_survivors`) |

`make start` still serves wasm on 127.0.0.1:8090. `make native` still
launches survivors. The 1.12 FrameXML client path is untouched except
Glue gaining `TouchPointerPlugin`. Original git still works; this tree
is a dirty overlay on top of it, not a rewritten history.
