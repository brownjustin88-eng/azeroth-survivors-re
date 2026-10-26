# Codebase review — S22 play first (2026-10-26)

Whole-tree read of `/home/justin/azeroth-survivors-re` (not a git diff), against
`docs/REVIEW_ASK.md` and `docs/ANDROID_REVIEW.md`. Play path is the native
GameActivity APK `house.azeroth.survivors` on SM-S908U (1440×3088, Adreno 730).
Goldshire already runs. This note is what to change next.

Constraints honoured: stay Rust/Bevy 0.18; MPQs stay beside the APK; no Magisk
(SM-S908U and kirkwood have no OEM unlock); overlay stays WASD / Space / F / C /
Esc; do not pack `/data` or MPQs; do not start `cargo-ndk` from this sitting.

Sitting facts used below: first launch pid 31390, Vulkan Adreno 730, mode 1
`renderFrameRate 120`, pipeline warm timed out with **25 pipelines pending**,
PSS ~3.5 G, pad-shrink APK is `fullUser`. Overlay is remapped WASD/Space/F/C/Esc.
Class HUD 1/2/3 are not on the overlay.

---

## 0. How the four pains map onto the tree

| Felt | Mechanism | Files |
|---|---|---|
| Cards 1/2/3 laggy | Level-up overlay is a desktop FX stack, then scaled by `height/768` (~4.0 in portrait). 1400² conic rays spin every real-time frame while `Time<Virtual>` is paused. Overlay pad is **hidden** in `Phase::LevelUp`, so 1/2/3 are card taps (or a keyboard), not overlay buttons. | `crates/benilla-app/src/survivors/cards.rs`, `hud.rs` (`LevelUpRoot`, `apply_ui_scale`, `show_panels`), `touch.rs` (`pad_visible`) |
| HUD not portrait | First APKs: `height/768` ≈ 4.02, logical width ~358, clump. Current: `min(w,h)/768` = 1.875, logical width 768; 1024-wide XP/menu is 1920 px on 1440, chrome clips. Punch-hole 100 px. | `hud.rs` `apply_ui_scale`, XP `Px(1024)`; `menu.rs` `TOWER_H`; dossier §5 |
| No landscape felt | Sitting APK was `screenOrientation=portrait`. Pad-shrink is `fullUser` + `configChanges` includes orientation. Rust still does `Window.resizable = false` and authors a 768-tall **landscape** menu. Auto-rotate off on the phone also yields no landscape. | `android/app/src/main/AndroidManifest.xml`; `crates/benilla-world/src/boot.rs`; `crates/benilla-app/src/video.rs` `apply_window_mode`; `menu.rs` |
| Options not visible | Aethus Linux ELF has `wenilla-rogue-app::screens::options` (pause Resume/Options/Quit). This clone never vendored that crate; FrameXML `BenillaOptionsFrame` is dark under `CaptureMode`. House overlay is `survivors/options.rs`. | `survivors/options.rs`; `hud.rs` `PauseRoot`; Aethus ELF `/home/justin/Downloads/AzerothSurvivors` |

---

## 1. Cheapen or replace level-up card FX on Android

### What the picker actually does

`cards.rs` authors three **236×330** `Val::Px` cards with:

- `BackgroundGradient` linear face
- `BoxShadow` blur 26 (swells to 18+22·hover every frame)
- `CardGlow` radial gradient 150 px tall
- gold-rimmed `SpellIcon` 74² with its own `BoxShadow`
- rarity hairline `LinearGradient`
- `CardShine` 70-wide band sweeping across
- `UiTransform` deal (ease-out-back, scale 0.6→1, rot −14°), per-frame **tilt** toward `RelativeCursorPosition`, bob, pick burst

`hud.rs` `LevelUpRoot` also spawns `cards::Rays`: a **1400×1400** node with a 24-stop **ConicGradient** plus RadialGradient. `animate_cards` rotates it every `Time<Real>` frame (`now * 0.12`) and pulses scale, **while the sim is paused**.

`apply_ui_scale` then multiplies every `Val::Px` by `height/768`. On the S22 portrait panel that is ~4.02:

