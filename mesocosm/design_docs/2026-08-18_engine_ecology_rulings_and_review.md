# Engine and Ecology Rulings: Critical Review (2026-08-18)

**Status: provisional engine rulings, reviewed against the live code through
2026-08-26.** The vessel brief remains authority for product identity,
camera, and presentation. The resident-views composition plan remains
authority for the shared-device seam and the completed A-F proofs. This
document records the consequences of those decisions, corrects the
overclaims that accumulated in discussion, and names the next proof gates.

The short conclusion is that the architecture survives review. Its useful
shape is a record with several projections, one shared GPU allocation path,
a hybrid renderer, and deterministic resolution around learned or evolved
proposals. The claims that fail are the claims that turn a reusable mechanism
into a finished subsystem.

## 1. Evidence labels

- **Standing** means an existing ratified document or landed proof already
  carries the decision.
- **Working ruling** means this is the recommended shape, but a named proof
  may still overturn it.
- **Correction** means the discussion stated more than the evidence supports.

## 2. Conclusions that survive

### 2.1 One record, several projections

**Standing.** A vessel consumes facts, places, motifs, bodies, and provenance
from the shared record, then realizes them at its own scale. Mesocosm,
Paredros, and Isometry do not need a permanent interchange format for their
render chunks. Their voxel resolutions and presentation policies may differ.

This does not abolish shared spatial semantics. The record still needs stable
place and region identities, coordinates and scale transforms, material and
biome vocabulary, provenance, and projection-version identity. Exact player
edits such as a tunnel or base also remain exact facts even when a later
vessel renders them differently.

There are therefore two fidelity regimes:

1. **Deep-time projection.** Facts and motifs survive; each vessel regenerates
   its own spatial realization.
2. **Contemporaneous construction.** Exact committed edits survive; another
   vessel may simplify their appearance but must not silently change their
   meaning or topology.

The cross-vessel render-chunk problem is narrowed to a shared spatial and
historical contract. It is not deleted.

### 2.2 One resident allocation path

**Standing.** `ResidentChunk` is the seam between exact world state, compute,
rendering, and persistence. CubeCL owns allocation outward; raw wgpu and Burn
views refer to the same allocation; revision and valid-read epochs guard use.
The seed crystal proved CubeCL growth and DDA rendering over one resident
chunk with zero CPU voxel or pixel transfer per frame.

The plane type matters:

- exact occupancy and material planes remain integer or fixed-point authority
  and are consumed by raw CubeCL kernels;
- Burn is a good producer and consumer of fixed-shape floating-point fields,
  including density, distance, light, learned growth, and candidate state;
- constraints and the record decide which candidate result becomes an exact
  committed fact.

"Burn produces the field" is accurate for learned and derived floating-point
fields. It is not the ownership rule for every voxel plane.

### 2.3 Hybrid rendering

**Standing core, incomplete composition proof.** Mere's `conatus-brick` now
owns the shared brick layout and DDA traversal core. Each vessel still owns its
camera, lighting, LOD, composition, picking, and readability policy.

- Mesocosm uses an orthographic terrarium section with a near/far slab and
  organism presentation composed over it.
- Paredros uses perspective traversal and sparse residency. A clipmap or mip
  joins the continuous pull-back when a real exact camera footprint exceeds
  its resident budget.
- Isometry may use orthographic traversal where live volume matters and baked
  sprites or meshes where a stable tabletop asset is cheaper.

Ground remains raymarched on the hot path. Renderling remains the mesh tenant
for dynamic bodies, baked surfaces, and impostors. Netrender owns composition.
This is shared traversal, not one complete renderer with three camera
matrices.

### 2.4 Fields, extraction, and draw

**Working ruling.** For a floating-point candidate or derived field, the bake
path is:

```text
Burn tensor field
    -> CubeCL count / prefix / scatter extraction
    -> GPU vertex range plus draw count
    -> renderling draw
```

The direction is correct and keeps data on one device. Stock renderling does
not yet expose the required uninitialized GPU-only vertex allocation. The
missing surface also crosses craballoc/crabslab concerns: contiguous capacity,
slab growth and lease invalidation, overflow reporting, compacted count, and
indirect or otherwise bounded draw submission.

Renderling's stock `Vertex` is 26 words. A procedural bake should measure
whether that ABI is acceptable before making it the universal extraction
target. A compact procedural vertex ABI may be the better fork seam.

This path is for bakes, colliders that require meshes, Isometry assets, distant
LOD, and forms that cannot stay raymarched. It is not a reason to mesh the
resident ground every frame.

