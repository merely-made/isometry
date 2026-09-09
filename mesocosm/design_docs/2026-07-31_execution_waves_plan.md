# Execution Waves

**Demoted to Wave 1/2 history, 2026-08-07 (audit).** This plan predates
the place-graph and general-model plans and no longer orders Mesocosm;
its "next" pointers (PD1b/PD2) were bypassed by independently landed
E-slices and G-probes. Until a successor dependency ledger exists, the
current chains are:

- `PD1b -> PD2 -> P3 -> PD3/PD4 -> P4` (processdef/phenotype lane), and
- `G0/G1 world adoption -> G2 integration -> G3 world wiring -> G4`
  (place-graph lane), with F0-F5 exploratory.

Wave 1/2 content below is preserved as history. The successor is the
[dependency ledger](2026-08-07_dependency_ledger.md).

**Status: in progress, 2026-08-01.** **Wave 1 is complete**: core, runtime,
mesher, renderer, a windowed host, and the Isometry projection, with every
done-condition met and tested. **Wave 1.3 dropped**: the render lane is decided (custom wgpu body
renderer, netrender owning the device), so there is no second host to compare
against. Ruled by Mark. This is the **authority on ordering** across the
games wing. It does not restate design;
it sequences the work the governing plans already specify and adds the
constraints that only appear once the order is fixed.

Governing plans, which own the *what*:

- [body pipeline and host probe](archive_docs/2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md) — the shared organ, the body document, R-phases
- [Mesocosm founding plan](2026-07-30_mesocosm_founding_plan.md) — vessel 1's design and M-phases
- [games wing founding record](2026-07-30_games_wing_founding.md) — the laws, and the proof pair as the next architectural threshold
- [phenotype plan](2026-07-31_phenotype_plan.md): Mesocosm's body rules and local proof dependencies
- [wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md): portable body identity and sovereign readings

---

## Wave 1 — Architecture and receipts

The purpose of this wave is to make later decisions cheap by producing
evidence rather than argument. Nothing here is a game yet, and that is
deliberate — but see the standing caveat in Wave 2.1, which is the reason this
wave is not allowed to expand.

### 1.1 `mesocosm-core` — **LANDED 2026-07-31**

Own the core: seeded simulation, ordered inputs, snapshots, state hashing, the
body graph, provenance, and the smallest possible metabolize/attach operation.

**The core owns game state; hosts only project it.** That sentence is the
whole architecture of this wave, and every later comparison depends on it
holding literally.

**Done when** the fixture replays identically, provenance round-trips, and
attaching a part changes the core's mass and collision state.

**All three met.** `crates/mesocosm-core`, 30 tests, clippy clean, no host or
graphics dependencies (serde and postcard only). The shared fixture lives in
`tests/replay.rs` and is what waves 1.2 and 1.3 replay.

The design decision worth carrying forward: **the core is integer-only.**
Voxel coordinates, masses in milligrams, and quarter-turn rotations are exact
on every platform, so a replay cannot diverge on a different machine's
floating-point behaviour. This is a stronger guarantee than the R0 note
anticipated, which had float physics behind a seam but still inside the
determinism argument. Float physics now sits wholly outside the core, and
`rapier`'s `enhanced-determinism` stops being load-bearing for replay
equivalence: a host may use floats freely, provided it does not feed derived
float state back into the core.

### 1.2 The Genet host — **COMPLETE 2026-07-31**

Build the custom host over the existing winit / wgpu / Cambium stack, probing
**Renderling** for voxel bodies. It consumes the core's state and intents
**without duplicating rules**.

**Done when** the new part is visible and physically legible, the host uses the
intended wgpu device, and uneven frame delivery does not change simulation
results.

**Split on landing, because the three conditions carry very different risk.**
The frame-delivery condition is a logic property provable without a window; the
other two need a GPU and a display. Doing the provable part first also serves
the confound rule, since the stepping it defines is shared by both hosts.

- **`mesocosm-runtime` landed** (`crates/mesocosm-runtime`, 12 tests, clippy
  clean): the fixed-step clock, the intent queue, the applied-intent trace, and
  `Receipt`. **Uneven frame delivery does not change simulation results** is
  met and tested: the same total elapsed time delivered in ragged chunks
  produces the same trace and the same state hash, and a step cap defers work
  rather than dropping it.
- **`mesocosm-mesh` landed** (`crates/mesocosm-mesh`, 24 tests, clippy clean):
  per-part greedy voxel meshing plus rigid placement, headless and integer-only.
  **The attachment hypothesis is met on everything derivable without a screen.**
  An end-to-end test eats a real morsel and checks all four claims at once:
  mass grows, the centre of mass moves toward the new part, the collision box
  grows, and the drawn body gains placements and reaches further, with
  provenance intact alongside the geometry.
- **`mesocosm-genet` landed** (`crates/mesocosm-genet`): a winit window, a wgpu
  surface, input as intents, and the shared runtime stepping the world. It
  holds no game state; if a rule ever appears in it, it is in the wrong crate.
  **All three done-conditions are now met.** A capture run of 300 frames
  produced **111 steps** — the window drew at roughly 162 fps while the
  simulation ticked at exactly 60 Hz, which is frame-delivery independence
  observed live rather than only unit-tested. The critter grew to five parts
  during the run and the frame was captured on exit.