| Authored | On glass (portrait, scale 4.02) |
|---|---|
| One card 236×330 | **948×1326** |
| Three in a row + 22 px gaps | **~3020 px** wide vs 1440 |
| Rays 1400² | **~5628²** conic+radial, spinning at 120 Hz |
| Deal translation 260 px | ~1045 px fly-in |

Bevy UI `BoxShadow` and `BackgroundGradient` (especially conic) are extra blur / gradient passes. Three of those plus a 5.6k conic disc on Adreno 730 at 1440×3088@120 is the hitch. `TouchPointerPlugin` parks a cursor at 1440×3088, so `RelativeCursorPosition` is live on every tap and the tilt/shadow path never idles.

`choose_card` also waits `PICK_SECS = 0.28` of burst animation before `hud::pick` applies. That is added input latency on top of the GPU hitch. If `pending_levels > 1`, `pick` immediately `deal_cards`s a new triple (despawn children, spawn three more gradient cards, `fill_icons` BLP from MPQ).

`touch.rs` `pad_visible` is `Phase::Playing` only. During LevelUp the stick/JUMP/F/C/II are gone. There are **no overlay 1/2/3**. Phone pick is tap-the-card. Digit1/2/3 only fire if GameActivity sees a keyboard.

Results screen (`OverRoot`) repeats the same pattern: radial vignette, five 128-wide tiles with `LinearGradient` + `BoxShadow` 16. Pause is a plain `BackgroundColor` — that one is fine.

### What to change (do this first; it is the felt lag)

Keep the desktop picker. Gate a **Low** UI quality on Android (resource, not a second crate).

1. **`crates/benilla-app/src/survivors/cards.rs`**
   - Add `SurvivorsUiQuality` (`High` / `Low`) in `survivors/mod.rs`, default `Low` on `cfg(target_os = "android")`, `High` elsewhere. Options (section 3) can raise it.
   - `Low` deal path: `BackgroundColor` + `BorderColor` only. **No** `BackgroundGradient`, **no** `BoxShadow`, **no** `CardGlow`, **no** `CardShine`, **no** `RelativeCursorPosition`, **no** per-frame tilt/bob.
   - `Low` pick: `PICK_SECS = 0.0` (or 0.05). Call `hud::pick` the same frame as the tap.
   - `deal_cards`: size from a `CardMetrics` resource (section 2), not `const CARD_W/H = 236/330`.
   - `animate_cards`: on `Low`, skip the rays query entirely; if a deal motion is kept, lerp translation/scale only (no rotation, no shadow writes).

2. **`crates/benilla-app/src/survivors/hud.rs` `setup_hud` LevelUpRoot**
   - Do not spawn `Rays` on `Low`. The 1400² conic node is the single most expensive widget in the mode.
   - Keep the dark `BackgroundColor` vignette; drop `BackgroundGradient` radial on `Low` (solid `srgba(0,0,0,0.82)` is enough).
   - `OfferRow`: set `flex_direction` from window aspect every layout tick (or on `WindowResized`):
     - **Portrait** (`height > width`): `FlexDirection::Column`, `row_gap` ~12, cards stacked.
     - **Landscape**: current row.
   - Hint line: on Android say `"Tap a card"` not `"Click a card, or press 1, 2 or 3"`.

3. **`crates/benilla-app/src/survivors/touch.rs` during `Phase::LevelUp`**
   - Show three large 1 / 2 / 3 buttons (same `PadBtn` pattern, or a new `LevelBtn(u8)`), stacked on the right, **or** full-width under the cards in portrait. Fire the same path as `choose_card`.
   - Hide stick / JUMP / F / C while the picker is up (already hidden). Keep II = pause if you want an escape; today pause is not reachable from LevelUp (`hud.rs` `keys` is empty in `Phase::LevelUp`).

4. **`OverRoot` in `hud.rs`**: same `Low` strip (solid fills, no tile gradients/shadows). Cheap; same GPU family.

Do **not** try to keep the desktop tilt-toward-cursor on the phone. The cursor is a synthetic finger. It fights the compositor.

---

## 2. HUD / menu layout from window aspect, both orientations

### The scale bug

```1495:1505:crates/benilla-app/src/survivors/hud.rs
/// The reference lays its UI out on a screen 768 units tall at any resolution (`useUiScale` off).
pub(super) fn apply_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let want = window.height() / 768.0;
    ...
}
```