### 2.5 Collision and physics

**Working ruling, with a sharp boundary.** Parry `Voxels` is the direct tactile
projection for committed exact occupancy. It has sparse chunks, an internal
acceleration structure, incremental voxel edits, and point, ray, contact, and
mass-property queries. A Ground revision or delta can update it at a bake
boundary. Terrain collision does not require surface extraction first.

A Burn-generated candidate field cannot become authoritative collision by
remaining GPU-only. It must pass constraints, commit to exact occupancy, and
then materialize the relevant delta for the CPU collision advisor. That
readback is a deliberate bake boundary, not a per-frame world copy.

Nexus remains gated on a real dynamic-body consumer. Its current GPU rigid
body path accepts triangle, polyline, convex, and primitive forms, but it does
not consume Parry `Voxels` directly. Complex body geometry is initialized from
CPU-side Parry shapes, and the current append path is more restricted than the
initial build. Nexus's useful parts can be decomposed into a Conatus backend
after an interop proof; it is not the terrain-collision answer today.

**Sharpened 2026-08-28 (Mark): Nexus is decomposed, never adopted.** When
the first vessel gate needs dynamic bodies — Paredros F5 material life and
F7 danger are the expected pulls — Nexus's useful parts arrive as a Conatus
backend behind the same `BodyWorld` vocabulary that already keeps Rapier
private, on the one-device CubeCL/wgpu lane R2 proved. It never becomes a
peer engine with its own vocabulary, which retires both standing hazards:
the upstream shader-build blocker (only the kernels that serve this stack
are taken) and the Parry-`Voxels` admission gap (its solver meets the world
through Conatus's voxel colliders). The consolidation map this ruling
belongs to lives once, in
`mere/design_docs/mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md`.

### 2.6 Evolved behavior

**Working ruling.** The useful part of Job Talle's
[neuroevolution experiment](https://jobtalle.com/neuroevolution_in_squids.html)
is the co-evolution of body affordances and a small inherited controller. Its
current squid network has no sensors and evolves autonomous locomotion against
a short generation score, so it is not yet a model of ecological behavior.

Mesocosm's safe seam is narrower:

```text
body + visible traits
    -> trait-gated senses and drives
    -> inherited, bounded policy
    -> proposed intent
    -> deterministic movement and ecology resolver
    -> authoritative events
```

The first fauna proof should replace target ranking inside the existing
movement policy. It must not replace terrain legality, movement costs,
resource conservation, event emission, or replay authority. Start with a
fixed small recurrent topology, quantized weights, trait-gated sensors and
effectors, and an inspectable decision trace. Let survival and reproduction
provide selection rather than inventing one global fitness score.

Flora should not be forced through the same controller. A developmental
grammar is the better source of form; a slow policy can allocate resources and
choose growth direction. Near fauna may run individual controllers while far
cohorts retain aggregated tendencies. Burn can batch candidate evaluation at
world generation or epoch review, but GPU floats remain proposals unless the
chosen intent or quantized state is recorded deterministically.

The visible trait graph remains the player-facing genome. An evolved policy
may express those traits; opaque weights must not bypass them.

### 2.7 The trait board

**Working ruling.** Mere's graph canvas, typed nodes, springs, exclusions, and
support queries make it a plausible substrate for the epoch review board. It
is not a completed review UI. A real Mesocosm adapter still has to prove trait
adjacency, contradiction and synergy feedback, preview of the compiled body,
undo, commit, keyboard access, and a replay-stable result.

### 2.8 Record, time, and peers

**Standing direction.** The three vessels can share an intent and event
grammar while keeping different schedulers: turn order, fixed real-time
steps, and epoch batches are not interchangeable clocks. A log alone is not a
runtime. Cross-host authority also needs causal order, ruleset and evaluator
version identity, conflict rules, snapshots and compaction, and spatial
invalidation for regeneration.

Integer and fixed-point state is the default for authoritative ecology across
hosts. That is a discipline, not an ontology. Exact rational forms or recorded
resolved outcomes are also valid when they replay identically. Floating-point
GPU results remain derived fields or proposals.

## 3. Critical corrections

| Discussion claim | Verdict | Corrected ruling |
|---|---|---|
| Broad-strokes fidelity kills the cross-vessel chunk problem. | Overstated. | It removes a permanent render-chunk interchange format. Shared spatial semantics and exact committed edits remain. |
| One tracer, three cameras. | Useful shorthand only. | Share brick layout and traversal. Each vessel owns rendering policy beyond camera math. |
| Mesocosm's renderer is nearly free. | Rejected. | Its ground renderer is cheaper than Paredros's, but organism presentation, section readability, picking, and epoch UI remain product work. |
| The trait board is Mere's canvas. | Plausible substrate. | It becomes true only after one Mesocosm review adapter completes a commit and replay receipt. |
| One intent log solves three time models. | Overstated. | The grammar can be shared; scheduling and conflict semantics remain distinct. |
| Every authoritative number is integer or fixed-point. | Good default, too absolute. | Require cross-host replay identity; integers, fixed point, exact forms, or recorded resolution may satisfy it. |
| Burn to CubeCL to renderling needs only a few fork lines. | Direction valid, scope understated. | The allocation API is small, but lifetime, capacity, count, ABI, and draw submission form the actual proof. R2 also found that CubeCL 0.10 cannot import renderling's slab allocation: the live path is one device, two allocators, and one device-local publication copy. |
| Parry or Nexus can collide with these forms. | Split answer. | R3 proves Parry ray, point, and contact-manifold queries over revision-gated committed occupancy. Nexus can later simulate baked dynamic shapes after an interop proof — as a decomposed Conatus backend per the sharpened ruling in §2.5, never a peer engine. |
| Squid-style neuroevolution supplies flora and fauna behavior. | Promising mechanism, wrong authority. | Evolve a bounded proposal policy behind existing deterministic ecology resolution; treat flora separately. |

## 4. Authority and projection flow

```text
shared record / Ground exact facts
    -> ResidentChunk exact and fixed planes
       -> CubeCL authoritative transforms
       -> Burn floating-point fields and candidates
          -> constraints + explicit commit
             -> new exact record revision

committed resident revision
    -> brick DDA                 hot ground image and depth
    -> CubeCL extraction         mesh bakes, bodies, LOD assets
       -> renderling             raster tenant
    -> Ground delta
       -> Parry Voxels           tactile and collision advice
    -> later Nexus adapter       dynamic rigid bodies only
```

Neither rendering, Burn, Parry, nor Nexus owns world truth. Each consumes a
revisioned projection with an explicit validity boundary.

## 5. Recommended build order

R1, R2, R3, B1, and V1 moved the shared engine past feasibility.
Projection-correct residency, the brick ABI/DDA ownership lift,
raymarch-depth composition (D1, 2026-08-26), the first real Mesocosm
tactile consumer through Conatus (T1, 2026-08-26), and the stable
resident brick cache (V1b, 2026-08-26) are now complete: the ordered
shared-engine consolidation chain is closed. What remains in §6 is
consumer-forced, not ordered work. Product work, including the epoch
slice, remains independently ordered by its own consumer gates rather
than serving as a prerequisite for every reusable mechanism.

### R1. Shared traversal profile: **COMPLETE 2026-08-20**

Run one brick/DDA data contract under a Mesocosm slab camera and a Paredros
perspective camera. Vessel-specific lighting and LOD policy may differ.

**Done when:** both headed views consume the same traversal implementation and
brick ABI; captures and timings are recorded; no renderer-wide abstraction is
introduced merely to hide their policy differences.

**Receipt, 2026-08-20.** `BrickTracer` now accepts a camera-neutral ray
basis: origin, forward, right, and up, plus projection kind and far cut. Its
WGSL keeps one pointer lookup, material atlas, ray-box intersection, and DDA;
only ray construction differs. The old `Flight` constructor remains a
perspective adapter, so the existing G2, carve, lease, and downlevel receipts
retain their source contract.

Mesocosm's real headed G2 frame now supplies a 16-voxel orthographic slab at
the ratified terrarium angle. On the RTX 4060 Laptop GPU at 1920×1080, frame
32 recorded 3 µs tracer preparation, zero steady brick and uniform upload,
and 1.238 ms in netrender's reported frame total. The inspected capture and
JSON receipt are `Code/testing/mesocosm/r1_terrarium.{png,json}`.

Paredros's real S0 room host consumes the same `mesocosm_lens::BrickTracer`
behind its historical `r1-proof` receipt while retaining its own close-perspective
eye and target. Its 64-frame 1280×720 headed run recorded 11.601–34.646 ms
overall, 12.552 ms median, zero steady brick upload, 2,357 capture colours,
and the unchanged position-log hash `0x27a905731c6bfc61`. Both receipts name
the same Rust `BrickMap` ABI: origin `[-9,0,-8]`, pointer extent `[18,3,17]`,
atlas extent `[128,16,128]`, 3,672 pointer bytes, and 262,144 atlas bytes.

This closes traversal reuse, not renderer unification. Mesocosm owns the
slab; Paredros owns the perspective rig, torch, and later LOD. R1 did not
create a renderer-wide camera abstraction or claim the later
raymarch-depth/renderling join.

### R1a. Shared brick traversal owner: **COMPLETE 2026-08-26**

Mere's `conatus-brick` at commit `28c07fab` now owns deterministic `BrickMap`
layout, projection-stamp carriage, `BrickTraceSpace`, and the camera-neutral
`BRICK_DDA_WGSL`. Mesocosm and Paredros each own their Ground source binding.
Mesocosm's shader retains camera, fog, material, body, and composition policy;
Paredros carries the shared organ in its default compile path while retaining
its own camera and residency selection.

The Mesocosm lens tests and Paredros V1 tests pass against that exact pushed
revision. The 96-frame headed Paredros rerun passed on the RTX 4060 Laptop GPU,
and its PNG remained byte-identical to the pre-extraction artifact. This
promotes the proven mechanism only. Quint still owns `ResidentChunk`; product
profiles still own source bindings; projection identity, frame cadence, and
lease scheduling remain provisional.

### D1. Raymarch depth composed with renderling: **COMPLETE 2026-08-26**

Brick-raymarch ground and renderling raster geometry occlude each other per
pixel on one device, closing the join R1 deliberately did not claim.

The mechanism is a shared depth attachment, not a second compose pass.
Renderling's stage already stores single-sampled standard-z `Depth32Float`
depth; `brick_dda` already returns hit distance. `mesocosm_lens::BrickTracer`
gained the camera-neutral half — a `clip_from_world` uniform, an `fs_depth`
entry writing `@builtin(frag_depth)` from the raster tenant's own matrix, a
lazily created depth-pipeline variant, and `encode_with_depth` under
`LessEqual` with the colour target loaded rather than cleared. The plain
`encode` path is untouched, and a headless lens test proves the join without
renderling by pre-filling depth and asserting the occlusion split; all 33
lens tests are green. Draw order and the receipt scene stay vessel-side:
Paredros's opt-in `d1_depth` bin drew its body and three cyan witness
pillars through renderling over the raymarched room and judged projected
point probes on the RTX 4060 — raster covering rock where nearer, the floor
covering a buried pillar base, a wholly sunken pillar invisible, with a
positive control in the same frame and the replay hash unchanged. Receipt at
`paredros/design_docs/2026-08-07_paredros_execution_plan.md` §3 D1 and
`Code/testing/paredros/d1_depth.{json,png}`.

One durable caution: the stage *replaces* its depth texture on size or
multisample changes, so a consumer fetches the depth view after each raster
draw; a held view silently tests the join against zeroed memory and loses
every pixel.

### T1. Terrarium picking through Conatus: **COMPLETE 2026-08-26**

The first real Mesocosm tactile consumer, chosen by Mark 2026-08-26:
pointer picking in the terrarium section. A cursor position becomes an
orthographic-slab ray, the ray asks Conatus, and the answer names the
critter, the exact ground voxel face, or nothing. This completes
`Ground -> GroundVoxelProfile -> Conatus` and, per R3's ruling, names the
adapter's permanent host crate: **mesocosm-runtime** (also ruled by Mark
2026-08-26). Rapier stays private inside `conatus`; Mesocosm holds Conatus
ids and arrays only, and picking is advice — it writes nothing into the
record.

The adapter keeps a `conatus::BodyWorld` beside the profile: one fixed
body whose `VoxelGrid` collider (cell size 1) holds the profile's occupied
cells in world coordinates (`chunk key * 8 + local cell`), synchronized
from `GroundVoxelUpdate` occupancy edits under the same
stale/regressed-revision refusals the profile enforces, plus fixed capsule
bodies for critters (a tapered pose capsule is carried at its mean radius;
taper is a recorded approximation). The consumer is a headed lens example
over the real G2 terrarium scene.

**Done when:** the runtime adapter refuses stale and regressed source
revisions atomically; a committed carve reaches the collider as exactly
its occupancy edits; a headed RTX 4060 run sweeps a deterministic cursor
trace across critter, terrain, and sky, and every ground answer matches an
exact oracle over `Ground` occupancy (independent of Rapier)
cell for cell, the critter position picks the critter, the sky position
picks nothing; a mid-run carve moves the swept column's answer exactly as
the oracle says; the pick log replays identically; and the capture plus
JSON receipt land at `Code/testing/mesocosm/t1_picking.{json,png}`.

**Receipt, 2026-08-26.** All hold. `mesocosm_runtime::tactile` owns
`TactileWorld`; its eighteen-crate test run proves the exact surface cell
under a vertical pick, the carve flowing through `sync` as occupancy
edits, the atomic stale refusal, and — via a ray through a capsule's
off-centre length — that capsule orientation is really applied. The
headed example `t1_picking` swept 22 judged cursor stops over the real G2
terrarium on the RTX 4060 Laptop GPU (Vulkan): 16 ground answers, every
one cell-for-cell and distance-for-distance against the axis-aligned
`Ground::solid` oracle (the terrarium slab's `-z` rays make the oracle an
integer scan, sharing no arithmetic with Rapier); 4 critter answers
through the capsule core; 2 nothing answers above the slab's tallest
surface. The mid-sweep carve took the first swept cell `[-20, 3, 26]`,
advanced Ground revision 0 to 1, reached the collider as exactly one
synced cell, and moved that cursor's answer as the oracle said. The whole
judgment ran twice from freshly grown worlds and agreed bit for bit
(`fnv1a64:f66edcef9ac2d3ac`). One judged edge arrived free: the first
stop's origin cell was itself solid at the slab's near face, and Rapier
and the oracle agreed at distance exactly zero. Capture and receipt at
`Code/testing/mesocosm/t1_picking.{json,png}`; the capture shows the
cursor on the critter with its class chip and marker.

Two boundaries worth keeping. Conatus queries answer through Rapier's
broad phase, which only learns colliders during a step, so this
never-simulating adapter settles it with a minimal zero-effect step after
topology changes (everything fixed, gravity zero); if Conatus grows a
query-refresh seam, that call is what it replaces. And a tapered pose
capsule is carried at its mean radius — the T1 stops pass through the
capsule core exactly so no radius model can disagree; a consumer needing
taper-exact grazing answers must first widen Conatus's capsule
vocabulary.

### V1b. Stable resident brick cache: **COMPLETE 2026-08-26**

The residency lane V1's ruling reserved and §6 carried: a stable
`ResidentChunk`-backed brick cache with per-brick publication and measured
allocator bytes, joining Quint-owned residency to `conatus-brick`
traversal. Ruled by Mark 2026-08-26: the slot-retaining, capacity-fixed
`BrickMap` variant lands in `conatus-brick` as new commits continuing
`codex/conatus-brick-lift` from `28c07fab`, with both consumers repinned;
the V1 1 MiB budget becomes the hard cap of the fixed allocation
(capacity 1,984 atlas slots plus the fixed pointer box); and the gate
closes with two receipts so quint's compute stack stays out of
paredros-room.

The mechanism: `BrickMap` gains a capacity mode whose atlas and pointer
extents never change — retained keys keep their atlas slots, evicted
slots recycle deterministically to loaded keys, and a retarget rewrites
the (kilobyte-scale) pointer volume wholesale while touching only loaded
bricks' 512-byte atlas slots. The tracer honours a changed-slot
declaration across a projection advance — full pointers, loaded slots
only — and validates an expected lease read epoch, closing "host promise"
into refused mismatches. Fixed extents mean the tracer's existing
equality check already yields zero texture recreation.

**Done when:** conatus-brick proves slot stability, deterministic
recycling, capacity and extent refusals, and pointer correctness across
retargets; the tracer proves a projection advance with a slot declaration
uploads exactly full pointers plus the loaded slots' atlas bytes while
rendering identically to a full rebuild, and refuses a lease with a
mismatched read epoch; Paredros's headed V1b travel run repeats the V1
zoom-and-travel trace over the stable cache with zero texture or
bind-group creation at every band transition, per-brick transition
uploads, retained one-frame recovery, and `wgpu` allocator-report bytes
recorded beside the logical count; the extended `resident_ground`
receipt publishes a carve as per-brick `commit_plane_patches`, leases
exactly the changed slots at a stated epoch, refuses a deliberately
mismatched epoch, and records CubeCL `memory_usage` bytes; and no
existing receipt changes behaviour.

**Receipt, 2026-08-26.** All hold. `conatus-brick` at `bd8f0044` (the
branch continuation Mark ruled) owns `BrickMap::with_capacity` and
`retarget`; its tests prove kept-slot stability, deterministic recycling,
pre-write refusals, and read parity with a rebuilt map; both consumers
repinned. The tracer honours a changed-slot declaration across a
projection advance — full pointers, loaded slots only — renders
pixel-identically to a from-scratch rebuild over retained textures, and
refuses a lease at any epoch other than the frame's stated one; all 35
lens tests are green.

The headed Paredros run holds the V1 trace over one cache of 1,791 slots:
931,376 fixed bytes under the 1 MiB cap (whole atlas rows make the
capacity 1,791 rather than the estimated 1,984 — the cap is the ruling,
the count derives), pointer box `[34, 3, 34]`, atlas `[128, 56, 128]`,
extents never moving. Six retargets each created zero textures and zero
bind groups and uploaded exactly the 13,872-byte pointer volume plus
their loaded slots: the close snap published pointers alone, and the
travel frame moved 47,664 bytes against V1's 795,144 — sixteen times
less. `wgpu`'s allocator report read 182,561,728 bytes after the first
frame and after the last: growth zero. Both rapid zooms recovered in one
frame. Receipt and capture at
`Code/testing/paredros/v1b_residency.{json,png}`.

The extended `resident_ground` receipt closes the Quint join: the carve
reaches the retained `ResidentChunk` as one 4-byte
`commit_plane_patch` with CubeCL's own accounting unchanged across the
commit (262,144 bytes in use, one allocation, growth zero), the tracer
takes a 512-byte one-brick lease at the stated epoch, and the same lease
at any other expected epoch is refused with the CPU fallback drawing the
committed picture pixel for pixel.

One instrument finding worth keeping: per-slot `write_texture` costs
about 0.37 ms per call on this machine, so a 1,290-slot reload took
485 ms naively. Consecutive slots occupy contiguous atlas boxes, so the
tracer batches a loaded set into a few strided box writes from the map's
own atlas slice — same bytes, 13.5 ms. Publication granularity stays
per-brick; only the submission count changed.

### R2. GPU mesh bake into renderling: **COMPLETE 2026-08-21**

Add the smallest safe GPU-only range allocation seam, extract one resident
field with CubeCL, and draw the resulting compacted vertices through
renderling.

**Done when:** the receipt proves buffer identity, contiguous capacity,
overflow behavior, compacted draw count, slab-growth invalidation, zero CPU
vertex staging, and a headed capture. The receipt decides full `Vertex` versus
a compact procedural ABI.

**Receipt, 2026-08-21.** Craballoc now allocates a contiguous GPU-only array
without queuing initial CPU values. Renderling exposes that capacity as
`Vertices<GpuOnlyArray>` and refuses a bounded draw count above it while
retaining the full allocation lease.

The probe generated a 48 by 48 floating field through Burn, then ran CubeCL
count, exclusive prefix, and scatter passes. It produced 7,182 vertices, equal
to the CPU count oracle. A deliberate 7,181-vertex capacity raised the GPU
overflow flag and renderling separately refused `capacity + 1`. The retained
range was 13,824 vertices, or 359,424 contiguous words. Forced slab growth
invalidated the old buffer, produced a new identity, and publication targeted
the buffer attached on the following commit. The allocation queued zero CPU
vertex values.

The discussion shorthand was wrong about one allocator. CubeCL 0.10 can reuse
the host's wgpu device, but its compute server cannot import an arbitrary
renderling slab buffer as a CubeCL handle. The proven path is therefore:

```text
Burn field
    -> CubeCL count / prefix / scatter in CubeCL-owned memory
    -> one device-local buffer copy
    -> GPU-only renderling range plus bounded draw count
```

Vertex contents never cross the CPU. Two u32 values, compacted count and
overflow, return at the asynchronous bake boundary. This is one device and two
allocators, not the claimed one-device/one-allocator path.

On the RTX 4060 Laptop GPU the final headed run drew the 7,182 vertices with 63
distinct scene-region colours and 31.6% non-background coverage. Full
renderling `Vertex` cost 746,928 bytes; a ten-word procedural position/colour/
normal ABI would cost 287,280 bytes, 2.6 times less. The ruling is to keep full
`Vertex` for bounded asynchronous bakes because it draws through stock
renderling after the small allocation seam. It is not the universal procedural
ABI. A compact path must be pulled by a high-density bake that proves the
bandwidth and shader complexity are worth a second draw contract.

### R3. Committed voxel collision: **COMPLETE 2026-08-21**

Project one Ground revision into Parry `Voxels`, apply a later Ground delta,
and compare ray, point, and contact results with exact occupancy.

**Done when:** unchanged regions remain untouched, changed voxels alter the
expected queries, revision mismatch is refused, and replay produces the same
collision answers. Only then test a learned field through propose, constrain,
commit, and collision projection.

**Receipt, 2026-08-21.** The standalone `crates/probes/parry-ground` receipt
consumes only public `Ground` revision, dirty-brick, key, material, and
occupancy APIs. It projected 136 stored Ground bricks into Parry 0.29
`Voxels`: 69,632 cells compared and 41,763 occupied voxels. Parry's internal
3D chunks are also 8 cubed, so they align with Ground's brick grid.

One boundary carve committed Ground revision 0 to 1 and removed 19 voxels
across four dirty bricks. The projection rescanned exactly those four regions,
2,048 cells, and changed exactly the 19 committed occupancies. The other 132
region occupancy signatures stayed unchanged. Parry's `set_voxel` maintained
the cross-brick neighbor masks needed to suppress internal collision edges;
the adapter did not rescan adjacent Ground materials.

The committed delta moved a downward ray hit from 2.5 to 4.5 voxel units,
changed point containment from true to false, and cleared a ball contact
manifold from one contact to zero. The ray result matched an exact Ground
occupancy oracle before and after. A stale source revision and a skipped target
revision were both refused before mutation. Re-growing the same seed and
replaying the carve reproduced the whole query receipt bit for bit.

Two API boundaries matter. Ray and point queries are direct `Voxels` traits,
while contact uses Parry's persistent contact-manifold dispatcher; its simpler
single-contact helper does not dispatch voxel shapes in 0.29. The finite
`Voxels` projection covers stored Ground bricks at `y >= 0`. Ground's implicit
solid bedrock below zero is an analytic half-space, not an infinite voxel
allocation.

This remains a host-side projection. `mesocosm-core` stays integer-only and
Parry-free. Promotion waits for the first tactile gameplay consumer to name the
adapter's permanent host crate; R3 proves the contract rather than choosing
that owner early.

### B1. Bounded evolved fauna policy: **COMPLETE 2026-08-21**

Insert one quantized recurrent policy at the existing target-choice seam.

**Done when:** fixed seeds replay identically, the decision trace names the
traits, sensed facts, and selected drive, the established behavior receipt
still passes, and distant cohort conservation is unchanged. A useful strange
behavior is evidence; a higher synthetic score alone is not.

**Receipt, 2026-08-21.** Near grazers and predators now rank directly visible
targets through one fixed three-drive recurrent topology. Five integer inputs
carry energy deficit, target nearness, relative mass, a sensed warning, and
recent target memory. The policy uses signed byte weights, signed integer
biases and recurrent state, and proposes only pursue, avoid, or hold. Sight,
grounded stepping, route legality, movement cost, feeding, and event emission
remain in their established resolvers.

The decision trace names the body's current feeding mode, reach, locomotion,
and sensory-part count; the exact local facts; all three drive scores; and the
selected target and drive. Warning input is available only when the body has a
`Sense` part. Feeding mode gates eligible prey, reach gates the visible set,
and locomotion still gates the resolver's step budget. A sensed warning can
therefore reverse the default pursuit into avoidance without granting the
policy any new terrain authority.

The genotype and recurrent state are snapshot authority. Each child inherits
its parent's weights with one bounded mutation derived from the filial seed,
then starts with clear recurrent memory. Survival and ordinary reproduction
therefore provide selection; B1 introduces no synthetic fitness score. Far
bodies, producers, and scavengers retain their previous policies, so the
individual controller does not leak across the tier line or force flora
through a fauna mechanism.

The grounded receipt snapshots and restores the full decision trace and
recurrent state, while a twin fixed-seed run produces the same state hash. The
pre-existing wall-and-door autonomous movement receipt still passes, including
grounded legality and replay. The 300-organism mixed-tier receipt also still
matches state hashes and exact far-cohort member, biomass, and energy totals.
This closes the authority-path choice in favour of quantized integer state.
Floating-point Burn batches remain candidate generators only if a later
consumer records or requantizes their proposals before authority.

### V1. Residency budget under Paredros scale: **COMPLETE 2026-08-21**

Let a concrete continuous-zoom scene pull the residency policy. Do not design
a universal pager without the scale pressure that makes its tradeoffs visible.

**Done when:** the scene records visible range, resident bytes, upload and
eviction cadence, camera hitch behavior, and recovery after rapid zoom or
travel.

**Receipt, 2026-08-21.** Paredros's feature-gated `v1_residency` binary grows
a 256-voxel-half-extent Ground and drives the ratified third-person camera
from distance 8 to 72 while it rises from 50 to 65 degrees. Visible range is
the four-corner ground-plane footprint plus one brick. The complete 6,091
brick region refuses the exact tracer's 4,096-brick ceiling, while the camera
pulls exact page radii 40, 88, and 128 through the new bounded
`BrickMap::from_ground_keys` seam, which requires an explicit projection
revision.

The far planning frame covers radius 127 with 1,411 bricks and 795,144 logical
pointer-plus-atlas bytes under a 1 MiB budget. Five page transitions over 96
frames moved 2,250,848 bytes. Warm page preparation took 230 to 1,463
microseconds. On the RTX 4060 Laptop GPU at 1280x720, frame spans were 4.385
to 32.271 ms, 5.918 ms median, with a 5.876 ms steady median. The four warm
transition frames took 5.534, 6.822, 5.995, and 6.724 ms; the maximum was an
unchanged close-page frame. Both abrupt zooms met their profile's 125 percent
recovery threshold on the next frame. The inspected final capture has 63
distinct colours. Paredros owns the camera and key selection; Mesocosm owns
exact Ground and `BrickMap` allocation.

Two corrections carry forward. The byte budget is logical payload, not
allocator-observed VRAM, and excludes driver rounding plus transition overlap.
The load and eviction counts are policy deltas. Extent changes still create
replacement textures and upload a complete page. Equal-sized pages now retain
the existing textures and fully republish when their explicit projection
revision changes.

**Verdict.** Exact sparse pages are sufficient for this first base-planning
view, so V1 does not justify building a clipmap yet. The permanent seam is a
stable `ResidentChunk`-backed brick cache with explicit projection revision,
per-brick publication, and allocator-observed bytes. Larger planning views or
travel can then pull clipmap or mip work if they exceed the exact budget.
Traversal now has a platform owner; this receipt still does not promote the
resident lease or frame contract.

### V1a. Projection identity for equal-sized pages: **COMPLETE 2026-08-26**

`BrickProjectionRevision` now identifies one selected key-to-slot projection,
separately from Ground's authoritative `BrickRevision`. Dynamic working-set
owners advance it when selection or slot assignment changes. The retained
tracer compares it before the cache-hit return, fully republishes a changed
equal-sized projection without rebuilding textures or bind groups, and keeps
an unchanged projection upload-silent.

Resident atlas leases carry the same projection revision. A lease from another
page is refused even when Ground revision, physical range, and slot coordinates
match, and the CPU path fills the frame instead. A changed-slot declaration
that names an unknown slot is refused before uploads or cache-stamp advance.
Paredros now detects key-set changes inside one zoom band and advances the
projection revision, removing V1's travel refusal. The lens owns a headless
equal-extent cache/lease receipt and Paredros owns a same-band policy test. Its
refreshed headed V1 trace moves the focus one brick inside the far band. Frame
72 advances projection revision 4 to 5, retains the 795,144-byte texture extent,
fully republishes it with zero texture or bind-group creation, and frame 73
uploads zero bytes. The 96-frame RTX 4060 Laptop GPU run also preserves
one-frame close/far recovery and produces a non-empty 62-colour capture.

This implements cache coherence for equal-sized travel. It does not make the
identity durable or cross-product. Its open items closed in V1b on
2026-08-26: lease `read_epoch` is now tracer-validated against the frame's
stated epoch, and incremental `ResidentChunk` publication with
allocator-observed bytes is proven on both the Paredros travel cache and
the `resident_ground` join. No cross-product frame or lease contract is
implied.

## 6. Open choices carried to proof

- ~~which Mesocosm tactile consumer completes
  `Ground -> GroundVoxelProfile -> Conatus` while Rapier remains private~~
  — chosen 2026-08-26 (Mark): terrarium pointer picking, adapter hosted in
  `mesocosm-runtime`; see T1;
- the exact shared spatial contract for contemporaneous construction across
  vessels;
- ~~the product-forced join between Quint-owned `ResidentChunk` residency and
  `conatus-brick` traversal, including publication granularity and an expected
  lease epoch or equivalent token~~ — closed 2026-08-26 as V1b: per-brick
  granularity, and the frame's expected read epoch validated by the tracer.

These remain consumer-forced contracts. They do not suspend unrelated product
work, and none authorizes a generic engine umbrella in advance.