- **`--frames N --capture PATH` keeps the windowed path verifiable** without a
  person sitting in front of it, which is the same discipline as the headless
  tests: run the real loop, leave evidence.

**Wave 1.2 is complete.** What remains open is not a condition but a judgment
already given: Mark's read of the first renders was that the parts read as
attached rather than floating, with the caveat that a still frame cannot show
what is *joined* — that wants motion, which the window now supplies.

**Why the runtime is shared rather than per-host.** If each host wrote its own
stepping, a divergence between hosts could be a divergence in *stepping*, and
the probe would measure the wrong thing. Both hosts drive the world through
this crate, so any difference is attributable. This is also the extraction
candidate the body pipeline plan named; it stays here, and small, until a
second consumer justifies lifting it.

### 1.3 ~~The Bevy host~~ — **DROPPED 2026-07-31**

The **host-engine comparison** is cancelled. Mesocosm renders through a small
custom wgpu body renderer with netrender owning the device and compositing.

**The reason is stronger than preference, and worth keeping so the decision is
defensible later.** Weighing the field established that **our rendering need is
tiny and specialised**: the mesher emits flat-shaded palette quads with no
textures, no interpolated normals, no skinning (parts are rigid by ruling), and
no authored materials. Engines are optimised for the opposite problem, authored
content with PBR materials, image-based lighting, and skeletal animation.
Renderling's entire feature set, and most of Bevy's rendering value, sits in the
part of the problem this game does not have. Roughly six hundred lines of
pipeline covers what we do need.

**What dropping the comparison costs, stated honestly.** We give up a *measured*
answer to "would an engine have been faster to work in." The receipts R2 was to
produce, adapter size, frame pacing, iteration quality, asset handling and
debugging, become absolute observations rather than comparisons. They are still
worth recording, and if the custom lane turns out to fight us, the engine
question can be reopened with real evidence from having built the thing once.

**Clarified 2026-08-04:** dropping the Bevy comparison did not cancel
productionizing the renderer or extracting a reusable voxel presentation
boundary after a second consumer. The landed `mesocosm-lens` also changed the
immediate renderer from a transient mesh scene to a heightfield/SDF march. Its
retained-resource V0 gate and same-device native/browser V1 gate are now
complete. V2 projection plurality is also complete: one real played body
revision now crosses Lens, mesh, and Isometry bake with matching part identity
and narrow graft invalidation receipts. Mesocosm renderer extraction stops
there. Paredros, not Mesocosm, is the consumer allowed to pull chunk streaming
and persistent mesh residency. The optional `vello_hybrid` headed-WebGL gate
belongs to netrender and does not block game work. See the engine landscape
§8.

**The confound rule retires with the comparison, but its insight does not.**
"Do not change two variables at once" still applies to any later A/B, and most
immediately to a 2.5D-versus-3D presentation choice, which must not be decided
inside some other change.

**Rejected with reasons, for the record:**

- **Renderling.** Alpha and self-described work in progress. The original
  reason here was too categorical: current `main`, rechecked 2026-08-04, has
  no repository `rust-toolchain` pin. It does retain rust-gpu/cargo-gpu shader
  machinery at a git revision and currently uses wgpu 26 while this wing uses
  wgpu 29. More importantly, its glTF, IBL, PBR, shadows, bloom, and GPU scene
  model solve a different workload. **Ideas worth taking without the whole
  dependency** are caller-owned device construction, `crabslab`/`craballoc`
  resource lifetime, headless image-diff tests, and shared CPU/GPU types.
- **Vello, for now.** If Mesocosm settles into a genuinely 2.5D stylised look,
  vello becomes very attractive, and it is already owned. It is not the probe
  target because **vello has no depth buffer**: a 2.5D vello lane would depth
  sort by painter's algorithm, which is a different rendering approach rather
  than a different host, and the geometry here is separate rigid boxes that a
  depth buffer handles without sorting artifacts. Revisit once the look is
  known.

**Adopted from the Renderling read: golden-image testing.** The visual half of
1.2 was going to be pure judgment. It need not be. wgpu renders headless to a
texture, so the renderer should be built headless-first and wrapped in a window,
which makes "the new part is visible" an assertable property rather than an
opinion. The opinion that remains is whether it looks *good*, which is the right
thing to leave to a human at a screen.

### 1.4 The Isometry projection — **COMPLETE 2026-07-31**

Teach `isometry-voxel` to consume **`BodyDocument v0`** and emit a sprite plus
a bake receipt. `.vox` remains an **authoring input**, never the interchange
object.

**Done when** one critter completes
`body document → Isometry sprite → body/profile round-trip`
with part provenance intact.

**Landed: the flattening half.** `mesocosm-mesh::flatten` composes a whole body
into one occupancy grid with a body-space origin, handling negative placements
by shifting the origin rather than clipping. That is precisely what
`isometry-voxel::bake_facing` consumes, so **Isometry's baker needs no change**;
what was missing was the adapter, not a feature.

**Open, and it is a decision rather than work: which side holds the seam.**
`Voxels` and `Palette` live in `isometry-voxel`; `Flattened` lives here. One of
four has to happen, and the choice is Mark's because it sets a coupling
direction for the whole wing:

