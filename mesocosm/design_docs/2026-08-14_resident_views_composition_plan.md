# Resident Views Composition Plan (2026-08-14)

**Status: founded 2026-08-14; lanes A-F, the seed crystal, real-Ground
composition, retained-allocation mutation, and strided subregion carriage are
landed in tracked code; Conatus voxel mechanics are adopted through
`GroundVoxelProfile`; equal-sized projection cache coherence was implemented
and headed-verified 2026-08-26; Mere's `conatus-brick` now owns the shared
brick ABI and camera-neutral DDA; V1b closed incremental per-brick
residency, allocator-observed bytes, and the tracer-validated lease epoch
2026-08-26 (engine review §5 V1b); field-plane visualization remains
open; the consumer-specific field admission boundary was recorded
2026-09-01.** How the voxel world, Burn/CubeCL, the
tracer, the mesher, collision, and persistence compose on one device.
Ratified direction (Mark, 2026-08-14): the voxel world
is Burn-addressable state while remaining voxel-authoritative state.
Burn does not import voxels into a second world representation; every
consumer receives a resident view of the same facts.

The platform half of this ruling (lanes are program shapes, the
allocator direction, the resident-views law, the tensor-bundle plane
taxonomy) lives in mere
(`design_docs/mere_docs/technical_architecture/2026-08-13_spatial_compute_plan.md`
section 0.5). This doc is the wing application: the seam type, the
ownership rows, and the first proof.