1.12 `useUiScale` off means: design 1024×768, scale so **height** is 768. That is a **landscape** rule. Portrait 1440×3088 makes height the long axis, so the short axis becomes ~358 design units. Everything authored for 1024-wide overflow:

| Widget | Authored | Portrait result |
|---|---|---|
| XP bar | 1024×13, `margin-left: -512` | ~4116 px bar on a 1440 panel |
| Cards row | 236×3 + gaps ≈ 752 | ~3020 px vs 1440 |
| Menu tower | left 28, w 206, h 590 | overlaps the 260-wide zone frame (need ~499 width, have ~358) |
| Minimap cluster | 192² top-right | ~772², eats the clock |
| Timer | font 34 | ~137 px glyphs |
| Overlay stick | `STICK_R` 32 **also** under `UiScale` | 256 px diameter after the 3× shrink |

`UiScale` is global. The overlay shrink in `touch.rs` (`STICK_R` 32, `BTN` 28×24) was fighting this scale, not a density bug. If you later drop scale to ~1.9, those constants become too small and must be retuned **independently** of HUD scale.

`menu.rs` is the 1.12 create/select chrome (`TOWER_H = 590`, zone frame 260×642, `ROW_W = 228`, Enter World 200×60 at bottom 30). `GluePlugin::fit_glue_canvas` pillarboxes only when aspect **> 1.672** (`portrait/framing.rs` `GLUE_BOX_ASPECT`). Portrait 1440/3088 ≈ 0.47 → **no bars**, and survivors' `MenuRoot` is not even a `GlueCanvas`. The tower and zone sit in one full-window node with absolute Px.

S22 Ultra punch-hole is **center top** in portrait. Timer + Level + Kills sit in that band (`setup_hud` top row). Manifest does not set `windowLayoutInDisplayCutoutMode`. Expect the clock under the camera hole.

`Window.resizable = false` in `benilla-world/src/boot.rs` (Android block) can ignore the surface size after `fullUser` rotation. `video.rs` `apply_window_mode` also forces `BorderlessFullscreen(Current)` and returns early if already borderless, so it will not pick up a new extent. `MainActivity.onConfigurationChanged` only re-pins refresh + hides system UI — it does not tell Rust the new size. winit/android-activity *should* emit `WindowResized` if the surface changes; `resizable = false` is the risk.

### What to change

1. **Replace `apply_ui_scale`** with a `SurvivorsLayout` resource, updated on boot and `WindowResized`:

   ```
   portrait = height > width
   if portrait { scale = min(w/768, h/1280) }  // or min(w,h)/768 as a first patch
   else        { scale = min(w/1024, h/768) }
   ```

   Log once: `window.width/height`, `physical_width/height`, `scale_factor`. `ANDROID_REVIEW.md` assumed `height() == 3088` (scale_factor 1). Confirm; Bevy logical vs physical on GameActivity is the first number to print.

   First patch that already helps: `want = window.height().min(window.width()) / 768.0` (portrait scale ~1.875, canvas ~768×1647). XP bar 1024 still overflows 768 — still needs a width-aware XP bar.

2. **Two menu layouts** in `menu.rs` / `hud.rs` `MenuRoot` (do not keep one absolute 768-tall landscape):
   - **Landscape:** current tower-left + zone-right + Enter World bottom. This is what 1920×1080 kirkwood wants (`scale ≈ 1080/768 = 1.41`, width 1366 design units — fits).
   - **Portrait:** `FlexDirection::Column`: compact race/class grid (drop `TOWER_H = 590`; no 642-tall zone frame). Zone list **scrolls** (`Overflow::scroll_y()`). Enter World full-width above the home indicator. Logo can shrink or sit in the remaining top band.

3. **In-run HUD, portrait:**
   - XP bar: `width: Val::Percent(100.0)` (or `min(1024, window)`), not 1024 + −512 margin.
   - Ability/passive rows: wrap, left, below the cutout inset.
   - Timer/level/kills: **below** the punch-hole. Pad top ~48–64 px, or JNI `WindowInsets.Type.displayCutout()` once (section 5).
   - Minimap: 192 design units at scale 1.875 ≈ 360 px — acceptable. At scale 4 it is not. After the scale fix, leave 192.
   - Mobility `F` slot: keep bottom-center, but lift it above the stick/XP so it does not sit under the thumb.