1. **Mesocosm depends on `isometry-voxel`** (git dep). Smallest diff. Couples a
   game to a VTT's asset crate, which is backwards but survivable since
   `isometry-voxel` is nearly standalone.
2. **Isometry depends on `mesocosm-core`.** Worse: a general VTT would depend on
   one specific game.
3. **Extract the body document into a neutral crate** both consume. Correct by
   the wing's own extraction rule, since two real consumers now exist, and it
   would be MIT/Apache under the licensing split as a genuinely reusable
   library. Costs a naming round and a new published crate.
4. **Couple by data, not types.** Mesocosm writes the grid through
   `mere.pack/v1`; Isometry reads it with its own thirty-line adapter. Most
   consistent with the wing's law that games interoperate through data rather
   than type dependencies, and the only option that scales to a third game.

**Ruled 2026-07-31: option 4, couple by data.** Mesocosm writes the body
document and its flattened grid through `mere.pack/v1`; Isometry reads them
with its own small adapter into `Voxels` and `Palette`. No shared type, no
cross-repo dependency, no naming round.

This is what the wing's interop law already required — games interoperate
through data rather than type dependencies — and it is the only option that
scales to a third consumer without rework. It also follows from the
presentation ruling: if projection negotiates by capability, the pack must
ship the *document*, so the seam is data by construction.

Revisit option 3, a neutral extracted crate, when a third consumer appears and
the schema has stopped moving.

**Booked cost, honestly:** a schema and a reader instead of a type the compiler
checks. Drift between writer and reader becomes a runtime failure rather than a
build failure, so the profile needs a version field and a refusal path from the
first commit. That is the trade the interop model makes everywhere else.

**Wave 1.4 is complete, 2026-07-31**, and the seam is proven with real bytes
rather than asserted. `mesocosm.body/v0` is the schema, `mesocosm-mesh::profile`
writes it, `isometry-voxel::body` reads it, and neither repo depends on the
other. A critter grown by incorporation crosses and bakes to a sprite from four
facings in which you can see what it ate.

Four things the build settled that the ruling did not:

**V0 crosses a projection, not the body.** The first cut put a whole
`BodyDocument` on the wire, and that quietly broke the ruling it was built to
satisfy: a reader would have needed `mesocosm-core` — attachment graph, pivots,
body plan and all — which is a type dependency wearing a data dependency's
clothes. "Isometry reads it with its own small adapter" is only true if the
adapter is small. So every field on the wire is a primitive, a fixed-size
array, or a `Vec` of those, and Isometry's mirror is about twenty lines.

This proved that optional appearance and provenance can cross through local
primitive mirrors. It did not prove that topology should stay home. The later
anatomy ruling makes primitive parent links part of portable identity at v1,
without putting a core type on the wire. A test named
`the_profile_carries_no_core_types` decodes the payload into a structurally
identical foreign mirror, so the next person to put a core type back on the
wire gets a red test rather than a review comment.

**The version field cannot live inside the payload.** Postcard is positional
with no field tags, so when the layout changes the decoder cannot reach the
version field to discover it should have refused — it fails as a malformed
decode, or succeeds and returns nonsense. The schema tag and version therefore
sit in a fixed-position header ahead of the payload: eight magic bytes and a
little-endian `u16`, checked before anything is decoded. Both sides have a test
proving a bumped version is diagnosed even when the payload is unreadable,
which is the case a field inside the payload cannot handle at all.

**Flattening destroys history, so attribution is a second grid.** `flatten`
composes every part into one occupancy grid and in doing so discards which part
wrote each cell — the grid records materials, not origins. That is right for
the mesher and wrong for this seam, because the wing's legibility rule is that
the world is colour-coded by role and a creature is colour-coded by history. A
sprite baked from materials alone cannot show where a limb was taken from. So
the profile carries a parallel grid naming the part behind every voxel, written
by the same loop that writes occupancy, so the two can never disagree about
which part won a contested cell.

**A committed fixture is what buys back the compile check.** Unit tests on each
side prove each side self-consistent and prove nothing about the pair.
`mesocosm-mesh/fixtures/critter.body` is real writer output, copied unchanged
into `isometry-voxel/tests/fixtures/`, and Isometry's suite reads it. If the
writer changes shape without bumping its version, Isometry goes red instead of
a player's sprite going quietly wrong. The copy is deliberately manual: a
fixture that synced itself would hide exactly the drift it exists to catch.

Also checked rather than assumed: **the two engines agree on axes.** `.vox`
needs a remap because MagicaVoxel is Z-up. Mesocosm is Y-up like Isometry — its
`Above` facing is axis 1 and its yaw rotates about axis 1 — and both use
`x + y * dx + z * dx * dy` cell order, so the importer copies straight across.
The arrays lining up would not have been evidence.

Receipt: `Code/testing/mesocosm/09_critter_crossed_to_isometry.png`, a 494x132
four-facing strip of a three-part critter, one part founding and two
incorporated, the founding trunk green and the taken parts ochre.