This is the graph architecture applied to spatial state. The graph is
not inside layout, rendering, search, or ML; it stays authoritative
while each consumer gets an appropriate resident reading. `Ground` is
not inside the tracer, Burn, or parry; same law. The wing had already
grown the authoritative half before the seam was named: `Ground`
carries `revision: u64` and a dirty-brick queue whose doc comment
states the law ("projection work queue, not world authority; the
revision and brick bytes carry the authoritative change").

## The seam

```
ResidentChunk
  identity
  world position and extent
  authoritative revision (mirrors Ground revision at materialization)
  typed channel planes
  device-buffer handles
  shape, strides, element types
  valid-read epoch
  dirty regions
```

Views produced without copying: Burn tensor view, CubeCL kernel view,
tracer/material view, mesher input, collision-refresh input,
persistence/checkpoint view. A view is one arrangement of the chunk,
never a replacement for it.

**Planes are a bundle, not one tensor.** Exact planes (occupancy,
palette id, provenance, flags), fixed-point simulation planes
(temperature, moisture, nutrient, pressure; replay-exact), learned and
derived planes (latents, light estimates; projections, drift is a
shrug), temporary planes (inference output, constraint masks, work
queues). The plane taxonomy is the fact-bearing ruling: exact and
fixed-point planes belong to the record's side of the doctrine;
everything else is projection.

**Allocation flows compute-side out.** Burn/CubeCL can hand its
allocations outward but cannot cheaply adopt a foreign buffer, so
planes are allocated through the CubeCL client (conatus's resident
allocations) and leased to tracer, mesher, and render tenants.

## The transition path

```
resident chunk
    | tensor view
Burn model or CubeCL process
    | proposed channel deltas (candidate planes)
constraint and game-rule projection
    | accepted delta
authoritative commit + revision bump (Ground)
    |
tracer / mesher / collision / persistence observe the revision
```

**Burn proposes; the record disposes.** Inference stays GPU-resident.
The CPU reads accepted deltas, checkpoints, hashes, and small
reductions at durability boundaries, never whole planes per tick (the
4-byte readback pattern from the resident lane). A commitment that
reads a projected channel snapshots it into the fact at commitment
time.

## Ownership

- **Mesocosm** owns channel meanings, authoritative voxel facts
  (`Ground`), constraints, and commits.
- **Conatus** owns its spatial-system phase schedule, resident allocations,
  leases, and the sequencing of its own compute and materialization work.
- **The host** owns the shared device lifecycle, inter-tenant submission order,
  completion and presentation policy, recovery, and the read epochs that make
  resident writes safe for consumers.
- **Burn/CubeCL** owns tensor evaluation and learned or numerical
  transforms.
- **Renderling/netrender** consumes resident buffers and revisions as
  tenants.
- **Parry** derives tactile query state from committed voxel revisions
  (never from candidate planes).
- **Persistence and peers** exchange canonical chunk facts and deltas,
  never GPU allocation details.

## Field admission boundary (2026-09-01)

`ResidentChunk` is a carriage and revision contract for admitted planes. It is
not a universal environmental simulator, and its spatial shape does not decide
the semantic dimension of every field. Before a product field enters this
path, its admission card names its authority class, consumer, domain,
resolution, cadence,
interpolation, sources and sinks, boundary conditions, numeric range,
conservation rule, scale reduction, and acceptance evidence.

That keeps different phenomena honest:

- soil nutrient and moisture can remain authoritative columns or shallow
  layers;
- light can be a derived attenuation or occlusion plane rather than saved
  photon state;
- gas, clouds, or submerged chemistry can use full 3D when vertical transport
  changes a process;
- roots and mycelia remain explicit transport networks over fields when
  direction, connection, and severing matter.

A 2D or network consumer may use its own resident arrangement or a compatible
view over chunk storage; it does not gain a dense 3D authoritative plane merely
because the GPU carrier can express one. Every extra channel gets a measured
resident-byte and update-cost receipt. The
[VDB paper](https://doi.org/10.1145/2487228.2487235) remains a reference for a
sparse, dynamic 3D consumer if one binds. Noita’s
[technical overview](https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-DesignAt)
shows that local material simulation can carry a roguelike, but its 2D
falling-sand design does not choose this game’s field dimensions.

Resident mips, clipmaps, and window retargets are sampling and presentation
views. If a field later changes its authoritative simulation resolution, its
admission contract separately defines restriction, refinement, conserved
integrals, boundary flux, residual state, zero-tick round trips, and the
comparison envelope for reduced evolution. Camera movement and allocation
pressure cannot select that simulation path.

## The first proof (partially closed)

Use one existing `Ground` region; invent no second chunk model.

1. Materialize its authoritative bricks into one resident chunk bundle.
2. Construct a Burn tensor view over a material-derived plane
   (temperature or moisture).
3. Run a simple 3D diffusion or NCA pass through Burn/CubeCL.
4. Have the brick tracer visualize that field from the same resident
   allocation.
5. Commit one constrained field delta back into world history.
6. Receipts: one device; one allocation per plane (proved by buffer
   identity, the P4 epoch style); zero per-tick bulk copies; revision
   coherence (every consumer observes the same revision); replay
   (facts reproduce the committed state with no simulation present);
   unchanged-frame silence (a static world causes no recomputation,
   gated on elapsed time, never frame counts).

The `3x3x3` material-derived pass below proves carriage, proposal, commit,
revision, and replay mechanics. It does not admit temperature or moisture as a
product-wide 3D simulation. The first gameplay field still passes the admission
boundary above.

The feature-gated `mesocosm-lens` `resident_ground` example now closes the
authority-shaped path through those pieces: a material-derived 3D Burn pass
proposes one scalar consequence; `Ground` validates and commits one integer
voxel removal; Quint writes the accepted four-byte aligned range into the
existing exact atlas plane and publishes revision 1 at read epoch 11;
`BrickTracer` consumes that retained allocation with zero CPU voxel upload;
and the serialized accepted delta replays without Burn. `commit_plane_patch`
refuses a stale source stamp, a non-advancing revision or read epoch, the wrong
scalar type, an invalid or unaligned range, and a dirty region outside the
chunk. It also refuses multi-plane bundles: advancing a bundle-wide stamp
after changing one plane would falsely mark every untouched plane current.
A future multi-plane form must validate and publish one batch.

The RTX 4060/Vulkan receipt proves the stronger identity claim: buffer,
offset, and size are identical before and after the Ground commit. It records
a four-byte proposal readback, a four-byte resident patch, revision `0 -> 1`,
one retained 262,144-byte atlas allocation, a 512-byte committed GPU copy, a
four-byte pointer upload, 184 changed pixels, zero atlas bytes on the unchanged
frame, and replay from the 112-byte accepted delta.

The subregion transport gate is also closed locally. `LeasedAtlas` now carries
a source origin, source bytes per row, and source rows per image. The tracer
repackages only the named strided rows into its aligned device-local staging
buffer, then copies that box into the destination texture. It refuses source
or destination overruns, unaligned buffer copies, and partial leases that do
not cover every region in the frame's declared `BrickChange`. The real-Ground
commit therefore moves one 8x8x8 brick, 512 bytes GPU-to-GPU, while its source
allocation remains the same full atlas. Field visualization remains a
separate derived-plane consumer.

Generation, batched brains, fluids, and learned growth then stack on
this seam rather than each negotiating its own import path.

**Sequencing:** the place-graph engine work (G2+) is live in this tree
and owns `Ground`'s internals right now. This proof starts after that
work settles; the seam consumes `Ground`'s public revision contract
and does not reach into brick internals.

## Lanes (2026-08-14)

The proof gate above is one sequence, but the work under it is not one
sequence. These are the independent lanes, each written so a cold
session can pick it up. Lane briefs are handoffs; the ownership column
is what makes two of them safe to run at once.

**The safety rule: disjoint file ownership, not good intentions.** This
tree and mere both had concurrent sessions editing them the day this
was written. Lanes B, C, and D live in `mere/crates/probes/`, which
`.gitignore` excludes wholesale, so they touch no tracked file and can
never appear in another session's staging. Lane A owns quint's source
alone. While lanes run concurrently, each commits its own paths and
names what it left unstaged, which is the concurrent-work exception to
the usual commit-the-whole-tree default.

### Lane A. The seam (keystone, unblocked)

`ResidentChunk` and its view constructors, in mere at
`crates/conatus/quint/src/`, behind the existing `field-gpu` feature.

**A module, not a crate.** quint already owns resident GPU allocations
(`quint::resident`), already gates wgpu optionally, and feature gating
already gives the wing its subset without dragging the field algebra.
Split it out only if chunk substrates outgrow that neighborhood or a
consumer needs the seam with none of quint's algebra.

**Done when:** a synthetic chunk yields a Burn tensor view and a raw
kernel view over the same allocation, proved by buffer identity rather
than equal contents; revision and valid-read epoch surface to a
consumer; planes allocate through the CubeCL client (the allocator
direction); no CPU whole-plane read exists in the path.

Blocks lanes D, E, F. Owns quint's source.

### Lane B. Harvest (unblocked, independent)

Port nexus's GPU radix sort and Karras LBVH build/refit into our
carriage, in an untracked probe. Carry the two caveats their source
records: the Windows and NVIDIA atomic-load workaround (wgpu#9221) and
the radix-sort bounds-check note.

**Done when:** sort correctness against a CPU reference on adversarial
keys (duplicates, already sorted, reversed, single value); tree
correctness (every leaf reachable, each node's box containing its
children's); a timing curve against the CPU Barnes-Hut reference; and
a positive control proving the GPU path actually ran, per the
".spv existing is not .spv running" finding.

Promotion into conatus is a separate decision after receipts.

### Lane C. The carriage decision (unblocked, independent)

Port quint's repulsion kernel to a CubeCL raw kernel in an untracked
probe, with Burn sharing the client so the handoff bridge is absent by
construction rather than by effort.

**Done when:** numeric agreement with `forces::repulsion_reference` at
the tolerance the existing handoff receipt used; timing measured
against the resident lane's 50k figure; and a plain statement of
whether the bridge code disappears. Either outcome is an answer: it
decides whether the explicit lane consolidates on CubeCL-JIT or keeps
both carriages on purpose.

### Lane D. Mechanism proof (complete 2026-08-14)

Burn tensor view over a synthetic chunk plane, one diffusion or NCA
pass, one constrained delta committed in the commit shape. No `Ground`,
no tracer. This clears proof steps 2, 3, and 5 without waiting on the
wing, so only the wing-specific receipts remain.

Receipt: Mere's ignored `crates/probes/resident-diffusion` constructs a
`3x3x3` Burn tensor view from `ResidentChunk`, runs one six-neighbor
diffusion pass on the RTX 4060, and reads one four-byte candidate value.
The centre moved `10.000 -> 4.000`; the record constraint reduced the
proposed `-6000` milli-unit delta to `-1000`, committed revision
`41 -> 42`, then reproduced the resulting `9000` value from serialized
integer history with Burn absent. This clears steps 2, 3, and 5 only.

### Lane E. Wing materialization (complete 2026-08-16)

`Ground` into a resident chunk bundle, consuming only `Ground`'s public
revision contract. Blocked on lane A and on the live place-graph work,
which owns `Ground`'s internals.

The public surface proved sufficient without reaching into those
internals, so the mechanism ran while G2 remained dirty. Mere's ignored
`crates/probes/ground-resident` consumes only `Ground::revision`,
`drain_dirty`, `keys`, `brick_materials`, and `Brick::{get,raw}`. One
radius-zero carve produced Ground revision 1 and dirty brick
`[-4, 0, -4]`; that public brick became one `ResidentChunk` with world
origin `[-32, 0, -32]`, `8x8x8` extent, one 512-byte exact-U8 CubeCL
allocation, matching revision 1, and host read epoch 1 on the RTX 4060.
Ground's raw order is y/z/x, so materialization performs one
revision-gated staging transpose into canonical world x/y/z order. It
is not a per-frame consumer bridge. Because the receipt used the live
dirty G2 tree based at `7b68db0`, it must be repeated before the exact
material histogram becomes a stable fixture. **Unblocked 2026-08-14:** that
G2 session landed at `7869c96` (verified green first: 316 core tests, 25 lens
tests, fmt, and clippy `-D warnings`).

**Closed 2026-08-16.** The rerun against the landed base reproduces the same
selection, and the histogram is now pinned as a fixture rather than reported:
brick `[-4, 0, -4]`, origin `[-32, 0, -32]`, and 512 voxels of
(air 130, rock 254, soil 128, other 0) after the one radius-zero carve. The
asserts carry their own diagnosis ("Ground generation changed"), and the
fixture assert was verified to fire by perturbing it, since an assertion that
has never failed is not a proven instrument.

### Lane F. Tracer lease (complete 2026-08-16)

The brick tracer reads the leased allocation; revision coherence and
unchanged-frame silence receipts. The permanent `mesocosm-lens`
integration was blocked on its tracer sources landing, and the public resident
seam was sufficient to prove the mechanism without editing that in-flight
work. **Unblocked 2026-08-14:** those sources landed at `7869c96`
(`BrickTracer`, `tracer.wgsl`, `BrickMap` in `bricks.rs`), so adoption is now
scopable against real code. The shape to reconcile: `BrickMap` owns an
`R8Uint` material atlas it rebuilds from `Ground`, while the lease offers the
same bytes as a `RawKernelView` on a CubeCL sub-allocation. Adoption means
choosing which of the two owns residency, not keeping both.

**Answered 2026-08-16, and the brief had it wrong.** `BrickTracer` binds
`texture_3d<u32>` and holds no storage buffer at all, because G2 built it
fragment-only for downlevel reach (`g2_glprobe` runs
`downlevel_webgl2_defaults`, and WebGL2 has neither compute nor storage
buffers). So adoption cannot mean "bind the lease" the way the probe's DDA
does; that would trade away the downlevel path G2 deliberately bought.

Adoption is a change of the atlas's **source**, not of the tracer's bindings:
the lease feeds the atlas texture by GPU copy on capable devices, and the CPU
upload stays as the downlevel fallback. Proven in the probe: the leased
allocation copies GPU-side into an `R8Uint` `texture_3d` and the 512 voxels
read back matching the lease exactly, with zero CPU upload bytes. One wrinkle
recorded because it will recur: buffer-to-texture copies require 256-byte
rows and a brick row is 8 bytes, so a device-local repack sits in the path;
every byte still stays on the GPU.

**Landed in tracked code (mesocosm `a1805d2`).** `BrickFrameInput` gains an
optional `LeasedAtlas`, and the tracer copies a producer's resident voxels
into the atlas texture instead of uploading them. `LeasedAtlas` is a plain
wgpu buffer/offset/size triple plus the revision it was materialized at,
deliberately not a producer's view type, so `mesocosm-lens` depends on no
compute stack and the buffer contract stays the meeting point. A lease whose
revision does not match the frame's is refused and the frame falls back to
the CPU upload, so a stale lease cannot present itself as current.
Diagnostics separate the paths (`leased_atlas_bytes` for what arrived
GPU-side, `brick_upload_bytes` unchanged as the CPU measure,
`stale_lease_rejections` for refusals), and the receipt asserts both
directions. 26 lens tests green, fmt and clippy `-D warnings` clean.

**The promotion trigger has fired.** The lease now has a shipped producer
(`ResidentChunk` in quint) and a shipped consumer (`BrickTracer`), which is
the condition §1 named.

**Reconciled 2026-08-16, and extraction is deliberately not the answer.**
The three spellings (`ResidentChunk`, the `SpatialBufferLease` sketch,
`LeasedAtlas`) were reviewed against each other. Extracting a shared type
would put a common crate between mere and mesocosm, and the whole reason
`LeasedAtlas` is a plain wgpu triple is that `mesocosm-lens` must depend on
no compute stack: the buffer contract is the meeting point. There is also
exactly **one** shipped consumer, and this plan's own rule promotes a
reusable view only after *multiple* consumers prove it. Extraction is
therefore premature by our own standard, and the trigger for it is a second
shipped consumer, not this one.

What the reconciliation did instead:

- **One producing-side definition.** `RawKernelView::lease()` yields a
  `SpatialLease` (buffer range, shape, element type, stamp). Consumers keep
  their own spelling; this is the single place the producing side says what
  those terms mean, so a host cannot invent a field or drop the stamp.
- **It closed a real hole rather than a stylistic one.** The consumer sized
  its copy from `extent` and never checked `extent` against `size`. A
  producer pools many planes into one buffer, so an overrunning extent does
  not fault, it reads whichever plane sits next in the pool and paints it
  into the world. `byte_len`/`fits` exist on both sides now, the tracer
  refuses a misfit lease as it refuses a stale one, and **both refusals fall
  back to the CPU upload** rather than leaving the atlas unwritten (the
  second half of the same bug: a refused frame would otherwise have shown
  whatever the atlas held before).
- Diagnostics separate every path: `leased_atlas_bytes`,
  `brick_upload_bytes`, `stale_lease_rejections`, `misfit_lease_rejections`.

The lesson worth keeping: the drift the audit feared was not two types
disagreeing about field names. It was two types disagreeing about *who
checks the bounds*, and neither did.

**The wgpu split found 2026-08-15 is resolved.** Mesocosm, netrender, Quint,
Burn, and CubeCL now compile on wgpu 30. Mere commit `6ce399ea` moved Quint to
Burn `0.22.0-pre.2` and CubeCL `0.11.0-pre.2`, retiring the local
`cubecl-wgpu` 0.10 backport. The retained real-Ground receipt compiles and runs
against that row and the published native `cubecl-runtime`. Browser manifest
and existing-device receipts closed in Mere `19d2cb60`. Because Cargo patches
are workspace-root scoped, Mesocosm would need to restate Mere's temporary
`cubecl-runtime` manifest patch if this opt-in receipt becomes a browser
target. Its current gate is native.

Mere's ignored `crates/probes/resident-tracer-lease` binds the exact 512-byte
U8 `RawKernelView` allocation directly into a buffer-backed voxel DDA. On the
RTX 4060, it recorded zero tracer-owned material-upload bytes. An unchanged
16 ms frame performed zero dispatches, bind-group rebuilds, uploads, or
readbacks; the same allocation refreshed after the configured 60-second
elapsed interval without reattachment or upload. A public `Ground` carve
advanced the observed revision/read epoch from `0/10` to `1/11`, changed
2973/4096 pixels, and caused the older revision to be rejected. This proves
the lease, coherence, and elapsed-time silence mechanism. It does not claim
that the in-flight `BrickTracer` has adopted the lease.

### Execution receipts (2026-08-14)

- **A complete.** `ResidentChunk` landed in Mere commit `986fdb91`
  (four quint paths; the concurrent epoch-carriage session swept them
  into its wider commit; audit confirms the paths are in that commit).
  Real-adapter allocation-identity and metadata receipts pass, and the
  identity receipt is the strong form: it resolves the CubeCL handle
  carried by the public Burn tensor and compares buffer, offset, and
  size against the raw view, rather than comparing two metadata values
  quint minted itself. `wgpu::Buffer` equality proxies to the inner
  handle, so that assertion is identity. A second receipt fixes a
  constraint worth stating plainly in the seam's own terms: exact U8
  planes are kernel-viewable but **not** Burn-tensor-viewable
  (`burn_f32_view` returns `BurnElementType`), so "Burn-addressable"
  means the float planes. Audit gap: the brief's "no CPU whole-plane
  read exists in the path" is untested; the test header describes the
  test's method, not a property of the seam. Lane F shows the right
  instrument (counted upload bytes, asserted zero). Add that counter or
  strike the condition.
- **B complete, local ignored probe.** Nexus `3083a43` radix/LBVH
  harvest passes duplicate/sorted/reversed/single sort cases, 4096-leaf
  reachability and containment, and the GPU canary. At 50k, GPU LBVH
  construction measured 5.042 ms beside the differently scoped 305.451
  ms CPU Barnes-Hut reference (a live in-process `seiche` control, not a
  remembered figure). This does not replace mass aggregation. Two audit
  notes: traversal and force accumulation are unmeasured, so no
  end-to-end claim exists and the two figures should not be read as a
  ratio; and the harvest landed as hand-written WGSL (`lbvh.wgsl`,
  `radix.wgsl`), so if lane C's consolidation is eventually made, the
  algorithm knowledge transfers but the artifact needs rewriting. B and
  C were briefed as independent; they are not, and the brief was wrong
  to say so.
- **C complete, verdict earned with a control (2026-08-16).** The
  audit of 2026-08-14 withdrew the original verdict because the probe
  compared a repulsion-only pass against a hardcoded full-frame figure
  from another session. The control has now run: the probe builds
  `quint::resident::Resident` on the same device in the same process
  and times `repulse_only` under the same wait boundary, both carriages
  warmed, 20 synchronized frames at n=50k on the RTX 4060/Vulkan,
  wgpu-30 row. **CubeCL 11.39 ms (9.97-12.52) vs incumbent rust-gpu
  15.84 ms (15.68-15.97), ratio 0.72; the incumbent confirmed it ran
  the committed SPIR-V via passthrough, so the comparison beat the real
  artifact, not the WGSL fallback.** JIT first launch, compile
  included, is 12.61 ms, one frame's worth. Numeric agreement stands at
  `1.26e-7` mean relative error. **Verdict: consolidate the explicit
  lane on CubeCL-JIT**, now aligned with the engine composition
  ruling's authored-in-CubeCL line rather than substituting for it. The
  audit's caution is also vindicated in passing: `repulse_only` alone
  measures above the old 12.8 ms "full frame" number on today's row,
  which proves the cross-session constant was never a comparable
  baseline.
- **D complete.** Synthetic tensor/process/commit mechanism above.
- **E complete against the live G2 tree.** The stable-base histogram rerun
  is pinned as a fixture.
- **F complete and adopted in tracked code.** Direct leased-buffer DDA,
  revision/read-epoch coherence, stale-lease and undersized-lease rejection,
  and elapsed-time silence pass. The downlevel CPU upload path remains.
- **Real-Ground composition and retention closed locally (2026-08-20/21).**
  `mesocosm-lens --example resident_ground`, behind the
  `resident-ground-receipt` feature, is the first source-level cross-repository
  consumer of Quint's `ResidentChunk`. On the RTX 4060/Vulkan it proved
  Burn/raw allocation identity, a four-byte candidate read, constrained
  `Ground` revision `0 -> 1`, a four-byte accepted patch into one retained
  262,144-byte atlas allocation, restamping at read epoch 11, four CPU bytes
  for its pointer projection, 184 changed pixels, unchanged-frame atlas
  silence, and replay from the 112-byte accepted delta. The identity assertion
  compares buffer, offset, and size across the commit. Source origin and
  row/image strides narrow the committed tracer copy to the changed 512-byte
  brick. Lease validation also proves that a partial source cannot suppress a
  full refresh or an unrelated changed slot.

## The engine composition ruling (2026-08-16)

Ratified by Mark, 2026-08-16, from the renderling/Burn/nexus/hecs
composition discussion. Four decisions and their consequences; this
section is the authority for the core module's shape.

### 1. The core render module: hybrid, raymarch-first

Designed on its own terms, not conditioned on what any prototype
currently calls (that framing was retracted): mesocosm and paredros are
the two consumers that will prove the seam, per the promotion rule.

1. **CubeCL passes** own procgen, growth/NCA, field simulation, and
   meshing only where meshes are genuinely needed (colliders, bakes,
   sprites, distant-LOD impostors). All authored kernels, all
   Burn-addressable by construction.
2. **The brick raymarch draws the ground**: color plus depth into real
   depth targets. No mesh extraction in the hot path, so LOD is brick
   clipmaps/mips; T-junction seam stitching is a mesh-path disease and
   returns only for distant-LOD bakes.
3. **Renderling rasters the dynamic bodies** as instanced meshes,
   depth-tested against the raymarched ground. Thousands of bodies are
   what its slab instancing is for (P3: 400k vertices,
   transforms-first); per-pixel SDF composition stays for the few, not
   the many.
4. **netrender composites** what it never parses, as already ratified.

Transforms flow GPU to GPU, solver buffer into instance slab; the P3
ambience lease is the receipt for exactly this bridge (3.6ms, epoch
control). The CPU reads only the sparse set gameplay touches. A
per-frame full transform readback into the ECS is refused; that copy
boundary is the one this architecture exists to delete.

### 2. The carriage line: author CubeCL, consume rust-gpu

Exclusivity applies to what we write, not what we run. Our kernels
(quint's, procgen, growth, the brick renderer, meshing) are authored in
CubeCL, where Burn synergy pays. rust-gpu remains in the stack as
*consumed artifacts* from upstreams that maintain them: renderling's
shaders, and nexus's khal-compiled kernels at adoption. The local
rust-gpu fork and rebuilt cargo-gpu are the maintenance tooling for
consumed artifacts, no longer an authoring lane.

### 3. Nexus: adopted at the bodies gate

Thousands of dynamic bodies is a bought capability by ruling, not a
speculative one. Nexus arrives as a dependency (same dimforge family as
the tactile tier; its kernels share rapier's types), meeting the stack
at the wgpu buffer contract. The 6,302-line solver is not ported.
Verified 2026-08-16: nexus is rigid-body only today (the umbrella crate
gates a lone `rbd` subsystem; the only "sph" in the tree is "swept
sphere"), so water-worlds fluids stay on the grid/channel plane lane.
Adoption-time gates, per the consumer-pull rule: khal's wgpu row
membership, and nexus's feature requirements against the greedy tenant.

**Superseded in shape, 2026-08-28:** the ruling sharpened (engine review
§2.5; the consolidation map in mere's conatus engine plan) — Nexus's
kernels arrive as a Conatus backend behind `BodyWorld`, never as a
stack-level dependency with its own vocabulary; the buffer-contract
meeting point above survives as the backend's internal seam.

### 4. Renderling: hands-on fork, not a frozen inheritance

Adopting renderling into the core module upgrades the fork's posture:
Mark accepts a hands-on approach. The 0.9-to-0.10 shader-source
migration (45 committed `.spv`, currently inherited rather than
reproducible) is a sanctioned work item, tooled by the rust-gpu
carriage we already built, with priority set by the first shader edit
the engine actually needs (voxel-aware lighting is the likely first
customer). The shader-edit wall is hereby a planned doorway.

### hecs, placed

Runtime index, never the record. Adopted at the gameplay tier (hecs
over bevy_ecs: small, MIT/Apache). Authoritative entity facts stay in
the record with their replay hashes; the ECS is a disposable projection
of them, exactly as GPU allocations are. It holds handles and gameplay
state, not a mirror of every body's pose.

### The seed crystal: the ruling's P0, landed 2026-08-16

One frame loop, all four roles composed, in mere's ignored
`crates/probes/seed-crystal` (windowed, winit): a crystal-growth
automaton and a voxel DDA raymarcher, both authored as CubeCL kernels,
advance and draw two ping-ponged 96-cubed F32 planes held by a
`ResidentChunk`; the raymarcher writes packed RGBA8 into a
CubeCL-allocated pixel plane; one pre-authorized buffer-to-texture
copy (3,686,400 bytes) feeds netrender's composition (chrome bar with
a motion sweep at scene-op boundary zero); the swapchain presents.
Burn addresses the same plane for the growth curve, one scalar
readback per 120 frames.

Receipts on the RTX 4060/Vulkan, wgpu-30 row, 1280x720, 900 frames:

- Steady median 5.91 ms, p95 12.62 ms, worst 37.47 ms; the first
  frame is 66 ms and carries both kernel JIT compiles (the carriage
  cost, one frame wide, matching lane C's measurement).
- Zero CPU-side voxel or pixel bytes per frame; the only per-frame
  GPU copy is the priced present copy.
- Allocation identity asserted at init (the Burn view and raw view of
  each plane are one buffer), and the kernels launch over the same
  CubeCL handles the Burn views resolve to.
- The growth curve read through Burn is monotone (100 to 77,129 cells
  over 450 ticks), and two runs agree on 77,129 exactly: the automaton
  is deterministic (hash-gated, ping-ponged, no atomics).
- The capture (`testing/mere/seed_crystal.png`) was inspected, not
  just measured: dendritic voxel crystal, face-shaded, sky gradient,
  chrome composed above. Guard measured per row against the row's own
  edge, because a gradient background makes corner-pixel guards count
  the sky as lit (the guard-calibration lesson, recurring).

Findings, for every CubeCL kernel that follows:

- The cube macro cannot expand `==` on floats, and a `let mut` local
  that is later *compared* needs a written type annotation (the
  comparison expansion erases back-inference; arithmetic-only locals
  infer fine). Band tests (`>`, `<`) replace equality.
- CubeCL 0.10 has no texture bindings, so a compute renderer's output
  is a storage buffer plus the one copy the ruling priced. Confirmed
  in practice; the copy is invisible at this scale.
- This probe is the first windowed harness on the real wgpu-30 row
  (P3's lock predates the move): `SurfaceTexture::present` moved to
  `Queue::present`, and `SurfaceConfiguration` grew a `color_space`
  field. Recorded so the next windowed probe does not rediscover them.
- Growth tuning is a design lever, not a bug: high accretion
  probability grows a boulder that self-blocks (every frontier cell
  gains two neighbours and the automaton freezes); low probability
  keeps single-neighbour tips alive and the structure dendritic. The
  freeze is legitimate physics for the rule, so the stall assert only
  fires below a minimum size.

### The remaining genuinely new system

VRAM residency policy: which chunks live resident, paging, the budget.
CubeCL's allocator is the pool (the 8MB pooled-buffer observation and
the sub-allocation triple rule stand); the policy above it is ours.

## Prior art, harvest, and standards (2026-08-14)

Per the borrowing doctrine: lean on mature references and standards,
harvest license-clean technique, and never adopt a tool for a
capability the stack intends to own. Licenses marked **verified** were
read from source today; everything else is verify-at-adoption.

- **Device and kernel language.** wgpu/WGSL is the W3C WebGPU standard
  (fetch the spec, cite the section, as always). CubeCL and Burn are
  MIT/Apache and already in the row. The rust-gpu AOT carriage is
  working in-tree.
- **Far field and spatial queries: harvest nexus.** MIT OR Apache-2.0
  (**verified**, workspace manifest; checkout at `Code/crates/nexus`,
  commit `3083a43`). Lift as kernels and patterns into conatus lanes,
  never as an engine dependency (conatus owns the capability). The
  shopping list: Karras-2012 LBVH (`src_rbd_shaders/broad_phase/lbvh.rs`,
  723 lines, paper linked in-source), the complete GPU radix sort
  (`src_rbd_shaders/utils/radix_sort/`, 9 files, 757 lines), the
  bottom-up refit with atomic arrival counters, the Windows+NVIDIA
  atomic-load workaround (wgpu#9221), and the radix-sort bounds-check
  caveat noted in `nexus_rbd2d/build.rs`.
- **Tracer.** In-house (the brick tracer exists). Its algorithm
  standard is Amanatides & Woo 1987 (voxel DDA); brickmap residency
  follows the van Wingerden brickmap thesis; NanoVDB is the reference
  layout for GPU-resident sparse voxel grids, to align with or diverge
  from knowingly.
- **Meshing (bake paths only).** The bonsairobo family
  (`fast-surface-nets-rs`, `block-mesh-rs`, `ndshape`) is the pure-Rust
  prior art; surface nets per Gibson 1998. Raymarch stays the hot
  path; meshing is for bakes and Isometry sprites.
- **Worldgen.** `noise` 0.9 is Apache-2.0/MIT (**verified**);
  `fastnoise-lite` as alternative. WFC per Gumin (MIT) with the
  in-house Isometry experience as the hard-rule layer under any
  learned prior. NCA per Mordvintsev et al. (distill.pub 2020).
  Diffusion worldgen stays refused for shipping; NCA plus decoder is
  the shippable shape.
- **Field simulation and fluids.** DIY on the standards: Stam's stable
  fluids and Bridson's course notes for grids; lattice Boltzmann as
  the grid-native alternative. salva stays refused (CPU SPH against a
  resident-grid design, and it lags rapier).
- **Agents.** Burn-native modules (GRU, autodiff in dev builds);
  evolution strategies per Salimans et al. 2017. Semantics are
  governed by the general-model plan, never imported from an RL crate.
- **Content interchange.** `dot_vox` is MIT (**verified**);
  MagicaVoxel `.vox` is the de facto interchange standard for voxel
  content packs.
- **Persistence and replication.** Owned: muniment/codicil for the
  store, retinue/personae for peers and identity (the dogfood rule;
  the Syncthing precedent). Minecraft's documented palette-section
  encoding is prior art for the idea of palette compression; the
  format itself is not adopted. Content addressing on the BLAKE3 row.
- **Physics.** parry `Voxels` derives tactile queries from committed
  revisions, under this wing's own three-tier ruling (integer
  authority, parry advisor, rapier in reserve). The mere canvas side
  (seiche) is unaffected.

Three flags:

- **Veloren is GPL-3.0**: read its worldgen writeups for technique,
  never copy code.
- **hecs is approved only as a gameplay runtime index.** `mesocosm-core`
  retains record authority and replay identity. Adoption begins with a
  concrete gameplay consumer and may not turn the runtime index into a
  second world representation. SoA columns aliased as tensors remain views
  over owned state rather than a storage transfer.
- **Nothing else in the critique's component list is auto-adopted.**
  Each entry composes through the seam or waits for a consumer.

## Product adoption correction (2026-08-23)

Mesocosm is now the first tracked consumer of Conatus's CPU voxel mechanics,
without transferring `Ground` authority. `mesocosm_core::voxel_profile` keeps
disposable `VoxelChunk<u8>` views beside the record, preserves Mesocosm's
global source revision separately from Conatus's per-chunk mechanics, and
lowers only occupancy-changing patches for tactile consumers. The adapter
refuses stale or regressed source revisions atomically. It does not enter a
snapshot, and a repeated unchanged source produces an empty update.

The focused receipt proves four things against the real `Ground`: constructing
and synchronizing the profile leaves snapshot bytes untouched; one accepted
carve produces revision-addressed chunk and occupancy changes; the same source
revision is silent on the next sync; and two replay-identical grounds produce
identical snapshot bytes and projection changes. This is first-consumer
evidence for the mechanics, not promotion of a cross-product voxel authority,
identity, resident lease, or frame contract.

The dependency baseline is also repaired. Mesocosm's Mere-family Git source
moved together from stale commit `c10d82bd` to `5767563c`, so the real
`resident_ground` example now compiles with Quint's `commit_plane_patch` API.
The example remains part of the concurrent resident-view lane and is not
claimed by this adapter commit.

## Projection cache correction (2026-08-26)

`BrickProjectionRevision` identifies a selected key-to-slot projection
separately from Ground's `BrickRevision`. The product working-set policy
advances it when selection changes, including travel inside one unchanged zoom
band. The keyed constructor requires that explicit identity rather than
silently assigning revision zero. `BrickTracer` retains equal-sized textures,
fully republishes them when the projection revision changes, and performs no
brick upload on a repeated unchanged stamp.

`LeasedAtlas` carries the same projection revision, so a valid buffer range
from another page cannot masquerade as the current slot mapping. That mismatch
is refused and falls back to the CPU atlas. Unknown changed slots are refused
before any upload or resident cache-stamp advance.

The lens has a headless equal-extent cache/lease test, and Paredros has policy
coverage for same-band travel. The refreshed 96-frame headed V1 trace closes
the gate on the RTX 4060 Laptop GPU: a one-brick far-page move advances
projection revision 4 to 5 while preserving the 795,144-byte extent, exchanges
66 loaded for 33 evicted bricks, fully republishes without texture or bind-group
creation, and uploads zero bytes on the next frame. The original four-brick
probe changed physical extent and was not evidence for this path.

## Brick traversal ownership lift (2026-08-26)

The reusable organ moved to Mere's `conatus-brick` at commit `28c07fab`
(the pin has since advanced to `bd8f0044`, which added V1b's capacity-fixed
retargeting mode).
That crate owns deterministic pointer/atlas layout, projection-stamp carriage,
`BrickTraceSpace`, and the camera-neutral `BRICK_DDA_WGSL`. Mesocosm's
`bricks.rs` now only binds authoritative Ground bytes into that layout; its
tracer shader retains camera, fog, material, body, look, and composition policy.
Paredros owns a separate Ground binding and consumes the same platform types on
its default compile path.

The exact pushed revision passes the Mesocosm lens suite and the Paredros V1
suite. The rerun headed Paredros artifact remained pixel-identical after the
extraction (`C881B3CCAAED1690B174196719E595D2326ACB19FE4D9399EC8DA288E317E61A`).
This lift does not merge the brick map with `ResidentChunk`: Quint still owns
resident allocations, while products still own source selection, camera, and
presentation.

This remains evidence over the lens-local texture owner. It does not promote
projection identity, frame cadence, or leases into a cross-product contract.
Both residency residues closed later the same day as V1b (engine review §5):
`LeasedAtlas::read_epoch` is now tracer-validated against a frame-stated
expected epoch, and the `ResidentChunk`-backed per-brick cache with
allocator-observed resident and transition bytes is proven by the Paredros
`v1b_residency` receipt and the extended `resident_ground` receipt.

## Stop rules

- No second world representation inside any consumer; a consumer that
  wants one is asking for a view type instead.
- No CPU whole-plane reads in the per-tick path; durability reads are
  deltas, hashes, checkpoints, reductions.
- Candidate planes never feed collision, persistence, or peers; only
  committed revisions do.
- A view is promoted to a shared contract only after multiple real
  consumers prove it (the lease's promotion rule).
- A field is admitted by a named simulation or presentation consumer; the
  existence of a resident plane type is not sufficient demand.
- A lower-dimensional or network phenomenon keeps its honest domain instead of
  being inflated to dense 3D for storage uniformity.
- Render mips, clipmaps, and residency windows remain views; any authoritative
  field-resolution transition has its own conservation and fidelity contract.