4. **Overlay scale is not HUD scale.** `touch.rs` must not use `Val::Px(STICK_R)` under global `UiScale`. Use `Val::VMin` / `Val::Percent` (Bevy 0.18 has them) so the stick is ~12–15% of the short axis regardless of HUD canvas. Hit-test already uses window fractions (`x < 0.42w`, `y > 0.52h`) — keep that, but origin/radius must match the **drawn** stick, not `STICK_R` in design px.

5. **Landscape must actually resize.**
   - `benilla-world/src/boot.rs`: `primary_window.resizable = true` on Android (fullscreen still fills the panel).
   - Manifest stays `fullUser` + existing `configChanges` (do not restart the activity; that would drop the world).
   - Options (section 3): `sensorPortrait` / `sensorLandscape` / `fullUser` via JNI `setRequestedOrientation`. Phone auto-rotate off is why “no landscape felt” can still happen after `fullUser`.
   - `apply_ui_scale` / `SurvivorsLayout` must run on `WindowResized`, not only when `UiScale` drifts.

6. **Cutout:** `AndroidManifest.xml` activity: `android:windowLayoutInDisplayCutoutMode="shortEdges"`. Pad HUD top (portrait) / left-or-right (landscape) from insets. `MainActivity.hideSystemUi` already uses `WindowInsetsController`; extend it to stash cutout rects for JNI.

Do not invent a second UI toolkit. This is still Bevy UI. The 1.12 glue widgets (`glue/widgets.rs`) can stay; only their **parent layout** changes with aspect.

---

## 3. Options: a survivors panel, not 1.12 GameMenu

**Do not wire `GameMenuFrame` / `UIOptionsFrame` / `BenillaOptionsFrame` into survivors.**

Why:

- `SurvivorsPlugin` inserts `CaptureMode` **on purpose** (`survivors/mod.rs`). `ui_script/lifecycle.rs` `ui_wanted` is then false, so FrameXML never loads. That is why pause is a Bevy overlay and why Esc is not ToggleGameMenu.
- Loading FrameXML pulls the Lua 5.1 VM, the toc, `GameMenuAdapters.xml`, `OptionsFrame.xml`, video dropdowns, keybind UI, logout/CAMP — a second product on the phone, and a 3 s+ main-thread job the wasm path already had to slice.
- 1.12 Options are `gxResolution` / `gxWindow` / `gxVSync` / `ffxGlow` for a desktop client. They do not know 1440 vs 1080 display **modes**, ADPF, orientation lock, or overlay scale.
- `ui_logout.rs` talks to a server (`CMSG_LOGOUT_REQUEST`). Survivors is server-less (`GamePlugins { connect: false, start: InWorld }`). Pause `Quit game` already writes `AppExit`.

`VideoPlugin` and `CvarPlugin` **do** still run. `ffxGlow` defaults to `"1"` (`cvars/table.rs`). `Grade` defaults **on** (`survivors/grade.rs`, `WOW_GRADE` unset). There is no in-mode UI to flip them. `G` is keyboard-only. The APK is `--no-default-features`, so `perf::hud` (the FPS pill, `cfg(feature = "dev")`) is **not in the binary**. `FpsJournalPlugin` ships, but `/console fpsJournal 1` has no chat/console in this mode.

### What landed (2026-10-26)

`crates/benilla-app/src/survivors/options.rs` is the Aethus listings as a Bevy overlay (own file). Pause is Resume / Options / Abandon / Quit. Pages: Interface, Action Bars, Combat, Key Bindings, Nameplates, Audio, Graphics. Audio/Graphics write live CVars; Interface/Combat/Nameplates write the CVars that exist; Action Bars / Key Bindings describe the overlay map. Esc/II closes Options then unpauses. FrameXML stays off.

Still open from this section (JNI / quality, not the listings):

| Row | Writes | Notes |
|---|---|---|
| Quality: Native 1440@120 / Balanced 1080×2316@120 | JNI `preferredDisplayModeId` (S22 mode 1 vs mode 8) | Do not use `gxResolution` — Android fullscreen ignores it (`video.rs`) |
| Orientation: Follow phone / Lock portrait / Lock landscape | `Activity.setRequestedOrientation` | How landscape becomes something Justin can feel with Auto-rotate off |
| UI FX: Fancy / Fast | `SurvivorsUiQuality` | Section 1 |
| Grade default off on Android | `GradeOn` | Extra fullscreen pass |