Open, and deliberately not built: the `mere.pack/v1` envelope itself. A pack
carries an inventory of content-addressed blobs, and this is what goes *inside*
a blob. Wiring the outer envelope means depending on eidetic, which is the
federation platform, and the wing's rule is that the platform is extracted from
shipped games rather than built before them. Mesocosm is a candidate for that
proof, not yet a consumer. The bytes are ready for it whenever the pack lane
opens.

### Where Bones goes

**Not its own lane**, and now **unblocked earlier**. The original rule was to
test core-owned storage against Bones inside the *winning* host, so the ECS
choice would not be mixed with a renderer choice. With one host, there is
nothing left to wait for: the question is whether `mesocosm-core`'s own storage
or Bones ECS serves the core better, and it is independent of rendering.

It stays deferred behind playfeel rather than behind the host, because a
storage model chosen before there is a game to store is chosen against
guesses.

**Rechecked 2026-08-04:** Bones remains renderer-neutral at the `bones_lib`
layer and keeps the useful combination of deterministic sessions,
snapshot/restore, reflected schemas, content-addressed assets, and Piccolo.
Its official renderer is still Bevy 0.11 and 2D-focused. Since
`mesocosm-core` now owns exact simulation, replay, snapshots, provenance, and
epochs, adopting Bones would currently replace authority rather than fill a
gap. It remains a focused missing-middle donor; reopen it only when two games
need one renderer-independent service that has no current owner. The voxel
engine research does not reopen an ECS migration.

---

## Wave 2 — Playfeel and proof

Begins once one host can make metabolize feel promising.

### 2.1 Mesocosm M0 playfeel

One enclosure, one critter, meaningful movement, eating, deposition,
incorporation, and a metabolic budget.

> **The standing caveat, and the reason Wave 1 is bounded:** architecture
> receipts cannot substitute for the founding condition. **Somebody wants
> another run.** No quantity of state-hash equivalence redeems a verb that is
> not worth repeating.

#### Phenotype gate inside 2.1, **ACTIVE 2026-07-31**

The [phenotype plan](2026-07-31_phenotype_plan.md) supplies the dependency
order. P0 through P2 have now removed the special played-body model and proven
one embodied consequence. P2's deferred biomass and upkeep account landed in
`d9af641`; body v1 still waits on the later local proofs.

1. **P0, meal choice, mechanically landed:** burn or incorporate one meal,
   with mutually exclusive receipts and consistent venom. The headed playfeel
   judgment remains open.
2. **P1, one organism model, landed 2026-08-01:** played and unplayed critters
   use the same body-bearing representation and control names an organism.
3. **P2, one embodied consequence, landed 2026-08-01:** gameplay reach is
   derived from anatomy, and severing the contributing process path removes it.
   Its biomass and upkeep reconciliation is also landed.
4. **Play before authoring infrastructure:** PD1a's allocation design is
   complete. The recipe-to-`BodyDocument` authority proof and live constructor
   join are also landed: founders, offspring, local chronicle regrowth, and the
   founder-preview seam share `Species::realize`. Next migrate native process
   identity and allocation in PD1b and play one native process at PD2. P3 branch
   transfer follows stable process identity and allocation. Static packs and
   Piccolo then replace the native authoring path before phenotype-based
   adaptation.

Branch transfer may begin after stable process identity. Phenotype-based
adaptation consumes the validated expression path. Contested resource flow
then consumes both. The wing v1 schema still waits for stable subject,
body-revision, part-address, and local process-reference proofs.

### 2.2 Epoch and ecology lab — **COMPLETE 2026-07-31**, with one finding

`mesocosm-core::epoch` models the adaptation phase headlessly: descending-complexity
initiative, simpler-lineage response, the complexity frontier, autonomous inactive
lineages, and three authored worlds. 35 tests. `cargo run -p mesocosm-core --example
ecology_lab` is the watching half.

**The initiative rule needed a type to be more than decoration.** Ordering by
complexity means nothing if every lineage scores against the same frozen world. So
`Standing` computes what a lineage faces from the roster's *current* state — world
pressure plus what the neighbours currently are — and commits land immediately. A
lineage acting later is answering a world the earlier ones have already changed,
which is exactly the compressed generation tempo the rule was ruled for.

**Complexity is derived, never stored.** It is the sum of what a lineage carries, so
initiative cannot be bought: acting first means being genuinely expensive, and upkeep
is superlinear so being expensive is charged for. That is what keeps a fruit fly
viable next to a cicada.

**Nothing in the round reads `played`.** Unplayed lineages adapt by the same code, so
a line the player left keeps changing while they are elsewhere. A test runs the same
round over a played and an unplayed roster and asserts the transcripts are identical.
Law C holding at the level of the simulation rather than the file format.

**Fitness punishes the worst answer, not the average.** Squared deficits, so a lineage
dies of the thing it is worst at and adaptation shores up weaknesses instead of piling
onto strengths. A linear score would let a lineage rationally ignore a lethal pressure
because it was excellent somewhere else.

**The three authored worlds produce three different creatures**, which is the whole
reason to author three. On the tidal shelf (crowding 8, dark 5) everything converges on
sense and fecundity; in the heavy deep (gravity 9, corrosive 6) on frame and shell; in
the long year (cold 8, drought 7) on insulation and water. Same code, same founders,
different answers.

