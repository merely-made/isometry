# Place-Graph Engine Plan (2026-08-05)

**Status: active substrate record, clarified 2026-09-01.** Founded from the
2026-08-04/05 engine rumination. The G labels below record the order in which
the substrate was assembled. They are historical indexing, not current
acceptance gates. Sibling to the
[render lane landscape](2026-07-30_engine_and_render_lane_landscape.md),
which owns renderer research and the V-gates, and subordinate to the
[execution waves plan](2026-07-31_execution_waves_plan.md) for ordering.
This plan owns the world substrate: the place graph, volumetric truth, the
two-scale execution substrate, and the composed slice that proves them
together.

## 0. Rulings this plan rests on (2026-08-05)

Recorded here and amended into the founding record, `CLAUDE.md`, and the
landscape doc in the same session:

1. **One authority per capability, stack-wide.** The anti-Spore rule's
   kernel: one simulation authority; projections plural and cheap; refuse a
   second authority, and refuse duplicating functionality the stack already
   owns. (Narrows "refuse a second simulation or a second renderer".)
2. **Shared engine organs are encouraged; sovereignty lives in the verbs.**
   Vessels share nouns (space, bodies, fields, time, provenance); each owns
   its verb and person (metabolize, address, adjudicate). An organ is
   shareable while it stays verb-neutral; when a component encodes what you
   do, it belongs to a game. (Narrows the founding's "no engine" clause,
   which guarded against coupling-as-obligation and genre convergence; that
   guard stands as: no shared genre, no shared schedule, no shared verbs.)
3. **Volumetric world truth is permitted.** The lens-only stop rule opens.
   The anti-Minecraft insight stays: no machinery adopted because prior art
   ships it; acceleration structures are admitted by trace.
4. **Verticality is a simulation affordance.** The third axis must be
   mechanically legible (wings escape ground threats, canopy holds
   resources, burrows hide). Any projection that makes those facts readable
   qualifies; 3D rendering is not the only way.
   *Camera amended 2026-08-06:* Mesocosm's camera is **pulled back, Rain
   World-style**, superseding the 08-05 Barony-march ruling. First person
   survives in agency (person is agency, not camera). The Barony/Delver
   first-person march reference moves to Paredros, which is the brick
   tracer's long-term primary consumer.
5. **World graphs must differ.** No two worlds share topology unless they
   are copies. Distinctness is receipted with graph metrics, not vibes.
6. **Nesting is elective and continuous.** Containers index one global
   coordinate space; an edge is never a portal; a place with no internal
   topology stays a leaf.
7. **Worldgen is hybrid.** Top-down skeleton (relief, watersheds), then
   bottom-up detail within regions.
8. **The unit of proof is a slice.** Engines fail at the joints. No gate is
   proven for the wing until the composed run exercises it beside the
   others.
9. **Design reference targets (2026-08-06).** Mesocosm: Rain World,
   Voxatron, Caves of Qud. Paredros: Barony, Delver, Gotcha Force,
   RimWorld. Isometry: Foundry, the Larian/Owlcat adaptations, Wildermyth,
   tactical RPGs virtual and tabletop. Reading that matters: none of the
   nine demands a heavyweight renderer; all are simulation-deep and
   presentation-modest. Investment order follows: agency, procedural
   bodies, place graph, storyteller over renderer horsepower.
10. **Physics is three tiers (2026-08-06).**
    - **Authority: integer, ours.** Occupancy and movement legality over
      brick truth, inside the replay hash. Never a physics engine.
    - **Advisor: parry queries + bespoke kinematics.** Raycast, shapecast,
      TOI, contacts from parry (no dynamics world, no stepping, no
      persisted handles); move-and-slide and verlet chain gait are owned
      code that quantizes into integer outcomes. The nalgebra seam is one
      adapter around a query library.
    - **Ambience: GPU, outcome-free.** Debris, foliage, cloth, ragdoll
      (nexus-shaped, eventually). GPU float ordering varies by hardware,
      so this tier is constitutionally barred from outcome-bearing facts.
    - Rapier-the-dynamics-engine is **in reserve**: it returns only for a
      proven constraint-dynamics need, and its documented cross-platform
      determinism mode is why it alone may then sit near the fact plane.
      Avian rejected 2026-08-06 (Bevy-coupled by design, parry underneath,
      no determinism story).
11. **Renderer tenancy follows the cohesion contract** (landscape §8.9):
    one device, one frame, declared capability profiles, glam at the
    presentation boundary, receipts. Renderling is the lead mesh-tenant
    *candidate* pending its fork probe; kiss3d is a donor (AOV
    segmentation bakes, 2D GI); nexus is ambience-tier pending its own
    device audit; rust-gpu is welcome, carried by the fork family.

## 1. The model