`ffxGlow` and the audio/graphics CVars already persist through `Cvars` → `benilla-config/config.toml`. Keep Esc / II as pause/unpause. Do not load `GameMenuAdapters.xml`. Optional later: a cog on `MenuRoot`.

---

## 4. Missing 3D M2 bodies on first Adreno launch

Sitting log: *“Pipeline warm timed out 25 pipelines on first cover.”*

`pipe_warm/mod.rs`:

- `WARM_TIMEOUT_SECS = 10.0` — sized for macOS Metal inline compile vs a 43 ms audio cycle (`WARM_REVEAL_PER_FRAME = 24`).
- Off macOS, compiles are **async**; `SetItemPipeline` **skips** the batch until `CachedPipelineState::Ok`. Skinned M2 draws nothing until the pipeline settles — that is “no bodies”, not a missing MPQ. `ANDROID.md` already recorded this.
- On timeout: `done = true`, cover releases, **`warmed_once` is not set** (good — next cover retries). Goldshire **is** the first cover. Bodies stay invisible until the leftover 25 compile live, one-by-one, at 120 Hz.
- `thread_qos::COMPILE_BURST` only changes Darwin QoS. On Android it is a no-op. The compiler is not deprioritised; the cover just gives up.
- Menagerie does include skinned + static layouts (`pipe_warm/menagerie.rs`). The miss is **time**, not a missing rig, unless a later census `warn!` names a view key. Survivors camera is Perspective (`hero.rs` `looking_at`), which the world-cam arm warms.

`loading_screen.rs` will not clear until `warm.satisfied()` **and** terrain presentable. Timeout forces `satisfied`, so the splash can drop on cubes/empty attach.

Attach itself is fine: `survivors/units.rs` `spawn_unit` builds the same `Guid` / `ObjectStore` / display id path `entities/attach` dresses. If `model.MPQ` were missing, HUD BLPs from `interface.MPQ` would also fail; they did not.

### What to change

1. **`pipe_warm/mod.rs`**
   - Android (or `cfg(target_os = "android")`): `WARM_TIMEOUT_SECS = 45.0` (60 if 45 still trips). Do **not** set `done = true` while `pending > 0` until that ceiling.
   - Default `WOW_WARM_SLICE` on Android to **8** (not 24) so the Adreno compiler is not flooded. Env override stays.
   - On timeout, `warn!` the pending count **and** dump `WOW_PIPE_TRACE` lines to `$BENILLA_HOME/Diagnostics/pipe-warm.txt` so the next sitting names the 25.
   - Keep cover art up; the loading bar can stall at “compiling shaders…” using `PipeWatch` created/settled.

2. **Persist a Vulkan pipeline cache** under `BENILLA_HOME` (wgpu 27 / Bevy 0.18 `PipelineCache` is in-memory today). Second launch should not pay the 25-pipeline tax. This is the real fix; a longer timeout only makes the first launch correct.

3. **Do not treat timeout as success in logs.** A Goldshire run with cubes is a failed warm, not a playable first boot.

4. Measure after the next `.so`: `adb logcat` for `pipeline warm: TIMED OUT` and `pipeline compiled LIVE`. If LIVE lines are skinned `WowModelMaterial` / samples=1 / HDR, the menagerie arm is right and only the budget is wrong.

Do not Magisk, do not precompile via a rooted GPU profiler, do not pack SPIR-V into the APK as a parallel asset story until the wgpu cache file exists.

---

## 5. House Android glue (`android/`, `touch.rs`, `touch_pointer.rs`, GameActivity 4.4.0)

### Keep (do not regress)