#### The finding: the phase converges, and nothing ever dies

Measured across five seeds x forty rounds x three worlds — 600 lineage-rounds:

- **Zero extinctions.** Not one, in any world, at any seed.
- **Convergence by round ~6.** After round eight essentially every remaining change
  is a `+1` tweak. Activity continues, but nothing consequential happens.

This is not a tuning problem. `EXTINCTION_FLOOR` is -400 and final fitnesses sit
around -12 to -24 — the roster is not near the floor, it is comfortable. The cause is
structural: **every pressure has a trait that answers it, and income is flat and
uncontested, so every lineage eventually solves its world and then stops.** Nothing
in the model can make a lineage lose.

Open question 3 in the [founding plan](2026-07-30_mesocosm_founding_plan.md) framed
extinction as emergent-preferred with evented pressure "only to keep worlds from
settling". **The lab says emergent extinction does not arise on its own under these
conditions**, so that "only" is doing more work than it looked like. Two candidate
answers, and the choice is a design ruling rather than a tuning pass:

1. **Contested income.** Bank is currently flat per lineage per round. If income were
   a share of a finite pool weighted by fitness, a lineage that fell behind would
   adapt more slowly, fall further behind, and eventually fail — extinction as a
   consequence of competition rather than of a threshold. This is the emergent answer
   open question 3 prefers, and it makes crowding mean something the fitness function
   cannot currently express. It also belongs to the **epoch** half, since how bank is
   earned is what the played phase is for, which is why the lab left it flat.
2. **Evented disturbance.** A world throws a glaciation, an impact, a plague: pressures
   move and a settled roster is suddenly wrong. Cheaper to build and immediately
   legible, but it makes the drama authored rather than grown.

They are not exclusive, and the honest reading is that **1 is the fix and 2 is the
seasoning**. Recorded rather than built: disturbances are not in this wave's target,
and contested income is a ruling about the epoch half that should not arrive as a side
effect of the adaptation lab.

### 2.3 Paredros social proof

**P0/P1 only**: one place, three companions, offers, confidence bands, one
standing agreement, and bounded tag-in. **Placeholder bodies** — this proof
does not wait on the body pipeline.

The proof is whether unfamiliar requests and trusted assignments *feel
socially different* without becoming friction theatre. That is a judgment, and
it is the right kind of judgment for this vessel: the failure mode is a
negotiation minigame attached to routine work.

### 2.4 The full Mesocosm–Isometry proof pair — **COMPLETE 2026-07-31**

The loop is closed with real bytes in both directions. Mesocosm writes a
critter, Isometry reads it into a roster slot and adds history, Mesocosm reads
the descendant back and founds the next generation. `mesocosm.chronicle/v0` is
the return schema; neither repo depends on the other.

**Law C is demonstrated, and the demonstration had to move to the consumer.**
Mesocosm proving its own two records identical is the easy half — a writer can
hardly be surprised by its own output. The claim that matters is that the
*consuming* game has no way to sort them, so the test lives in
`isometry-campaign/tests/proof_pair.rs`: given two files and nothing else, every
question the campaign can ask returns the same kind of answer.

Two things that had to be true for the proof to mean anything:

- **The played critter is genuinely played.** `tests/proof_pair.rs` drives the
  world — hunts the nearest organism, walks at it, eats it, four hundred times —
  rather than hand-building a body. A hand-built body would prove that two
  structs with the same fields compare equal, which is not the claim.
- **The generated critter has a real history, at a real size.** A blank-slate
  RNG creature is trivially distinguishable from a played one, and so is a small
  one: **nobody needs an `is_player_made` flag to break Law C when the part
  count already gives it away.** The first generator capped at five parts while
  a well-fed critter reached forty-seven, so the distributions were disjoint and
  a consumer could have guessed origin from size alone. The generator now
  reaches the sizes play reaches, and both suites test for the overlap.

**The roster slot is the one Isometry already had.** An arriving creature
becomes a `WorldCharacter`, the same struct an authored NPC uses. A separate
`ImportedCharacter` would have failed Law C by existing. And that struct carries
a `faction`, so this seam is exactly where a borg becomes a character in the
sense ruled on 2026-07-31.

**The keystone is now three properties of a type rather than three phrases.**

- *Additive facts*: `append` is the only mutation a chronicle has. No edit, no
  delete, so set union is a legal merge and no game can quietly rewrite another's
  record.
- *Opaque preservation*: a deed carries the writing game's own vessel name, its
  own verb, and a payload nobody else parses. `tests/homecoming.rs` asserts that
  Isometry's facts come home byte for byte and survive being re-emitted, because
  fact loss happens by omission and omissions are not caught by review.
- *Deferred interpretation*: Isometry records what happened; Mesocosm decides
  what it means for a body. `Chronicle::found` reads the record without
  consuming it, so the next game sees everything this one saw.

**The round trip forced a distinction the design had not named.** A game's own
verbs are opaque, so their payload can be anything — Isometry writes JSON. But a
verb *two games both act on* is a contract, and needs an agreed payload. Writing
a `HistoryEvent` whose kind is `"lost-part"` therefore does **not** claim a lost
part; `record_loss` does, carrying the little-endian `u32` Mesocosm reads.
Narrating a loss and claiming one in another game's anatomy are different acts,
and a game that guessed from the prose would be inventing consequences for
somebody else's fiction. The shared vocabulary is deliberately one verb long:
each addition is a coupling two vessels must keep in step forever.

