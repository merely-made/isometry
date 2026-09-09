# Engine and Render Lane: Landscape

**Status: historical research, updated 2026-08-04.** Candidate labels in
this document do not override the engine composition ruling in
[2026-08-14_resident_views_composition_plan.md](2026-08-14_resident_views_composition_plan.md)
or its
[2026-08-18 critical review](2026-08-18_engine_ecology_rulings_and_review.md).
This document was rewritten after a
review found two systematic faults in the first draft: it counted an
ingredient shelf as a nearly finished engine, and it let "engine" mean two
different things in one document. Both are fixed below. The body-pipeline decision
and its probe are history: the plan was archived 2026-08-07
(`archive_docs/2026-08-07/`), with R4 (the extraction review) transferred
to the Paredros execution plan, firing after S3.

External facts were re-verified on 2026-07-30 against crates.io and GitHub.
Bones, Renderling, wgpu, and the voxel-engine field were refreshed on
2026-08-04. Version numbers move; recheck before committing.

## Decision index (2026-08-07, per audit: decisions split from research)

The live decisions, so nobody re-reads nine hundred lines of history to
find them. Everything not listed here is research record.

- **Vello 0.9 is the sole shipping rasterizer**; `vello_hybrid` is a
  credible future second backend, not a migration
  (netrender-notes/2026-08-04_rasterizer_backend_seam.md).
- **Tenancy follows the §8.9 cohesion contract**: one device, one frame,
  declared capability profiles, glam boundary, receipts, typed layouts.
- **Renderling is the lead mesh-tenant candidate** on receipts (§1 table:
  wgpu-29 fork green 95/95, device unity proven, wing-shaped scene
  rendered); kiss3d is a donor; Avian rejected; nexus ambience-only.
- **V0, V1, V2 are landed with receipts** (§8.6); D0 (headed WebGL) is
  the open render gate; V3/V4 wait on their consumers.
- **Capability profiles** (§8.5): raster baseline must run downlevel;
  storage/compute tenants are WebGPU-enhanced only.
- **Presentation-amplification tier** and the **relief lab** are adopted
  directions (place-graph plan Findings, 2026-08-07); sim-tier/render-LOD
  share facts, never events (audit correction).
- **Camera rulings live in the place-graph plan §0**, not here.
- Stop rules §8.7 and the donor ledger §8.3 remain binding.

---

## 0. What is actually being chosen

Three vessels with three different presentation needs. They do not have to
share a renderer — see §5 for what they *must* share.

Working dimensionality. **The Mesocosm row was ruled 2026-08-05**: the pivot
camera stands (worldgen x Barony, first person) and volumetric world truth
is permitted; see the
[place-graph engine plan](2026-08-05_place_graph_engine_plan.md) §0.
Verticality is a *simulation affordance*: the third axis must be
mechanically legible (wings escape, canopy holds resources, burrows hide),
and 3D rendering is not the only projection that can make it so. The other
two rows remain proposals:

| Vessel | Proposed | Notes |
| ------ | -------- | ----- |
| Mesocosm | Pulled-back embodied camera over volumetric truth (**amended 2026-08-06**) | Rain World-style; supersedes the 08-05 Barony-march camera while keeping volumetric truth. First person survives in agency. References: Rain World, Voxatron, Caves of Qud. |
| Paredros | First-person/close camera, 3D-lite (**references ruled 2026-08-06**) | Barony, Delver, Gotcha Force, RimWorld. Delver-class ceiling: many simple animated bodies, not fidelity. Primary consumer of the brick tracer. |
| Isometry | 3D, distant camera | **Conflicts with a standing ruling.** See below. References: Foundry, Larian/Owlcat, Wildermyth. |

**Camera is not person.** The first draft wrote "Paredros: 3D, first person",
which collides two vocabularies. The wing's person grammar describes *agency*
— Paredros is second person because companions are peers you address rather
than units you command. That is orthogonal to where the camera sits. A
second-person game can use a first-person camera; saying "first person" about
Paredros invites exactly the drift the person grammar exists to detect. Say
**camera distance** when discussing renderers, and **person** only when
discussing agency.

**Isometry caveat, flagged rather than adopted.** Isometry's `CLAUDE.md` says
camera freedom is explicitly not a near-term rendering task: the locked
isometric 2D lens is the shipped one, later 2.5D/3D modes are allowed because
voxel source models dissolve the facing-art explosion, *but they need their
own plan and render lane*. Isometry also renders through genet's DOM today, so
a 3D lens is a different renderer rather than a camera change. Moving Isometry
to 3D is a real scope change to a shipping project and belongs in a plan in
that repo, not as a side effect of this wing's choices.

If Mesocosm lands at 2.5D, the wing's hardest renderer is Paredros, which
inverts the founding record's assumption that vessel 1 is the render pressure
vessel.

---

## 1. Pure-Rust engines

Verified 2026-07-30.