| Piece | Where | Why |
|---|---|---|
| `games-activity` **4.4.0** | `android/gradle/libs.versions.toml` | 2.0.2 Java vs android-activity 0.6.1 C++ aborts `RegisterNatives` |
| **No prefab** | `android/app/build.gradle` | crate ships GameActivity C++ |
| `cargo ndk -P 31` | `scripts/android-build.sh` | API 21 sysroot has no `libaaudio.so` |
| `extractNativeLibs=false` + `useLegacyPackaging false` | manifest / gradle | page-aligned mmap of the `.so` |
| Vulkan required | manifest | correct for Adreno 730; kirkwood is a later flavor |
| `fullUser` + `configChanges` including orientation | manifest | pad-shrink already did this |
| MPQs on `/sdcard/azeroth-survivors/Data` | `warcraft-survivors/src/lib.rs`, `benilla-formats/src/install.rs` | never pack |
| `TouchPointerPlugin` hold-cursor | `touch_pointer.rs`, registered in `glue/mod.rs` | GlueClicks is `Pressed → Hovered` |
| Stick skip id | `CursorSkipTouch` | left thumb must not click the world |
| `WOW_MSAA=off` | `bind_android_paths` | 1440@120 4× is waste |
| `PresentMode::Fifo` + borderless fullscreen | `boot.rs`, `video.rs` | vsync to the 120 Hz mode pin |
| `GilrsPlugin` disabled | `boot.rs` | turn on only after the APK stays up; map onto `TouchPad` |

`Cargo.lock` has `android-activity 0.6.1`. Leave it.

### Change

1. **`MainActivity.java` ADPF is on the UI tid.** `createHintSession(new int[] { Process.myTid() })` reports Choreographer dt, not Bevy render. After the first frame, JNI from Rust (`ndk` 0.9 / `jni` 0.22 already in android-activity’s graph) should create a session for the render thread (8 Gen 1 **CPU 7 / X2**) at 8.33 ms. Until then the Java stub is a polite lie. `thread_qos.rs` is macOS-only; Android’s equivalent is this session + `setSustainedPerformanceMode` (already on).

2. **`boot.rs`**: `resizable = true` on Android (section 2). Keep Fifo + Vulkan + `MemoryHints::Performance`.

3. **Display cutout:** `windowLayoutInDisplayCutoutMode=shortEdges`; pass insets to Rust; pad HUD.

4. **Quality mode pin:** `pinHighRefreshRate()` currently picks the highest Hz at the **current** physical size (mode 1 = 1440×3088@120). Options “Balanced” should pick mode **8** (1080×2316@120) by matching `getPhysicalWidth/Height`, not by hoping Game Booster. Do this in Java (no rustc bounce) and/or JNI.

5. **`touch.rs`**
   - Overlay sizes in `VMin`/`Percent`, not `Px` × `UiScale`.
   - LevelUp 1/2/3 buttons (section 1).
   - Stick origin is the first touch in the quadrant, good. Deadzone: treat `mag < 8` physical px as zero (today `mag < 1.0` logical — with scale_factor 1 that is 1 px, noisy).
   - Later: haptic on JUMP (`VIBRATE` permission not declared yet). Not this sitting.

6. **`touch_pointer.rs`**
   - Fine for glue taps. During `Phase::Playing` + stick skip it correctly ignores the left finger.
   - Pinch → `MouseWheel` Pixel is what `hero::place_camera` expects (`* 0.02`). Keep.
   - Do not synthesise mouse-move for the stick finger (already skipped).

7. **In-game FPS without FrameXML.** APK has no `perf::hud` (`--no-default-features`). **Landed in tree (next APK):** Android `bind_android_paths` sets `WOW_FPS_JOURNAL` to `$BENILLA_HOME/Diagnostics/fps-journal.csv`; survivors HUD `CostPill` shows mean_ms / p95 at 4 Hz. `dumpsys gfxinfo` still only sees 2 Skia frames; the game is a Vulkan SurfaceView. Read the CSV before further quality-cvar guesses.

8. **Permissions.** `MANAGE_EXTERNAL_STORAGE` is granted by `scripts/android-install.sh` `appops`. House path is fine. If a cold phone install has no MPQs, show a Bevy dialog with the path `/sdcard/azeroth-survivors/Data` rather than a black world. No INTERNET permission — keep it that way (`wenilla-host` `/data` must not become a phone fetch).

9. **Leftovers.** `android/app/CMakeLists.txt` + `cpp/dummy.cpp` are an unused “convince gradle to pack libc++_shared” hack. `android-build.sh` already `cp`s `libc++_shared.so` from the NDK sysroot. Do not enable `externalNativeBuild`. Can delete the dummy later; do not wire CMake (SDK CMake is missing on TheCore).

