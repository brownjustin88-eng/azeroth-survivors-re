# Whole-tree review ask — 2026-10-26

Justin asked grok to back the tree up, put a read-only MCP of this
codebase on rented compute (grok-compile **57731778**), and have
**claude-chat** (and other seats) review the **whole** tree, not a git
diff, and write recommendations.

## Backup (done)

`/home/justin/backups/azeroth-survivors-re-20261026/`

- `HEAD.bundle` — git `--all` at `dd5e330`
- `worktree.tar.gz` — dirty tree, **no** `target/`, **no** `wow-data/`, **no** MPQs
- `app-debug-sitting.apk` — playable APK from this sitting
- `dirty.diff` + `MANIFEST.txt` + `SHA256SUMS`

## MCP (read-only source)

- Process lives on grok-compile, jailed to `/root/azeroth-survivors-re`
- TheCore loopback: `http://127.0.0.1:9830/mcp` (SSH local-forward)
- Cloud Custom card: `~/.config/pko/azeroth-survivors.connector.txt` (mode 600)
- Tools: `azeroth_summary`, `azeroth_tree`, `azeroth_read`, `azeroth_grep`,
  `azeroth_git_status`, `azeroth_git_log`
- **Do not paste the bearer into Hall.** Funnel URL rotates.
- **Do not serve `/data`, MPQs, or `wow-data`.**

Local seats that already have `pko-local-metal` can also `pko_read` the
TheCore clone at `/home/justin/azeroth-survivors-re`.

## What Justin felt on the S22 Ultra trial

Gameplay works (Goldshire, levels, kills). Pain:

1. **Upgrade cards picked with 1 / 2 / 3 are very laggy.**
2. **HUD is not sized for portrait** (S22 Ultra 1440×3088, density ~600).
3. **No landscape mode felt.** The sitting APK was `screenOrientation=portrait`.
   `fullUser` + `onConfigurationChanged` is in the pad-shrink rebuild.
4. **Options menu was not visible** on the OutThisLife pause (Resume /
   Abandon / Quit). Aethus's Linux build has a full Options overlay.

## Where that lives in this tree

| Symptom | Code |
|---|---|
| Card lag | `crates/benilla-app/src/survivors/cards.rs` — fixed `Val::Px(236×330)`, LinearGradient face, BoxShadow 26, RadialGradient glow, SpellIcon 74², shine band, per-frame `animate_cards` tilt toward `RelativeCursorPosition`. `hud.rs` LevelUpRoot also spins a **1400×1400** Conic+Radial `Rays` node every frame. TouchPointer synthesizes a cursor at 1440×3088. |
| Portrait size | `hud.rs` `apply_ui_scale` = `window.height() / 768`. Portrait 3088 px → scale ~4.0. Cards stay authored desktop Px. Overlay was ~3× too big; shrink is in `touch.rs` (`STICK_R` 32). |
| Landscape | `android/app/src/main/AndroidManifest.xml` `android:screenOrientation`. Menu `menu.rs` is 1.12 create/select on a landscape 768-tall canvas (`TOWER_H=590`, zone `ROW_W=228`). |
| Options | Aethus `wenilla-rogue-app::screens::options` (Linux ELF). OutThisLife pause had no Options rung; FrameXML `BenillaOptionsFrame` is dark under `CaptureMode`. House overlay: `survivors/options.rs`. |

## Standing constraints for reviewers

- Stay native Rust/Bevy. No Unity / Unreal / Godot port. No browser play path.
- MPQs stay beside the APK. Never pack them. Never put wenilla-host `/data` on the public internet.
- SM-S908U and kirkwood **cannot Magisk** (no OEM unlock). Do not recommend it.
- Do not destroy Vast 47993302 / 55473579. Keep 57731778 until this APK sitting is finalized.
- Overlay buttons are remapped WASD / Space / F / C / Esc. F = class mobility at level 5. C = CamMode. II = pause.

## Please write

A recommendations note (whole tree, Android play first) covering:

1. Cheapening or replacing the level-up card FX on Android (stack vs row in portrait).
2. HUD/menu layout from window aspect, both orientations.
3. Whether survivors needs a real options panel (quality, rotate hint, audio) vs wiring 1.12 GameMenu.
4. Missing 3D M2 bodies on first Adreno launch (pipeline warm timeout ~25).
5. Anything in the house Android glue (`android/`, `touch.rs`, `touch_pointer.rs`, GameActivity 4.4.0) that should change.

Drop the note at `docs/CODEBASE_REVIEW_20261026.md` in this tree (TheCore path
above, or the grok-compile clone). Do not commit unless Justin asks.
