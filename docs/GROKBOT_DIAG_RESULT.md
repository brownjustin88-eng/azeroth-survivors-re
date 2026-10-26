# Grok Bot s22-diag result (Hall `tv-1792998272066`)

Clone `/home/justin/azeroth-survivors-re-diag` `s22-diag` @ `e4331cc` on
`9815742`. Nothing pushed. `house-s22-apk` untouched.

## Taken from the phone

Hero and 11–24 enemies spawn, load, and attach fully (56 parts, textures
ready, meshes in camera, none hidden). 967 pipelines compiled, 651 on
the skinned path. Only the blob shadow draws. `WOW_WARM_SKIP` gives the
same hero log and still no bodies, so the 45 s warm timeout is **not**
the cause. Draw problem.

Next on that clone: forced bind pose and MeshTag slot coloring, suspect
`wow_skin_model` in `wow_model.wgsl` (bone rows from a storage buffer in
the vertex stage, slot from the mesh tag, collapse to zero area on this
Adreno).

## Desktop vs Android in this tree (house search, no rebuild)

There is **no** `cfg(target_os = "android")` in the WGSL. The skinned
vertex path is the same binary source on desktop and the phone.

Android-only render forks in house Rust:

- `benilla-world/src/boot.rs`: Vulkan backend, `WgpuSettingsPriority::Functionality`,
  `MemoryHints::Performance`, compute pool 4.
- `cvars/table.rs`: `maxFps` default 30.
- `pipe_warm`: slice 4 / 45 s (ruled out by the skip-warm run).

`WowModelExt` binds the shared light buffer with
`visibility(vertex, fragment)` (`materials.rs` ~120). `wow_skin_model`
(`wow_model.wgsl` ~253–284):

1. `wow_rig_slot` = `(get_tag(instance) >> 19) & 0x7ff` (`mesh_tag.rs`
   bits 19..=29; slot 0 is never allocated).
2. `base = wow_light.rig_table[slot]`
3. Four weighted reads of `wow_light.palettes[3*(base+index)+0/1/2]`
4. Those three rows become the affine; clip uses
   `frame_from_local * p + (rig_origin[slot] − camera)` so a zero
   affine collapses the mesh to a point.

`palettes` is the **runtime-sized tail** of one SSBO (`wow_shared_light`).
Fixed prefix includes `prop_probes: array<vec4<f32>, 57344>` (8192 slots
× 7 rows) then slot tables, then `MAX_PALETTE_BONES = 131_072` × 48
bytes (~6.3 MiB of palette). Dynamic index into that tail in the
**vertex** stage is the Adreno-shaped hole. Blob shadows never take this
path: they are the effect/decal projector (`blob_shadow.rs`), which is
why they still draw.

Joint attrs: `Uint16x4` indices + `Float32x4` weights at locations 10/11
(`ATTRIBUTE_WOW_JOINT_INDEX`), not Bevy’s own skinning.

Unskinned world M2s (`#else` of `WOW_RIG_SKIN`) use
`mesh_world_from_local` and never touch `palettes`. Trees/terrain
visible with missing unit bodies isolates the skinned vertex read.

Chrome on this S22 also drawing no Aethus characters is the same class:
WebGPU on Android often has no working vertex storage buffers; the
native APK still *compiles* 651 skinned pipelines if the adapter
advertises the feature and the driver then serves zeros.

Forced bind pose (identity, skip `palettes[]`) vs slot coloring
(`get_tag`) is the right split. This seat is not editing the diag clone.