10. **kirkwood later, not this APK flavor.** `vulkan.level required` will refuse a box with no Vulkan. TV wants landscape, 1920×1080@60, glow off, `GilrsPlugin` on, GLES fallback (`WGPU_BACKEND=gl`) if `pm list features` has no Vulkan. Product flavor or runtime backend pick — **not** a second engine, **not** Unity/Godot, **not** ANGLE/Zink. ADB `:5555` is still off; do not start a second `tv_limb`.

11. **Gamepad.** House pad is Wii U Pro (HID). After the phone `.so` is stable: re-enable `GilrsPlugin` and map left stick → `TouchPad.dir`, south JUMP, east F, north C, start pause. Wiimote Classic is 8BitDo GBros/Mayflash, not an in-app L2CAP decoder this sitting.

12. **Audio.** Sitting: `SCHED_FIFO` denied, user-interactive QoS. Expected without Magisk; do not recommend root. cpal→AAudio is why `-P 31` exists. Oboe exclusive is a later mixer pass (`sound/output/cpal.rs`, kira Android). Not the card-lag fix.

---

## 6. Other load-bearing (survivors / benilla-app / android)

These are not the four pains, but they sit on the same play path.

1. **`GamePlugins` still registers the whole 1.12 UI stack** (`game_plugins.rs`: auction, mail, talent, gossip, … + `UiScriptPlugin`). Survivors only skips the FrameXML **load**. Systems still exist. A later pass should `run_if(not survivors)` or split a `SurvivorsGamePlugins` so the phone is not scheduling empty 1.12 UI every frame. Do not do this in the same `.so` as the card/HUD fix unless it is a pure `run_if`.

2. **`minimap.rs` `compose`** writes a 192² RGBA buffer **every play frame** (bilinear sample + GPU `Image` upload) at 120 Hz. Cheap compared to 5.6k conic cards, but throttle to 10–15 Hz on Android (`Low` quality) so the IO/render thread is not uploading minimap texels during the picker.

3. **FFXGlow + Grade at 1440×3088@120.** Glow is a full-res capture + ¼ downsample + H/V Gauss + combine (`benilla-world/src/ffx_glow.rs`). Grade is another fullscreen pass after tonemap (`survivors/grade.rs`). Default both **off** on Android until the journal says the GPU is idle. `ffxGlow` is already a cvar; grade is a survivors resource. Options rows in section 3.

4. **Clutter / doodads** in Goldshire at native 1440. `WOW_CLUTTER_DENSITY` exists for captures. A quality step can drop it without Magisk. Measure first.

5. **Single-threaded `Update` / `PostUpdate` / `ExtractSchedule`** (`lib.rs` `launch`) is correct on 8 Gen 1 — most systems are non-Send (mlua, kira). Do not “fix” this with `WOW_MT_UPDATE=1` on the phone.

6. **IO pool 2–8** (`boot.rs`) is the right place for BLP/ADT. Do not decode the MPQ chain into RAM (kirkwood is 4 GB; S22 PSS 3.5 G is already fat). mmap stays.

7. **`fill_icons` / `fill_art`** pull BLPs through `WorldAssets::sprite_texture` on the main thread when cards spawn. After a level-up that is a hitch **behind** the FX hitch. Cache is a `Local<HashMap>`. Preload the class pool’s `SpellIcon`s at run start so a pick does not decode.

8. **LevelUp cannot pause.** `hud.rs` `keys`: `Phase::LevelUp => {}`. II is hidden. If a picker hitch hits, there is no Esc. While FX is still fancy, let II/Esc drop the picker without picking (or apply nothing and unpause). Once `PICK_SECS = 0` and FX is cheap, this is less urgent.

9. **Build flags (next `.so`, not this review’s job).** `android-build.sh` already uses `-P 31`, `CARGO_BUILD_JOBS=2`, `codegen-units=4`. Later: `+neon,+fp16,+dotprod` then `profile.ship` fat LTO on a box that is not TheCore’s 16 GiB. First APK without LTO was the right call.

10. **Carries.** `AGENTS.md`: changes inside upstream files stay `#[cfg(target_os = "android")]` (or a new house file). `survivors/`, `android/`, `warcraft-survivors/`, `touch_pointer.rs` are house. `benilla-world/src/boot.rs` and `benilla-app/src/video.rs` Android blocks already follow that. Keep it. Do not fight the next upstream merge for a UI-scale fix that belongs in `survivors/hud.rs`.