**Law A is visible as an absence.** A chronicle carries provenance and history
and no coordinates. `geometry_did_not_travel_and_that_is_the_law` founds the same
record twice at different scales and gets the same lineage with different
anatomy, because the descendant is regrown here. That is what keeps the round
trip cheap and keeps another game from dictating this one's bodies.

**Framing was factored out.** `mesocosm-core::wire` now owns the magic-plus-
version header both schemas ride, so the refusal contract is written once and
every reader in the wing refuses for the same reasons. `profile.rs` lost 84
lines to it. `mesocosm-mesh` reads `PartOrigin` from core rather than defining
its own, so the wire form of provenance has one definition.

Fixtures, all committed and all real output: `played.chronicle` and
`rng.chronicle` from `cargo run -p mesocosm-core --example emit_chronicles`,
`returned.chronicle` from `cargo run -p isometry-campaign --example emit_return`.
The copies between repos are manual on purpose.

---

## Deferred, with reasons

Not "later" as a vague hope — deferred because each becomes much easier to
*name correctly* after M0, the host probe, and the first real cross-game
round-trip:

| Deferred | Why it waits |
| -------- | ------------ |
| Procedural world generation | Authored worlds first (2.2); generation is a question about a grammar that is still being read against real play |
| Paredros city simulation | The settlement's shape depends on what the social proof (2.3) shows is worth simulating |
| A generic storyteller | Extraction discipline: build one inside a vessel, extract at the second consumer |
| Shared game-runtime extraction | Same rule. Paredros is not yet a consumer of anything |

---

## Findings

- **2026-07-31, wave 1.1.** An integer-only core removes float determinism
  from the replay argument entirely, rather than managing it. Voxel units,
  milligram masses, quarter-turn rotations, and an `i128` accumulator for
  centre-of-mass are exact everywhere. Consequence for wave 1.3: the two hosts
  must agree on a state hash, and neither host's physics backend can affect
  that, because host physics never writes to the core. If a host ever needs to
  push a derived float back in, that is the moment this guarantee breaks and
  the decision should be surfaced rather than absorbed.
- **2026-07-31, wave 1.1.** Rejections are recorded outcomes rather than
  errors. An intent that cannot apply still advances the tick and returns
  `Outcome::Rejected`, so a replay that rejects the same intents stays
  identical. This matters for the host probe: a host that silently drops an
  invalid intent instead of submitting it will diverge.
- **2026-07-31, wave 1.1.** The body needed `world_yaw` alongside
  `world_offset`. Position alone would let a projection draw every part
  unrotated, which was a real gap in the portable document rather than a
  rendering detail.
- **2026-07-31, wave 1.2.** The fixed-step clock must accumulate in *rational*
  form, scaling elapsed microseconds by the tick rate rather than dividing by a
  precomputed interval. **1_000_000 is not divisible by 60**, so a precomputed
  16666 us interval runs fast and gains a step roughly every 25 seconds. The
  first implementation here asserted divisibility, which would have refused
  60 Hz outright; the second rounded, which drifts. Scaling is exact for every
  tick rate and refuses none. A host probe that ran long enough would have
  found this as an unexplained divergence between two hosts polling at
  different rates.
- **2026-07-31, wave 1.2.** A step cap must **defer** rather than drop. Capping
  by discarding the remainder would make the simulation depend on how badly the
  host stalled, which is exactly the property the fixed step exists to remove.
- **2026-07-31, wave 1.2.** **Per-part meshing makes attachment cheap, and the
  plan's rigid-part posture is what buys it.** A part's geometry depends only on
  its volume, so it is cacheable by `VolumeRef`: incorporating a part adds one
  placement and, only if the volume is new, one mesh. Nothing already on the
  body is remeshed, which is verified rather than assumed. This also means faces
  are never merged across a joint, which is correct rather than a shortcut,
  since merging across parts would weld the body into one mesh and lose the
  ability to move a limb.
- **2026-07-31, wave 1.2.** A missing volume is a **reported error**, not a
  skipped part. The tempting behaviour is to draw what resolves and move on,
  which yields an invisible limb and a silent divergence between a body's
  physics and its picture. `MeshError::MissingVolume` names the part.
- ~~**2026-07-31, wave 1.2.** The mesher is the shared organ… Isometry's baker
  wants the same quads.~~ **Wrong, corrected 2026-07-31 by reading the code.**
  `isometry-voxel::bake_facing` takes a `Voxels` **occupancy grid** and projects
  it voxel by voxel with depth sorting. It has never wanted quads.

  The corrected shape is better, and smaller. There are **two projections off
  one document**: a live renderer wants quads meshed per part and kept separate
  so limbs can move; a sprite baker wants a single grid, because a baked sprite
  has no moving parts. `mesocosm-mesh::flatten` composes a body into one grid
  and is the adapter that lane needs, so **Isometry's baker requires no change
  at all**. The shared organ is the body *document*, which is what the body
  pipeline plan said in the first place; the quads were my embellishment.
