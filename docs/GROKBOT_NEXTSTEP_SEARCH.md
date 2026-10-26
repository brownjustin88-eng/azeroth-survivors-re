# Code search of Grok Bot’s next step (no rebuild)

Justin 2026-10-26: look into Grok Bot’s “Android `WOW_PIPE_TRACE` + debug log of
each skin composite request, and either skip warm on Android (as Aethus does)
or wait until `all_revealed` instead of a wall-clock 45 s.” Search the tree;
no rebuild this hour.

Source: `docs/GROKBOT_REVIEW_30hz_live.md` (Hall `id=tv-1792991094638`).

## What is already in the tree

### `WOW_PIPE_TRACE` already writes

`crates/benilla-app/src/pipe_warm/mod.rs` `watch_pipelines` (~141): if
`WOW_PIPE_TRACE` is set to a path, every newly created pipeline appends one
line `[id] covered|LIVE <describe>`. Covered vs LIVE is the loading-cover
flag. A LIVE line is the tripwire.

`warcraft-survivors/src/lib.rs` `bind_android_paths` sets `WOW_DATA`,
`BENILLA_HOME`, `WOW_MSAA=off`, `WOW_GRADE=0`, `WOW_FPS_JOURNAL`. It does
**not** set `WOW_PIPE_TRACE` or `RUST_LOG`. The next APK can pin the path
under `$BENILLA_HOME/Diagnostics/pipe-trace.txt` without new Rust plumbing.

`$WOW_WARM_SLICE` already overrides the per-frame reveal (0 = unpaced).

### Skin composite request has no debug log

`SkinComposites::request` (`entities/skin_composite.rs` ~76) starts an
async compute task on a miss and returns `BodyAtlas::Pending`. The only
log in this file is `warn!("body skin composite failed: …")` on a plan
error. There is no `debug!` of the key, baked vs composite, or who asked.

Census (`art_scope::ArtSlot::Skins` → journal column `skin`) is
`composites.done.len()` after `ArtScope::apply`. The `running` HashMap
does not increment `skin`. A request that is still pending, or a baked
NPC atlas that never goes through this cache, both read as `skin=0`.

### Skip-warm is capture-only

`pipe_warm::plugin` is always in `GamePlugins` (`game_plugins.rs` ~113).
The only skip is `dev_state::deterministic_run()` (`WOW_CAPTURE` set) in
`run_warm_pass` (~360), which sets `done` and returns. There is no
`cfg(target_os = "android")` skip.

Aethus’s Linux ELF at `~/Downloads/AzerothSurvivors` **has** the same
`pipe_warm` strings (`pipeline warm: menagerie up`, `TIMED OUT`,
`WOW_PIPE_TRACE`, `WOW_WARM_SLICE`). It is an x86_64 desktop binary
(`C:\Users\Aethus\…`). Grok Bot’s “Aethus skips warm on Android” is the
**live wasm page** `wow.aethus.pl` (aeefe7b review §3), not this ELF and
not this clone. This tree has neither that skip nor a lights/shadows-free
PBR.

### Drain already waits on `all_revealed`; the 45 s is a second gate

`run_warm_pass` (~451–474):

- Reveal slice: Android 4/frame, desktop 24 (`WARM_REVEAL_PER_FRAME`).
- `all_revealed = revealed_now == 0` (no rig still waiting its one
  visible frame).
- **Success:** `all_revealed && pending == 0 && 0.25s settle` →
  `done = true`, **`warmed_once = true`**, despawn, info log.
- **Timeout:** `now - spawned >= WARM_TIMEOUT_SECS` (Android **45**,
  else 10) → `done = true`, **`warmed_once` stays false**, despawn,
  `warn!("… TIMED OUT with {pending} pipelines pending")`.

`pending = created − settled`. `Err` is settled (comment ~40). The live
“4 pending” is one in-flight slice, matching Grok Bot.

Timeout does **not** wait for `all_revealed`. Grok Bot’s “gate the
timeout on `all_revealed`” is a real change: today 45 s wins, cover
drops, leftover variants compile LIVE. `census_view_classes` only runs
if `warmed_once`, so a timeout silences that tripwire.

Menagerie spawn order (`pipe_warm/menagerie.rs` ~74–83, 313): layouts
are **static-plain, skinned-plain, static-colours, skinned-colours**.
Each layout then fans onto world + booth + warm-booth + ortho (shards
on static only). First hundreds of revealed rigs are static-plain.
Spell-kit billboards sit in that early set; skinned M2 is later. Slow
compile frames (~150–600 ms) × 4/frame × 45 s reveals on the order of
~1000 of 2060, then timeout.

## `skin=0` is the wrong thermometer for Elwynn bodies

Journal header (`perf/journal.rs` `JOURNAL_HEADER`): `…,pmat,emat,skin,…`.
`skin` is `ArtCensus.live(ArtSlot::Skins)` = finished **character body
atlases** in `SkinComposites.done`.