| Engine | Version (registry) | Shape | License | Posture |
| ------ | ------------------ | ----- | ------- | ------- |
| **Bevy** | **0.19.0** (2026-06-19) | Data-driven ECS, 2D + 3D, ~44k stars, large plugin ecosystem | MIT/Apache | Too large to fork; the idiom is plugins. **Cost to book: Bevy documents frequent breaking releases**, and two minor versions landed in the months around this survey. |
| **Fyrox** | **1.0.1** (2026-03-28) | 2D/3D engine, **the only Rust engine shipping a real scene editor** | MIT | Forkable in principle. But its current renderer is `fyrox-graphics-gl`, which makes shared-device composition with genet a poor fit. Treat as a possible **Paredros** engine, not a reusable stack organ. |
| **ggez** | **0.10.0** (2026-06-03) | 2D framework, LÖVE-shaped; now explicitly wgpu-based with event, timing, resource, and audio defaults | MIT | An alternative *host*, not a missing component. |
| **Bones** | `bones_framework` 0.4.0 on crates.io (2024-09-12) — **but git-active, pushed 2026-04-24**, 305 stars, not archived | **Renderer-neutral game logic**: deterministic ECS, snapshots, asset server, schema reflection, Piccolo (Lua) integration. Bevy renderer optional | MIT/Apache | **The most interesting row.** Addresses the missing middle (§2) rather than the renderer. Piccolo is already Isometry's scripting engine, which is an uncanny adjacency. |
| **Renderling** | 0.4.9 on crates.io (2024-09-20) — **git-active, pushed 2026-07-19**, 238 stars | GPU-driven wgpu + rust-gpu renderer: PBR, glTF scene machinery, headless image tests | MIT/Apache | **Promoted 2026-08-06: lead mesh-tenant candidate**, pending its fork probe. Caller-owned `Context::new(adapter, device, queue, target)` is the only candidate satisfying device unity unmodified. Fork = wgpu/naga 26→29 only; SPIR-V ships precompiled, spirv-std pin stays. rust-gpu welcomed by ruling. WebGPU-enhanced profile only (storage-buffer slabs). |
| **kiss3d** | 0.45.1 (Dimforge revival 2026-01) | wgpu 29, glam (left nalgebra 0.39; parry removed 0.41), glTF/skinning/animation, PBR, headless OffscreenSurface, 2D suite, AOV outputs incl. per-object segmentation | Apache-2.0 | **Donor, ruled 2026-08-06.** Verified: no external-device path (all seven constructors internal-only; `CanvasSetup` = vsync + AA), so device unity requires surgery renderling doesn't need, and its scene scope duplicates renderling's. Harvest: AOV segmentation ids as an Isometry sprite-bake harness; 2D GI ideas. |
| **Avian** | 0.7 (avian2d/avian3d, targets Bevy 0.19) | ECS physics, parry collision underneath | MIT/Apache | **Rejected 2026-08-06.** "Made with Bevy, for Bevy" is structural; adopting it without Bevy drags a runtime for a math-seam cosmetic; no determinism story. |
| **Nexus** | git, heavy development ("still missing many features") | **2D and 3D** GPU rigid-body dynamics via rust-gpu/cargo-gpu, WebGPU. Colliders (box, ball, convex, trimesh, heightfield), joints (ball, fixed, prismatic, revolute), contact resolution; the project's own framing is "rapier on the GPU". Documents a wasm32 browser target. No fluids/deformables/particles today despite the multiphysics name | MIT OR Apache-2.0 | **Ambience tier only, ruled 2026-08-06** (place-graph plan §0.10): GPU float ordering varies by hardware, so it is constitutionally barred from outcome-bearing facts. Browser support ragged. Enters only after the renderling seam proves out and its own external-device audit passes. **Corrected 2026-08-13**: this row previously said "GPU rigid-body compute" with no dimensionality and Apache-2.0 alone, which led a conatus analysis to dismiss Nexus as 3D-only. A 2D variant exists and is the same shape as seiche's rapier2d substrate. |
| **Macroquad** | — | Minimal 2D | MIT/Apache | **Ruled out.** [RUSTSEC-2025-0035](https://rustsec.org/advisories/RUSTSEC-2025-0035.html): multiple soundness issues, unprincipled mutable statics enabling use-after-free from safe code, **all versions affected, no patched release**, and the advisory notes fixing them is not treated as a priority. |
| Ambient | — | Rust/WASM multiplayer engine | — | **Paused indefinitely.** An architectural *donor* to read, not a candidate to adopt. |
| Amethyst, Piston | — | Legacy | — | Dead. Amethyst's ECS lineage fed Bevy. |
| rend3, Nannou, Blue Engine, Tetra | — | Renderer / creative-coding / niche 2D | — | Superseded here by Renderling, or not a fit. |

**A note on registry-stale, git-active projects.** Bones and Renderling both
look abandoned on crates.io and are not: their GitHub repositories saw pushes
in April and July 2026 respectively. This stack already has the convention for
that situation — the cambium family is git-only by ruling, tracked by branch
rather than pinned — so a stale registry version is not disqualifying here the
way it would be elsewhere. It *is* a maintenance signal worth weighing.

---

## 2. The correction: the shelf is not an engine

The first draft claimed "eleven of thirteen components are already owned" and
concluded the stack owns most of a game engine's parts. That is
inventory-true and architecture-misleading, and the corrected claim is:

> The stack already owns most of the **platform-facing boundary** and several
> important simulation primitives. **The missing middle is a coherent game
> runtime.**

That is still a strong position, and worth stating precisely.

### What genuinely exists — a credible host skeleton

- winit window, input, device, and presentation ownership (`cambium-winit`, genet's winit host)
- a real 2D renderer with external-texture composition (`netrender`, vello backend shipped)
- native UI, layout, and text (`cambium`, `sprigging`, `genet-layout`, `parley`)
- a host-neutral actor boundary (`armillary`)
- audio proven in a real interactive genet application (Firewheel, via Hocket)
- persistence and append-only history primitives (`muniment`, `codicil`)
- rapier2d already in the tree (`seiche`)
- field algebra and evaluation (`numen`, `quint`)
- voxel recipes and sprite baking (`isometry-voxel`)
- two working application patterns to copy (Hocket; Isometry's pure-core / views / native-host split)

### What is missing, and is load-bearing

Not one of these is supplied by the shelf:

- fixed-timestep simulation and clock policy
- an authoritative gameplay world or ECS
- snapshot, checkpoint, and deterministic replay lifecycle
- input actions, rebinding, controllers, device churn
- asset dependency graph, hot reload, content addressing
- scene, prefab, and level representation
- camera, animation, particles, lighting, materials
- navigation and spatial queries above raw collision
- game audio mixing, emitters, streaming, spatialization
- diagnostics, inspection, simulation stepping
- packaging and content-build tooling

And three specific over-claims from the first draft, corrected:

- **`armillary` is an actor harness, not a game scheduler.** It does not
  supply a fixed timestep or a simulation clock policy.
- **`codicil` is a linear replay primitive, not a game deed log.** A typed
  deed log with schema evolution, indexes, checkpoints, and forks is work on
  top of it, and Law A's `(context, chosen, foregone, cause-link)` record is
  exactly that work.
- **`muniment` ships an in-memory backend and expects the host to supply
  durable storage.** Isometry supplies redb; a game must supply its own.
- **Firewheel is proven for Hocket's audio graph**, which is not the same as
  positional game audio for Paredros.
- **`numen`/`quint` are field mathematics**, not ecological scheduling or AI.

### The opportunity this reveals

Better than the first draft's version. Mesocosm can be the consumer that
extracts a small reusable **game runtime layer** sitting between the existing
host boundary and a game's rules — fixed step, input intents, snapshot and
replay, asset graph — while Paredros stays free to choose a heavier 3D engine
later. That is the stack's normal extraction pattern applied one layer up.

---

## 3. Approaches worth probing

1. **Bevy as-is.** Fastest to a playable M0; accept that it owns the loop, and
   book the breaking-release cadence as a real cost.
2. **Fyrox for Paredros specifically.** Gets an editor. Its GL renderer makes
   it a poor genet-composition citizen, so treat it as a self-contained host
   for the heavy-3D vessel rather than a shared organ.
3. **Custom loop, 2.5D, on the existing host skeleton.** `winit` +
   `netrender`/vello + `rapier2d`, with bodies rendered to a shared wgpu
   texture that netrender composites and cambium decorates. Maximum reuse,
   minimum new rendering, and it fits Mesocosm's proposed dimensionality.
4. **Custom loop, 3D.** `winit` + `wgpu` + a mesher + `rapier3d`, with
   **Renderling** as the probe target rather than a from-scratch renderer.
5. **Bones as the missing middle.** Renderer-neutral: it could supply the
   deterministic ECS, snapshots, and asset server *inside* lane 3, which is
   the layer that is actually undecided.
6. **Split the bet.** Mesocosm custom-2.5D, Paredros on a heavier engine
   later. Permitted by §5.

The architecture lane 3 implies, which the review sketched and I endorse:

```text
genet / winit kernel  (device, window, input, presentation)
    |-- cambium UI, inspection, chrome
    |-- mesocosm fixed-step world
    |     |-- rapier2d bodies
    |     |-- ecology + lineage systems
    |     `-- numen/quint fields
    `-- body renderer -> shared wgpu texture -> netrender composite
```

**The hidden gap in that diagram is the body renderer**, and it is the whole
subject of the plan doc. `isometry-voxel` bakes appearances; it has never
shown that an incorporated part can attach *during play*, acquire collision
and mass, rotate correctly, move the center of balance, and stay visually
legible. "Voxel bodies still work in 2.5D" is a hypothesis, not a finding.

---

## 4. How to decide

By probe, with one correction that matters: **both hosts must consume the same
`mesocosm-core`**, or the experiment compares two implementations of the game
instead of two hosts.

```text
mesocosm-core          (deterministic rules, traits, metabolism, lineage,
                        input intents — no rendering, no host)
    |-- mesocosm-genet  (custom 2.5D lane)
    `-- mesocosm-bevy   (engine lane)
```

Same seed, same recorded input trace, and record:

- final simulation-state hash
- save/reload and replay equivalence
- fixed-tick behaviour under uneven frame delivery
- input-to-present latency and frame pacing
- **whether one new body part can be attached and metabolized mid-run**
- rapier reconciliation and collider rebuilding cost
- lines of adapter code, and any duplicated ownership
- whether the host creates a second wgpu device
- headed inspection and debugging quality
- asset and content iteration behaviour

"Does the verb feel right?" stays the product judgment. These receipts say
whether a lane is helping or fighting it.

A **third, smaller experiment belongs inside the genet lane**: `mesocosm-core`
storage versus Bones ECS. Bevy-versus-genet mostly tests host ownership;
core-versus-Bones tests the layer that is genuinely undecided.

---

## 5. What must stay shared, restated

The first draft said "renderer choice is free" and then listed a shared
*world model*, which was too strong in one direction and too vague in the
other. Corrected, with the sharing rule made explicit per layer:

| Layer | Sharing rule |
| ----- | ------------ |
| World identity and fact substrate | **Shared** |
| Identity, provenance, deed vocabulary | **Shared** |
| Portable critter profile | **Extracted from two consumers** |
| Game simulation runtime | May share crates, may differ |
| Host shell and event loop | Per vessel |
| Renderer and camera | Per vessel |

Two consequences:

- **"Shared world model" becomes "shared world identity and fact substrate."**
  Mesocosm's live ecology, Paredros' settlement, and Isometry's campaign state
  will never be one in-memory model. They append compatible facts to one
  world.
- **`isometry-voxel` recipes are no longer canonical.** They are an excellent
  first projection codec and probably Isometry's projection, but making them
  *the* appearance format lets today's renderer leak into the substrate. The
  portable artifact carries **body topology, incorporated-part provenance,
  loud inherited signatures, and optional projection recipes**; each vessel
  derives its own presentation.

### The anti-Spore law, restated

The founding record's one-substrate rule and this document's "renderers need
not be shared" appear to conflict. They do not, once the law is stated at the
right altitude:

> **A vessel must not create a private replacement for shared world identity,
> provenance, and causal history.**

A vessel may absolutely have its own renderer, event loop, ECS, or physics
dimensionality. What hollowed Spore was five stages that shared no world, not
five stages that shared no renderer.

---

## 6. Findings

- **2026-07-30**: the `wgpu-*` siblings are a web-embedding family
  (`wgpu-graft` = Servo texture grafting, `wgpu-weld` = CEF accelerated OSR,
  `wgpu-scry` = system-webview capture). None renders geometry, so none
  shortens this path. `netrender` and vello apply, but to the 2D lane only.
- **2026-07-30**: `seiche` wraps rapier2d 0.33, so the physics family is
  already in the tree and understood.
- **2026-07-30, verified versions**: Bevy 0.19.0 (2026-06-19), Fyrox 1.0.1
  (2026-03-28), ggez 0.10.0 (2026-06-03). Macroquad carries an unpatched
  soundness advisory affecting all versions. Bones (`bones_framework` 0.4.0,
  registry 2024-09-12) and Renderling (0.4.9, registry 2024-09-20) are both
  **registry-stale but git-active** (pushes 2026-04-24 and 2026-07-19), which
  this stack already has a convention for.
- **2026-07-30**: `block-mesh` remains the meshing reference —
  `visible_block_faces` ≈ 40M quads/s on one core, `greedy_quads` ≈ ⅓ the
  triangles at ~3× the time.

---

## 7. The person ladder and the hybrid lens (research round 2026-08-03)

Prompted by the pivot: the custom prism pipeline dies, the game wants a
Barony-flavoured first-person view of a No Man's Sky-flavoured biosphere, the
player should feel like slugcat, and Mark asked whether the stack's 2D
strengths can carry it, and whether sprites can be more than billboards,
cheap. Research, no decision; probes decide.

### The contradiction dissolves into three dimensionalities

"Sacrifice 3D" and "height and gravity should count for wings" stop
conflicting once simulation, world rendering, and body rendering are allowed
different answers:

- **Simulation: 2.5D columns.** Terrain is an integer heightfield per
  column plus an air band above it. Height and gravity are real rules
  (staying aloft costs energy; wings occupy the band; falling is a thing),
  but nothing stacks, which is what places already ruled ("height is not a
  place") and what worldgen, crowding, and dispersal already assume.
- **World rendering: raycast the heightfield.** The Voxel Space lineage
  (Comanche 1992): each screen column marches a ray across a heightmap and
  fills vertical spans. The modern GPU form is one fullscreen shader (a
  hobby implementation reports ~500fps), and the engine's entire input is
  **two images: a heightmap and a colormap**. That is the load-bearing fact
  for this stack: the vello lane can *paint* those images. Worldgen
  generates biome maps per place; the renderer just marches them. Rolling
  organic terrain, fog, no meshes, no scene graph.
- **Bodies: the parts graph, not a billboard.** Three candidate forms, in
  ascending ambition, all driven by the same anatomy:
  1. **Per-part cards**: one small quad per part, placed by its attachment
     frame in 3D, texture painted by vello, chains driven host-side by
     seiche. Parts occlude and parallax as the body turns; all art stays
     2D. The Rain World technique is exactly this (2D chains, IK feet)
     lifted into a 3D placement.
  2. **Sprite stacking**: a part as a stack of horizontal slices (the
     MagicaVoxel-to-slices pipeline; SpriteStack, many GameMaker/PICO-8
     games). Genuinely 3D-reading rotation from 2D draws, thousands of
     sprites cheap. Honest limit: it reads from above-ish and degrades at
     eye level, so it serves Paredros' over-shoulder and Isometry's
     pulled-back distances better than Mesocosm's first person.
  3. **Skeleton-driven SDF capsules**: model a critter as smooth-unioned
     capsules along its part chains and raymarch it (iq's canon; the
     technique behind Claybook-style clay). Squash, stretch, breathing,
     and morphing are math on the field rather than rig work, and
     **smooth-union is kleptoplasty made visible**: a grafted part blends
     into the body exactly the way the mechanic means it. Cost: a
     fullscreen march is real GPU work and an art-direction commitment
     (the clay look), and debugging lives in shaders.

### The unified-raymarch candidate

The three lanes above share one loop. A single fullscreen pass can march
terrain heightfield and critter capsule-fields together: the same ray, the
same fog, one engine measured in hundreds of lines of WGSL rather than a
renderer. Everything it consumes is 2D data the stack generates (height and
colour maps from vello; capsule chains from the parts graph via seiche).
The camera ladder becomes a parameter: Mesocosm holds the camera at a
critter's eye, Paredros pulls it over a shoulder, Isometry lifts it to the
scene, and no vessel grows an engine, which is the anti-Spore law satisfied
by construction rather than discipline.

Netrender is the frame owner in this shape: its render graph takes encode
callbacks as tasks, so the march is a `Task` between vello layers, the HUD
composites above, and `mesocosm-render`'s custom pipeline dies into a graph
node rather than being replaced by a second custom pipeline.

### What the stack contributes, named

vello paints every input (terrain maps, part textures, HUD); seiche drives
chains host-side (f32 presentation, integer core untouched); netrender owns
the frame and the composite; genet's cambium lane arrives for the epoch
screen; the sprite-hull tracer turns rendered art into collision hulls. The
raymarch shader is the one genuinely new piece, and it is small enough to
become a shared component (a biosphere lens) if Paredros pulls.

### Probes, with targets

- ~~**Terrain probe**~~ **LANDED 2026-08-03** as `crates/mesocosm-lens`:
  two passes (march + grade), maps synthesised from a `Places` partition
  (one biome per place, golden-angle tints, fbm relief), both souls
  captured along a flight at `testing/mesocosm/16_lens_*.png`. The vista
  shots show rolling multi-biome terrain into banded dithered fog (retro)
  and smooth haze (clay) from the same march. The probe confirms: the
  engine is two small WGSL files, the world is two images, a soul is a
  uniform block, and place borders read as biome borders. CPU map
  synthesis stands in for the vello lane without changing the contract.
  Two findings: the march's step-growth budget silently capped view
  distance at ~155 units until the growth rate matched the far plane (a
  vista rendered as pure sky), and banded fog plus starved palette carries
  most of the retro soul on its own. Dither-in-motion remains a headed
  judgement; three consecutive frames per soul are captured for it.
- ~~**Critter probe**~~ **LANDED 2026-08-03**: a follow-the-leader capsule
  chain (`lens::critter`, the Rain World constraint technique, ~100 lines,
  no physics engine) smooth-unioned into the same march, walking a wander
  over the terrain. Receipts at `testing/mesocosm/17_critter_*.png`: a
  tapered caterpillar-form body cresting a ridge with visible ground
  contact, fully legible under both grades; the retro dither reads as body
  texture rather than noise. **The gate passes: it reads as an animal.**
  Chain findings: spacing must be ~2x segment radius or the body blends
  into a blob; the smooth-min k must be small against the radii; a relax
  term toward the leader's heading is what lets a still body settle
  instead of freezing its wiggle; and turn radii must exceed a body length
  or a loper reads as a coiler. One shader bug worth remembering: seeding
  a smooth-min from a 1e9 sentinel makes `mix(1e9, d, 1.0)` cancel
  catastrophically at f32 and the whole field reads as a hit -- the
  giant-sphere artefact. Seed from the first primitive, never a sentinel.
  Deliberately absent yet: legs, eyes, contact shadows, and the
  parts-graph binding (the probe chain is hand-shaped; the game's chains
  derive from anatomy).
- **Fallback**: if the march disappoints on perf or art, per-part cards on
  the same chains are the retreat, and sprite stacking stays evaluated for
  the two pulled-back vessels.

### Rulings (Mark, 2026-08-03)

1. **The soul question is a styling matter.** The march yields colour,
   depth, and normals; a parametric **grade** stage (internal resolution,
   palette LUT, dither matrix, lighting ramp, fog curve) turns every look
   into a small data block worldgen can emit the way it emits a heightmap.
   Retro and clay are two grade blocks, and everything between them is a
   space. Mark leans retro; the probe renders both.
2. **Lineage grades: yes.** Successful lineages may shift their territory's
   grade, so a region can be felt on entering. Same derived-not-stored rule
   as every look: computed from world state, never in the snapshot.
3. **HUD inside or above the grade is a setting**, because it touches
   accessibility. Not a design fork; both stay cheap.
4. The layering is **march (geometry) → field fx (world-driven, the same
   ScalarField that feeds physics, so fx cannot lie) → grade (the soul)**.
5. Probes proceed, with the dither-in-motion check flagged (Obra Dinn's
   lesson: ordered dither shimmers under camera motion; judge the retro
   grade while flying, not from stills).

---

## 8. From proofs to a netrender-quality voxel presentation family (research round 2026-08-04)

**Status 2026-08-04: V0, V1, and V2 landed.** `mesocosm-lens` now has one
retained encode path for live presentation and capture; that path has entered
a same-device `netrender` frame on native and headed Browser WebGPU; and one
played `BodyDocument` revision now projects through Lens, the headless mesh,
and Isometry's independent reader and sprite baker. V3 remains a Paredros
pull, not the next Mesocosm renderer task. The independent downlevel rasterizer
gate in §8.8 is the only open host-capability proof from this research round.

The question for this round was not "which voxel engine should Mesocosm use?"
It was what the landed proofs would have to become before they deserve the
same confidence as `netrender`: reusable ownership boundaries, retained GPU
resources, native and browser hosts, capability fallbacks, diagnostics,
capture/replay receipts, and clean licensing.

The first answer needs correcting before any architecture is drawn. There is
not one live voxel renderer hiding in the wing. There are three projections
of shared body and world authority:

| Consumer | Projection it has or needs | Pressure it applies |
| --- | --- | --- |
| **Mesocosm** | Heightfield march plus SDF/capsule bodies, then field effects and grade | Frequently changing camera and pose over comparatively stable world maps; native and browser presentation |
| **Paredros** | Close-camera 3D bodies and eventually editable settlement/world geometry | Persistent geometry, chunks, collision, streaming, navigation, destruction, and many independently moving bodies |
| **Isometry** | Deterministic voxel-to-sprite bake | Portable, cacheable, pixel-authoritative interchange rather than a live 3D scene |

One authoritative body may feed all three. Making all three consume one mesh,
one storage layout, or one renderer would move presentation into the substrate
and violate the boundary this document already established. The reusable
thing is therefore a **voxel presentation family**: common snapshot identity,
revision discipline, materials, queries, GPU ownership conventions, and
receipts, with more than one projection strategy.

Vello's place is equally specific. The heightfield/SDF pass is direct wgpu.
Vello paints maps, effects, HUD, and chrome where useful. `netrender` owns or
adopts the device and composes the passes. Voxel geometry does not render
"through Vello"; the voxel and Vello passes are siblings on one device and in
one frame graph.

### 8.1 What the live proofs actually establish

`mesocosm-mesh` is deterministic and headless. A part mesh depends only on its
content-addressed `VolumeRef`, and rigid parts remain separately projectable.
That is already the right authority boundary for a mesh projection.

`mesocosm-render` proves caller-owned wgpu composition, but its live draw path
still flattens every scene into a fresh vertex vector, creates a depth texture,
and creates and uploads a vertex buffer on every call. `mesocosm-genet` calls
`mesh_body` while projecting a scene. The path is a visibility proof, not a
retained renderer.

`mesocosm-lens` proves the newer presentation direction more directly:

- `Lens::with_device` accepts the host's device and queue;
- the march and grade are ordinary render pipelines using sampled textures,
  uniform buffers, and direct draws, with no compute or storage-buffer
  requirement in the baseline;
- seeded map synthesis and capsule posing are deterministic;
- the native test suite passes, the crate compiles for
  `wasm32-unknown-unknown`, and V1 runs headed through Browser WebGPU.

Before V0, its capture-first `render_with` uploaded the complete height,
colour, and palette textures every call; recreated intermediate targets,
uniform buffers, and bind groups; created its own encoder; submitted it; and
blocked on a staging-buffer readback. V0 split **encoding** from **capture**.
One retained lens now records into the caller's encoder and target through
`Lens::encode`; the pixel-returning path is a receipt adapter over that same
entry point.

The earlier Wasm compile was portability evidence only. V1 added the browser
canvas host and a headed Browser WebGPU receipt. That headed run caught one
real target defect that compile-only evidence missed: `std::time::Instant`
panicked in Wasm. The renderer now uses `web_time::Instant` on that target.
The raster baseline therefore has native and browser runtime evidence; a
WebGL2 fallback remains only a possible future profile.

`netrender` supplies the quality comparison, not another voxel algorithm. Its
relevant properties are externally supplied `WgpuHandles`, retained caches,
render-graph callbacks, external-texture composition, scene capture/replay,
GPU readback and image oracles, and `Renderer::last_frame_timings()`. A voxel
lane reaches netrender quality when it carries comparable ownership,
observability, and receipts, even though its rendering algorithms differ.

One phrase from the first research summary also needs tightening. A raymarched
frame is never literally free; it still shades the screen. The correct
invariant is:

> **Unchanged projection inputs cause no remesh, resource creation, full-map
> upload, or readback. Only the render passes remain.**

Camera, grade, or pose changes should write their small buffers. A changed map
region should upload that region. Resize should recreate size-dependent
targets. Everything else is resource churn and should be visible in metrics.

### 8.2 Bones, restored to the comparison

Bones was considered beside Renderling in the first landscape and should not
have disappeared from the voxel round. It answers a different question.

Current `main`, checked 2026-08-04, still makes the distinction clearly:

- `bones_lib` has no renderer or math types. It supplies game/session
  organization over Bones ECS and assets.
- Bones ECS is designed for deterministic iteration, snapshot/restore, and
  schema-reflected access from systems outside Rust.
- `bones_framework` adds 2D-oriented render vocabulary, audio, localization,
  UI, networking, and optional Piccolo scripting.
- the official renderer remains a Bevy 0.11 integration with WebGL2 enabled by
  default. Bones' own documentation says 3D requires a custom integration.
- the repository is MIT OR Apache-2.0, targets stable Rust, and checks native
  and `wasm32-unknown-unknown` in CI. Its published crates remain at 0.4.0 and
  much of the dependency set is from the Bevy 0.11 generation.

The useful correspondence is real: renderer-neutral sessions, snapshots,
reflected schemas, content-addressed asset loading, hot reload, and Piccolo
scripts are all problems this wing cares about. A Bones `Session` even has an
interesting family resemblance to one live game instance.

Adopting Bones now would nevertheless replace working authority rather than
fill a hole. `mesocosm-core` already owns exact integer simulation, snapshots,
ordered intents, rejections, replay hashes, body provenance, epochs, and
cross-game records. The stack also has its own pack/engram direction and its
own bounded Piccolo `ProcessDef` lane. Moving those into a second ECS, asset
server, and scripting host would duplicate semantics before it removed code.

The posture is therefore:

> **Bones is the missing-middle donor and a possible focused probe, not a
> voxel renderer and not a pending Mesocosm migration.**

Reopen a Bones dependency only when two real games need the same missing
renderer-independent service and the existing stack has no owner. A valid
probe must preserve the same simulation receipt and state hash, measure the
adapter and duplicated ownership, compile on native and Wasm, and remove the
losing path. Architectural resemblance alone is not a trigger.

### 8.3 Donor ledger

| Donor | Transferable value | Posture |
| --- | --- | --- |
| [`block-mesh`](https://github.com/bonsairobo/block-mesh-rs) | Small generic mesher boundary, padded neighborhoods, material-aware merge classes | Direct dependency or benchmark candidate; keep the current deterministic mesher as the oracle until traces justify replacement |
| [`fast-surface-nets`](https://docs.rs/fast-surface-nets/latest/fast_surface_nets/) and [Transvoxel](https://transvoxel.org/) | Smooth SDF surfaces and crack-free smooth-terrain LOD | Alternate projections only after a real smooth body or biome requires them |
| [`bevy_voxel_world`](https://github.com/splashdust/bevy_voxel_world) | Procedural base plus sparse edits, remesh events, custom mesh delegates, data resolution separate from mesh resolution | Architecture donor; Bevy ownership prevents direct adoption here |
| [Godot Voxel Tools](https://github.com/Zylann/godot_voxel) | Prioritized task pools, streaming and save lifecycles, collider cost, memory pools, and queue diagnostics | Strongest full reusable-engine reference |
| [Renderling](https://github.com/schell/renderling) | Caller-owned wgpu context, refcounted GPU slabs through `crabslab`/`craballoc`, headless image tests, GPU-first residency | Focused donor. Current `main` is wgpu 26 and still carries rust-gpu/cargo-gpu machinery; Mesocosm is wgpu 29 and does not need its PBR/glTF scene model |
| [Veloren](https://gitlab.com/veloren/veloren) and [Luanti](https://docs.luanti.org/for-engine-devs/basic-data-structures/) | Shipped chunk scheduling, client/server authority, request throttling, unload policy, background mesh work | Study patterns. Their reciprocal licenses also argue against casual code transfer into a permissive core |
| [Sector's Edge](https://github.com/Vercidium/voxel-mesh-generation) and [`stb_voxel_render.h`](https://github.com/nothings/stb/blob/master/stb_voxel_render.h) | Hot remesh latency, reused scratch storage, packed quads, palette/material/AO representation | Algorithm and format donors; benchmark against the real mutation workload |
| [Voxelis](https://github.com/WildPixelGames/voxelis) and [Woxel](https://github.com/NemoInfo/woxel) | SVO-DAG and VDB/HDDA alternatives | Research and benchmark specimens. Highly varied evolving bodies may erase their compression wins |
| [Mosaic](https://github.com/sevenevesai/mosaic) | Revisioned asynchronous jobs, boundary invalidation, latency classes, persistent residency, derived light volumes | Study only. No repository license was found, and its fixed GPU pool can silently refuse work |
| [Bones](https://github.com/fishfolk/bones) | Deterministic sessions, snapshots, schema reflection, assets, Piccolo, networking | Missing-middle donor, orthogonal to rendering |
| [Bonsai](https://github.com/scallyw4g/bonsai) (+ [poof](https://github.com/scallyw4g/poof)) | Hot-reload worldgen loop ("voxel shadertoy"); GPU second-stage terrain decoration reading derivatives; SDF layer-brush editing; profiler-first culture; poof = schema-derived editor UI (livery's TOML-DB codegen is our analog) | **Technique donor, adopted three ways 2026-08-07** (place-graph plan Findings): the relief lab, the presentation-amplification tier, and sim/render LOD unification. Caveat: the whole-world view-distance mechanism is **undisclosed** (no LOD documentation found in README or the author's HN comments); treat that headline as unverified. Ten years solo, WTFPL, 2.0.0-alpha, kept alive by a real consumer game pulling scope |

The data-structure conclusion is deliberately plural. Dense padded arrays are
excellent meshing snapshots. Bitplanes are excellent derived occupancy and
query indexes. SVOs, DAGs, and VDBs win when repetition and sparsity match
their costs. The presentation family must accept more than one provider and
benchmark actual body mutations and Paredros edits. It must not turn any one
acceleration structure into world truth.

### 8.4 Ownership model

Working labels below name responsibilities, not approved crate names:

```text
MPL game authority
    body topology, world state, provenance, intents, revisions
                    |
        immutable projection snapshot
        content keys + revision + dirty regions
                    |
       +------------+-------------+
       |                          |
  lens projection            mesh projection
  height/SDF/grade           greedy/surface/other
       |                          |
       +------------+-------------+
                    |
        caller-owned wgpu device and frame
        retained resources + explicit admission
                    |
       +------------+-------------+
       |                          |
  netrender/Vello chrome       capture/readback
```

The common core may eventually own read-only volume access, channels,
materials, content keys, revisions, dirty regions, query indexes, and
projection receipts. Rendering strategies own their caches. Collision
derives from the same immutable snapshot rather than treating the display
mesh as authority — per the 2026-08-06 three-tier ruling (place-graph plan
§0.10) that means **parry queries plus owned kinematics** at the advisor
tier, with rapier-the-dynamics-engine in reserve for a proven
constraint-dynamics need. Game adapters remain MPL-2.0.

Do not mint permissively licensed engine crates merely because this diagram is
clean. Per the repository license boundary, a library becomes MIT OR
Apache-2.0 only after its reusable boundary is real and a second consumer
proves it. Original game assets remain CC BY-SA 4.0.

### 8.5 Capability profiles

Capability detection chooses an execution path, never a game rule.

| Profile | Required shape | Intended targets |
| --- | --- | --- |
| **Raster baseline** | Sampled 2D textures, uniforms, ordinary render passes and direct draws; CPU scheduling and culling | Native and browser WebGPU with standard Vello; the Lens half is also proven under WebGL2-class limits |
| **Downlevel raster** | The raster baseline plus a vector backend needing no storage buffers or compute | Native GL is probed with `vello_hybrid`; headed Wasm WebGL remains the admission gate |
| **WebGPU enhanced** | Storage buffers, compute work, indirect draws, optional timestamp queries and optional worker-backed CPU jobs | Browsers and native adapters that report the features |
| **Native enhanced** | Real multi-draw-indirect-count, larger resident arenas, Rayon jobs, timestamp instrumentation | Native adapters only |

The baseline must not require Wasm threads. Shared-memory worker builds need
cross-origin isolation and remain an optional second artifact. Browser support
is admitted by a headed browser run, not by `cargo check`.

### 8.6 Gates

#### V0. Retain the landed lens — **LANDED 2026-08-04**

Replace the capture-shaped production API with a render-task boundary that
records march and grade into a caller encoder and target. Retain map textures,
palette, uniform buffers, bind groups, and size-dependent targets. Upload only
changed map regions; camera, pose, and grade changes write only their small
buffers. Keep capture as a readback adapter over the same task.

**Done when:** an unchanged world and grade create no GPU resources and upload
no map bytes across frames; a camera or pose change updates only the relevant
buffer; resize recreates only size-dependent targets; native capture still
matches the existing receipts; and diagnostics report CPU preparation,
uploads, allocations, march, grade, and readback separately.

**Met.** The retained `Lens` owns map and palette textures, uniform buffers,
bind groups, its marched intermediate, and reusable capture resources.
Callers own the command encoder, output target, and submission. Explicit
`MapRevision` plus `MapChange::Region` drives full or rectangular uploads.
The native GPU tests prove:

- a second unchanged frame creates zero resources and uploads zero bytes;
- camera and pose changes each upload only their corresponding uniform;
- a 2 by 2 height-and-colour edit uploads exactly 20 bytes;
- resize recreates size-dependent targets while leaving map uploads at zero;
- march, grade, readback, preparation, upload, and resource counts are
  reported separately.

The compatibility receipt was checked against a clean detached build of the
pre-V0 `HEAD`: all six lope frames are pixel-identical. The older files under
`testing/mesocosm/17_critter_*.png` do not match that clean pre-V0 build, so
they are historical visual references rather than a current regression
oracle. Native tests, examples, strict clippy, and
`wasm32-unknown-unknown` compilation pass.

V0 deliberately did not replace the live Genet scene. Genet projected the
real `BodyDocument` through `mesocosm-render`, while the early Lens receipts
used synthesized heightfields and a sculpted `CritterPose`. V2 has since bound
the real played body through `BodyLensProjection`. Its terrain remains a
synthesized presentation map; deriving that map from the simulated ecology is
a separate world-projection problem and is not smuggled into the body proof.

#### V1. Enter netrender's frame and prove the browser — **LANDED 2026-08-04**

Run the retained lens on netrender's `WgpuHandles` and compose its external
texture into netrender's master at an explicit scene boundary. Add
target-specific browser wgpu setup and a canvas host without creating a
second device. Netrender currently retains submission ownership around its
external-texture seam, so V1 proves one device and one composed frame rather
than claiming one command encoder.

**Done when:** the same serialized world, camera, body pose, and grade render
headed on native and in a browser; both produce screenshot and JSON receipts
kept as generated proof artifacts; the receipt records adapter, backend,
limits, and selected profile; and frame timings are visible beside
netrender's spans.

**Met.** `LensScene` serializes the maps, flight, grade, and pose with
`postcard`. Both hosts decoded the same 329,261-byte scene with digest
`fnv1a64:9d34d7f2688bcf73`, then passed the same wgpu device and queue to the
lens and netrender. The lens rendered to an RGBA external texture; netrender
inserted it at scene boundary zero, redrew Vello chrome over it, and exposed
the composed master for presentation.

The native receipt records Vulkan on an NVIDIA GeForce RTX 4060 Laptop GPU;
the headed browser receipt records `BrowserWebGpu`. Both select the
`raster-baseline` profile and include limits, formats, lens diagnostics,
netrender's timing spans, and dirty-tile counts. On the second frame both
reported zero map bytes, uniform bytes, resource creations, bind-group
rebuilds, readback bytes, and dirty tiles while still running one march and
one grade pass. Native and browser screenshots plus JSON receipts were
captured under the local proof directory. Strict native and Wasm clippy and
the full `mesocosm-lens` target suite pass.

V1 still uses a deterministic host scene. It does not replace Genet's live
`BodyDocument` projection; that remains the V2 boundary.

#### V2. Prove projection plurality — **LANDED 2026-08-04**

Bind the real parts graph into the SDF/capsule lens while preserving the
existing per-part mesh and Isometry flatten/bake projections. Each may simplify
appearance differently, but each points back to the same part addresses and
provenance.

**Done when:** one played body revision can be marched live, meshed headlessly,
and baked for Isometry; changing one part invalidates only the products that
depend on that part or region; and none of the three projection outputs is
needed to decode the body document.

**Met.** `BodyLensProjection` reads the authoritative `BodyDocument` and
simplifies each living voxel part to one capsule. Every capsule retains a
sidecar record of its `PartId`, `VolumeRef`, provenance, and exact dependency
digest. Resolved parent placement participates in that digest, so moving a
parent invalidates its descendants; mass does not, because the Lens does not
read mass. Bodies above the baseline's 96-capsule uniform capacity now return
an explicit admission error rather than being silently truncated.

The native V2 receipt played the real world from three parts through one more
incorporation. The body plan produced a mirrored pair, parts 3 and 4, yielding
a five-part body revision `fnv1a64:1e04e4325650d6c7`. Only parts 3 and 4 changed
in the Lens dependency map and mesh placements. The attributed flattened
profile changed 48 cells, all owned by those same two part addresses. Every
Lens capsule, mesh placement, and profile attribution agreed on part identity
and provenance. The 260-byte `BodyDocument` round-tripped without reading any
projection output.

Isometry then read the emitted 877-byte `mesocosm.body/v0` profile through its
own mirror type and baked a 398 by 164 four-facing sprite strip with 34,372
opaque pixels. Writer and reader receipts agree on profile digest
`fnv1a64:6656da7e804ca21d`. `critter_sprite` now accepts an arbitrary `--body`
input so this crossing is executable rather than tied to the committed
fixture.

The migration surfaced one adjacent defect: `mesh_body` iterated the complete
historical part vector and therefore still drew severed parts. It now consumes
`BodyDocument::living`, while the body record continues to retain the loss.
The focused Mesocosm and Isometry suites cover the corrected boundary.

**Follow-through 2026-08-05:** the axial catalogue had still bypassed this
boundary through a renderer-only `critter::Body::from_plan`. That parallel body
constructor is gone. `mesocosm-core::development` now turns `Recipe + Soma`
into an authoritative `BodyDocument`, and the Lens menagerie projects it
through `BodyLensProjection`. V2 therefore binds a body source, not one growth
method: somatic incorporation and filial development meet at the same parts
graph. The phenotype lifecycle adopted that source on 2026-08-05: world
founders, offspring, returned-chronicle regrowth, and founder previews now use
the same developer. This remains a rendering boundary rather than a renderer
owning biology.

#### V3. Let Paredros pull world services

Only the second live 3D consumer justifies chunk requests, border
invalidation, priority classes, async mesh jobs, persistent geometry arenas,
eviction, collider and navigation derivation, and network edit payloads.

**Done when:** a Paredros settlement edit produces one revisioned snapshot,
updates only affected render, collision, query, and navigation products,
rejects stale jobs, never transmits a GPU mesh as authority, and reports its
resident and queued work. That proof is the extraction trigger for a
permissive shared library.

#### V4. Admit advanced renderers by trace

Compute culling, indirect submission, binary greedy meshing, smooth terrain
LOD, SVO/VDB raymarching, and richer lighting remain optional strategies.

**Done when:** a captured native or browser workload identifies the current
bottleneck, one candidate improves that trace without changing simulation or
interchange semantics, and the baseline remains available on weaker adapters.

### 8.7 Stop rules

- Do not turn Mesocosm into a chunk-meshing engine because most voxel prior art
  was built for Minecraft-shaped worlds. (Narrowed 2026-08-05: volumetric world
  truth is now permitted and planned; the guard that stands is admission by
  trace, never by prior-art availability. See the
  [place-graph engine plan](2026-08-05_place_graph_engine_plan.md).)
- Do not make Paredros inherit the lens if close-camera editing and destruction
  prove that meshes serve it better.
- Do not migrate `mesocosm-core` into Bones because the concepts resemble one
  another. Require a missing service, a second consumer, and a removal proof.
- Do not adopt all of Renderling to obtain its allocator or image tests. Probe
  the focused donor crate or pattern against wgpu 29 first.
- Do not call compile-only Wasm evidence browser support.
- Do not allow caches, meshes, colliders, heightmaps, SDFs, or DAGs to become
  authoritative body or world formats.
- Do not let asynchronous work commit without matching its content key and
  revision.
- Do not silently drop oversized GPU admissions. Return an explicit failure,
  evict deliberately, and expose both in diagnostics.
- Do not copy from Mosaic while its license remains absent.

### 8.8 WebGL2-class downlevel probe receipt (2026-08-04)

Question: does the wing's render stack fit WebGL2-class constraints, and
could `vello_hybrid` (0.1.0, published 2026-07-29, `wgpu ^29.0.3`, matching
the workspace pin) serve as a downlevel 2D rasterizer behind netrender's
seam? Probe ran scratchpad-side; nothing landed in the tree.

Context that makes the seam narrow: netrender hands vello only nine
`SceneOp` variants, lowered through four vello calls (`fill`, `stroke`,
`draw_glyphs`, `push_layer`/`pop_layer`). Masks, blurs, backdrop filters,
and color matrices are netrender's own fragment-shader wgpu passes and
never reach the rasterizer. `vello_hybrid`'s shader set is sampled
textures plus one uniform block; no storage buffers, no compute.

Ran on RTX 4060 under `Limits::downlevel_webgl2_defaults()` (storage
buffers and compute zeroed, max texture 2048), twice: Vulkan, then wgpu's
real GL backend, which exercises naga's WGSL-to-GLSL emission for every
shader involved. Identical results on both:

- **mesocosm-lens march + grade + capsule critter: passes.** Retro and
  clay grades both render correctly restricted. The world pass is
  WebGL2-class as landed.
- **vello_hybrid: passes everything netrender would send it.** Fills,
  strokes, linear gradient, clip layer, all six `SceneBlendMode` mixes
  under SrcOver, opacity layer, blur filter layer, glyphs (Arial via
  glifo, atlas path, caching off). Zero validation errors.
- **Mask layers panic** (`unimplemented!`, scene.rs:734), as documented.
  Irrelevant to the seam: netrender never sends vello a mask.

Still ungated: a headed browser run (ANGLE validation, wasm async init,
no blocking readback). Per §8.5, browser support is admitted by that run
only. The probe retires the *capability* risk, not the browser receipt.

Consequence: an anyrender-style pluggable rasterizer behind the nine-op
seam is viable, and hybrid's known panic surface does not intersect
netrender's usage. Glyph atlas caching is experimental upstream; the
probe rendered with it off.

That is algorithmic viability, not a claim that netrender is backend-neutral
today. Its retained surface state still stores `vello::Scene` and its timing
and error vocabulary still name Vello. A real integration belongs in
netrender: keep `netrender::Scene`, invalidation, filters, external textures,
and presentation common; move lowered tile scenes into backend-owned state;
report the selected rasterizer in receipts; and rename the raster timing span
at that boundary. Mesocosm should only select the resulting capability
profile.

#### D0. Prove the downlevel host

Build netrender with `vello_hybrid`, compile the existing V1 serialized scene
for wgpu's Wasm WebGL backend, and run it headed without blocking readback.
Exercise glyphs, clips, gradients, every supported blend, filters, and the
external Lens texture. Masks remain outside the admitted scene vocabulary.

**Done when:** the browser receipt reports the WebGL backend and hybrid
rasterizer, matches the V1 scene digest, renders visible Lens and chrome
coverage, exposes backend-neutral timing spans, and reports no validation
errors or silent operation drops. This is a netrender gate and does not block
Mesocosm feature work.

### 8.9 Tenancy: the cohesion contract (2026-08-06)

Holistic cohesiveness is the non-negotiable (Mark, 2026-08-06). Composing
the stack from parts and forks is encouraged; every renderer or compute
tenant signs this contract:

1. **One instance, adapter, device, queue per process.** netrender's
   `WgpuHandles` is the authority. No tenant creates a device, ever.
2. **One frame.** Tenants record into a caller encoder or produce external
   textures composed at explicit scene boundaries (the V1 seam, receipt-
   proven). Tenants never present; the host owns the surface.
3. **Declared capability profile** (§8.5). Storage-buffer/compute tenants
   (renderling, nexus) are WebGPU-enhanced only; the raster baseline
   (march/tracer, vello_hybrid lane) must keep working without them. The
   frame degrades by dropping tenants, never by forking architectures.
4. **One presentation math**: glam at every tenant boundary; the nalgebra
   adapter lives only at the parry/rapier seam.
5. **One receipts culture**: headless goldens, timing spans beside
   netrender's, adapter/backend/profile recorded in every receipt.
6. **One authority contract underneath**: integer facts, replay hashes,
   three-tier physics (place-graph plan §0.10). No tenant's state is ever
   world truth.
7. **Shared typed layouts**: GPU struct layouts derive from Rust types
   (crabslab in the rust-gpu family; encase-style derivation in WGSL
   lanes). The `CritterParams` vec4-packing scar is the argument.

rust-gpu is welcome (ruled 2026-08-06), carried by the fork family
(renderling now, nexus later if admitted), pins reconciled by us; the
wing's own WGSL lanes stay WGSL.

### Recommendation

Stop Mesocosm renderer extraction here. Keep the deterministic per-part mesher
as the mesh oracle and return to game pressure. Run **D0** in netrender when a
downlevel browser target is wanted; it is independent of body projection.
Let Paredros pull V3 streaming and residency when it exists as the second live
consumer. Bones stays visible in the record as the strongest missing-middle
donor, but it does not sit between voxel data and the GPU.