- **2026-07-31, wave 1.2. A part's local origin is its lowest corner, not a
  pivot, and the body document needs to say which it wants.** This is the first
  finding that came from *looking at a render* rather than from a test, which is
  the argument for keeping a human in this loop. Every visual test passed while
  the first rendered body had its limbs **floating in space beside the torso**,
  because a test can assert that a part is drawn and that the silhouette
  widened, and both were true of a detached limb.

  Two consequences follow, and they are the same underspecification seen twice.
  Attaching a part flush requires knowing that part's size, so an author cannot
  write an offset without consulting the volume. And **rotation turns a part
  about its corner**, so a limb that was flush swings off its joint when yawed,
  which is why the example holds yaw at zero.

  The fix belongs in the body document as a **per-part pivot**, the origin an
  attachment frame is measured from and a rotation turns about. The plan already
  calls for the `.vox` importer to strip marker voxels and write explicit
  attachment frames; this says what those frames must carry. Deferred rather
  than done, because the right pivot convention is easier to choose once real
  authored parts exist. Recorded so it is not rediscovered as a rendering bug.

---

## Progress

- **2026-07-31**: waves ruled and recorded.
- **2026-07-31**: **wave 1.1 landed.** `crates/mesocosm-core` with body graph,
  provenance, seeded stream, ordered intents, whole-world snapshots, and FNV
  state hashing. 30 tests, clippy clean. Deps: serde, postcard. The fixture in
  `tests/replay.rs` is the shared artifact waves 1.2 and 1.3 compare against.
- **2026-07-31**: **wave 1.2 part one landed.** `crates/mesocosm-runtime`:
  drift-free fixed-step clock, intent queue, applied-intent trace, and
  `Receipt` for host comparison. 12 tests, clippy clean. The frame-delivery
  done-condition is met; the window, device, and body projection remain.
- **2026-07-31**: **wave 1.2 part two landed.** `crates/mesocosm-mesh`:
  per-part greedy voxel meshing, rigid placement, and an end-to-end attachment
  test over the real simulation. 24 tests, clippy clean. 66 tests across the
  workspace. Only the window and the device remain in wave 1.2.
- **2026-07-31**: **wave 1.2 complete.** `crates/mesocosm-genet`: winit window,
  wgpu surface, input as intents, shared runtime stepping the world, and a
  `--frames/--capture` mode that keeps the windowed path verifiable. A 300
  frame run produced 111 steps at 60 Hz, so the loop drew at ~162 fps while
  the simulation ticked fixed. 82 tests across the workspace, clippy clean.
  Five crates: core, runtime, mesh, render, genet.
- **2026-07-31, wave 2.1 (playtest).** Mark ran the window and it found two
  defects no test had. **Drawing only the body made the game unplayable**: the
  camera framed body bounds and morsels were not drawn, so moving did nothing
  visible and there was nothing to act on. Fixed by rendering scenes with a
  following camera and dimming what is out of reach. **And placement stacked
  parts**: the fixture cycled six faces by `body.len() % 6`, so the seventh
  part landed on the first and a well-fed critter collapsed into a z-fighting
  pile. Fixed by checking the flattened body for free space before proposing an
  attachment, with a regression test asserting placed voxels equal expected
  voxels, since an overlap silently loses voxels to overwriting.

  The general lesson: **both defects were invisible to the test suite and
  obvious within seconds of playing.** Tests assert that a part is drawn and
  that the body grew; neither notices that the player cannot see the world or
  that the body is a pile. Wave 2.1's standing caveat is doing real work.
- **2026-07-31.** First free face gives **compact clumping**, not silhouette. A
  real growth policy will want directional bias, symmetry, or trait-driven
  morphology; the current one only guarantees no overlap. Recorded because a
  blob that reads as a blob is a design question, not a bug.
- **2026-07-31. Pivots, and the value of counting recurrences.** The
  corner-origin gap was recorded once as a rendering curiosity, then showed up
  as a placement awkwardness, then as a wrong centre of mass, then as a
  malformed AABB. **Four defects, one missing concept**, and each was fixed
  locally before anyone counted. The lesson worth keeping: a finding that
  recurs is not four findings, and the second recurrence is the signal to fix
  the cause rather than the instance.

  The fix also paid a dividend that was invisible while the bug stood: yaw had
  been pinned to zero everywhere, in the fixture and the plan and the growth
  resolver, each place with its own comment explaining why. Rotation now works,
  which unblocks the articulation that the presentation ruling depends on.
- **2026-07-31. The ecology substrate landed, and running it found three
  things no design pass had.** `Morsel` became `Organism`: kingdom, lifecycle
  stage, age, and a per-tick step, with the world stepping every organism on
  every tick whether or not the player acted.

  **One: producers had no limit.** Fixing energy from nothing with no
  competition made the population exponential — 75 organisms became 1530 in
  600 ticks — and biomass share is meaningless when everyone's share grows.
  Fixed with density-dependent income on a coarse grid, floored at rent so a
  shaded stand *stagnates* rather than starving. The first attempt had no
  floor, and an entire patch of identical plants crossed the starvation line
  on the same tick and went extinct instead of thinning.

  **Two: consumers could not eat.** They paid upkeep and earned nothing, so
  they were guaranteed to starve and every world converged to producers only.
  The trophic cycle had a missing rung and nothing in the design documents
  caught it, because on paper "consumers eat producers" reads as already true.

  **Three: producers alone are unbounded, and that is correct.** Crowding
  limits what a patch supports; dispersal escapes the patch. What actually
  regulates a pasture is something grazing it. A test asserting producers
  self-limit was asserting the wrong thing, and the right test is that a
  *mixed* world holds its population.

  Result: 75 alive to 74 alive over 600 unattended ticks, all three kingdoms
  coexisting, with carrion cycling. The world now goes somewhere on its own.