11. **Compile placement.** aarch64 `.so` stays on TheCore (warm `target/aarch64-linux-android`). grok-compile **57731778** stays until this APK sitting is finalized. Do not rsync `--exclude target` (drops `crates/benilla-app/src/target/`). Do not destroy Vast 47993302 / 55473579.

12. **Do not** port to Unity/Godot/Unreal. **Do not** make the browser the phone play path. **Do not** pack MPQs. **Do not** Magisk.

---

## 7. Suggested order of work (one `.so` at a time)

Java/XML-only does not bounce `cargo-ndk`. Use that.

| Step | Bounce rustc? | What |
|---|---|---|
| A | No | Manifest `shortEdges` cutout; Options JNI stubs for orientation + display mode 8 (can land empty). |
| B | **Yes** (the one that fixes the sitting) | `SurvivorsUiQuality::Low` on Android: strip card/rays/shadows/gradients; stack `OfferRow` in portrait; `PICK_SECS = 0`; LevelUp 1/2/3 overlay buttons; `apply_ui_scale` → `min(w,h)/768` (or full `SurvivorsLayout`); XP bar `Percent(100)`; overlay `VMin`; `resizable = true`; pause **Options** panel (quality/glow/grade/orient/FX); Android warm timeout 45 s + slice 8; auto FPS journal to `BENILLA_HOME`. |
| C | After a Goldshire minute with journal | If GPU bound: default glow/grade off, offer mode 8. If bodies still missing: pipeline cache file + longer cover copy. |
| D | Later | Portrait menu reflow (tower above scrolling zones); ADPF render-tid JNI; gilrs; kirkwood flavor; clutter; `GamePlugins` survivors trim. |

Do not start `cargo-ndk` until Justin asks for step B. Do not commit this note unless asked.

---

## 8. Files to touch (checklist)

**Card / HUD / pause (house, `survivors/`):**

- `crates/benilla-app/src/survivors/cards.rs` — Low FX, metrics, instant pick
- `crates/benilla-app/src/survivors/hud.rs` — `apply_ui_scale`, LevelUpRoot rays, OfferRow dir, XP bar, Pause Options, OverRoot, hint text, LevelUp Esc
- `crates/benilla-app/src/survivors/menu.rs` — portrait stack (step D)
- `crates/benilla-app/src/survivors/touch.rs` — VMin overlay, LevelUp 1/2/3
- `crates/benilla-app/src/survivors/mod.rs` — `SurvivorsUiQuality`, layout resource
- `crates/benilla-app/src/survivors/grade.rs` — Android default off
- `crates/benilla-app/src/survivors/minimap.rs` — optional compose throttle
- `CONTROLS.md` — tap-a-card; Options on pause; overlay 1/2/3 during level-up

**Warm / window (cfg-gated where upstream):**

- `crates/benilla-app/src/pipe_warm/mod.rs` — timeout, slice, trace dump
- `crates/benilla-world/src/boot.rs` — `resizable = true`
- `crates/benilla-app/src/video.rs` — already forces Fifo/fullscreen; do not let Options fight it
- `crates/warcraft-survivors/src/lib.rs` — already binds `WOW_DATA` / `BENILLA_HOME` / `WOW_MSAA`

**Android glue:**

- `android/app/src/main/java/house/azeroth/survivors/MainActivity.java` — ADPF render tid (JNI), display-mode 8, cutout insets, `setRequestedOrientation`
- `android/app/src/main/AndroidManifest.xml` — `shortEdges`; keep `fullUser`, vulkan required, no INTERNET
- `android/gradle/libs.versions.toml` — stay on `gamesActivity = 4.4.0`
- `crates/benilla-app/src/touch_pointer.rs` — leave behaviour; maybe skip mouse during LevelUp stick (N/A, pad hidden)

**Do not touch for this sitting:** `GameMenuAdapters.xml`, `OptionsFrame.xml`, `ui_logout.rs`, `ui_script/lifecycle.rs` (keep `CaptureMode` skipping FrameXML), `CMakeLists.txt` dummy, Gilrs, kirkwood GLES, wasm play path.