One continuous integer coordinate space (the core's `[i32; 3]`), with
layers of meaning over it:

- **Place graph (macro).** Places own regions of the space: a generation
  recipe, a revision, dirty state, a simulation tier. Adjacency is derived
  from the landscape (link where travel is possible), never asserted.
  Nested containers are elective: a burrow system is a subgraph on a place;
  a meadow stays a leaf. The chartulary resemblance is noted and not
  depended on; a portable profile follows two real consumers, per the
  standing rule.
- **Brick truth (micro).** Per-region brick maps: coarse grids of pointers
  to dense material bricks; an absent pointer is air or unloaded. Dense
  bricks because they hash, diff, and serialize flat; SVO/DAG stay
  benchmark specimens (landscape §8.3 stands). Interiors are carved bricks
  in the same space.
- **Two-scale execution target.** Places near the played body run embodied
  individuals; distant places may run deterministic cohorts once the
  conversion and comparison receipts in the
  [scale plan](2026-08-29_scale_plan.md)
  land. The incumbent tick is not there yet: `organism/ecology.rs` still
  advances every organism record and derives `Cohort` only as a conserved
  summary. Promotion and demotion already happen at a hops-distance boundary
  with hysteresis, but that line currently selects locomotion and perception
  detail rather than changing canonical representation.

**Scale names are not authorities (clarified 2026-09-01).** “Macro” and
“micro” above describe the spatial index and voxel detail. They do not name
separate simulations. The future individual and cohort forms are two
canonical representations of one world; the trophic web is a derived reading
of their stocks and flows. The standard transition verbs are **materialize**
and **aggregate** because hydration already means water in this ecology. A
zero-tick round trip must preserve the aggregate record exactly, while
evolution over time is compared against an all-individual reference under a
declared error envelope. Camera distance, render LOD, and cache residency
cannot choose simulation detail. Recorded focus and other world facts may.
The [playable ecology plan's downstream gates](2026-08-31_playable_ecology_plan.md#7-downstream-architecture-gates)
route that scale-owned contract into PE6.

Presentation stays a family of projections of one truth (landscape §8):
a brick-map DDA raymarch grows out of the landed march for the
first-person lens; `mesocosm-mesh` pointed at world bricks is the raster
projection; the grade is unchanged; bodies ride the landed
`BodyLensProjection` (V2). Fields (numen) attach to places and bricks. Tactile
advice derives from the same snapshot through the landed T1 adapter
(`mesocosm_runtime::tactile` over Conatus, Rapier private inside it;
§8.4's advisor tier unchanged).

## 2. The slice: the burrow run

One composed scenario used to expose joins between systems.

Worldgen lays a place graph over one voxel space; one region generates
with a burrow. A catalogue critter hunts the player by real sight-lines
through real bricks. The player flees or carves, and hides inside
geometry. The hunter follows across the threshold without a stutter.

The original integration claims, now exercised in one run, were:

- the run replay-hashes identically from its recorded intents;
- the brick tracer renders it headed, native and browser, both souls, with
  fps and frame spans recorded beside netrender's (V1 harness reused);
- a carve lands: intent, dirty brick, region re-upload, collision refresh,
  with the latency recorded;
- the burrow needs no special casing in tracer, collision, or perception;
- per-tick sight-line cost is recorded at the target critter population.

The run is an integration specimen, not a game-completion test. Whether one
unfinished encounter feels tense neither validates nor blocks the engine. The
useful next move is to connect another independently owned fact to the same
scene and see which behavior appears without an authored branch.

The engine document (profile, capability matrix, external references) is
written after this run exists, from its numbers, not before.

## 3. Build history

### G0. A graph worth the name — **COMPLETE 2026-08-08** (constructor 2026-08-06; genesis adoption 2026-08-08)

Landed additively as `Places::grown(seed, side, extent) -> Grown` in
`places/relief.rs` (integer diamond-square, 65², own seed, percentile sea)
and `places/grown.rs` (traversability-derived links: climb + ford limits
over sampled crossings, Chebyshev-2 candidates, union-find reconnection
over least-bad blocked passes; `Nest` interiors where ruggedness earns
them). `scatter` and its serialized shape are untouched, because `Places`
rides inside snapshots and world genesis draws from a shared stream after
it; adoption is a one-line swap in `world/genesis.rs` deferred while that
file is in flight. Receipts (all green, strict clippy): same-seed
bit-equality; per-seed topology uniqueness over an 8-seed corpus; the
lattice-regression test (non-uniform interior degree, every seed); edge,
diameter, nest, and **bridge** counts all varying across the corpus with
at least one world growing a chokepoint; whole-world connectivity;
congruence (every unlinked grid-adjacent pair fails the crossing test,
which required making `crossing` direction-canonical after integer stride
rounding made A→B and B→A disagree); serde round-trip. Threshold
calibration kept as an `#[ignore]` spectrum test.

Replace `Places::scatter`'s lattice. Today: stratified jittered sites with
hardcoded 4-connected links, so every world is topologically identical
(uniform interior degree, fixed diameter, zero bridges, hubs, or dead
ends), and `links` disagrees with `at()`'s nearest-site partition, which
can make diagonal Voronoi neighbours the graph denies. Build the hybrid
generator: relief and watersheds top-down, local detail bottom-up,
adjacency derived from traversability, elective nesting where interiors
have topology.

**Done when:** the same seed reproduces the same graph bit-exactly; across
a seed corpus, degree distribution, bridge count, cycle structure,
diameter, and nesting depth all vary, and a lattice-regression test
rejects uniform degree; derived links agree with the partition (no
Voronoi-adjacent pair denied without a landscape reason); the reckoning
consumers (`hops`, `spread`, `scale`) keep their existing receipts.

### G1. Brick truth with a lifecycle — **COMPLETE 2026-08-08** (container 2026-08-06; world adoption 2026-08-08)

Landed additively as `places/bricks.rs`: `Ground::grow(&Grown, extent)`
raises dense 8³ bricks (ordered map, serde-flat) from the relief; nests
realize as **roofed burrows anchored at the highest column near their
host** (a low host digs into its hillside rather than cratering; rooms
scale to afforded depth; every chamber keeps a ceiling). Each also has a
direct one-voxel descending entry to a roofed room: a former vertical shaft
looked like an interior but could not be climbed by the owned near-stepper.
`carve` bumps one
revision and marks dirty bricks; `drain_dirty` is the projection's upload
discipline. Occupancy (`solid`, `stands`) and integer line-of-sight
(`sees`) land here too, seeding G3's perception. Receipts, all green with
strict clippy: same-world bit-equality + serde round-trip; identical
carves replay to identical bytes; a radius-1 carve dirties ≤8 bricks and
carving air is not an edit; burrows are roofed voids near every nest; every
generated entry step is legal and ends under a roof;
hills block sight and a bored tunnel grants it (the first scan version
walked into a burrow corridor, which is its own kind of receipt); and in
`mesocosm-mesh/tests/ground_projection.rs`, every matter-bearing brick
meshes through the same `mesh_volume` path bodies use, with clean bricks'
meshes untouched by a neighbour's carve. The original remaining G1 pieces —
carve as an ordered `Intent`, `Ground` inside the world snapshot/replay hash,
and genesis wiring — all landed with the `grown` swap on 2026-08-08.

Place-keyed brick regions over the one space; carve as an ordered intent;
dirty regions revisioned (the lens's `MapRevision`/`MapChange` discipline
generalized); the snapshot/replay hash covers bricks; `mesocosm-mesh`
consumes a world brick as a `VolumeSource` for the raster projection;
occupancy derives per brick for collision and perception queries. The dirty
queue is projection-only: it is omitted from snapshots and equality, so a
host's upload cadence cannot alter replay state.

**Done when:** a carved world replays to an identical hash; an unchanged
world uploads zero brick bytes across frames; a carve uploads only its
region; occupancy answers stand, burrow, and see for the near tier.

### G2. The tracer, riding the landed V1 harness — **COMPLETE 2026-08-14**

Fragment-only brick-map DDA in the lens's retained pattern: pointers and
brick atlas in 3D textures, no storage buffers, no compute; both souls;
SDF bodies composited as now. *Camera note (2026-08-06):* the DDA is
camera-neutral; with Mesocosm pulled back (ruling 4), the tracer's
long-term primary consumer is Paredros's first-person lens, and
Mesocosm's shipped projection is chosen at playfeel within the
pulled-back framing (mesh raster, side-view, or a pulled-back trace).
G2's proof value (volumetric presentation + downlevel receipts) is
unchanged.

**Done when:** the tracer renders a carved region with interiors correctly
occluded; downlevel and wgpu-GL receipts pass (the glprobe pattern,
landscape §8.8); a browser receipt lands through the same composition seam
and recording discipline V1 proved; 1080p fps is recorded on the 4060 and
on GL, and the number is reported whatever it is.

**2026-08-14, P0 Ground tracer.** `BrickMap` is a lens-owned, rebuildable
projection of `Ground`: `R32Uint` 3D pointers address a dense `R8Uint` 3D
material atlas, with slot zero as air. `BrickTracer` does fragment-only DDA
against those textures, encodes into a caller-owned target, and owns neither
world state nor submission. A real seed-4242 hill and bored horizontal
tunnel receipt confirms the visual lifecycle: the initial 265,612-byte upload
is followed by 43 removed voxels in two changed brick slots, a 1,032-byte
upload, and changed capture pixels. Steady frames upload zero brick bytes.
This proves Ground-to-texture provenance and narrow carve propagation. It does
not by itself prove composition, host, browser, downlevel, or 1080p evidence.

**2026-08-14, P1 composed cross-host receipt.** `BrickFrameInput` now carries
an optional presentation-only `CritterPose`; the DDA shader sphere-traces its
capsules and lets the nearer of its SDF hit and Ground hit own each pixel.
`g2_frame` is the headed harness: generated seed-4242 Ground is reconstructed
into `BrickMap`, one SDF body is composed with it, and the caller-owned trace
texture enters netrender at external scene boundary zero. The native release
run presented the composed 1920×1080 frame on the RTX 4060/Vulkan with scene
digest `fnv1a64:411f3c4f92446b5a`; the steady second frame made no brick or
uniform upload and recorded netrender's 991µs span ledger. The browser ran
the same 232,803-byte scene digest, trace and netrender path headed through
Browser WebGPU at 960×540, again with a steady no-upload frame and no browser
validation errors. The rendered canvas, not merely the receipt text, was
inspected: voxel ground, nearer SDF body, and chrome were all visible.

`g2_glprobe` requested `Limits::downlevel_webgl2_defaults()` on wgpu's real
GL backend (maximum 2D texture 2048): both clay and retro body-plus-Ground
frames rendered at 960×540, the grade change changed pixels, and its second
frame uploaded zero Ground bytes. `g2_bench` performs synchronized, readback-
free steady DDA frames at 1920×1080, including Ground and the SDF body:
**482.4 fps median** (2,073µs, 1,825–3,219µs) on Vulkan and **520.8 fps
median** (1,920µs, 1,697–2,641µs) on GL. These are tracer spans, deliberately
reported beside rather than mistaken for the headed netrender frame budget.

### G3. The near tier — **core LANDED 2026-08-06; ecology drive/tier slice LANDED 2026-08-07; player ingress, autonomous occupancy/sight wiring, and 300-body scale receipt LANDED 2026-08-14**

Landed additively as `places/near.rs` (kinematic `step` with slide, climb,
gravity settle, and doorway drops; `spot` sight; `Tier`/`TierLine` with
promote/demote hysteresis over hops). The former `places/hunt.rs` probe
supplied the historical chase receipt, which passes on the real seed-4242
ground: a
half-speed hunter acquires its quarry, loses it behind a hill, projects
along its heading, is forced off a cliff edge, re-acquires at a bored
den's mouth, and follows it inside, with per-tick continuity asserted
throughout (no teleports). Determinism receipt: identical runs
bit-identical. The former standalone Hunter probe measured **300 hunters in
~62µs** (release), but it did not exercise world ecology, Ground, or the
far-cohort projection and is retained as probe evidence only. Tier receipt:
the line does not flap crossing the band, and a far hunter wakes on
promotion. Two movement laws were earned by failure, both the same
lesson: **preferences order, they never refuse** — climb-when-descending
and the cliff-edge comfort drop each deadlocked an agent until the
preference became an ordering with a forced fallback. The Hunter wiring was
deliberately not adopted. E0-E4 now wires the tier line, far-tier receipts,
and anatomy-driven drives into the world's organisms; the old chase receipt
remains probe evidence rather than an authoritative FSM.

**Superseded in destination, 2026-08-06.** The `Hunter` FSM is **a probe,
not the design**. Search/Stalk/Memory is authored behaviour with a tuned
patience constant, and it duplicates a feeding model the ecology already
owns. Its lasting contribution is proving the *queries* work: sight
through terrain, pursuit across a burrow threshold, one-voxel continuity,
and the movement law that preferences order rather than refuse. The
[general model plan](2026-08-06_general_model_plan.md) gate **E4**
replaces it with drive-and-affordance selection, where pursuit is what a
fast, large-mouthed, starving body does about a reachable meal. Do not
build further behaviour on the FSM.

Perception as sight-lines (brick DDA or parry raycast), advisor-tier
locomotion per ruling 10 (parry queries + owned move-and-slide + verlet
chain gait, quantized into integer outcomes, no persisted handles), hunt
and flee for catalogue critters; tier promotion and demotion at a hops
boundary with hysteresis.

**Done when:** a hunter acquires, pursues, loses, and re-acquires the
player across a place boundary and a burrow threshold without stutter or
teleportation; per-tick agent cost is recorded at the target population;
the individual far-body path stays within its existing receipts and its
derived cohort projection conserves the recorded scalars.

**2026-08-14, V0 player closure.** Genesis now puts every founder on real
brick footing. `Intent::Move` resolves one `near::step` toward its requested
offset, charges only horizontal distance actually travelled, and cannot use a
large offset to cross a wall or slope. The focused receipt finds a real
climb-blocking face, records a carve that turns it into a doorway, crosses it,
and replays to an identical hash. This is deliberately not G3 completion:
`ecology::disperse` still uses its abstract integer step and has not yet read
`Ground` occupancy or `spot` sight.

**2026-08-14, V1 autonomous closure.** Replayed worlds now call
`ecology::step_with_ground`: near-tier food and carrion acquisition uses
`spot`, pursuit takes one or more bounded `near::step`s under the existing
locomotion budget, and exhausted near bodies wander by a legal grounded step.
Far bodies retain their place-graph movement and perception, while graph
traversal and near-tier birth keep embodied bodies on valid surface footing.
`Cohort` remains a derived conservation projection here, not the executing
far-tier record. The receipt
finds a generated occluding wall, proves an autonomous predator does not steer
through it, records a carve, then proves it enters the opened doorway and
replays identically. Population-cost and full burrow-run receipts remain open.

**2026-08-14, V2 grounded scale receipt.**
`grounded_ecology_receipt` starts from the actual seed-4242 world with 300
living Near founders, warms a disposable clone, then measures five
independent, identical 64-tick idle windows. Refreshed after the G4 perception
work, this Windows host's release build reports **897.05µs/tick** median
(887.95–928.89µs; 903.81µs mean), and all samples finish at the same state
hash. The receipt also rejects an
unfooted living Near body and rejects any scalar disagreement between raw Far
bodies and their cohort projection. It replaces the old Hunter number as the
G3 capacity evidence; it is a host-specific measurement, not a frame-budget
guarantee. Full player-and-burrow composition remains G4.

### G4. The burrow run — **COMPOSED 2026-08-21; interaction expansion continues**

G0 through G3 now meet in §2's scenario. P0-P8 below are the assembly record,
not a reason to stop extending the causal joins.

**2026-08-14, P0 one-world doorway run.** `burrow_run_receipt` and the
Lens `burrow_run` example now begin from one real `World`, rather than an
adjacent demo Ground. A played producer begins occluded from a near consumer
by generated brick terrain; an ordered `Idle`, then legal `Carve`, opens the
doorway; Ground sight changes; the consumer enters by its ordinary grounded
ecology step; and a twin produces the same outcomes and final replay hash.
The lens refreshes just one dirty atlas slot (516 bytes) and the DDA capture
changes after the eight-voxel carve. This is the composed authority and
projection join, but not the full G4 verdict: it is a generated occluding
doorway, not yet a selected roofed burrow encounter; the live run is not yet
headed through the G2 netrender/browser harness; population sight cost and
the human tension judgment remain open.

**2026-08-14, P1 generated-entry correction.** A nest's old vertical hollow
was a visual and raycastable burrow, not one an embodied body could enter:
`near::step` cannot climb a shaft. Generation now produces a direct,
one-voxel descending route into a roofed room and relocates the generated
room cluster at that entry. The Ground receipt walks every route edge through
the actual stepper. The world receipt then places a producer under that roof
and a Near consumer at the mouth: on `Idle` it sees the producer, descends on
the generated route, remains grounded, and twin-replays identically. This
settles actual continuous burrow ingress, not the full G4 scene. The existing
hide/reacquire-and-carve proof is still a selected generated doorway; combining
it with a turning interior needs a local path-search policy, not another
geometric special case.

**2026-08-14, P2 local lost-sight pursuit.** `LastSeen` is now serialized
organism state, rather than a host perception cache: a Near consumer refreshes
it from direct `spot` acquisition; it lasts for eight failed perception ticks
and clears on expiry, target death, or a tier change. `route_step` is a small,
deterministic breadth-first search over the actual `near::step` transition
relation, limited to an eight-voxel horizon and 256 examined stances. It adds
no collision map and makes no global-path claim. The generated-entry receipt
proves direct observation writes the replayed state. A separate one-world
receipt carves an L-shaped bore with the ordinary Ground primitive, puts a
target beyond an occluded turn, and proves a consumer's remembered target
takes the legal first detour and twin-replays. The expiry and Near/Far boundary
are unit-receipted too. Reacquiring a *moving* target through that turning
interior, the headless G2 scene composition, performance at population, and
the human tension judgment remain G4 work.

**2026-08-14, P3 external-frame closure.** The Lens `burrow_run` receipt no
longer stops at a tracer capture. Its same-device `G4Frame` encodes the
World-derived DDA view into an external texture and passes that texture through
netrender, which owns the final master frame. The before/after frames differ
after the recorded eight-voxel carve; the after frame carries one dirty atlas
slot and 516 brick-upload bytes into the composed master. The native receipt
also records netrender span count and writes that actual master as PNG. This
proves the headless frame seam for this run, not browser parity for this
particular scenario: the existing browser/downlevel G2 host remains a generic
Ground projection proof. A browser-directed burrow-run harness, moving-target
reacquisition through the turn, population cost under sight/routing, and the
human tension judgment remain G4 work.

**2026-08-14, P4 moving-target reacquisition.** The World receipt now uses
the played producer, rather than a fixed quarry: it is seen by a Near
consumer, moves through an ordinary player-carved L turn out of sight, leaves
the consumer following its serialized last-seen position, and is reacquired
within the eight-tick local window. Every step stays grounded and the whole
intent trace twin-replays. This composes acquisition, pursuit, loss, and
reacquisition through a real turning obstruction. It is intentionally a local
carved-turn proof, not the final §2 claim across both a generated burrow
threshold and a place boundary. Those two conditions, the browser-directed
burrow run, population sight/routing cost, and the human tension judgment still
keep G4 open.

**2026-08-14, P5 embodied scene subject.** The G4 frame no longer supplies a
handmade capsule as its player. It projects the controlled organism's
authoritative `BodyDocument` through `BodyLensProjection`, checks every living
part became an SDF capsule, and records that body revision with the composed
frame receipt. Thus the DDA terrain, played body, and netrender master all
come from the same World run without introducing a presentation body format.
The selected player currently has one living root part, so this is a real
ownership join rather than a multi-part visual stress case. The remaining G4
conditions are unchanged.

**2026-08-16, P6 sight cost at population.** §2 asks for per-tick
sight-line cost at the target critter population, and the existing
`grounded_ecology_receipt` did not answer it: it measures 64 ticks over
which the founders fall from 300 Near to 145, so its figure averages a
shrinking population. For a capacity question that is the whole
question. `sight_cost_receipt` measures a short window with the
population intact and sweeps it, so level and shape are both visible.
This Windows host, release: 75 bodies 133us/tick, 150 -> 398,
300 -> 1234, 600 -> 3747. **At the target, a tick with 300 Near bodies
seeing and routing costs about 1.23ms, roughly 7.8% of a 16.7ms
frame.** Per-body cost grows 2.31x across a 4x population: super-linear,
so the pairwise sight term is real, but well short of the 4x that fully
pairwise would give, and the tier line holds the target with room. The
span is a whole `Idle` tick with sight and routing active rather than
sight isolated, since there is no sight-off control to subtract; the
shape rather than the level is what implicates sight. Its assert
catches a blow-up rather than freezing a constant.

**2026-08-21, P7 browser-directed doorway run (implemented locally).**
The V1/G2 headed host now accepts a scenario module rather than owning one
hardcoded G2 document. `g4_frame` uses that same surface, device, tracer,
external-texture, netrender, and receipt path for P0's independently generated
seed-4242 Worlds and ordered `Idle`/`Carve` trace. Native Vulkan presented at
1920×1080; Browser WebGPU presented at 960×540. Both reported replay hash
`10803197323918604988`, scene digest `fnv1a64:ef6d3107b0e8b508`, eight removed
voxels, one dirty slot, a 516-byte brick upload, zero readback, and the
netrender span ledger. The browser canvas was inspected headed and emitted no
warnings or errors.

This receipt also corrected two presentation lies found during the run.
P0's hunter-side capture changed pixels but was mostly a wall, so the browser
receipt uses the judgment harness's actual subject position: the hidden
player's eye watches the authoritative hunter body enter the opening. That
view initially looked horizontally above the lower doorway; the browser and
judgment harness now derive the downward pitch from eye to threshold. The page
also puts the appended canvas before the long JSON receipt visually. These are
camera and host corrections over the same World and intents, not new scenario
facts. The two-frame timings are receipts, not an fps benchmark; G2's
synchronized 1080p spans remain the performance evidence.

**2026-08-21, P8 generated crossing.** The final machine claim now has one
run rather than an inference across fixtures. Seed 0 grows a five-stance nest
entry. The threshold is its first step; its third step crosses Place 3 into
Place 0. The played producer descends three ordinary `Intent::Move` steps and
the autonomous consumer selects `Pursue` on every ecology tick, following
exactly one generated stance behind. Both bodies cross the place edge, the
hunter crosses the burrow threshold, every position is grounded and unique,
and an independent twin returns the same seven history events and state hash
`7286350926008272852`. `Grown::nest_entries` is the narrow read model that
lets this receipt and a projection name the generator's exact route; terrain
generation still owns the construction rule and Ground remains the collision
authority.

The shared headed host presents that same three-intent run. Native Vulkan at
1920×1080 and Browser WebGPU at 960×540 reported scene digest
`fnv1a64:e90af60c63d5c6ec`, the same replay hash, revision zero before and
after, zero dirty slots, zero brick upload on the final movement frame, zero
readback, and the netrender span ledger. Native's final frame reported 163 µs
of tracer preparation and 4591 µs in netrender; the browser reported 0 µs and
1300 µs respectively. These four-frame timings are receipts rather than a
benchmark. The browser canvas was inspected headed and its diagnostics log was
empty. A too-close first camera cut through the wide hunter, so the final
projection keeps the actual player eye, aims at the body centre, and scales the
body to fit the one-voxel separation. World state and the ordered trace did not
change.

**2026-08-21, P9 body-terrain composition.** The universal two-high movement
column is now only the compatibility wrapper. `WalkerShape` derives a turning
cross-section from each live `BodyDocument::aabb`: one primitive mass segment
maps to one Ground voxel, height remains exact at that scale, and the shorter
horizontal axis lets an elongated body align with a passage until facing
becomes authoritative. Player movement, near pursuit, remembered routing,
wander, graph-to-Ground realization, founders, and births all use the derived
shape.

The resulting behavior needs no hunter variant. At the same generated seed-0
threshold, compact and broad consumers both select `Pursue`; the compact body
descends while the broad body is diverted onto the roof. Both branches replay
identically. Live anatomy owns the answer: adding a broad part widens the
cross-section, severing it narrows the body again, and incorporation that would
expand a body through its current tunnel rolls the whole meal transaction back
as `NoRoom`. `body_clearance_receipt` keeps the two pursuit branches visible.

The deliberate approximation is now explicit: this is a turning
cross-section, not oriented collision. Facing, gait, and deformability can
refine it later without adding another body-size field or changing Ground
authority.

**2026-08-21, P10 body-perception composition.** `spot_for` now derives the
terrain ray's endpoints from both live `WalkerShape`s. A baseline body keeps
the established one-voxel head offset; anatomy taller than the old walker
lifts its top-centre sight point while the stance-to-stance horizon remains
bounded. The ecology tick's temporary living and carrion target views carry
those derived shapes, so feeding, fauna policy, lost-sight acquisition, and
scavenging all ask Ground about the bodies that are actually present.

At generated seed 0, the same observer stance `[-15, 15, -16]`, prey stance
`[-10, 16, -16]`, terrain, range, and default fauna policy split on one live
part. The compact hunter's sight point `[-15, 16, -16]` is occluded and it
records no target decision. Adding a narrow vertical part keeps passage width
unchanged, raises the sight point to `[-15, 17, -16]`, clears the ridge, and
selects `Pursue`. Independent copies replay to the same hashes;
`body_sight_receipt` exposes the pair.

This is body-height sight, not a claim that the top of every critter is an
eye. `Process::Sense` already exists, but Ground has no derived location for a
living sensor part yet. A later composition can replace the top-centre fallback
with sensor-part positions without changing sight authority or storing another
perception profile.

## 4. Wing checks

During G1, before the brick container's shape hardens: read it against
Paredros's settlement-edit pressure (landscape V3 wording: one revisioned
snapshot, only affected products update, stale jobs rejected) and against
Isometry's bake reads. The shared organ serves three vessels or it is
Mesocosm-local and says so. Extraction to a permissive crate follows the
standing rule: after two real consumers, never declared in advance.

## 5. Stop rules

- No portals. If crossing a boundary ever reads as a scene transition, the
  design is wrong, not the tuning.
- No second authority. Caches, meshes, SDFs, occupancy planes, and DAGs
  never become world truth.
- No camera, render LOD, clipmap, or cache-residency decision may select the
  authoritative simulation representation.
- No cohort execution claim lands on projection-only evidence. It needs an
  exact zero-tick round trip and a measured evolution envelope against the
  all-individual reference.
- No chunk machinery by availability; admit acceleration structures by
  trace against real mutation workloads (V4 discipline).
- No graph ships without its distinctness receipts.
- Compile-only wasm evidence is not browser support.
- A component proven alone is weak evidence; joins are the unit of progress.
- Prefer another causal join over declaring the current encounter finished.

## Findings

- 2026-09-01, **the shipped cohort is a projection, not an execution tier.**
  `step_inner` advances the individual organism roster, then calls
  `cohort::from_organisms` to populate conservation tallies. `Cohort` carries
  count, biomass, energy, and summed age, but not enough state to execute
  lifecycle, reserves, process, relationship, or evidence semantics. S3 must
  therefore promote a second canonical representation deliberately rather
  than treating the current summary as one.
- 2026-08-07, **three directions adopted from the bonsai reading**
  (landscape §8.3 carries the donor row and the unverified-claim caveat):
  1. **The relief lab.** Worldgen tuning becomes an instrument: live seed
     and parameter twiddling over `grown` + `Ground`, re-rendered through
     the tracer per keystroke. Bonsai buys its "voxel shadertoy" loop by
     moving generation to the GPU; we get the same loop keeping authority
     on the CPU, because authoritative gen is deliberately coarse (65²
     relief, ~400 bricks, milliseconds). G0's distinctness receipts
     become feelable, not just assertable. Instrument for G0/G2, not a
     gate.
  2. **The presentation-amplification tier.** The terrain twin of the
     ambience tier, same constitutional line: GPU-side micro-detail
     (micro-relief within faces, scatter, texture variation) derived
     from authoritative bricks, **never entering the replay hash**,
     enhanced capability profile only. The grade already amplifies
     colour; this extends the same contract to apparent geometry.
     Bonsai's "second-stage decoration reading terrain derivatives" is
     the worked example.
  3. **Sim/render LOD: shared facts, separate events.** *Corrected by
     audit, 2026-08-07: the first form of this direction ("one line
     serving both, demotion as the same event") was wrong.* Simulation
     tier depends on the authoritative recorded focus and its
     transitions; render LOD may depend on a local camera, and under
     one-state-N-windows there are many cameras and one authority. The
     two consumers share distance and region *facts* (hops, place
     membership) and nothing else. Far places may still collapse to
     hulls or mips for rendering, per viewer, without touching tier
     state.


- 2026-08-06, **renderling wgpu-29 port receipt** (fork at
  `Code/crates/renderling` + `Code/crates/crabslab`): the 26→29 bump is
  mechanical. ~277 initial errors were mostly a two-wgpu split (craballoc
  0.3.1 pins wgpu 26; forked at the `v0.6.6` tag + three edits: wgpu pin,
  `PollType::Wait` fields, `pub fn len`). Renderling itself took a scripted
  pass (descriptor field renames, `Option`-wrapped depth state,
  `bind_group_layouts` Option-wrapping, `MipmapFilterMode`,
  `CurrentSurfaceTexture`, `experimental_features`) plus hand fixes. Lib and
  renderling-ui check clean; test modules pass in isolation including image
  goldens. Open: a cross-test device-teardown leak
  (VUID-vkDestroyDevice-05137, OOM accumulation over 95 sequential
  device creations) to chase before the suite is a receipt; upstream's
  suite also assumes parallel per-test devices, which our no-concurrent-
  test rule already forbids. Next gate: `Context::new` on netrender's
  `WgpuHandles` with a composed external texture and a headless golden.
- 2026-08-06, **device-unity probe PASSED**. One instance/adapter/device/
  queue (netrender `WgpuHandles`, feature union `REQUIRED_FEATURES` +
  renderling's four, intersected with the adapter). Renderling
  `Context::new(RenderTarget::from(texture), ...)` rendered a three-
  triangle ortho stage into an `Rgba8UnormSrgb` texture on the shared
  device; netrender composed it at scene-op boundary 0 under vello chrome
  via `ExternalTextureComposite` and presented a master. Headless readback:
  72 distinct colors, full coverage, spans recorded (vello_render 5.1ms,
  master_compose 48µs, tenant render outside netrender's spans). Zero
  copies, zero validation errors. Cohesion contract clauses 1 and 2 hold
  for renderling as a tenant. Probe: scratchpad `rlprobe`; receipt PNG
  captured. Renderling's mesh-tenant candidacy is now **proven at the
  seam**; remaining before the seat is confirmed: the device-teardown leak
  chase, a wing-shaped scene (voxel mesh body via `mesocosm-mesh`), and a
  browser receipt per D0 discipline.
- 2026-08-06, **leak found, fixed, suite green: 95/95.** Bisection
  (context-only survives 120 cycles; context+stage OOMs) plus wgpu's
  alive-resource report (exactly 8 buffers leaked per Stage create/drop,
  zero textures) attributed the teardown leak to a **self-referential Arc
  cycle in craballoc's `SlabBuffer`**: the allocator stores a `SlabBuffer`
  inside the `Arc<RwLock<Option<SlabBuffer>>>` slot that the stored copy's
  own `source_slab_buffer` field points back at. Fork fix: the back-pointer
  is now `Weak` (allocator owns the strong slot; handles upgrade while it
  lives; a dead allocator means no newer buffer can exist). Receipts: stage
  churn now leaks zero buffers over 24 cycles; the long-lived frame loop
  was already clean (flat counts over 600 frames); renderling's full suite
  passes 95/95 single-threaded on wgpu 29. Upstream-worthy fix; the fork
  carries it for now.
- 2026-08-06, **wing-shaped scene through renderling**. A real body grown
  by play (`World::new(2024, 80)`, the fixture's eat loop, 10 living
  parts), greedy-meshed by `mesocosm-mesh`, triangulated by
  `mesocosm-render::build_vertices`, rendered by renderling (perspective
  camera, per-face normals) on netrender's device, composed under vello
  chrome. 120 triangles, every vertex tracing to a part with provenance;
  receipt PNG captured. The pipeline holds end to end with no hand-placed
  geometry. The image also re-demonstrates the known appearance gap: solid
  same-tag volumes read as one blocky mass, which is the axial-plan and
  V2-capsule work's territory, not the tenant's. Remaining for the seat:
  the browser receipt (D0 discipline).

## Progress

- 2026-09-01: reconciled the substrate record with the scale and playable
  ecology plans. The current Near/Far flag selects execution detail over an
  individual roster; future individual/cohort materialization is now stated
  as a gated target. Macro trophic state remains derived, and simulation
  fidelity is explicitly independent of per-view render residency.
- 2026-08-05: plan founded from the engine rumination. Fundamentals
  rulings recorded in §0 and amended into the founding record, `CLAUDE.md`,
  and the landscape doc the same session. V1 and V2 landed 2026-08-04
  (landscape §8.6), so G2 rides a proven browser harness and real bodies.
- 2026-08-06: rulings 9-11 added (reference targets, three-tier physics,
  renderer tenancy). Mesocosm camera amended to pulled back (ruling 4);
  G2 and G3 reworded to match. Renderling fork scout begun (wgpu 26→29
  bump feasibility).
- 2026-08-06, later: renderling tenant proven (device unity, leak fix,
  95/95, wing-shaped scene; see Findings). **G0 constructor landed**:
  `Places::grown` with full distinctness, congruence, and connectivity
  receipts; genesis adoption deferred to its owner. Worlds now differ.
- 2026-08-06, later still: **G1 container landed** (`Ground`): brick
  truth raised from the relief, roofed burrows under nests, carve with
  revision + dirty discipline, occupancy and sight queries, and the
  mesh-projection receipt through `mesh_volume`. World wiring (carve
  intent, snapshot hash, genesis) rides the same deferred swap as G0.
- 2026-08-06, end of day: **G2 probe receipt** (tracer: ~1900/~1420 fps
  Vulkan/GL, pixel-identical, lifecycle in pixels) and **G3 core landed**
  (near-tier movement, tiers with hysteresis, the Hunter, the chase
  receipt on real ground, 300 hunters at ~62µs/tick). All four gates now
  have landed cores or probe receipts; G4's burrow run is composition
  work plus the deferred world adoption.
- 2026-08-06, **G2 tracer probe receipt** (scratchpad `rlprobe`, bin
  `tracer`). Fragment-only brick-map DDA over the real seed-4242
  `Ground`: pointers in an R32Uint 3D texture (16×4×16), materials in an
  R8Uint 3D atlas (128²×64, 383 bricks), one vec4-packed uniform, no
  storage buffers, no compute, device requested at
  `downlevel_webgl2_defaults`. Numbers, reported as found: **~1900 fps on
  Vulkan, ~1420 fps on wgpu-GL at 960×540** with a 420-step budget, and
  the two backends render **pixel-identical** captures (1 px in 518,400
  beyond tolerance 2). The edit lifecycle ran in pixels: a 72-voxel bore
  carved in ~18µs, 8 dirty bricks re-uploaded (4,096 bytes) in ~10ms
  (first-call overhead included), interior then rendered with a rock
  ceiling and correct occlusion out the opening. Findings along the way:
  hit-distance-in-alpha washes PNG receipts (force opaque on capture);
  nest chambers are too small to photograph from inside, so the interior
  receipt is a carved bore, which is the better lifecycle demo anyway.
  Remaining for G2 proper: port into the lens's retained pattern
  (`MapRevision`-style uploads), grade/souls pass, SDF body compositing,
  netrender frame entry, browser receipt.
- 2026-08-07: bonsai reading adopted as three directions (relief lab,
  presentation-amplification tier, sim/render LOD unification); donor row
  and caveat in landscape §8.3.
- **2026-08-08: G0/G1 world adoption landed.** Genesis grows
  (`Places::grown(seed ^ PLACE_SALT, ...)` reuses the exact site-draw
  sequence the old scatter consumed, so the partition is bit-identical
  and only links, relief, and ground are new); `World` owns a serialized
  `Ground` inside the replay hash; `Intent::Carve` is an ordered intent
  with anatomy reach legality, an `Outcome::Carved { at, removed }`, and
  an `Event::Carved` in history (carving air is not an event). Receipts:
  carve replays to identical hashes on a twin, survives snapshot,
  refuses beyond reach; the full core suite is green and strict clippy
  clean. Finding: grown links are better-connected than the lattice
  (the shipped 3x3 enclosure's diameter is exactly 2, the demote
  threshold), so the tier receipt now finds a maximally distant pair
  rather than assuming corners; a larger PLACE_SIDE would widen the far
  tier and is a world-size question, flagged, not silently retuned.
- 2026-08-14: G3 world ingress and autonomous embodiment landed, followed by
  the grounded 300-founder scale receipt. Remaining on the chain: G2
  integration and G4 composition.
- 2026-08-14: **G2 complete.** The retained Ground-to-DDA core now composes
  nearer SDF bodies, enters netrender's external-texture seam, renders headed
  through Browser WebGPU, passes wgpu-GL under the WebGL2-class limit profile,
  and carries synchronized 1080p Vulkan and GL tracer measurements. G4 is the
  remaining composed game-pressure gate.