- **2026-07-31. Simulacra and signalling landed.** Organisms carry a `signal`
  (what they advertise), `venom_mg` (what they actually do to something that
  eats them), and a `guise` (the kingdom they appear to belong to). The gap
  between claim and truth is the mechanic.

  Both directions of lie exist, because one direction alone is just a hazard.
  **Batesian**: warns without a bite, so it is safe and only something that
  learned better will risk it. **Aggressive**: looks plain and bites hard,
  which is the trap that makes reading the world worth doing. A live world
  reports 91 warning, 76 armed, of which 24 bluff and 9 trap.

  **The tell is diegetic**, per the unfair-versus-unknowable line: a thing
  wearing a producer's look but living a consumer's life does not gain mass in
  open ground, because it is not fixing anything. Watch it and the lie shows.
  A test asserts exactly that divergence.

  **A lie is heritable.** Offspring inherit signal, venom, and guise, so a
  mimic is a *lineage* you can learn rather than a coin flip per organism.
  That is what makes the knowledge worth carrying across a death.

  And the renderer does not leak it: warning colours are drawn because a
  signal must be seen to mean anything, but a bluffer wears the same colours
  as something genuinely armed. Reach is dimmed because that is information
  the player is entitled to; honesty is not.
- **2026-07-31. "A tad abstract."** Mark's read of the signalling capture, and
  correct: the simulation had plants, grazers, corpses, saplings and giants in
  it, and the render had **confetti**. Every organism was an identically-shaped
  box in a hash-derived colour, so a rich ecology was invisible. This is Law B
  turned on the world rather than on the player's marks — depth nobody can
  perceive is procedural noise, whoever it belongs to.

  Three cheap changes, all presentation:

  - **Colour by kingdom**: green makes its own living, ochre eats, violet works
    the dead. A role is now readable at a glance.
  - **Colour by *guise*, never by kingdom.** A simulacrum is drawn as the thing
    it pretends to be, so the picture can be lied to. The renderer must not
    leak what the player has to learn.
  - **Scale by stage**: a juvenile is visibly smaller than what it will become,
    which is what makes waiting a decision rather than a delay. The dead drain
    toward grey.

  The player's own body deliberately keeps **material** colours rather than
  kingdom colours, because its parts came from different species and that
  patchwork *is* its provenance. The world is colour-coded by role; you are
  colour-coded by history.
- **2026-07-31**: **wave 1.4 complete.**  crosses from
   to  with no shared type and no
  cross-repo dependency. 139 tests in mesocosm, 22 in isometry-voxel, both
  clippy clean. The first cut embedded a whole  and had to be
  rebuilt as a flat projection, because a reader would have needed
   to decode it, which is the type dependency the ruling
  forbade. Details in §1.4.
- **2026-07-31**: **wave 2.4 complete.** The proof pair closes with real bytes
  both directions: `mesocosm.chronicle/v0` out, Isometry adds a roster slot
  and history, Mesocosm founds the descendant. 163 tests in mesocosm, 271 in
  isometry, both clippy clean. Two findings worth carrying: a generator that
  only makes small creatures breaks Law C without any marker, because part
  count becomes the tell; and a verb two games both act on is a contract
  needing an agreed payload, distinct from a game's own opaque vocabulary.
  Details in §2.4.
- **2026-07-31**: **wave 2.2 complete.** `mesocosm-core::epoch`: initiative by
  derived complexity, `Standing` so commits land before the next lineage
  scores, the frontier gate, and three authored worlds that produce three
  different creatures. 198 tests across the workspace, clippy clean. **One
  finding worth a ruling**: across 600 lineage-rounds there were zero
  extinctions and the phase converges by round six, because every pressure has
  an answer and income is flat and uncontested. Contested income is the
  emergent fix and belongs to the epoch half; see §2.2.
- **2026-08-01:** P1, P2, and P2's biomass/upkeep reconciliation are landed.
  The ProcessDef plan now gives allocation its own design pass, plays one
  native process before pack and Lua infrastructure, schedules P3 branch
  transfer after stable process identity, and keeps bounded Piccolo authoring
  before P4 adaptation.
- **2026-08-05:** PD1a is complete and the first recipe-to-anatomy authority
  proof landed. `Recipe + Soma` now develops an authoritative `BodyDocument`
  which V2 Lens projects.
- **2026-08-05:** the live founding and filial constructor join landed before
  PD1b. Founders and offspring realize recipes under the world's snapshotted
  palette, returned chronicles regrow through the same developer, and
  incorporation remains somatic growth during an epoch.
