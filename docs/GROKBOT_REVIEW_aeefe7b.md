# Grok Bot read-only review of `aeefe7b`

Hall :8765, 2026-10-26 04:02Z, `id=tv-1792987349758`. Seat **Grok Bot**.
Nothing pushed, built, or sideloaded by that seat. grok filed this copy
so the note survives compact.

Source line: `@grok thanks for the sitting. Read-only review of aeefe7b…`

## 1. Portrait HUD

Short-side scale gives a portrait logical width of 768 units
(1440 / 1.875). XP bar is still `Px(1024)` with `-512` margin
(`hud.rs` ~342), so ~128 units clip each side. Options panel is
`Px(720)` (`options.rs` ~152), tight.

Proposed fix: XP `Percent(100)`, Options `max_width Percent(96)`.
Cards already wrap (`4×236 + gaps = 1010`).

Touch buttons `BTN_W 28` / `BTN_H 24` (`touch.rs` ~45) land at
~52×45 px: too small for thumbs.

## 2. Level-up lag

`Rays` (`hud.rs` ~676) is 1400 units ≈ 2625 px square, a 24-stop
conic plus radial gradient, rotated and rescaled every frame
(`cards.rs` ~410), over a full-screen radial root, plus four
animated `BoxShadow` blurs (`cards.rs` ~479). Several full-panel
gradient fills per frame.

On Android: drop `Rays` or use a static baked image, no per-frame
transform, fixed blur. Confirm with `gpu_ui` in the fps journal.

## 3. Missing 3D bodies

The dossier says bodies stay missing all run. Slow pipelines would
pop in late, so this reads like pipelines that **fail** (a retryable
`Err` counts as pending, `pipe_warm/mod.rs` ~40). 45 s will not cure
that, and slice 4 at 30 Hz is ~120 rigs/s vs ~2900 before.

The live wow.aethus.pl page turns the warm pass **off** on Android
and forces a lights/shadows-free PBR shader because the Adreno
compiler crashed on the full one. This tree has neither.

Next: logcat with `WOW_PIPE_TRACE`, grep pipeline errors; write our
own Android lite M2 material; warm unpaced at 120 Hz behind the
cover, then drop to `maxFps`. Cause is inference until that log
exists.

## 4. 30 Hz

Right first step, not the whole fix. 1440×3088 fill is the big cost;
1080 mode (or render scale) at 60 likely runs cooler and feels
better than 1440@30. Suggest 60 + 1080 default, thermal-headroom
step down to 30, ADPF on the render thread. No in-run temps yet, so
this is judgment.

Browser vs native: the web build is WebGPU/wasm, one thread, static
extracted data; the APK is Vulkan, reads MPQs, warm pass on, full
shader.

## grok notes (same hour, not yet coded)

- cargo-ndk is still compiling `benilla_app`. No Rust edits until
  that APK sideloads, or the compile is wasted.
- XP Percent / Options 96% / drop Rays on Android: next layout pass.
- Thumb size vs Justin’s “3× smaller” overlay request: tension; wait
  for Justin.
- 1080@60 was already the later JNI mode-8 “Balanced” item. Keep it
  after we have dumpsys + journal from the 30 Hz APK.
- ADPF is still on the Java UI tid. Pinning the Bevy render tid /
  CPU 7 is later.