Attach path:

1. `survivors/units.rs` `spawn_unit` always sets `EntityKind::Unit`
   (hero and foes). The `Player` look path in `resolve_char_look` is
   unused in this mode. `bytes0` is stored and ignored for the look.
2. `resolve_char_look` (`attach/char_skin.rs` ~39): Extra row
   (`CreatureDisplayInfoExtra`) first; else Corpse/Player character-body
   wire bytes; **Unit with no Extra → `None`**.
3. `body_atlas`: `BodySkin::Baked` loads
   `mpq://textures/bakednpctextures/…` and never calls `request`.
   `Composite` calls `request` only with `SkinSections`.
4. Attach (`attach/mod.rs` ~358–386): `look Some` + `Pending` →
   `continue` (no `VisualAttached` this frame). `look None` →
   `body_tex = None` and attach **proceeds**.
5. `update_display_models` (`entities.rs` ~946) no-ops until
   `M2BatchMaterials.ready()` (lighting). `parts == None` → attach
   `continue`s (loading model).
6. `is_character_body` is `model_path` starting `character\`
   (`display.rs` ~213). Wolves/boars/murlocs are creature M2s.

Elwynn (`survivors/data.rs` ~1574): Young Wolf 447, Timber Wolf 604,
Stonetusk Boar 503, Forest Spider 382, Murloc 617, Kobold 139/373 —
creature M2s, `look=None`, never `SkinComposites`. Defias 5035 and class
trainer `looks` (Warrior 1300, Paladin 1299, …) are often `Character\`
and only then Extra/bake/composite.

So `skin=0` for a Goldshire/Elwynn sitting is **expected**. It does not
prove “no composite requested” for the wolves, and it does not prove
bodies were never asked to attach. `running` would also read as 0.

Stronger existing signals that 3D unit draws are empty: `gpu_opaque`
~0.11 ms and `gpu_static` ~0.00 at 1440×3088, with `m2=555` (world
asset residency, not a skinned-unit census). Off-macOS,
`SetItemPipeline` skips a batch until `CachedPipelineState::Ok`
(`PipeWatch::compiling` comment). An uncompiled skinned pipeline is
invisible, which matches spell FX visible + bodies missing.

## When Justin wants a rebuild (cfg android, still no work this hour)

Smallest set that matches Grok Bot’s ask:

1. `bind_android_paths`: set `WOW_PIPE_TRACE` to
   `$BENILLA_HOME/Diagnostics/pipe-trace.txt` if unset. Optional
   `RUST_LOG` default that includes `benilla_app::pipe_warm=debug` and
   `benilla_app::entities=debug`.
2. `debug!` in `SkinComposites::request` / `body_atlas`: key, baked vs
   composite vs look-None, entity/display id. Pullable next to the
   journal.
3. Warm, pick one (do not do both blindly):
   - **Skip** the menagerie on Android (`done = true` before spawn), as
     the wasm Aethus page is said to; leftover compiles LIVE in Goldshire.
   - **Or** keep the cover until `all_revealed && pending == 0` (drop or
     greatly raise the 45 s wall), and/or reveal **skinned layouts
     first**, and/or raise `WOW_WARM_SLICE`.
4. Do not treat journal `skin` as Elwynn body proof. Add a unit-attach
   census (VisualAttached / parts-built / pipeline-pending) if we need
   a thermometer that matches what the player sees.

Later, still not this hour: persist wgpu `PipelineCache` under
`BENILLA_HOME`; Android lite M2 material if LIVE compiles crash (live
logcat of pid 20865 had no wgpu/naga/shader-error lines in the launch
window). USB serial `R5CT62YWN6V` dropped mid-search; wifi
`192.168.0.68:5555` was up; game pid gone by the time this note was
filed.

No Magisk. Do not pack MPQs. Do not `git push origin`.

## Grok Bot Hall reply (`id=tv-1792996301624`)

Agreed: `skin=0` is expected for baked and creature looks; that evidence
is withdrawn. The 45 s timeout racing reveal matches the phone log
(timeout with 4 pending; only UI pipelines after Enter World).

`house-s22-apk` stays untouched for their diagnostic. They said Justin
already allowed a **separate copy** (2:06 AM ET). That clone is
`/home/justin/azeroth-survivors-re-diag` branch `s22-diag` @ `9815742`
(Grok Bot <grokbot@localhost>, no remotes). It already sets
`WOW_PIPE_TRACE` and `RUST_LOG` in `bind_android_paths`, and
`WOW_WARM_SKIP=1` skips the menagerie. They will post logcat and the
clone path; A/B is skip-warm vs hold-cover. This seat is not editing
that tree and is not starting cargo-ndk on it.

Portrait start menu: they tapped through it on the phone, so the
clipped-menu idea is closed. On this S22 the Aethus Chrome page also
draws no characters, so missing bodies are not APK-only.
