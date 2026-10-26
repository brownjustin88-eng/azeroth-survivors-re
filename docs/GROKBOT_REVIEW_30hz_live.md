# Grok Bot live pass on pid 20865 (30 Hz APK)

Hall :8765, 2026-10-26 05:04Z, `id=tv-1792991094638`. Seat **Grok Bot**.
Read-only: no logcat clear, no installs, no rebuild.

## Taken

30.0 fps (mean 33.36, p95 37.88) on pid 20865 is a solid base.

### Pipeline warm is not 4 stuck compiles

Menagerie up 00:55:56 (2060 variants, 4/frame) → TIMED OUT 4 pending
00:56:41 → cover cleared 46.8s. Zero wgpu / naga / validation /
shader-error / panic lines for 20865. Crash buffer only holds the old
10-25 SIGSEGV.

`pending = created − settled` (an `Err` counts as settled). Unrevealed
rigs never queue, so ~257 frames × 4/frame reached only ~1000 of 2060.
The 4 (and the earlier 25 against a slice of 24) is **one slice in
flight**, not 4 stuck compiles. Both Async Compute threads idle after
(~0.3s CPU per 10s).

The compiled-LIVE census only runs if `warmed_once`, which stays false
after a timeout, so its silence proves nothing.

### Bodies: no skin composites at all

Journal `skin` column (body composites) = **0 in every row**. grok
re-pulled 546 rows, t=560s, skin min/max/sum 0. `gpu_opaque` ~0.11 ms,
`gpu_static` ~0.00 ms at 3088×1440: almost no opaque 3D is drawn.
`m2` sits at 555 (assets present).

Adreno compile-failure theory is **not supported**. Bodies are missing
**upstream**: no composite is ever requested or finished.

### Build stamp vs house branch

Log: `benilla build 0.2.0 · dd5e330 · 2026-10-15 · release`.
`BENILLA_GIT_SHA` is frozen by `benilla-buildstamp` from `git rev-parse
HEAD` when that crate last built. cargo-ndk started while HEAD was still
origin `dd5e330`; house FPS code was already on disk (maxFps 30, glow 0
prove it). Docs commits `df65831` / `29f4d24` are not in the `.so`.

Next rebuild after a commit on `house-s22-apk` will stamp the house SHA.

## Smallest next (cfg android only, when Justin wants a rebuild)

Set `WOW_PIPE_TRACE` plus `RUST_LOG=debug` in `bind_android_paths`, log
each skin composite request, and either skip the warm on Android as
Aethus does or gate the timeout on `all_revealed`.
