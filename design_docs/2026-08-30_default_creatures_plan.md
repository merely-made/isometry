# Default Creatures Plan (2026-08-30)

**Status: implementation through DC4 landed; visual acceptance remains open
(refreshed 2026-09-04).** DC4's roster ships as the default founding, but Mark
still sees capsules rather than readable voxel critters. The
[phenotype plan section 8](2026-07-31_phenotype_plan.md#8-visible-voxel-bodies-integration-2026-09-04)
now coordinates procedural content, the live voxel-body path and presentation
through VB0-VB5. The earlier claim that none of the remaining problem is
anatomy is withdrawn: both placeholder content and lossy presentation need
proof. DC5's historical colour options are inputs to that integration, not
the final remaining task. Founded on the ruling that closed
the TD series in
[`2026-08-29_terrarium_dynamics_plan.md`](2026-08-29_terrarium_dynamics_plan.md)
§"The series closes here". This plan owns the *body* half of that ruling; the
update stage is the other half and is not this plan.

---

## 0. The ruling

Mark, 2026-08-30, quoted in the terrarium dynamics plan:

> "making creatures just sorta voxel chunks is pretty weird. let's start from
> the presumption that things have senses, limbs, and set roles. let's start
> with default, well tuned creatures with bodies, yes, made of voxels, but a
> bunch of small ones, recognizably flora and fauna like how the isometry
> tokens aren't just sorta cylinders... we're shooting for imagined creatures
> made of voxels here. the game is not beginning at the start of evolution.
> we're not dealing with single cells inventing life's basics. there should be
> some built in expectations of capacities."

And, owning the second playtest's finding
([`2026-08-28_played_slice_plan.md`](2026-08-28_played_slice_plan.md), Findings
2026-08-29): bodies read as "abstract voxel shapes", and the direction was
"shrink those down and put a few together in the shape of a body plan, maybe
you have something that looks more like a critter or flora then."

Two demands, and they are not the same demand:

1. **Founding starts from authored, capable archetypes** rather than a per-tier
   random draw. Senses, limbs and roles are presumed rather than rolled for.
2. **A body is many small voxel parts arranged into a body plan**, at the
   Isometry-token bar, rather than a handful of large blocks.

## 1. What this replaces, and what it keeps

**Replaced: the founding draw.** `axis::seed` (`crates/mesocosm-core/src/axis.rs:386`)
draws one recipe per founding species from a salted stream — a head, one to four
stretches, and an appendage lottery (`rng.below(6)`: 2/6 limb, 1/6 feeler on the
first stretch, 1/6 plate on the last, else bare). That draw is the direct cause
of TD11's seventh finding: sense organs land per *tier*, so five of ten seeds
found a world where **not one body has a sensing part**, and 307 of 3,070
founding fauna can see. Archetypes retire the lottery. `axis::seed` stays in the
tree as the generator it always was — it is what a *soup* world would still use
(founding plan §"Where new lineages come from": any world may begin anywhere on
the trait ladder) — but it stops being how the shipping enclosure founds.

**Kept: the representation.** Archetypes are **data in the existing machinery**,
not a parallel system:

- A `Recipe` (tagmata, appendages, lexicon, variance) is still what a lineage
  carries and what `Species::realize` reads.
- `Soma::develop` still varies an individual off it, so kin still resemble each
  other without being clones.
- `develop_body` is still the one path from recipe to `BodyDocument`, still
  conserving `mass_mg` exactly, still refusing an under-provisioned birth.
- `plan::classify` still reads a role off geometry. Nothing gets a stat flag.

The precedent is already in the tree: `axis::catalogue`
(`crates/mesocosm-core/src/axis/catalogue.rs`) hand-writes centipede,
millipede, insect, spider, tetrapod and snake as in-code `Recipe` constants. Its
own doc comment says "these are reference points, not content"; this plan adds a
**second, sibling set that is content**, and does not repurpose the catalogue —
the catalogue's job is to prove the four axial rules reach real animals, and
these are imagined creatures.

**The enabling ruling already exists.** Founding plan
([`2026-07-30_mesocosm_founding_plan.md`](2026-07-30_mesocosm_founding_plan.md),
§"The authoring caution"): *"author the organisms, generate the arrangements.
That is the wave 2.2 ruling (three authored worlds, not procedural generation)
holding one level further down, at the bestiary."* This plan is that sentence
executed. The Vesta reading it cites — "roughly a dozen organisms each carrying
one strange mechanism followed all the way through: complexity from specificity,
not quantity" — sizes the roster below.

**Two standing rules bind and are not negotiable here.** *Do not let a stage
grow its own engine* and *refuse any shift that needs a second simulation
authority* (`CLAUDE.md`, Important Don'ts). An archetype file that produced
bodies by a route other than `Recipe → Soma → develop_body` would be exactly the
second authority, and would re-open the split account PD0 closed. Every proposal
below stays inside the one path.

**And the phenotype plan's boundary is untouched.** 2026-08-03 ruling: "the
adaptation editor previews a founder phenotype but commits changes to the axial
recipe, `BodyPlan`, process-expression rules... as one heritable program. This
confirms the generator's recipe boundary rather than turning its first rendered
body into a lineage template." An archetype is a *starting* recipe. It is not a
template the epoch boundary must respect, and a lineage that mutates away from
its archetype is the point rather than a defect.

---

## 2. The part-scale arithmetic

**This is the load-bearing section.** S1 ruled "fix the world, not the bodies"
for the *world size* question and gave its reason — `part_ceiling_mg` prices
voxel volume, so shrinking half-extents would drag the economy TD6 and TD7 had
just tuned (`2026-08-29_scale_plan.md`, Findings 2026-08-29). This plan is the
deliberate body-side change S1 deferred, so it owes that arithmetic in full.

All figures below are derived from the formulas in
`crates/mesocosm-core/src/organism/ecology/rates.rs` and
`crates/mesocosm-core/src/organism.rs`, not measured. Measuring them against the
instrument is a done-condition of slice DC2.

### 2.1 The four numbers a body's economy reads

| reading | source | formula |
| --- | --- | --- |
| `part_ceiling_mg` | `rates.rs:119` | `voxels × 100 / 125`, floored, min 1 — **0.8 mg per voxel** |
| `mass_ceiling_mg` | `organism.rs:360` | sum of `part_ceiling_mg` over living parts |
| `actuator_span` | `organism.rs:300` | sum of longest half-extent over parts performing `Contract` |
| `sensor_span` | `organism.rs:327` | the same, over parts performing `Sense` |

and every rate is one of these two shapes (`rates.rs:200`, `build_multiple`):

```text
build_multiple = (ceiling + span × 100) / ceiling
rent   = 1 + m^0.75 × build_multiple / 62          (TD7)
bite   = 3 × m^0.75 × build_multiple / m_ref^0.75  (TD9)
decay  = 4 × m^0.75 × build_multiple / m_ref^0.75  (TD9)
sight  = 8 × build_multiple(sensor_span)           (TD11)
breeding gate = 33% of ceiling                     (TD8)
```

`Role` is read from geometry (`plan.rs:108`) and `Role → Process` is fixed
(`process.rs:54`): **Limb → Contract, Mass → Intake, Sensor → Sense, Plate →
nothing.** So of the four roles, only Limb and Sensor can move an economy
number at all.

### 2.2 The three findings

**Finding 1 — part *count* appears nowhere in the economy.** Not in the ceiling,
not in the build multiple, not in any rate. Ceiling is total voxel volume; span
is a sum over a subset of parts. **A body re-carved into more, smaller parts at
constant total voxel volume has an identical adult mass, an identical breeding
gate, and identical allometric terms.** The only per-part cost is integer
flooring — each part floors its own ceiling independently, so N parts lose up to
N−1 mg against one. On a 1,284 mg body at 33 parts that is at most 2.5%.

**Finding 2 — Mass and Plate detail is free; Limb and Sensor detail is priced.**
Mass has no span term and Plate has no process, so an archetype may spend as
many Mass and Plate parts on its silhouette as the *render* budget allows, at
exactly zero economic cost. Limbs and sensors are counted by their span, and
span is what the whole TD series is normalized against.

**Finding 3 — the economy is scale-free in size and is NOT scale-free in
slenderness.** This is the trap, and it is the one number an archetype palette
must respect. Define a template's **build price**:

```text
build price = 100 × longest_half_extent / part_ceiling_mg
            = 125 × longest_half_extent / voxel_volume
```

It is what one part of that shape contributes to a body's build multiple, per
milligram of the body it hangs on. Ceiling is cubic in half-extent; span is
linear; so **build price scales as 1 / cross-sectional area.**

| template | voxels | ceiling | span | build price |
| --- | ---: | ---: | ---: | ---: |
| limb `[4,1,1]` (the primitive palette) | 81 | 64 | 4 | **6.25** |
| limb `[5,1,1]` | 99 | 79 | 5 | 6.33 |
| limb `[3,1,1]` | 63 | 50 | 3 | 6.00 |
| limb `[5,2,2]` | 275 | 220 | 5 | 2.27 |
| limb `[3,1,0]` (thinned to a 3×1 cross-section) | 21 | 16 | 3 | **18.75** |
| limb `[3,0,0]` (thinned to a 1×1 rod) | 7 | 5 | 3 | **60.0** |
| sensor `[1,1,1]` (the primitive palette) | 27 | 21 | 1 | **4.76** |
| sensor `[1,1,0]` | 9 | 7 | 1 | 14.29 |
| sensor `[1,0,0]` | 3 | 2 | 1 | 50.0 |
| sensor `[0,0,0]` | 1 | 1 | **0** | 0 |

TD7's stated bound — "a body made of nothing but limbs reads `4 × 100 / 64` =
6.25, so no anatomy can price itself past ~7x" (`rates.rs:317`) — is **exactly
the primitive limb's build price**. TD11's "no anatomy may see the enclosure,
46 voxels" (`rates.rs:281`) is `8 × (1 + 4.76)`. Both bounds are properties of
the palette, asserted in doc comments as if they were properties of the game. A
finer palette that thins its limbs raises the first bound from 7× to 20× or 60×
in silence.

Two smaller traps in the same table. `classify` returns `Sensor` only when every
half-extent is ≤ 1, and `Limb` only when one axis is more than twice the others
— so **the smallest legal limb is `[3,1,1]`, seven voxels long.** Limbs cannot be
made much shorter; they can only be made thinner, which is the expensive
direction. And a `[0,0,0]` sensor is legal, one voxel, and contributes span
**zero** (`unsigned_abs().max()` of `[0,0,0]` is 0), so a body of one-voxel eyes
is blind. An archetype's *decorative* eye voxels and its *functional* sense
organs are therefore not the same parts.

### 2.3 The rule this gives archetype authoring

> **Conserve total voxel volume per tier, and keep every Limb template at a 3×3
> cross-section and every Sensor at `[1,1,1]`. Then no `REFERENCE_*` constant
> needs re-deriving, and every TD-series number lands in the band it was tuned
> in.**

The formulas being ceiling-normalized *is* enough — but only because those two
invariants hold. Break the volume invariant and the whole ecology moves in
absolute mass. Break the cross-section invariant and the build multiple moves
while the ceiling does not, which is the drag S1 warned about arriving through a
door S1 did not name.

The invariant should be **asserted, not commented**: `PartPalette::validate`
(`development.rs:75`) already refuses a template whose geometry does not
classify as its role. It should also refuse a Limb or Sensor template whose
build price exceeds the primitive palette's. That is the diagnostics-assert-
invariants pattern, and it turns a doc comment into a test.

### 2.4 The worked comparison

Two carvings of one grazer, matched exactly on total voxel volume. **Carving A**
is a representative body the current `axis::seed` produces; **carving B** is a
proposed archetype. Illustrative, not compile-ready.

**A — today (15 parts).** Recipe `[Tagma(1, Mouth), Tagma(3, Limb),
Tagma(4, None)]`: 9 mass `[2,2,2]` + 6 limb `[4,1,1]`.
**B — archetype (33 parts).** 8 trunk `[2,2,1]`, 1 head `[2,1,1]`, 1 snout
`[2,1,0]`, 2 eyes `[1,1,1]`, 6 legs `[3,1,1]`, 6 feet `[2,1,0]`, 6 dorsal
plates `[3,3,0]`, 3 tail `[2,1,1]`.

| | A (15 parts) | B (33 parts) |
| --- | ---: | ---: |
| total voxel volume | 1,611 | **1,611** |
| `mass_ceiling_mg` | 1,284 | **1,284** |
| distinct part shapes | 2 | 6 |
| `actuator_span` | 24 | 18 |
| `sensor_span` | 0 | 2 |
| build multiple | 2.87 | 2.40 |
| rent at adult mass | 10 mg/tick | 9 mg/tick |
| bite at adult mass | 58 mg | 49 mg |
| sight horizon | **8** voxels | **9** voxels |
| breeding gate (33% of ceiling) | 423 mg | 423 mg |
| birth floor (`biomass/4 ≥ parts`) | parent ≥ 60 mg | parent ≥ 132 mg |
| ticks of reserve at full | 128 | 142 |

Read across: **doubling the part count moved nothing.** The two columns differ
only where B chose a `[3,1,1]` leg over a `[4,1,1]` one and grew two eyes, and
both differences are inside the band TD7 measured for seeded fauna ("roughly 2
to 4 swing per reference segment"). B's build multiple is *lower* than A's, so
if anything the archetype is gentler on the economy than the draw it replaces.

The producer case is stronger still: a producer is all Mass parts and reads
`actuator_span` 0, so its build multiple is exactly 1 and every rate reduces to
the plain `allometric_rate` it has always been. **A voxel fern of 60 small parts
is bit-identical in the ecology to today's 22-block stalk of the same volume.**
Since the pyramid founds 610 of 916 organisms as producers, that is most of what
the eye sees, obtained for nothing.

### 2.5 Where the part count *does* bind

| constraint | threshold | today | at a 33-part archetype |
| --- | --- | --- | --- |
| `MAX_ROSTER_CAPSULES` = 10 — every non-played body | 10 parts | 15 (33% dropped) | 33 (**70% dropped**) |
| `MAX_CAPSULES` = 96 — the played body is not posed at all past it | 96 living parts | ~15 at genesis; **61 measured after ten meals** | 33 at genesis |
| birth at TD8's gate rather than later: `ceiling ≥ 12.1 × parts` | mean part ≥ 15.2 voxels | 107 voxels | 49 voxels |
| fertility at all: `ceiling ≥ 4 × parts` | mean part ≥ 5 voxels | ✓ | ✓ |
| genesis draw `100 + rng.below(400)` ≥ part count | ≤ 100 parts | ✓ | ✓ |
| TD7/TD11 bounds | build price ≤ 6.25 / 4.76 | 6.25 / 4.76 | must be held (§2.3) |

The fertility rows are worth stating because they are silent failures. A birth
realizes the recipe at `cost = parent.biomass_mg() / OFFSPRING_COST`
(`breeding.rs:49`) and `continue`s if `realize` returns `InsufficientMass`
(`breeding.rs:65`) — no matter is created and no entropy spent, so the world
stays sound, but the lineage simply stops breeding. A body whose mean part falls
below **5 voxels** is permanently sterile; below **15.2 voxels** it breeds later
than the gate TD8 tuned. Both are far away at 33 parts. Neither is far away at
300.

**The conclusion of this section: the render budget binds first, by a wide
margin, and the economy does not need re-deriving.**

---

## 3. The render budget

### 3.1 The arithmetic that exists

`mesocosm-lens` traces fragment-only for downlevel reach, so poses live in a
**uniform**, and the binding it must fit is
`Limits::downlevel_webgl2_defaults().max_uniform_buffer_binding_size` =
**16,384 bytes** (`lib.rs:42-80`; `g2_glprobe` requests those limits, so it is a
live ceiling). Two structures share the budget
(`crates/mesocosm-lens/src/tracer/params.rs`):

- `TraceParams` = camera 64 + space + fog 16 + look 16 + `clip_from_world` 64 +
  `CritterParams`. `CritterParams` is bounds 16 + tint 16 + eyes 32 + `32 ×
  MAX_CAPSULES` = **3,136 B at 96**. Header and pose together ≈ 3.4 KiB, about
  **21% of the downlevel limit**.
- the roster buffer = 16 + `MAX_ROSTER × RosterPose`, `RosterPose` = 16 + 16 +
  `32 × MAX_ROSTER_CAPSULES` = **352 B at 10**; total `16 + 40 × 352` =
  **14,096 B, 86.0%** of the downlevel limit.

So with `M` members at `C` capsules each the roster costs `16 + M(32 + 32C)`,
and the budget is:

```text
downlevel (16,384 B):  M × (C + 1) ≤ 511
desktop   (65,536 B):  M × (C + 1) ≤ 2,047
```

| configuration | M × (C+1) | downlevel | desktop |
| --- | ---: | --- | --- |
| 40 members × 10 capsules (today) | 440 | 86% ✓ | 21% ✓ |
| 40 × 11 | 480 | 94% ✓ | 23% ✓ |
| **40 × 33 (an archetype roster)** | **1,360** | **266% ✗** | **66% ✓** |
| 24 × 33 | 816 | 160% ✗ | 40% ✓ |
| 15 × 33 | 510 | 100% ✓ | 25% ✓ |
| 40 × 50 | 2,040 | ✗ | 100% ✓ |

### 3.2 Three facts this exposes

**The played pose's 96 is arbitrary.** `TraceParams` spends only 21% of the
downlevel limit at `MAX_CAPSULES = 96`; the headroom allows roughly **500**
capsules before the downlevel binding is reached. 96 is not a hardware number.

**Past 96 living parts a body is silently invisible.** `BodyLensProjection::project`
returns `TooManyCapsules` (`lens/src/body.rs:74`), and `pose_at` in the host
swallows it — `BodyLensProjection::project(...).ok()` (`genet/src/section.rs:475`),
documented as "a body past the lens's capsule limit yields `None` and the
section traces without it rather than refusing the frame". The first playtest's
body went **52 → 61 parts in ten meals**; five parts a meal reaches 96 in seven
more. This is a code-verified mechanism for the second playtest's finding, *"the
followed critter disappeared while he was not dead"*, whose cause that plan
records as unknown. It is a candidate, not a confirmation: proving it means
replaying `mark_playtest2` and reading the part count on the frame the body
vanished.

**Roster truncation drops the wrong ten parts.** `RosterPose::from_pose` takes
`pose.capsules.iter().take(10)` in document order (`params.rs:121`), and document
order is `BodyDocument::living()`, which is the axial chain from the root
outward. A truncated body is therefore its head end, not its silhouette. At 15
parts that loses a third; at 33 it loses 70% and every non-played critter in the
terrarium reads as a stump — the exact opposite of what the ruling asks for.

### 3.3 What is proposed

Staged, cheapest first. The last stage is Mark's.

- **DC-R1 (free, no budget change).** Order a roster member's capsules by
  descending radius before truncating, so ten capsules spend themselves on the
  silhouette. Surface `capsules_dropped` beside the existing
  `BrickDiagnostics::roster_dropped`, and make `pose_at`'s `None` loud in the
  receipt rather than silent on the frame.
- **DC-R2 (cheap, stays downlevel-safe).** Raise `MAX_CAPSULES` from 96 to
  **256**. Cost: `TraceParams` goes ≈3.4 KiB → ≈8.5 KiB, **52% of the downlevel
  limit**, still fits. This retires the vanish for a played body carrying an
  archetype plus a long career of meals. Raise `MAX_ROSTER_CAPSULES` 10 → 11,
  which is all the downlevel roster budget has (94%).
- **DC-R3 (the ruling).** The downlevel roster budget cannot carry
  many-small-voxel bodies at forty members — it is short by 2.7×, and no
  rearrangement of 511 fixes that. Two answers:
  - **(a) Profile the roster.** Make roster capacity a construction parameter
    with a downlevel-safe default and a desktop profile; Mesocosm's genet host
    requests desktop and gets **40 × 33 at 66% of the desktop limit**, with room
    to reach 40 × 50. Cheap, reversible, and it keeps `mesocosm-lens` honest for
    a future browser tenant instead of quietly dropping the WebGL2 reach the
    tracer was built for. **This is my recommendation.**
  - **(b) Take bodies off the uniform path.** Bodies are voxel volumes and the
    tracer already marches a brick atlas for terrain; voxelizing bodies into a
    second small atlas removes the uniform limit entirely and is what "made of
    many small voxels" literally asks for. It needs per-frame paging for moving
    bodies, whose nearest existing pattern is S2's Paredros-proven
    `BrickMap::with_capacity`/`retarget`. This is the honest long answer and it
    belongs on the scale ladder beside S2/S4, not inside this plan.

  Recommendation: **(a) now, (b) recorded as scale-ladder work.**
- **Colour is a separate question and is flagged, not answered.** A pose carries
  **one tint** (`BodyPlacement::tint`, `CritterParams::tint_count`), so a body is
  one colour whatever its parts. The Isometry bar is a **seven-entry palette on
  one figure**. Three options: a four-entry per-pose palette indexed by the
  part's `Role` (+64 B per pose, derived, no authoring); a full per-capsule tint
  (+16 B per capsule, +50% on the roster — 40 × 33 then needs 2,020 of the
  desktop 2,047, i.e. exactly at the wall); or leave it single-tinted. The first
  is cheap and gets legs, plates and eyes reading differently from the trunk.
  Mark's.

**Note on the existing open ruling.** ~~The rulings register §10 ("Decide how
to lift the 40-body roster cap") asks whether to raise `MAX_ROSTER` or hand
far bodies to silhouettes.~~ **Ruled by Mark, 2026-09-02: far bodies go to
silhouettes** rather than raising the capsule caps; it lands with S4 zoom.
That question is about *how many bodies*; this one is about *how much of each
body*. They share the same 511/2,047 budget and were ruled together.

---

## 4. The archetype roster

### 4.1 The machinery change archetypes require

**`PartPalette` admits exactly four shapes per world** — one template per `Role`
(`development.rs:32`), and `develop_body` reads `palette.template(role)` for
every part it makes. **A body today cannot contain more than four distinct
shapes**, which is why bodies read as repeated blocks and why no amount of extra
segments reaches the bar. Isometry's reference figure (`isometry-voxel::demo::hero`,
a 10×24×8 humanoid) is built from **fourteen axis-aligned box fills of eight
distinct extents**, plus two one-voxel eyes and a seven-colour palette. Shape
vocabulary, not part count, is the gap.

The minimal change that keeps one authority:

- **`PartPalette` widens to a small fixed array per role** — e.g.
  `mass: [Option<PartTemplate>; 4]` and likewise for limb, plate and sensor —
  staying `Copy`, staying bounded, staying snapshotted with the world. It
  remains world state, which is its stated purpose ("keeping the palette as
  world state means another world can admit different materials without changing
  a lineage's recipe", `development.rs:41`), and it keeps `plan.rs`'s intent
  honest: *if nothing long and thin lives in your world, you cannot grow limbs*.
- **`Tagma` gains two shape selectors** — which template its segments are made
  of and which its appendages are — as small indices into that palette. This is
  the same kind of field `per_segment` already is, it stays serde-stable, and it
  keeps `Recipe` the single heritable program.
- **`PartPalette::validate` gains the build-price check** from §2.3.

Rejected: an archetype format that builds a `BodyDocument` directly. That is the
second authority, and it re-opens the split account PD0 closed.

**This reorders the RNG stream and every existing hash breaks.** Accepted, as it
has been every round this week (scale plan: "hashes break by design"). Fixtures
re-record.

### 4.2 How many, and which

Sized by the founding plan's own Vesta reading — *complexity from specificity,
not quantity*, roughly a dozen organisms each carrying one idea followed
through — and by two structural pressures. Genesis founds exactly **three**
non-played species today (`genesis.rs:91`, one per kingdom), which is why TD10
found kinship's discount cancels: each tier is one interbreeding species. And
the pyramid founds **610 / 229 / 77** producers / consumers / decomposers, so
producer variety is most of what a capture shows.

Proposed: **eight archetypes**, and the number is Mark's.

| tier | count | what each is for |
| --- | ---: | --- |
| Producer (`Symmetry::Radial`) | 3 | a low ground mat; a mid many-fronded shrub; a tall single-stalk with a crown. All Mass parts, so all three are economically free (§2.4) and they carry the terrarium's whole visual field. |
| Consumer (`Symmetry::Bilateral`) | 3 | a browsing hexapod; a low fast pursuit form; a small armoured opportunist. See §4.3 — the grazer/predator distinction is currently blocked. |
| Decomposer (`Symmetry::None`) | 2 | a spreading crust; a mobile detritivore that walks to carrion. |

**No names are proposed.** Naming rounds in this repo are deliberate and carry
crates.io / game / studio / trademark checks (`CLAUDE.md`, Terminology). The
slices below use role-descriptive identifiers (`producer_mat`,
`consumer_browser`, `decomposer_crust`) and **the naming round is flagged as
Mark's** — including whether these want in-world names at all, given that a
founding lineage is deliberately unnamed (`species.rs:66`: "they were there
before anybody arrived to name them").

### 4.3 Two blockers the roster runs straight into

**Every limbed consumer is a predator.** `Organism::feeding_mode`
(`organism.rs:284`) reads `Kingdom::Consumer if self.body.performs(Process::Contract)
=> FeedingMode::Predator`, else `Grazer`. Grazer-vs-predator is today *exactly*
"has a limb or not". The ruling says every archetype presumes limbs — so an
authored grazer becomes a predator and the grazer disappears from the world.
This must be ruled before the consumer archetypes are authored. Options, with
the legwork done:

- Read a different anatomy fact: a mouth's *geometry*. `Appendage::Mouth` maps
  to `Role::Mass` today (`axis.rs:95`); a jaw-shaped mouth (a Limb-classified
  part at the head) versus a cropping mouth (a Mass one) would make the
  distinction geometric, which is the anti-Spore posture the whole role system
  takes. Costs: a mouth that classifies as Limb also adds actuator span, so a
  predator would pay and earn more than a grazer — arguably correct.
- Read the lexicon: a line that can express a jaw kind. Ties feeding mode to
  acquisition, which is the kleptoplasty loop.
- Declare it: split `Kingdom::Consumer`, or carry feeding mode on the `Species`.
  Cheapest, and the one that most looks like a stat flag.

**Symmetry *is* kingdom.** `Kingdom::from_symmetry` (`organism.rs:73`) maps
Radial → Producer, Bilateral → Consumer, None → Decomposer, and `Organism::kingdom`
reads it off `body.plan.symmetry`. So an archetype's silhouette symmetry is not
free: a radial predator or a bilateral plant is currently unrepresentable, and
`BodyPlan::mirrors` only pairs appendages for `Bilateral`, so producers and
decomposers get no mirrored growth at all. Authoring will hit this immediately —
a radial frond wants its fronds around the stalk, which `BodyPlan` cannot
express. Whether symmetry stays the kingdom reading is Mark's; it is a
one-reading change with a wide blast radius (`rates.rs` tests, `genesis.rs`, the
minimap's lineage colouring).

---

## 5. What founding becomes

The shape of `World::with_development_palette` (`world/genesis.rs`) is kept, and
so is every guarantee it currently makes:

- **The pyramid survives exactly.** `pyramid(count)` still composes 2/3 : 1/4 :
  rest, still asserted per seed at 610 / 229 / 77.
- **The kingdom floor survives.** Every tier is still founded.
- **The age and gestation stagger survive**, still proportional to each
  founder's own `lifespan_for_mass` / `gestation_for_mass`.
- **The seeded-stream discipline survives.** Recipes still draw from their own
  salted stream (`RECIPE_SALT`) so body generation never advances the ecology
  stream, and the founder's development seed still comes from `DEVELOPMENT_SALT`.

What changes is one step. Where genesis today maps one species per kingdom and
calls `axis::seed`, it instead **draws an archetype from that tier's archetype
list** off the same salted stream, and installs the archetype's recipe. Species
ids run `2..=(1 + archetype_count)` instead of `2..=4`, so the world founds eight
lineages rather than three — which is also what TD10's structural finding wanted.

**Individual variation still comes from `Soma::develop`**, which is the existing
answer and needs no new mechanism: each founder varies its per-tagma segment
count by the recipe's `variance` and may lose an appendage to developmental
absence. An archetype should set `variance` deliberately rather than inheriting
`axis::seed`'s `1 + rng.below(2)` — a hexapod with a varying leg count is a
different creature, while a frond with a varying frond count is the same plant.

The played critter (`OrganismId(0)`, `SpeciesId(1)`) currently founds with
`Recipe::default_founding()` — one bare four-segment stretch, the simplest body
there is. Under this plan it founds from a consumer archetype like everything
else. Whether the player should start from the *same* archetype every seed, or
draw one, is Mark's.

---

## 6. Slices, with done-conditions

Each slice lands with a receipt in `Code/testing/mesocosm/` and re-recorded
fixtures. Conservation (`cargo test -p mesocosm-core --test matter`) is the gate
on every one of them, at any part scale.

### DC1 — the palette learns more than four shapes

`PartPalette` widens to a fixed array per role; `Tagma` carries shape selectors;
`validate` gains the build-price assertion from §2.3. No archetype yet: the
primitive palette is expressed in the new shape and every existing recipe reads
the same templates it read before.

**Done when:** the primitive palette in the new form produces bodies
**bit-identical** to today's for every seed 1–10 at genesis (the null change is
the receipt); `validate` refuses a Limb template whose build price exceeds 6.25
and a Sensor above 4.76, with a test for each; conservation exact; workspace
tests and `clippy -D warnings` clean.

### DC1.5 — kingdom unbinds from symmetry

Added by §6.5's second ruling, and ordered before any archetype because every
archetype's recipe depends on which anatomy makes it what it is. A fourth
`Process` expressed by `Role::Plate`; `Kingdom` and `FeedingMode` read off
feeding organs; `Kingdom::from_symmetry` retired and `Symmetry` left as pure
body-plan geometry; `axis::seed` drawing a body that reads as the tier it was
asked for, since the pyramid no longer authors a `Kingdom` field.

**Done when:** every founding body in seeds 1–10 reads the kingdom its tier
drew, asserted as a test rather than probed; both consumer readings are
reachable at founding; conservation exact; the ten-seed instrument verdicts no
worse than DC1's (0 breathes / 10 thins / 0 boil / 0 collapse), with movement in
either direction reported rather than absorbed; fixtures re-recorded (hashes
move by design) and `--replay` exiting 0 against a re-recorded trace and 1
against a falsified one; workspace tests, `clippy -D warnings` and `fmt` clean;
and the decomposer's anatomical reading either found and stated, or returned to
Mark unbuilt rather than shipped as a stat flag.

### DC2 — one archetype, measured against the economy

The `consumer_browser` archetype of §2.4's carving B, authored as a `Recipe`
plus its palette entries, installed for the consumer tier only. Everything else
founds as it does today, so the arm is isolable.

**Done when:** the instrument (`population_instrument.rs`) runs its ten seeds
with the archetype arm on and off; the measured `mass_ceiling_mg`,
`actuator_span`, `sensor_span`, build multiple, rent, bite and sight of a
founded archetype body **match §2.4's table**, with any divergence explained
rather than absorbed; the ten-seed verdicts are no worse than TD11's (thins is
the standing baseline, not breathes — the TD ruling retired breathes as a gate);
total matter identical across every sample; **no `REFERENCE_*` or `*_BASE`
constant is touched**, and if one has to be, the slice stops and returns to Mark.

### DC3 — the render budget

DC-R1 and DC-R2 from §3.3; DC-R3 waits on the ruling.

**Done when:** a roster member truncated to its capsule budget is truncated by
descending radius, proven by a capture pair at the same seed and frame;
`MAX_CAPSULES` is 256 and a headed run confirms the uniform binding is accepted
at the downlevel limits `g2_glprobe` requests; a body pushed past the cap is
**reported** in the host receipt rather than silently unposed; the played-slice
demo trace replays to its hash unchanged (this is all presentation and cannot
reach a trace).

### DC4 — the eight archetypes, and the Isometry bar

The full roster, genesis founding one lineage per archetype, the pyramid and
floor asserted unchanged.

**Done when:** every seed 1–10 founds all eight lineages with the pyramid intact
(610/229/77) and the kingdom floor holding; **every founded fauna body has at
least one part performing `Sense` and at least one performing `Contract`** —
TD11's census, run again, reading 3,070 of 3,070 instead of 307; the ten-seed
instrument verdicts and matter conservation hold; and the receipt is a set of
headed captures at the ruled framing that **Mark looks at and calls critters**.
That last condition is a judgment and no test supplies it, exactly as P0's was.

### DC5 — colour, if ruled

Whichever of §3.3's three colour options Mark takes.

**Done when:** the ruled option is implemented, the budget arithmetic in
`lib.rs`'s cap comment is updated to match, and a capture shows a body whose
legs, plates and sense organs read differently from its trunk.

---

## 6.5 The two blockers, ruled (2026-08-30, Mark)

Both §4.3 blockers were ruled the day the plan landed:

- **Grazer vs predator: mouth geometry.** A jaw — a Limb-classified mouth
  part at the head — makes a predator; a cropping, Mass-classified mouth
  makes a grazer. Geometric, the same anti-Spore posture the role system
  takes, and a jaw adds actuator span so a predator pays and earns more,
  which is correct.
- **Kingdom unbinds from symmetry.** The deeper fix, taken deliberately:
  kingdom becomes a reading of feeding anatomy rather than of
  `body.plan.symmetry`, and symmetry becomes pure body-plan geometry. This
  pulls the forms brief's Stage 1 in as a dependency — a producer must be
  readable from *fixing anatomy*, and `Role::Plate` ("Fins, plates,
  leaves") is the already-named geometry for it. The decomposer's anatomy
  reading is the open design question the implementing slice must surface
  if no honest reading exists. Known consequences, priced in the forms
  brief's Stage 1 cost list and accepted: replay hashes move
  (`feeding_mode` reaches `state_hash` via the fauna decision trace),
  `CohortKey` semantics, `betrays_itself`, and guise's referent. The
  mixotroph-as-prey question (register §13) stays deferred — archetypes
  are single-mode, so edibility keeps reading the derived kingdom until a
  body can honestly do both.

The slice order gains a step: the unbinding is DC1.5 — after the palette
widens and before any archetype is authored, since every archetype's
recipe depends on which anatomy makes it what it is.

## 7. Open questions — Mark's

1. **How many archetypes, and are eight the right eight?** §4.2 proposes 3/3/2.
2. **What separates a grazer from a predator once every consumer has limbs?**
   §4.3, first blocker. This one blocks DC4 and has no defensible default.
3. **Does symmetry stay the kingdom reading?** §4.3, second blocker. It
   constrains every archetype's silhouette and it is not obviously worth its
   cost.
4. **The render ruling: profile the roster for desktop, or move bodies off the
   uniform path?** §3.3 DC-R3. Recommendation on record; the choice is not mine.
   Rule it together with the rulings register's §10.
5. **Colour: per-role palette, per-capsule tint, or stay single-tinted?** §3.3.
6. **Does the played critter start from a fixed archetype or a drawn one?** §5.
7. **Do the archetypes get in-world names, or stay unnamed the way founding
   lineages deliberately are?** §4.2. Either way the naming round is yours; no
   names are proposed here.
8. **Does `axis::seed` stay in the tree as the soup world's generator, or go?**
   §1 assumes it stays. Since DC4 it has no caller in the shipping founding —
   only `Founding::Drawn`, which the instrument keeps as its baseline arm.
9. ~~Which axis does the section look down?~~ **Ruled by Mark, 2026-09-02:
   measure before ruling.** (DC4) The terrarium view looks along `-z` and a
   body's segments chain along `+z`, so every critter is drawn end-on and the
   roster's whole part budget is invisible. Depth is illegible with bodies
   drawn end-on. A measured presentation slice renders the same tick under
   three cameras through the DT4 scenario harness before ruling: the section
   turned so bodies chain across the view; a shallow oblique tilt; and a
   rotatable local isometric cube with the interior cut (the sibling Isometry
   projection, which would need its own brief against the 2026-08-18
   first-person ruling). This is the cheapest legibility the plan has left
   and it is presentation only, a yaw when the section poses a body, or a
   camera that is not axis-aligned. Findings, DC4. **Measured 2026-09-04**:
   `--camera side|across|oblique` over one tick of the golden replay, three
   captures and a contact sheet, with the isometric cube priced rather than
   built. **Ruled by Mark 2026-09-04: oblique.** The section keeps its
   framing and tilts twenty degrees in yaw and pitch, so part budgets count
   and depth reads as a stagger. The presentation slice that follows makes
   oblique the default, fixes the ground's top-face palette (painted
   lavender by a grade a level camera never exercised), and looks at the
   near ground occluding the lowest bodies. **Landed 2026-09-04**: oblique
   is the default, the palette is fixed at the grade with `side` still
   byte-identical, and the occlusion cost measured away at 0 of 40 bodies
   more than half hidden under either the level or the tilted section.
   Closed.
10. ~~Is the roster's ecology acceptable, and if not which half moves?~~
    **Ruled by Mark, 2026-09-02: fixed by a body, not a constant; a diagnosis
    slice measures first.** (DC4) Eight of ten seeds collapse against the
    drawn baseline's zero. The two diagnostic arms in the Progress entry say
    which half of the roster carries it; the fix is a body, per the stop
    rule against moving a constant, and the diagnosis slice runs before the
    fix is chosen. **Diagnosed 2026-09-04**, Findings, DC4 Q10. The stand is
    innocent — no producer body moves a seed — and the producer tier's
    `income - rent - outflow` is the verdict, negative in every collapsing
    seed and positive in every seed that holds. The roster's only jaw carries
    it: a `FeedingMode::Predator` has no kingdom restriction on its prey, so
    `consumer_pursuit` grazes the stand at the roster's largest bite, and
    replacing those 76 bodies cuts grazing 61% and flips the balance. Both
    walls were measured on ten lucky seeds: over thirty the roster collapses
    17/30 and the drawn baseline 1/30. **No body change measured holds the
    corridor** — the best halves the failure rate to 9/30. Two candidates are
    proposed and neither is applied; the choice between their costs (D
    deletes predation, F is the only arm that ever breathes) is Mark's, and
    so is whether the prey-set rule itself should move.

---

## 8. What this plan does not do

- **The update stage.** The other half of the closing ruling — each lineage
  evaluating and refining its bodyplan and trajectory at the epoch boundary — is
  PS2 and the epoch boundary plan's, not this one. This plan gives that stage
  something worth refining; it does not build it.
- **Evolution mechanics.** NPC lineages still never mutate their recipes
  (`World::learn_from` returns early for anything but the controlled critter).
  That is the traits brief's incorporation half and stays open.
- **New part types.** No new `Role`, no new `Appendage`, no new `Process`. The
  archetypes are built from the four roles and six appendages that exist. If an
  archetype genuinely cannot be expressed, that is a finding to bring back, not
  a licence to add a role.
- **World size, framing, or the scale ladder.** S2/S3/S4 are untouched. The
  traced-body option in §3.3(b) is *recorded* for that ladder, not scheduled
  here.
- **The economy's constants.** DC2's done-condition is explicitly that no
  `REFERENCE_*` or `*_BASE` constant moves. If the measurement says one must,
  the slice stops.

---

## Findings

- **2026-09-04 (DC4, Q10): the diagnosis. The stand is innocent; the roster's
  only jaw carries the collapse; and both walls of the corridor were measured
  on ten lucky seeds.** Measurement only — no constant moved, nothing in the
  ecology was built, and no roster body was changed. The instrument's own
  arithmetic was restated in a scratch harness and validated against the
  shipped receipt before anything was read off it: the roster arm reproduces
  `dc4_roster.json` exactly, per seed, including every end state and every
  decision tick (0 breathes / 2 thins / 8 collapse; baseline 0/10/0/0).

  **The account that goes negative is the producer tier's, and its sign is
  the verdict.** Per-lineage income, rent and outflow were reduced off the
  flow record by the same split `flow::Accounts` makes. In all eight
  collapsing seeds the stand's peak headcount is *exactly* its founding 610 —
  it never adds one net body — and `income - rent - outflow` lands in a tight
  band from **-332k to -363k mg**. In the two that hold it is **+652k and
  +1,016k**, and the stand grows to 832 and 1,035. Producer births are 110-373
  against 2,267 and 3,239. The founding stand then ages out together at
  ~2,400 ticks (`lifespan_for_mass` at the `100 + rng.below(400)` founder
  draw), which is exactly when the producer lines die. Matter is conserved to
  the milligram in all ten; in the collapsing seeds ~2.20M of the world's
  2.21M mg ends in the soil with nothing left to take it up. **This is a
  conversion failure, not a matter shortage.**

  | seed | verdict | first line to die | stand peak | producer balance |
  | ---: | --- | --- | ---: | ---: |
  | 1 | thins | consumer_pursuit @2686 | 832 | +1,016,284 |
  | 2 | collapse | consumer_pursuit @2585 | 610 | -363,039 |
  | 3 | collapse | producer_stalk @1850 | 610 | -347,665 |
  | 4 | collapse | decomposer_crust @1396 | 610 | -331,914 |
  | 5 | collapse | decomposer_crust @1272 | 610 | -362,542 |
  | 6 | collapse | consumer_pursuit @2126 | 610 | -359,993 |
  | 7 | thins | consumer_pursuit @4704 | 1,035 | +651,725 |
  | 8 | collapse | decomposer_crust @353 | 610 | -355,359 |
  | 9 | collapse | consumer_pursuit @1694 | 610 | -345,844 |
  | 10 | collapse | decomposer_crust @1664 | 610 | -356,432 |

  `consumer_pursuit` is the first line to die in 7 of 10 seeds; the armoured
  is the last survivor in 9 of 10.

  **Attribution by ablation, with a positive control.** A founding lineage's
  recipe is rewritten to a sibling archetype's immediately after founding and
  its founders are re-grown under the same development seed and drawn mass.
  Breeding is asexual (`breeding::bear` takes one parent), so this is
  population-identical to founding the tier without that archetype: same
  founder count, same positions, same tier shares, only the body moves. **All
  eight no-op ablations (each lineage rewritten with its own recipe)
  reproduce the roster arm to the tick**, which is what licenses the rest.

  | one body changed | 1-10 | 11-30 | **all 30** |
  | --- | ---: | ---: | ---: |
  | roster as it ships | 8 | 9 | **17** |
  | any producer archetype to a sibling | 8 | — | — |
  | consumer_browser to pursuit | 8 | — | — |
  | consumer_armoured to browser | 8 | — | — |
  | **consumer_pursuit to armoured** | **5** | **4** | **9** |
  | decomposer_crust to detritivore | 5 | 10 | 15 |
  | decomposer_detritivore to crust | 6 | 9 | 15 |

  **Three things the plan did not expect.**

  *(1) A predator eats the stand.* `movement.rs:70` filters a Grazer to
  `target.kingdom == Kingdom::Producer`; `perception.rs:206` reads
  `mode == FeedingMode::Predator || target.kingdom == Kingdom::Producer`. **A
  Predator has no kingdom restriction on its prey at all.** `consumer_pursuit`
  is the roster's only predator, founds 76 of 229 consumers, and carries the
  largest bite in the roster (57 mg against 49 and 43) — so a third of the
  consumer tier grazes the stand harder than either dedicated grazer *and*
  preys on the fauna. Replacing those 76 bodies cuts grazing off the stand by
  **61%** (15.37M to 6.02M mg over ten seeds and 3,000 ticks) and flips the
  producer balance from **-1,803,710 to +334,317**. §6.5's grazer/predator
  ruling — "a jaw makes a predator" — is one shape selector on one recipe, and
  it silently bought that lineage an unrestricted prey set on top of a bigger
  bite. **This is a reading of the ecology as it stands, not a proposal to
  change it**; the prey-set rule is Mark's.

  *(2) The stand is innocent at the body level too.* **No producer body moves
  a single seed**, which is the stand-only arm's verdict re-measured one
  archetype at a time. And blinding the grazers does not relieve the stand —
  it roughly halves the time to collapse (browser feelers off: 900-1500
  against the roster's 2200-3200; armoured feelers off: 700-2200), across all
  eight collapsing seeds. So "an authored animal is a capable animal" is not
  the mechanism the 2026-08-31 entry read it as. Capability helps.

  *(3) Both walls were measured on ten lucky seeds.* Over thirty seeds the
  roster collapses **17/30 (57%)**, not 8/10, and the drawn baseline reads
  **1 breathes / 28 thins / 1 collapse**, not 0/10/0/0. The corridor is real —
  1 in 30 against 17 in 30 — but Q10's "eight of ten against zero" overstates
  both walls, and the ten-seed instrument cannot tell a three-seed move from
  luck. Two candidates that looked like three-seed movers in-sample
  (`decomposer_crust`, `decomposer_detritivore`) do not replicate at all.

  **The part-budget steps, over all thirty seeds.**

  | candidate | breathes | thins | **collapse /30** |
  | --- | ---: | ---: | ---: |
  | A roster as it ships | 0 | 13 | **17** |
  | B pursuit takes the armoured's whole body | 0 | 21 | **9** |
  | C pursuit jaw to crop only | 0 | 16 | 14 |
  | D **pursuit jaw to crop, and -1 leg pair** | 0 | 21 | **9** |
  | E pursuit keeps its jaw, -1 leg pair | 2 | 13 | 15 |
  | F **every consumer -1 leg pair** | **5** | 15 | **10** |

  **Proposal, and it does not hold the corridor.** No body change measured
  holds it: the corridor's floor is zero collapse and the best arm is 9 in 30,
  so what follows halves the failure rate rather than closing it, and saying
  otherwise would be dressing up the measurement. Two candidates are worth
  ruling between, and they trade against each other rather than ranking. **D
  is the smallest**: two tokens on one archetype — `consumer_pursuit`'s mouth
  from `SHAPE_JAW` to the `Mass` bank's `SHAPE_CROP`, and one leg-bearing
  segment off its first `Limb` stretch (span 22 to 12, bite 57 to ~44, gate
  514). It matches the whole-body swap at 9/30 while keeping eight distinct
  bodies; it moves seeds 2, 6, 8, 12, 14, 15, 16, 17 and does not move 3, 4,
  5, 9, 10, 11, 21, 26; visually it costs one leg pair (six legs to four) and
  a mouth part changing from `[4,1,1]` to `[2,1,0]`, which is nothing at the
  ruled framing. **Its real cost is that it deletes predation from the
  world**, which §6.5 ruled in deliberately, so it buys the corridor by giving
  up a design commitment. **F is the only arm that ever breathes**: one leg
  pair off all three consumers reads 10/30 collapse and **5 breathes**, where
  the roster, the drawn baseline and every other arm read 0, 1 and 0-2. It
  keeps the jaw, keeps predation and keeps all eight archetypes; it costs a
  visible leg pair on each consumer, and the armoured drops from four legs to
  two, which will read oddly on a body whose whole idea is that it is low and
  covered. **Neither is applied to the shipped roster.** Which cost is the
  acceptable one is a design call, not a measurement, and it is Mark's.

  **Receipts.** `cargo test -p mesocosm-core --release` green; `cargo clippy
  --workspace --all-targets -D warnings` clean; `cargo fmt --all --check`
  clean. No file under `src/` changed — the diagnosis ran entirely through the
  public API (`World::founded`, `lineages_mut`, `Species::realize`,
  `World::flows`), in two scratch examples deleted at the end of the slice, so
  the tree carries no new harness. Every run wrote to explicit scratch paths;
  `dc4_roster.json` and the golden `ps1_played.*` fixture were read and never
  written, and all five are md5-identical to backups taken before the slice
  (`dc4_roster.json` `82cb5f0d94924f297eae22d7ec562ded`).

- **2026-09-04 (DC4, Q9): oblique landed as the default, and the two costs the
  measurement named settled — one fixed, one measured away.** Mark ruled
  `oblique` off the sheet below the same day; this entry is what shipping it
  found. `--camera` keeps all three arms, because they are the evidence the
  ruling rests on; the default moved, and `--help`, the module docs, the
  receipt and the scenario snapshot moved with it. The 2026-08-18 vessel briefs
  carry a dated amendment under their Mesocosm camera ruling: the word
  "side-on" is superseded, everything the ruling gives as grounds is not.

  **The top-face palette, and why it was never a tracer fault.** The retro
  grade quantises red, green and blue on three *independent* five-step ramps.
  That is not a palette — it is a lattice — and a colour crosses its three
  boundaries at three different brightnesses. For the faces a level section
  draws it never showed: they are dark, and their channels sit far apart. A top
  face is lit from above (`0.38 + 0.62 cos` against the sun reaches 0.906,
  where a vertical face reaches 0.58-0.64) and is then mixed toward the fog
  colour `(0.66, 0.66, 0.72)`, which is *cool*; soil's blue starts furthest
  from it, so blue climbs fastest and takes a step the red and green have not.
  The whole sequence was predicted from the two material constants and matched
  the capture exactly: a soil top face walked `(64,64,0)` olive, `(64,64,64)`,
  `(127,127,127)`, then `(191,127,191)` pink-violet at the fifth of six fog
  bands; rock reached blue-violet `(64,64,127)` at the first. That is the
  lavender, and it is a hue neither material has anywhere.

  **The fix is a second ladder, not a second palette.** Ground seen from above
  now climbs the same five steps along its own colour line — the brightness is
  stepped and the hue is left alone, so a step is always a step in light and
  never a step in hue. Soil lit from above lands on `(127,80,44)`, a lighter
  warmer brown of the same soil, and holds its family through the fog bands
  instead of crossing out of it; rock lands on `(102,108,127)`, its own cool
  grey. The look stays starved: the reachable colours are (material hue) x (fog
  band) x (step), which is smaller and more palette-like than the RGB lattice
  it replaces. **Vertical faces are untouched.**

  **The gate had to be measured too, and the first one was wrong.** The obvious
  test — the face normal points up — is not a level-camera no-op, because
  conatus's brick DDA seeds its normal to `+y` and returns that seed unchanged
  when a ray *begins* inside solid, which under the level section is the whole
  block of terrain reaching the camera's near plane. Gating on the normal alone
  moved 268k pixels of the shipped `side` capture. The gate is the normal
  **and** a downward ray: an orthographic section marches its forward, and
  `side` and `across` have `forward.y` exactly zero, so it is false for every
  pixel of a level frame by construction. `--camera side` then replays the
  golden fixture to a capture that is **byte-identical** to the `ps1_played.png`
  on record (sha256 `3b260e95...`), which is the receipt.

  **The occlusion cost was not real.** Measured rather than argued, at the same
  tick 816 the sheet used, by sampling seven points along every capsule of
  every rostered body and marching each sample back to the camera's own near
  plane through `Ground::solid`: **0 of 40 rostered bodies are more than half
  hidden under `oblique`, and 0 of 40 under `side`.** Oblique hides *less*, not
  more — mean hidden fraction 0.013 against side's 0.029, worst single body
  0.100 against 0.133, and no body over a quarter under either. The reason is
  geometric and obvious once measured: a camera tilted down reaches a body from
  above, and there is less ground above a body than beside it. The impression
  on the contact sheet was the near ground *filling the bottom of the frame*,
  which is framing working as intended rather than bodies being lost. **The
  instrument was proved before the negative was believed**: the same probe on
  the `across` arm reads 6 of 33 bodies more than half hidden, 23 over a
  quarter, mean 0.343, worst 0.743. So nothing moved — not the slab's vertical
  centre, not the follow-centre clamp. The probe was scaffolding and was
  deleted with the slice.

  **What the fix also changed, reported rather than hidden.** The band of
  terrain that reaches the camera's near plane — the rays that begin inside
  solid, carrying the DDA's fabricated `+y` — now takes the hue ladder too,
  because nothing in the shader can tell that normal from a real one. It reads
  as a flat cool slate `(102,108,127)` where it used to read dark grey, about a
  tenth of the frame along the bottom edge. It is the same surface either way
  and it is drawn in rock's own family, but it is lighter than it was. The
  honest name for the underlying problem is that a ray starting inside solid
  should not report a lit top face at all, and that lives in conatus's
  `brick_dda.wgsl`, not here. **Mark's**, if the band should read differently.

  **Receipts.** `cargo test -p mesocosm-core --release --test matter --test
  flows --test succession --test embodied` green (99 tests); `cargo test -p
  mesocosm-genet -p mesocosm-views -p mesocosm-runtime` green; `cargo test -p
  mesocosm-lens --lib` green (48, the three new grade tests among them — the
  lens *integration* suite is known-slow and was not run, so this is its lib
  tests only); `cargo test --workspace --exclude mesocosm-lens` green; clippy
  `-D warnings` clean in both profiles; `cargo fmt --all --check` clean. The
  golden replay exits 0 at `081b4ba4bdc46190` under the new default, and a
  scenario falsified on that hash exits 1. `ps1_played.png` was re-recorded
  under the default with an explicit path — it is the fixture's photograph and
  the fixture now looks different — and a fresh `oblique_default.png` was
  captured at tick 816 and read: part budgets legible (three and four capsules
  countable on individual bodies), ground warm and stepped with no violet, the
  vitals panel and the minimap intact. The golden trace, its hash and
  `dc4_roster.json` were read and never written.

  **The three grade tests are rendered, not reasoned**
  (`lens::tracer_tests::grade`). They drive the headless brick tracer at the
  section's own numbers: the tilted frame never splits green more than half a
  rung below both neighbours (the artefact is exactly one rung); soil lit from
  above actually reaches the screen as warm soil, which is the positive control
  the first assertion needs; and a level frame still lands on every rung of the
  per-channel ladder, which is the no-op proof and is what caught the first
  gate. The ordered dither is off in these three and only these three, because
  it scatters individual pixels onto neighbouring rungs by design and the tests
  are about which ladder a surface climbs. Forcing the old path fails the two
  tilted tests and leaves the level one passing, which is the discrimination
  check.

- **2026-09-04 (DC4, Q9): three cameras over one tick, measured. `oblique`
  is the only arm that shows a body's part budget *and* keeps the section.**
  The instrument is `--camera side|across|oblique`
  (`genet::section::camera`), a presentation flag beside `--slab`: it hands
  the tracer a different forward and up and changes nothing else. The
  2026-08-31 finding's `section::FORWARD` constant is now
  `CameraMode::forward`, and `side` reproduces it exactly.

  **What each mode is, mathematically.** World up stays `+y` in all three,
  and `TraceCamera::orthographic_slab` orthonormalizes what it is handed, so
  a mode is three unit vectors and no new pipeline. `side` looks down `-z`
  and is the shipped section unchanged. `across` looks down `-x`; screen
  right becomes `-z`, so a body's `+z` chain projects to the full screen
  horizontal. `oblique` yaws 20 degrees off `-z` **and** pitches 20 degrees
  down — both, because a yaw alone only shortens the chain along the
  horizontal and never makes a diagonal; the `+z` chain projects to
  `(-sin 20, -cos 20 sin 20)`, a diagonal 0.47 of the across span. The
  roster's cull window turned with the camera (it is now the camera's own
  oriented box, exact for all three and numerically identical to the old
  axis-aligned box under `side`), and the bedrock clamp now reads the
  camera's real vertical reach — 28 for the two level modes, 29.05 for
  `oblique`, which leans its slab depth into the vertical.

  **The captures.** One golden replay
  (`ps1_played.trace.json`) held at **tick 816**, live hash
  `35143e3984979d41`, under each mode:
  `camera_side.png`, `camera_across.png`, `camera_oblique.png`, composed as
  `camera_compare.png`. All three replays then ran the recording out and
  exited 0 at `081b4ba4bdc46190`, and all three receipts report roster 40.

  - **`side` (control).** Part budgets **illegible**: every body in the
    frame, played and rostered alike, reads as one fat capsule with at most
    a shoulder beside it — exactly the end-on finding, now on a photograph. Depth between
    bodies **does not read** — bodies at different `z` occlude with no
    stagger and no size cue, so a crowd is a row. What it keeps: the ground
    silhouette is the clearest of the three (stepped relief against sky, the
    vertical structure the section exists for) and there are no chrome
    collisions.
  - **`across`.** Part budgets are **there but not attributable**: chains do
    run the full screen width, but the slab's long axis is now `z`, so forty
    rostered bodies at one tint pack shoulder to shoulder into a continuous
    hedge and no body's own extent can be picked out of its neighbours'.
    Depth between bodies is **worse than `side`**, not better. The ground
    silhouette is **lost**: relief varies along `x`, which is now depth, so
    the terrain degenerates to a flat brown table ending in a hard vertical
    edge where the slab stops. Chrome does not collide, but the vitals panel
    now sits over solid hedge rather than sky.
  - **`oblique`.** Part budgets **legible**: three and four capsules can be
    counted stepping up-and-right on individual bodies. Depth between bodies
    **reads** — nearer bodies overlap further ones with a visible stagger,
    which neither level mode gives. Ground silhouette **survives**, as a
    staircase rather than a clean profile. What got worse: the ground's *top*
    faces come into view for the first time and the retro grade (palette 3)
    paints them lavender against the soil's dark red — a palette never
    exercised by a level camera, not a tracer fault, and it is the one thing
    on the sheet that looks like a bug; and the near ground now occludes the
    lowest bodies, which `side` never did.

  **Nothing is ruled here.** The three arms and their costs are on record;
  the camera is Mark's.

  **The isometric cube, priced rather than built (Q9's third option).** The
  sibling projection is `isometry-voxel::bake`, and it is not a camera: it is
  a CPU sprite baker that rotates a model about vertical in four 90-degree
  steps, projects each voxel's top vertex into a 2:1 grid and splats a
  three-tone cube stamp through a z-buffer, emitting an RGBA sheet. Its own
  module note says the live adjustable-camera modes are "a later, wgpu
  concern", so nothing there renders a world, and it has no notion of
  cutting an interior. Mesocosm would not borrow it; it would point its own
  tracer down `(-1,-1,-1)/sqrt 3`, which this slice's `CameraMode` already
  makes a one-line addition. The two pieces that are genuinely missing are
  (a) **the cube**: the tracer's ray interval clips *along the view axis*, so
  the shipped slab is a diagonal cut under an isometric camera rather than a
  world-aligned box around the played critter, and a rotatable cube whose
  contents do not change as it turns needs a world-box clip the tracer does
  not express; and (b) **rotation as an interaction**, which is input,
  a persisted host angle, and the follow-centre policy for a camera the
  player aims — the cull window follows for free, since it is already the
  camera's own oriented box. Against the 2026-08-18 ruling, "first person
  names the agency, never the camera" survives an isometric cube only if
  *rotating it is not a move*: the cube is a way of looking at where you
  already are, the way `--slab` and the arrow-key pan are, and it must not
  become a way of choosing where to be. The moment the cube's orientation
  decides what a critter can reach, sense or step onto, the camera has
  started naming the person and the ruling is broken. That is a brief, and
  this slice does not write it.

- **2026-08-31 (DC4): the section looks straight down every body's own axis,
  and that is why nothing reads as a critter.** `develop_body` chains segments
  along `+z`; the ruled terrarium section looks along `-z`
  (`genet::section::FORWARD`). So a thirty-part animal is drawn **end-on**: the
  browser at slab 28 is a green blob with two side stubs, and its whole length
  — the thing the roster spends its part budget on — is a single pixel column
  deep. Turning the same slab a quarter (`dc4_*.png` against
  `dc4_*_axial.png`) shows the body plan immediately. **Neither cut shows
  everything**: broadside shows the axial chain and hides the limb pairs,
  which point at the camera; axial shows the limbs and hides the chain. An
  orthographic section cannot show both, because the two are perpendicular by
  construction — which is exactly why DC2's capture used the menagerie's
  three-quarter camera. This is the cheapest legibility left in the plan and it
  is presentation, not anatomy: either the section carries a small yaw when it
  poses a body, or the camera stops being axis-aligned. **Mark's**, and it
  belongs beside DC5's colour question. **Ruled 2026-09-02: measure before
  ruling.** A measured presentation slice renders the same tick under three
  cameras through the DT4 scenario harness before ruling: the section turned
  so bodies chain across the view, a shallow oblique tilt, and a rotatable
  local isometric cube with the interior cut (the sibling Isometry
  projection, which would need its own brief against the 2026-08-18
  first-person ruling). See §7 question 9.

- **2026-08-31 (DC4): a tall stalk is not expressible, and the roster hit it on
  the first plant.** §4.2 asks for "a tall single-stalk with a crown". Segments
  chain along one axis and nothing stacks along `y`, so the only vertical a
  body has is the height of a single appendage: the tallest thing the palette
  can build is a `[4,4,1]` frond standing eight voxels over its segment.
  `producer_stalk` is therefore a sixteen-segment ground runner with a crown of
  fronds at one end, and it reads as a bead chain with a box on its head
  (`dc4_producer_stalk.png`). Raising a stalk needs a second growth axis —
  either a `Facing`-aware `develop_body` or a recipe that can branch — and that
  is a body-machinery change, not this slice's. Recorded rather than worked
  around.

- **2026-08-31 (DC4): the roster is a worse ecology than the browser alone, and
  the pursuit form is not the fix DC2's finding predicted.** Ten seeds:
  **0 breathes / 2 thins / 0 boil / 8 collapse**, against DC2's browser-alone
  arm re-measured at 0/6/0/4 and the drawn baseline's 0/10/0/0. Founding a
  predator beside the browser was the remedy DC2 named — a consumer tier that
  is one interbreeding species cannot hold both readings, so install three —
  and it does not land. What the end states say is that the failure changed
  shape rather than easing: the **baseline** ends with a stand and no consumers
  (nine seeds of ten end `1100-1500 / 0 / 40-160` P/C/D), while the **roster**
  ends with consumers and no stand (`0 / 3-12 / 0` in seven of ten). The world
  that founds capable fauna keeps them and loses the plants.

- **2026-08-31 (DC4): the whole regression is in the authored *fauna*; the
  authored stand carries none of it.** Two diagnostic arms split the roster in
  half. **Producers authored, fauna drawn: 0 breathes / 10 thins / 0 boil / 0
  collapse** — the baseline's verdicts exactly, and the standing producer counts
  are *higher* than the baseline's in nine seeds of ten (1,328-1,567 against
  1,140-1,534). **Fauna authored, stand drawn: 0 / 2 / 0 / 8** — the full
  roster's verdict, to the seed. So the archetype stand is a strict improvement
  and the archetype animals are what empty the world.

  And the reason is the ruling itself, arriving as a price. DC1.5 measured that
  the drawn consumer tier founds **blind** in nine seeds of ten (sensor span
  zero) and sessile in the tenth, and the drawn decomposer tier is the same
  lottery. The roster founds 306 non-played fauna a seed of which **every single
  one** has a working eye, something that contracts, and the mouth its line
  feeds by — which is DC4's second done-condition, met, and is also the thing
  that strips the stand. *Founding capable animals is the plan's whole point and
  it costs eight seeds in ten.* The remedy is a constant or a body, and rent,
  bite and the breeding gate are `REFERENCE_*` territory the stop rule forbids
  touching here, so this returns to Mark rather than being tuned away.
  `dc4_roster.json`.

- **2026-08-31 (DC4): armour is a position, and the position rule is four
  lines of geometry the body already carried.** The DC1.5 residue said no
  archetype could wear a shell without reading Producer. The rule Mark set —
  fixing reads only from plates in canopy positions — turns out to need nothing
  stored: a plate is in the canopy when its attachment offset points up rather
  than sideways, and when no living part's lowest voxel is above its highest.
  Both are read off `Attachment` and `half_extent`. `Yaw` turns about `y`, so a
  part's height is the plain sum of the `y` offsets up its chain and the whole
  reading is one forward pass in document order. `consumer_armoured` wears ten
  plates, performs `Process::Fix`, and reads **Consumer**; the three producers
  and every drawn fronded stretch still read Producer.
  `organism/kingdom.rs::canopy`.

- **2026-08-31 (DC4): the `Mass` and `Plate` banks are full at four, `Sensor`
  can only ever hold two, and the roster fits.** DC1 left `PALETTE_SHAPES` as
  Mark's budget question. Measured rather than guessed: the eight archetypes
  spend **4 of 4 `Mass`** (`[2,2,2]`, `[2,1,1]`, `[2,2,1]`, `[2,1,0]`), **4 of
  4 `Plate`** (`[4,4,1]`, `[3,3,0]`, `[4,0,4]`, `[0,4,4]`), **2 of 4 `Limb`**
  (`[4,1,1]` as the jaw, `[3,1,1]` as the leg) and **2 of 4 `Sensor`**. The
  `Sensor` spare is unspendable: the §2.3 build-price guard admits exactly
  `[1,1,1]` and `[0,0,0]` and refuses every other shape that classifies as one,
  so two is the whole vocabulary. The `Limb` spare is real but small — at a 3x3
  cross-section the guard admits only `[3,1,1]` and `[4,1,1]`, and anything
  longer has to be thicker. **The budget was not raised**; a ninth archetype
  wanting a new bulk or leaf shape is what would raise it.

- **2026-08-31 (DC4): the absence guard had to widen twice, and both widenings
  are the same argument.** DC1.5 stopped `Soma::develop` taking a mouth. DC4
  had to stop it taking a **lit plate** — a plant realized at one leafing
  segment loses its whole canopy and is born a decomposer — and then a
  **feeler**, because a line with one sensory stretch bears a blind founder
  about one birth in twelve, which is TD11's defect reintroduced by lottery.
  The rule that fell out is cleaner than the patches: **absence may take limbs,
  vanes and covering, and nothing a body's kingdom or its senses are read
  off.** Armour is variation; a leaf and an eye are not. Two archetypes also
  had to be reshaped around the same hazard rather than guarded — the crust's
  holdfasts went from two segments to three (at variance 1 a stretch of two can
  realize as one and then lose it) and the armoured form's two single-segment
  leg stretches became one of two, because absence is drawn per stretch and
  both can come up short. `axis.rs::Soma::develop`,
  `archetype::tests::development_cannot_blind_or_strand_a_fauna_archetype`.

- **2026-08-31 (DC4): the archetype-bodied played critter is a five-column
  walker and can no longer use the carve-a-doorway move.** `WalkerShape`
  derives a passage radius from the body's shorter horizontal extent, and the
  browser's leg pairs make that sixteen voxels — radius 2, a 5x5 footprint. A
  radius-2 body cannot *stand* beside a two-high face at all, because the face
  is inside its own footprint, so no carve can open a door for it: the world's
  own terrain offers no such stance in 129 x 129 columns. The fixture
  (`world::tests::a_carve_opens_a_grounded_step_and_replays`) now drives a
  standard walker on purpose. This is a gameplay consequence of the ruling,
  not a defect in carving, and it is worth a look before the roster ships:
  either the played body is smaller than the roster's, or a carve widens with
  the body that asks for it.

- **2026-08-31 (DC4): a developed body's segments overlap by one voxel plane,
  and the mesh path has never been asked about it.** `develop_body` places a
  child segment at `parent_half + own_half`, and a part of half-extent `h`
  spans `2h + 1` voxels, so consecutive segments share exactly one plane. The
  capsule projection does not care; `mesocosm_mesh::flatten` does, and loses a
  voxel per joint. It only surfaced now because the played critter used to
  found with a handful of parts and now founds with thirty-three
  (`genet::fixture::tests::parts_never_land_on_top_of_each_other`). Pre-existing
  and untouched; the fixture starts from a single root instead.

- **2026-08-31 (DC3): the vanish is confirmed, on a live body, by
  counterfactual.** §3.2 called it a candidate. A headed run grew the played
  critter to **349 living parts** and the trace of it replays exactly
  (`dc3_vanish.trace.json`, `97bf29f3ca1fd587`). At `df2a741` that replay draws
  **no body at all** while the receipt says `body_parts: 349` and the minimap
  shows the critter alive; after DC3 the same replay draws it truncated and
  reports the 93 parts it could not carry. Cause and cure both proven on the
  same trace. `dc3_vanish_before.png`, `dc3_vanish_after.png`.

- **2026-08-31 (DC3): the `mark_playtest2` trace no longer grows the body it
  grew.** DC1, DC1.5 and DC2 all moved founding, so the preserved trace
  diverges from its recorded hash immediately; its critter reaches 58 parts by
  frame 10 and is dead by frame 30, ending `body_parts: 0`. It is still a good
  world driver — the roster capture pair uses it — but it is no longer evidence
  about *that* body, and the vanish had to be re-grown to be shown. Playtest
  traces are worth re-recording when founding moves.

- **2026-08-31 (DC3): descending radius keeps the bulk and drops the fronds,
  and on a stand of producers that is not obviously prettier.** DC-R1 landed
  exactly as §3.3 specifies and the change is unambiguous — at one seed and
  frame the visible stalks go from about 25 to about 13 — but the parts it
  drops are the tall thin `[4,4,1]` fronds and the parts it keeps are the round
  root masses. Silhouette *area* is what the argument asks for and what it
  delivers; silhouette *reading* may want **extent** (`radius² × length`) as
  the key instead, which is the same four lines. Ordering key returned to Mark
  rather than substituted. `dc3_roster_before.png`, `dc3_roster_after.png`.

- **2026-08-31 (DC3): one capsule cannot carry the primitive `Plate`.** The
  radius rule fixed the archetype's six shapes but `[4,4,1]` is 4:1
  anisotropic, and the mean takes it from radius 1 to **2.5** — the standing
  crop goes from thin blades to thick columns. A geometric mean gives 2.0 and
  is no better; there is no single radius that represents a 9×9×3 slab. The
  honest answer is a second capsule for plate-shaped parts, which is a
  projection redesign and belongs with DC5's colour question rather than in
  DC3. `lens/src/body.rs::capsule_for`.

- **2026-08-31 (DC2): carving B's exactness came from its six dorsal plates,
  and post-DC1.5 a consumer cannot wear them.** `Role::Plate` performs
  `Process::Fix` now, so §2.4's "6 dorsal plates `[3,3,0]`" would make the
  browser read **Producer** rather than Consumer — the armour collision DC1.5
  named as a standing constraint, hit on the very first archetype exactly as it
  predicted. Replacing them is not free arithmetic either. Holding *both* of
  carving B's totals (1,611 voxels **and** 1,284 mg) requires some part whose
  voxel count is not a multiple of five, because `part_ceiling_mg` is
  `voxels * 4 / 5` floored and 4 x 1,611 / 5 is not an integer; the plates were
  the only fine-grained parts doing that. **The smallest `Mass` shape whose
  volume is not a multiple of five is `[3,3,3]` — 343 voxels, 274 mg**, a
  seventh of the whole body in one block, because every smaller candidate
  (`[3,1,1]`, `[3,3,1]`, `[1,1,1]`, `[3,3,0]`) classifies as Limb, Plate or
  Sensor rather than Mass. So the choice was 1,284 mg with a chunky body or a
  fine-grained body one or two voxels off the volume, and the slice took the
  milligram: every *rate* in §2.4's column reads off the ceiling and the spans,
  and none reads off the voxel count directly. `axis/archetype.rs`, §2.2.

- **2026-08-31 (DC2): a decorative sense voxel is also an arithmetic tool.**
  §2.2 named `[0,0,0]` as a legal one-voxel `Sensor` contributing span **zero**,
  and it turns out to be the one part in the vocabulary that can move a body's
  ceiling by a single milligram without moving any span. Two of them are what
  land the browser on 1,284 mg exactly at 33 parts; without them the same
  ceiling forces a `[3,3,3]` block and the part count falls to 15, which is
  carving A's. `archetype::SPECK`.

- **2026-08-31 (DC2): the authored grazer is not worse than the naive one; the
  *tier* is.** Ten seeds with the consumer tier founding the browser:
  **1 breathes / 5 thins / 0 boil / 4 collapse**, against DC1.5's naive mobile
  grazer at 1/6/0/3 and the standing baseline's 0/10/0/0 — the same regime, one
  seed either way, and it is a regime the body cannot leave. Measured beside it,
  the reason: **DC1.5's transitional draw founds a *predator* tier in nine of
  ten seeds** (ceiling 1,420-3,148, actuator span 20-128, sensor span **zero**
  — the drawn consumer is blind), and the tenth (seed 2) is a *sessile* grazer,
  which is the one baseline seed where consumers survive to the horizon. The
  archetype arm founds 230 mobile grazers in *every* seed, at a breeding gate of
  423 mg against the drawn consumer's 468-1,038, with a working eye (sight 9
  against 8) and the whole 610-strong stand as food. A tier that is one
  interbreeding species (TD10) therefore cannot hold both readings, so DC2's arm
  is an all-grazer world by construction. **This is a roster question, not a
  body question**, and it is DC4's to answer: §4.2's consumer tier is three
  archetypes, and a founding that installs the browser without the pursuit form
  beside it is measuring half a roster. `dc2_browser.json`.

- **2026-08-31 (DC2): what a collapsed world does is empty into the ground.**
  Seed 4 under the arm decides at tick 500 and ends 0/2/15 P/C/D with soil at
  2,184,886 mg of a 2,202,302 mg total — **99.2% of the world's matter in the
  ground** — against the same seed's baseline 1158/0/71 and 93,651 mg of soil.
  Seed 9 is the same shape (2,184,330 of 2,209,340). Matter is conserved to the
  milligram in every sample of every run, both arms, so the collapse is a
  redistribution rather than a leak: the grazers strip the stand faster than it
  regrows, die, and nothing is left to lift the matter back out of the soil.

- **2026-08-31 (DC2): the lens flattens the whole widened palette to one
  radius.** `BodyLensProjection::capsule_for` (`lens/src/body.rs:174`) reduces a
  part to an axis, a run, and **one radius — the smaller of the two cross-axis
  half-extents, floored at 1**. Every shape the archetype uses (`[2,1,1]`,
  `[2,2,1]`, `[2,1,0]`, `[3,1,1]`, `[1,1,1]`, `[0,0,0]`) has a cross-section of
  1, so all six project to radius-1 capsules differing only in length, and the
  one-voxel decorative speck draws exactly as large as a working eye. The
  primitive `[2,2,2]` segment projects at radius 2, which is why a
  many-small-parts body reads *thinner* than the blocks it replaced rather than
  more detailed. **DC1's shape vocabulary cannot reach the eye through this
  projection**, and neither can DC4's roster: this belongs beside DC3's render
  work and DC5's colour question, and it is cheaper than either (the radius rule
  is four lines). `lens/src/body.rs`, `dc2_browser.png`.

- **2026-08-30 (founding): part count appears nowhere in the body economy.**
  `mass_ceiling_mg` is total voxel volume × 0.8; the build multiple is a span
  sum over Limb and Sensor parts only; Mass has no span term and Plate has no
  process at all. A body re-carved into more, smaller parts at constant total
  voxel volume is economically identical. §2.2.

- **2026-08-30 (founding): the TD7 and TD11 bounds are properties of the
  palette, written as if they were properties of the game.** "No anatomy can
  price itself past ~7x" is `1 + 100 × 4 / 64`, the primitive limb's build price;
  "no anatomy may see the enclosure, 46 voxels" is `8 × (1 + 100 / 21)`, the
  primitive sensor's. Build price scales as `1 / cross-sectional area`, so a
  palette that thins its limbs from a 3×3 cross-section to 3×1 raises the first
  bound from 7× to 20×, and to 1×1 raises it to 61×, silently. The guard is one
  inequality and it belongs in `PartPalette::validate`. §2.2, §2.3.

- **2026-08-30 (founding): a body cannot contain more than four distinct
  shapes.** `PartPalette` holds exactly one `PartTemplate` per `Role` and
  `develop_body` reads `palette.template(role)` for every part. That, not part
  count, is why bodies read as repeated blocks: Isometry's reference figure uses
  eight distinct box extents and seven colours on one 10×24×8 model, and
  Mesocosm's bodies use one shape and one colour. §4.1.

- **2026-08-30 (founding): past 96 living parts the played body is silently not
  drawn.** `BodyLensProjection::project` returns `TooManyCapsules`
  (`lens/src/body.rs:74`) and the host's `pose_at` maps it to `None`
  (`genet/src/section.rs:475`), documented as tracing the frame without it. The
  first playtest's body reached 61 parts in ten meals. This is a code-verified
  mechanism for the second playtest's "the followed critter disappeared while he
  was not dead", whose cause that plan records as unknown — a candidate to prove
  against the preserved `mark_playtest2` trace, not a confirmation. §3.2.

- **2026-08-30 (founding): a roster member is truncated to its first ten parts
  in document order**, which is the axial chain from the root outward, so a
  truncated body is its head end rather than its silhouette
  (`params.rs:121`). Reordering by descending capsule radius costs nothing and
  is a strict improvement at any part scale. §3.2, DC-R1.

- **2026-08-30 (founding): the downlevel roster budget cannot carry
  many-small-voxel bodies.** `M × (C + 1) ≤ 511` at the WebGL2 uniform limit
  against `40 × 34 = 1,360` needed — short by 2.7×, and no rearrangement of 511
  fixes it. Desktop's `≤ 2,047` carries it at 66%. Meanwhile the *played* pose
  spends only 21% of the downlevel limit at `MAX_CAPSULES = 96`, so that
  particular constant is arbitrary and has room to roughly 500. §3.1, §3.2.

- **2026-08-30 (founding): grazer versus predator is currently "has a limb or
  not".** `Organism::feeding_mode` reads `body.performs(Process::Contract)`
  (`organism.rs:288`). The ruling presumes limbs on every archetype, so
  authoring capable consumers deletes the grazer from the world unless the
  distinction moves first. No default is defensible; it is ruled or the roster
  waits. §4.3.

- **2026-08-30 (founding): symmetry *is* kingdom.** `Kingdom::from_symmetry`
  maps Radial → Producer, Bilateral → Consumer, None → Decomposer, and
  `BodyPlan::mirrors` pairs appendages only for Bilateral. Producer and
  decomposer archetypes therefore get no mirrored growth and no radial
  arrangement mechanism, which authoring will hit on the first frond. §4.3.

- **2026-08-30 (founding): the classifier floors how small a functional part can
  be.** `classify` reads Sensor only when every half-extent is ≤ 1 and Limb only
  when one axis exceeds twice the others, so the smallest legal limb is `[3,1,1]`
  — seven voxels long. Limbs can be made thinner but barely shorter, and thinner
  is the expensive direction. Separately, a `[0,0,0]` sensor is legal, is one
  voxel, and contributes span **zero**, so decorative eye voxels and functional
  sense organs are not the same parts. §2.2.

- **2026-08-30 (founding): a body of very small parts becomes sterile, quietly.**
  A birth realizes the recipe at `parent.biomass_mg() / 4` and `continue`s on
  `InsufficientMass` (`breeding.rs:65`) — sound, but the lineage stops breeding.
  The thresholds are mean part volume **5 voxels** (sterile) and **15.2 voxels**
  (breeds later than TD8's gate). At 33 parts on today's volume the mean is 49
  voxels, so both are far off; at 300 parts they are not. §2.5.

- **2026-08-30 (DC1): the §2.3 guard caps limb *length*, not only thinness.**
  Holding build price at or below the primitive limb's 6.25 refuses `[5,1,1]`
  (6.33) and `[6,1,1]` (6.45) as well as `[3,1,0]` (18.75), because span is
  linear in the long axis while the ceiling is cubic in all three. **At a 3x3
  cross-section the longest admissible Limb is `[4,1,1]`**; a longer limb has
  to be thicker, e.g. `[5,2,2]` at 2.27. This is a constraint on archetype
  authoring that §2.2's table implies but does not state, and DC2's legs
  (`[3,1,1]`, 6.00) sit inside it. `development.rs`, `overpriced`.

- **2026-08-30 (DC1): widening the palette is free at the ecology scale, not
  merely at the body scale.** The ten-seed instrument's per-sample curves —
  alive counts, biomass, births, deaths, soil, total matter — are byte-identical
  to TD11's across all ten baseline seeds and all five control seeds, not just
  equal in verdict. The snapshot format moved (`RoleShapes` and two `Tagma`
  fields), so `state_hash` moved and the demo trace re-recorded; nothing the
  simulation computes moved with it. `dc1_palette.json` against
  `td11_chain.json`.

- **2026-08-31 (DC1.5): the unbinding leaves the verdicts alone and leaves more
  of the world alive.** Ten seeds, same horizon, same constants: **0 breathes /
  10 thins / 0 boil / 0 collapse**, identical to DC1's, control all collapse,
  matter conserved to the milligram in every sample and identical per seed, zero
  escapees. What moved is underneath the verdict. **Decomposers survive to the
  10,000-tick horizon in 9 of 10 seeds against DC1's 5**, and standing biomass
  is up in 8 of 10 (seed 8: 370,074 -> 1,131,018 mg; seed 9: 381,867 ->
  864,135; seed 2 is the one that fell, 691,333 -> 389,601). Consumers still die
  out in 9 of 10, unchanged. The likely mechanism is that a fixing part brings
  its own ceiling, so the stand is made of bigger producers that hold more
  matter in bodies rather than in the ground — the second-order effect the forms
  brief's §A predicted, landing in the direction it hoped for. `dc15_kingdom.json`
  against `dc1_palette.json`.

- **2026-08-31 (DC1.5): the decomposer's anatomy is the *absence* of a feeding
  organ, and that is an honest reading rather than a shrug.** §6.5 named this
  the open design question. There is no positive saprotroph organ to read and
  inventing one would have been a stat flag with a shape on it, but the reading
  was already in the tree: `Process::Intake`'s own doc comment is "a mouth, a
  gut, **an absorbing surface**", and every body's bulk segments perform it. A
  saprotroph digests outside itself and takes the result in across its whole
  surface — no ingesting organ, which is exactly what the real ones do. So
  producer and consumer are positive readings (a part that performs `Fix`; a
  mouth borne under the head) and **decomposer is what a body reads as when it
  carries neither**. It costs no machinery and it is stated as precedence rather
  than discovered. The consequence to watch is that the residual is also what a
  body reads as after it *loses* its mouth: `organism/kingdom.rs`.

- **2026-08-31 (DC1.5): expressing the fixing process by `Role::Plate` makes
  every flat part a leaf.** The role's doc reads "Fins, plates, leaves" and the
  ruling took it as the named geometry, but the three are now one process:
  armour fixes, and a fin fixes. The founding consumer draw therefore had to
  drop its plate arm — a consumer that grew one would be the deferred mixotroph
  — so **founding consumers can no longer be armoured at all**. That is a
  standing constraint on DC4's roster: no archetype can carry a shell without
  reading as a producer until either the reading separates a frond from a plate
  (a second geometry, or a position rule) or the mixotroph question is ruled.
  `process.rs`, `axis.rs::seed`.

- **2026-08-31 (DC1.5): developmental absence could take a mouth, and with it a
  kingdom.** `Soma::develop` drew a 1-in-12 absence per appendage-bearing tagma,
  and the mouth sits on the head tagma, which the founding draw gives one
  segment. Under the new reading that was a 1-in-12 chance for **every consumer
  lineage** of realizing a body with no mouth — a founder born a decomposer into
  a consumer's tier, and the census failing at ~8%. Guarded: absence may not
  take a feeding organ, because an individual missing one limb is variation and
  an individual missing its mouth is a stillbirth. `axis.rs::Soma::develop`.

- **2026-08-31 (DC1.5): a bare root reads Decomposer, so the fixture
  constructor had to grow anatomy.** `Organism::founding` is called from 44
  sites across tests, examples and the lens fixtures, every one of them handing
  it a `Kingdom` and expecting the body to be one. Post-unbinding a single root
  part reads Decomposer whatever it was asked for, so the constructor now
  attaches the smallest organ of the role its kingdom names — a `[3,2,1]` frond
  above, a `[2,1,1]` crop or a `[3,1,1]` jaw below — with the milligram taken
  out of the root, so the body still weighs what it was given. **A consumer gets
  a jaw when its own root is Limb-classified and a crop otherwise**, which
  reproduces every existing fixture's feeding mode exactly, since the old rule
  was "does any part perform `Contract`" and a fixture's only part was its root.
  Two fixtures still moved measurably and are updated in place: a hunter's
  walker height (a jaw hangs below the head) and a metabolize test whose literal
  `[5,0,0]` offset no longer cleared the body the seed draws. `organism.rs`.

- **2026-08-31 (DC1.5): founding *mobile grazers* costs three seeds in ten —
  measured, not argued.** The first cut of the transitional draw took the
  consumer head's geometry from an independent coin flip, which founds something
  the world has never had. Before DC1.5, `Grazer` meant "no living part performs
  `Contract`", which also meant **sessile**: a grazer could not cross the
  enclosure, so its pressure was local by construction. A crop-mouthed line that
  draws limbs can, and a grazer eats *only* producers (`movement.rs:68`), so all
  230 consumers' appetite lands on the stand and nothing eats them back. Ten
  seeds under the coin flip: **1 breathes / 6 thins / 0 boil / 3 collapse**,
  against DC1's 0/10/0/0. The correlation is clean — the three collapses (seeds
  4, 6, 7) are all grazer-tier worlds, and the two predator-tier worlds (1 and
  2) read *breathes* and *thins*, seed 1 being the only breathes in the set.
  Collapsed worlds empty into the ground: seed 4 ends 6/12/0 P/C/D with soil at
  2,079,663 mg of 2,202,302 total, against DC1's 1430/0/0 and 71,814 mg of soil.
  The draw was therefore changed to the null-change rule — **the mouth follows
  the legs** — and founding a mobile grazer is left for DC2 to do deliberately
  with the instrument watching. This is the number a grazer archetype has to
  beat, and it is the first hard evidence that §4.3's first blocker was not only
  a legibility problem.

- **2026-08-31 (DC1.5): incorporation can now mint a mouth, and severing can
  take one.** `growth::resolve` places a Mass-classified part at the plan's Mass
  preference, which defaults to `Below`, nearest the root first — precisely the
  attachment the reading calls a mouth. So a body that eats something bulky
  grows a feeding organ and changes kingdom, and a body whose mouth is severed
  or arrives severed through a `Chronicle` loses one. Both are the point of a
  reading rather than a field, and both are **latent in the shipping loop**: no
  ecology rule severs a part, and only the played critter incorporates. The
  fuller kleptoplasty payoff the forms brief's §A wants is *not* reachable yet —
  `land` grafts the eaten organism as one part shaped like its **root**, and
  every root in a founded world is the Mass template, so nothing can be eaten
  into a fixing part. `growth.rs`, `world/act.rs::land`, `chronicle.rs`.

## Progress

- **2026-09-04 (DC4, Q9): the measured three-camera slice.** The instrument
  Mark's 2026-09-02 ruling asked for, and nothing beyond it. `--camera` is a
  presentation flag with three modes (`genet::section::camera`, 300 lines
  with its tests): it picks the tracer's forward and up and touches no world
  state, no intent and no snapshot the world reads. Two things generalized
  with it, both narrower than they sound — the roster's cull window became
  the camera's own oriented box (exact for every mode, numerically identical
  to the old axis-aligned box under `side`), and the follow centre's bedrock
  clamp now reads the camera's real vertical reach instead of the
  half-height, because a tilted slab frames 29.05 voxels where a level one
  frames 28. `Section::new` took a `Framing` rather than a ninth argument.
  The contact-sheet captions are a new two-field view in `mesocosm-views`
  (`caption`), because no text rendering lives in this repo.

  **Receipts.** Three scenario replays of the golden trace, one per mode
  (`camera_{side,across,oblique}.scenario`), each exit 0 at
  `081b4ba4bdc46190` with the mode asserted off the host's own reading and
  the capture taken at a shared tick 816 / live hash `35143e3984979d41` —
  which is the identity claim the three captures rest on. Receipts and
  trailing captures at explicit non-default paths; the golden `ps1_played.*`
  fixture was read and never written, and **the `side` arm's own final frame
  is byte-identical to it** (sha256 `3b260e9505ea7a97...` for both
  `ps1_played.png` and `camera_side_final.png`) — the control arm is the
  shipped section to the byte, not merely to the eye. `cargo test -p mesocosm-core --test
  matter --test flows --test succession --test embodied --release` green
  (99 tests). `cargo test -p mesocosm-genet -p mesocosm-views
  -p mesocosm-runtime` green. `cargo test --workspace --exclude
  mesocosm-lens` green. `clippy --workspace --all-targets -D warnings` clean
  in both profiles, `cargo fmt --all --check` clean. Contact sheet:
  `camera_compare.png` (5768x1142, three 1920x1080 panes, full resolution),
  built by `examples/camera_compare.rs`.

  **Nothing ruled.** The readings are in the Findings entry above and the
  camera stays Mark's. The isometric cube is priced there and not built, per
  the slice's own stop rule.

- **2026-09-02, three rulings recorded (doc only).** §7 question 9 (section
  camera): measure before ruling, a measured presentation slice renders the
  same tick under three cameras through the DT4 scenario harness (turned
  section, shallow oblique tilt, rotatable local isometric cube) before a
  camera is chosen. §7 question 10 (roster ecology corridor): fixed by a
  body, not a constant; a diagnosis slice measures first. The open rulings
  register's §10 (far roster): far bodies go to silhouettes rather than
  raising the capsule caps, landing with S4 zoom. No code changed.

- **2026-08-31 (DC4): the eight archetypes, and the Isometry bar.** Landed
  against §6's DC4 done-conditions and both of the 2026-08-31 rulings. **The
  last done-condition is Mark's and no test supplies it**; the captures are
  below and read honestly.

  **The position rule, as implemented.** `Kingdom::of_body` asked
  `body.performs(Process::Fix)`; it now asks `body.canopy()`
  (`organism/kingdom.rs`), and the whole rule is stated in one block there:

  > A plate is in a **canopy position** when (1) it is hung above what it grows
  > from — its attachment offset has a positive `y` and no larger component on
  > the lateral axis — and (2) nothing on the body stands over it: no living
  > part's lowest voxel is higher than this plate's highest. The body fixes when
  > it holds at least one such plate.

  The axial component of the offset is deliberately not compared, because along
  the axis is *where on a segment* an appendage sits — a slot — rather than
  which way it faces; a mat's pads sit at `[0, +2, ±4]` and are canopy. `Yaw`
  turns about `y`, so a part's height is the sum of the `y` offsets up its
  chain and one forward pass in document order resolves every one of them; the
  reading short-circuits on `performs(Fix)` first, so a body with no plate pays
  what it always paid. Development learned to hang a plate two ways:
  `ARMOUR_SHAPE` is `JAW_SHAPE`'s trick one bank over, so a plate selector at
  or above it is worn as a **mirrored flank pair** instead of held above, and
  `Appendage::covers` is the one predicate that says so. Nothing new is stored
  on a part, so a grown or grafted plate reads the same way as a developed one.

  **The eight recipes.** Palette, tagmata, and what the world reads off each.
  `Mass` shapes: block `[2,2,2]`, slim `[2,1,1]`, broad `[2,2,1]`, crop
  `[2,1,0]`. `Plate`: frond `[4,4,1]`, blade `[3,3,0]`, pad `[4,0,4]`, shell
  `[0,4,4]`. `Limb`: jaw `[4,1,1]`, leg `[3,1,1]`. `Sensor`: eye `[1,1,1]`,
  speck `[0,0,0]`.

  | archetype | tagmata | parts | shapes it uses | reads as, and why |
  | --- | ---: | ---: | --- | --- |
  | `producer_mat` | 2 | 41 | block; pad x2 per segment | Producer — 26 pads held above their tiles |
  | `producer_shrub` | 3 | 41 | block, slim; frond, blade | Producer — 8 fronds and 16 blades, all lit |
  | `producer_stalk` | 3 | 26 | block; frond | Producer — a crown of 6 fronds on a 17-segment runner |
  | `consumer_browser` | 8 | 33 | slim, broad, block, crop; leg; eye, speck | Consumer / Grazer — a `Mass` crop under the head |
  | `consumer_pursuit` | 6 | 30 | slim, block; jaw, leg; eye | Consumer / **Predator** — the mouth is drawn from the `Limb` bank |
  | `consumer_armoured` | 5 | 29 | slim, broad, block, crop; shell (covering), leg; eye | Consumer / Grazer — 10 plates, **none of them lit** |
  | `decomposer_crust` | 3 | 36 | broad, crop; pad (covering), leg; eye | Decomposer — no mouth, and its pads lie beside it |
  | `decomposer_detritivore` | 5 | 31 | slim, block; leg; eye, speck | Decomposer — nothing hangs under its head |

  **Senses, as chosen.** Every fauna archetype carries working eyes (span > 0)
  and something that contracts; **the three producers carry neither**, which
  keeps their build multiple at exactly 1 and is what §2.4 says makes a stand
  free. The crust is *not* sightless — a decomposer that could not tell where
  the dead were would be scenery — and its contractile parts are holdfast
  threads rather than legs, which is also what the drawn decomposer tier has
  always had.

  **The economy, measured off developed bodies.**

  | archetype | parts | ceiling mg | actuator | sensor | build | rent | bite | sight | gate |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | `producer_mat` | 41 | 3,164 | 0 | 0 | 1.00 | 7 | 40 | 8 | 1,044 |
  | `producer_shrub` | 41 | 3,364 | 0 | 0 | 1.00 | 8 | 42 | 8 | 1,110 |
  | `producer_stalk` | 26 | 3,164 | 0 | 0 | 1.00 | 7 | 40 | 8 | 1,044 |
  | `consumer_browser` | 33 | 1,284 | 18 | 2 | 2.40 | 9 | 49 | 9 | 423 |
  | `consumer_pursuit` | 30 | 1,610 | 22 | 2 | 2.37 | 10 | 57 | 8 | 531 |
  | `consumer_armoured` | 29 | 1,694 | 12 | 2 | 1.71 | 8 | 43 | 8 | 559 |
  | `decomposer_crust` | 36 | 1,558 | 18 | 2 | 2.16 | 9 | 51 | 9 | 514 |
  | `decomposer_detritivore` | 31 | 1,384 | 24 | 2 | 2.73 | 10 | 59 | 9 | 456 |

  Every build multiple sits between 1.00 and 2.73, well inside the band TD7
  tuned (its stated ceiling is ~7x and is a property of the palette). **The
  browser's DC2 column is unmoved**, asserted as its own test. And §2.3's real
  demand — *conserve total voxel volume per tier* — is asserted rather than
  eyeballed: `the_roster_conserves_each_tiers_scale` measures the transitional
  draw's own mean adult mass per tier and holds the roster within a fifth of
  it (producers 3,231 against the draw's 3,238; consumers 1,529 against 1,547;
  decomposers 1,471 against 1,438). The first cut of the roster was **2.7x
  under** on producers, which would have written TD6's inverted pyramid back
  into the body plans; the test is what caught it.

  **Founding.** `Founding` became the per-tier set DC2's residue asked for: a
  variant names, per kingdom, the list of authored bodies that tier founds, and
  an empty list means it still draws. `Founding::Roster` is now the **default**,
  so `World::new` — the shipping enclosure — founds eight non-played lineages
  instead of three. Species ids are laid out as consecutive blocks in the same
  shuffled tier order the TD2b floor already used, and within a tier the
  archetypes take turns, so every archetype founds in every seed and the tier
  splits evenly without spending a draw on it: **204/203/203 producers,
  77/76/76 consumers, 39/38 decomposers**. The pyramid (610/229/77) and the
  kingdom floor hold as tests. The played critter takes the first body of the
  consumer tier — §5's open question answered provisionally, not ruled.

  **The census, and it is a test.**
  `world::genesis::tests::every_founded_fauna_body_senses_and_contracts` asserts
  that every founded body that is not a producer — 229 consumers, 77
  decomposers and the played critter, **307 a seed** — performs `Sense` with a
  non-zero sensor span *and* performs `Contract`, over seeds 1-10. **3,070 of
  3,070**, against TD11's 307 of 3,070. A sibling test in `axis::archetype`
  holds the same claim over 4,096 development seeds per archetype, because a
  one-in-a-hundred-and-forty-four founder is what a ten-seed world finds and a
  bench test misses.

  **The instrument, six arms, ten seeds each.**

  | arm | founding | breathes | thins | boil | collapse |
  | --- | --- | ---: | ---: | ---: | ---: |
  | baseline | every tier draws (DC1.5's founding) | 0 | 10 | 0 | 0 |
  | archetype | consumer tier founds the browser alone (DC2) | 0 | 6 | 0 | 4 |
  | **roster** | **all eight archetypes (DC4, and what ships)** | **0** | **2** | **0** | **8** |
  | stand only | producers authored, fauna drawn | 0 | 10 | 0 | 0 |
  | fauna only | consumers and decomposers authored, stand drawn | 0 | 2 | 0 | 8 |
  | control | one founder, no producer | — | — | — | 5 of 5 |

  Matter conserved to the milligram in every sample of every run, zero escapees,
  control all collapse. Per-kingdom end states (P/C/D at the horizon, by seed):

  ```text
  baseline  1:1288/0/62  2:180/65/0  3:1534/0/64  4:1140/0/48  5:1333/0/89
            6:1403/0/40  7:1402/0/45  8:1354/0/161 9:1439/0/0   10:1427/0/42
  archetype 1:126/20/0   2:696/0/0    3:576/0/0    4:0/1/7      5:0/1/0
            6:339/10/0   7:212/4/0    8:1213/0/0   9:1/1/12     10:1/15/0
  roster    1:722/7/0    2:0/3/0      3:4/12/0     4:0/11/0     5:0/12/0
            6:2/3/11     7:430/0/0    8:0/1/0      9:0/3/0      10:0/6/2
  stand     1:1493/0/26  2:1528/0/76  3:1489/0/65  4:1384/0/85  5:1533/0/103
            6:1567/0/79  7:1481/0/40  8:1494/0/80  9:1337/0/0   10:1328/0/0
  fauna     1:438/0/0    2:0/5/3      3:1/4/2      4:0/18/0     5:0/16/2
            6:3/10/0     7:372/1/0    8:11/2/0     9:1/4/9      10:0/16/1
  ```

  **Does the pursuit form land as DC2's finding predicted? No.** DC2 read the
  browser-alone arm's four collapses as "a tier is one interbreeding species, so
  install three", and installing three makes it eight. The two diagnostic arms
  say why the prediction missed: the trouble was never that the tier held one
  reading, it is that **an authored animal is a capable animal** and a world of
  306 seeing, walking, feeding fauna outstrips its stand. The stand-only arm
  proves the producers are innocent — better than the draw, in fact — and the
  fauna-only arm reproduces the roster exactly. The archetype arm's own numbers
  also moved a seed from *breathes* to *thins* against DC2's recorded 1/5/0/4,
  because the absence guard widened underneath it; reported rather than
  absorbed.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 51 s). `cargo test --workspace` green (567 tests, mesocosm-lens at
  `--test-threads=1`), `clippy --workspace --all-targets -D warnings` clean,
  `cargo fmt --all --check` clean, `cargo check -p paredros-room --features
  r1-proof` clean. **No `REFERENCE_*` or `*_BASE` constant was touched.** Demo
  fixture re-recorded — same 120 intents, hash `86af868ebb97e90b` ->
  **`2295790889f3ccd5`**; `--replay` at the default path exits **0** and
  reports the match, and a trace with one bit flipped in its recorded hash
  exits **1**.

  **The truncation key, ruled to extent.** `widest`
  (`lens/tracer/params.rs`) orders by `radius² × (length + 2·radius)` instead
  of the fatter endpoint radius. **The cap term is not decoration**: `|b − a|`
  is exactly zero for every ball-shaped part, and the primitive `[2,2,2]` trunk
  segment is a ball, so a literal `radius² × length` would rank every trunk
  segment below every sliver. Two tests: a long thin frond outranks a short fat
  bead, and a ball still ranks by its radius. The capture pair is one seed and
  one frame of the shipping founding (`dc4_truncation_before.png` /
  `dc4_truncation_after.png`, 28 roster members, 640 capsules past the budget):
  **18,311 pixels of 2,073,600 differ, 0.9%**. The change is real and small,
  and the reason it is small is worth saying: the DC3-era complaint was a
  producer stand of `[4,4,1]` fronds against `[2,2,2]` root masses, and DC3's
  own capsule radius rule already separates those two (2.5 against 2.0). Where
  extent still decides is `producer_mat`, whose pads and segments **tie at
  radius 2.0** and split 32 to 16 on extent — so the mats keep their leaves
  instead of their root chain. On this frame that is a handful of bodies.

  **What the captures actually show, honestly.** All at the ruled framing —
  an orthographic slab of half-height 28 over the grown enclosure — and the
  first thing to say is the finding above: the section looks along `-z` and a
  body's segments chain along `+z`, so *the game's own camera draws every
  critter end-on*. `dc4_<name>_axial.png` is that cut and it is uniformly
  useless: the browser is a green blob with two side stubs. The eight
  `dc4_<name>.png` are the same slab turned a quarter, which is the only cut
  that shows a body plan, and `dc4_roster.png` is a contact sheet of those
  eight.

  - `producer_mat` — a row of fifteen round beads with one continuous flat
    green slab lying over them. It reads as a low hedge or a mossy log; the
    twenty-six pads merge into a single surface rather than reading as leaves.
    Closest of the three to its brief.
  - `producer_shrub` — two runs of upright green pills over a row of beads,
    eight big and sixteen small. **The most plant-like body in the roster**:
    the two leaf sizes read as two kinds of foliage on one plant.
  - `producer_stalk` — a seventeen-bead chain lying on the ground with a single
    box-shaped crown floating over one end. Not a stalk. This is the
    no-vertical-axis finding, seen.
  - `consumer_browser` — a fine bead chain, thicker in the middle, with two
    ringed circles at the head. A caterpillar. DC2's verdict stands.
  - `consumer_pursuit` — a bead chain with five big balls at the shoulder, two
    ringed circles (the leg pairs, end-on) and a small blob under the head that
    is the jaw. Reads as a fatter caterpillar; the jaw does not read as a jaw.
  - `consumer_armoured` — **the one that reads.** A small head with two ringed
    eyes, a stubby tail, and the ten flank shells fused into one long smooth
    carapace bar. It reads as an armoured slug or a woodlouse, and it is the
    body the position rule exists for.
  - `decomposer_crust` — broadside it is almost nothing: its skirts are lateral
    and point at the camera. Its *axial* cut is the good one — a round head
    between two big lobes, which reads oddly like a small face.
  - `decomposer_detritivore` — a long bead chain with **four** ringed balls in a
    row where its leg segments are. The rings are the only thing separating it
    from the browser at this framing, and they are the legs seen end-on rather
    than anything the body was authored to show.

  And `dc4_enclosure.png`, the living world at the ruled framing, twelve frames
  in, 40 roster members and 1,003 capsules past the budget: a band of tall
  green columns over low domes and pale lumps, denser and more varied than any
  earlier capture in this plan, with real vertical structure in the stand.
  Nothing in it is a critter yet. **The honest summary: the roster is a real
  bestiary in the data and a field of smooth blobs on screen, and the three
  things standing between them are all presentation — the axis the section
  looks down, one tint per body (DC5), and a capsule projection that cannot
  represent a slab (DC3's own finding).**

  **Split, per the ceiling.** `population_instrument.rs` was 598 and is now a
  directory: `main.rs` 280, `measure.rs` 279, `receipt.rs` 136.
  `axis/archetype.rs` is 407 with its tests in `archetype/tests.rs` (401);
  `axis.rs` 578 — the one to watch next — `development.rs` 511,
  `organism/kingdom.rs` 409, `world/genesis.rs` 506, `genesis/tests.rs` 417.
  `world/tests.rs` is 1,290 and was already past the ceiling before this slice.

  **Outside the stated boundary, and why.** Two files beyond mesocosm-core, the
  lens's truncation key and the fixtures had to move, both consequences rather
  than choices. `genet::fixture::volumes` enumerated **one shape per role** and
  so resolved four of the twelve the shipping palette now admits, which made
  `mesh_body` fail on the first archetype body; it reads
  `Founding::default().palette()` and every admitted shape now, so the table
  cannot fall behind the core again. And `mesocosm-lens/examples/dc4_roster.rs`
  is new — the archetype receipts needed the section's own orthographic slab,
  which the `menagerie` example's perspective camera cannot give.

  **Left for Mark.** Five, in rough order of how much they cost.

  1. **The section's axis.** Every body is drawn end-on at the ruled framing.
     Cheapest legibility left, presentation only, and it is what stands between
     these captures and the bar. Findings, DC4. **Ruled 2026-09-02: measure
     before ruling**, see §7 question 9.
  2. **The roster's ecology.** Eight collapses in ten, and the diagnostic arms
     put it entirely in the authored fauna. The fix is a constant or a body and
     the stop rule forbids the first, so it is yours. **Ruled 2026-09-02:
     fixed by a body, not a constant; a diagnosis slice measures first**,
     see §7 question 10.
  3. **Colour**, DC5 and §3.3, unchanged.
  4. **DC-R3**, unchanged and still §3.3's ruling.
  5. **Names**, §4.2 and §7's question 7 — eight role-descriptive identifiers
     now exist and none of them is a name.

  And two smaller ones this slice answered provisionally rather than ruled: the
  played critter takes the consumer tier's first body (§5), and the crust is
  the one archetype that stretched its brief — it is a *creeping* crust, with
  eyes and holdfast threads, because the census admits no sightless fauna.

- **2026-08-31 (DC3): the render budget, and the radius rule with it.** Landed
  against §6's DC3 done-conditions. DC-R1 and DC-R2 from §3.3; **DC-R3 is
  untouched and still Mark's**. `mesocosm-core` was not opened.

  **DC-R1, truncation by descending radius.** `RosterPose::from_pose`
  (`lens/src/tracer/params.rs`) picked a member's first ten capsules in
  document order, which is the axial chain from the root outward, so a
  truncated body was its head end. It now keeps the widest capsules — ordered
  by the fatter of the two endpoint radii, ties going to the earlier capsule,
  then re-sorted back into document order so the upload is stable frame to
  frame. Bodies inside the budget take a length comparison and no allocation.
  Capsules the budget could not carry are counted in a new
  `BrickDiagnostics::roster_capsules_dropped` beside the existing
  `roster_dropped`, and the host puts both in its receipt.

  **DC-R2, the numbers.** `MAX_CAPSULES` 96 → **256** and
  `MAX_ROSTER_CAPSULES` 10 → **11**, exactly §3.3. Measured rather than
  recited, and the binding test in `tracer/params.rs` now asserts each one:
  `CritterParams` 3 136 → **8 256 B**, the whole frame uniform **8 464 B =
  51.7%** of the 16 384 B downlevel binding (§3.3 estimated 52%); `RosterPose`
  352 → **384 B**, the roster binding 14 096 → **15 376 B = 93.8%**; and
  `M × (C + 1)` is 480 against the 511 ceiling, where 12 capsules each would be
  520. The heightmap lane's hardcoded `192` in `renderer.rs`, `helpers.rs` and
  `march.wgsl` is gone: the array size is injected from `MAX_CAPSULES` the way
  the tracer already injects the roster's, so the two layouts cannot drift.
  `g2_glprobe` accepts the new binding at `downlevel_webgl2_defaults`, byte for
  byte the same receipt it wrote before.

  **The vanish, retired, and caught in the act.** `BodyLensProjection` gained
  `project_truncated` — additive, `project` refuses past the cap exactly as it
  did, so `paredros-room --features r1-proof` still checks clean. The host's
  `pose_at` (`genet/src/section.rs`) uses it: a body past the budget is drawn
  truncated to its widest parts and the overflow is reported, where the old
  `.ok()` turned the refusal into no body at all. **The receipt is a
  counterfactual on one trace.** A headed `--auto-eat` run grew the played body
  to **349 living parts** (`dc3_vanish.trace.json`, hash `97bf29f3ca1fd587`,
  replays exactly). Replayed at `df2a741`: hash matches, `body_parts: 349`, the
  minimap shows the critter alive — and the section draws **nothing**
  (`dc3_vanish_before.png`, 211 body-tint pixels, all of them the minimap dot).
  Replayed after: same hash, same 349 parts, the body on screen
  (`dc3_vanish_after.png`, 2 092 pixels) and the run says out loud "body: 93 of
  349 parts past the lens capsule budget, drawn truncated to its widest". That
  is §3.2's candidate mechanism confirmed on a live body, not a hypothesis.

  **The preserved `mark_playtest2` trace no longer reaches 304 parts**, and
  this is worth recording rather than working around. Replayed today it
  diverges at once (DC1, DC1.5 and DC2 all moved founding), the played critter
  reaches 58 parts by frame 10 and is dead by frame 30, and the run ends
  `body_parts: 0`. The trace is still a good *world* driver — it is what the
  roster capture pair below uses — but it can no longer grow the body it grew,
  so the vanish had to be demonstrated on a trace recorded today.

  **The capsule radius rule** — DC2's finding, the four lines it called the
  cheapest legibility here. `capsule_for` took the *smaller* of the two
  cross-axis half-extents and floored it at 1; it now takes their **mean**,
  floored at half a voxel, and the run is measured from that. On the
  archetype's six shapes that is three radii where there was one: speck
  `[0,0,0]` and crop `[2,1,0]` at 0.5, eye `[1,1,1]` / slim `[2,1,1]` / leg
  `[3,1,1]` at 1.0, broad `[2,2,1]` at 1.5. The primitive `[2,2,2]` segment is
  unchanged at 2.0, so nothing the palette already drew got fatter — except one
  thing, below.

  **What the two capture pairs actually show, honestly.**

  *The radius rule* (`dc3_radius_before.png` / `dc3_radius_after.png`, the
  `menagerie` example's DC2 arm at the same framing). It is still a segmented
  bead-chain, still one tint, and the legs still read as bumps — DC2's verdict
  stands and DC4's bar is not met. But two things changed and both are the
  right direction: the cropping mouth was a bead the same size as the trunk and
  is now a thin flat blade lying under the head, so it reads as a *different
  organ*; and the broad segments now bulge against the slim ones, so the body
  has a thorax instead of a constant diameter. The decorative specks shrank to
  nubs and no longer draw as working eyes, which was the finding's sharpest
  complaint. So: more legible, not yet a creature.

  *The truncation order* (`dc3_roster_before.png` / `dc3_roster_after.png`,
  `mark_playtest2` replayed to frame 400, DC-R1 alone with nothing else
  changed). The change is unambiguous and **its aesthetic sign is not.** These
  forty roster members are producers: a root mass at ground level with tall
  thin `[4,4,1]` fronds above it. Descending radius keeps the fat root blobs
  and drops the fronds, so the stand goes from about twenty-five visible
  stalks to about thirteen and reads lower and rounder. The plan's argument is
  about silhouette *area* and area is what it delivers; whether a stand of
  bulbs reads better than a stand of blades is a judgement, and on this
  particular frame I would not claim it does. **A candidate refinement for
  Mark: order by capsule *extent* (`radius² × length`) rather than radius**,
  which keeps a long thin frond over a small fat bead and is the same four
  lines. §3.3 says radius, so radius is what landed.

  **One shape got fatter, and it is the same frond.** The primitive `Plate`
  `[4,4,1]` has a 4:1 cross-section, and the mean takes it from radius 1 to
  **2.5**. In the section that turns the standing crop from thin blades into
  thick columns (visible in `dc3_roster_after.png`, which carries both
  changes). A geometric mean would give 2.0 and is no better. A single capsule
  cannot represent a 9×9×3 slab, and giving the plate two capsules is a
  projection redesign this slice was told not to do. **Mark's**, alongside the
  ordering key above.

  **The load-bearing receipt: the demo trace replays to its hash unchanged.**
  `ps1_played.trace.json` replays to `86af868ebb97e90b` and exits 0. DC3 is all
  presentation and reached no trace, exactly as §6 predicted — this is the
  second slice in the series not to break a hash.

  **Files and ceilings.** `lens/src/body.rs` 349 → 495, `tracer/params.rs`
  192 → 291, `genet/src/section.rs` 525 → 569. All under the 600 ceiling;
  `body.rs` is the one to watch next. Receipts in `Code/testing/mesocosm/`:
  `dc3_radius_{before,after}.png`, `dc3_roster_{before,after}.png`,
  `dc3_vanish_{before,after}.{png,json}`, `dc3_vanish.trace.json`. The
  preserved `mark_playtest2.*` files are untouched.

  **Left for Mark.** Three. **DC-R3**, unchanged and still the ruling §3.3
  asks for. **The roster ordering key** — radius, as landed, or extent.
  **The `[4,4,1]` plate**, which one capsule cannot carry at any radius rule.

- **2026-08-31 (DC2): one archetype, measured against the economy.** Landed
  against §6's DC2 done-conditions. **The verdict moved and is reported rather
  than absorbed** — see the third finding above; nothing here is a claim that
  the arm should ship.

  **The body, as authored.** `axis::archetype` is new (286 lines) and is the
  catalogue's sibling and its opposite: the catalogue says "reference points,
  not content", and this module is content. `consumer_browser` is eight tagmata
  and **33 parts**, the same count §2.4's carving B has:

  | tagma | segments | segment shape | bears | shape |
  | --- | ---: | --- | --- | --- |
  | head | 1 | `[2,1,1]` | Mouth x1 | crop `[2,1,0]` |
  | face | 1 | `[2,1,1]` | Feeler x1 | eye `[1,1,1]` (mirrored: 2) |
  | nares | 1 | `[2,1,1]` | Feeler x1 | speck `[0,0,0]` (mirrored: 2) |
  | neck | 5 | `[2,1,1]` | — | — |
  | shoulder | 1 | `[2,2,1]` | Limb x1 | leg `[3,1,1]` (2) |
  | chest | 1 | `[2,2,2]` | — | — |
  | haunches | 2 | `[2,2,1]` | Limb x1 | leg `[3,1,1]` (4) |
  | tail | 10 | `[2,1,1]` | — | — |

  Six distinct shapes, as carving B has. Four of them are new palette entries in
  **spare slots** — `[2,1,1]`, `[2,2,1]` and `[2,1,0]` in `Mass`, `[3,1,1]` in
  `Limb`, `[0,0,0]` in `Sensor` — so `PALETTE_SHAPES` stays four and
  `PartPalette::primitive`'s defaults do not move, which is what makes the arm
  isolable. It reads **Consumer** because the head bears a mouth and nothing on
  it fixes, and **Grazer** because that mouth is a `Mass`-classified crop.
  `variance` is **0**, per §5 ("a hexapod with a varying leg count is a
  different creature"); kin are still not clones because `Soma::develop`'s
  developmental absence survives, so ~30% of founders are born short a leg pair,
  an eye pair or their specks. Absence cannot take the mouth (DC1.5's guard) and
  the body carries no `Plate` at all, so **no individual can develop out of its
  kingdom** — asserted over 256 development seeds.

  **The measurement, against §2.4's table.**

  | reading | §2.4 carving B | DC2 measured | |
  | --- | ---: | ---: | --- |
  | parts | 33 | **33** | ✓ |
  | distinct part shapes | 6 | **6** | ✓ |
  | `mass_ceiling_mg` | 1,284 | **1,284** | ✓ |
  | `actuator_span` | 18 | **18** | ✓ |
  | `sensor_span` | 2 | **2** | ✓ |
  | build multiple | 2.40 | **3,084 / 1,284 = 2.40** | ✓ |
  | rent at adult mass | 9 mg/tick | **9** | ✓ |
  | bite at adult mass | 49 mg | **49** | ✓ |
  | sight horizon | 9 voxels | **9** | ✓ |
  | breeding gate | 423 mg | **423** | ✓ |
  | birth floor | parent >= 132 mg | **132** | ✓ |
  | ticks of reserve at full | 142 | **142** | ✓ |
  | total voxel volume | 1,611 | **1,609** | **-2** |

  **One divergence, and it is arithmetic rather than judgement.** The body is
  two voxels short of carving B's volume. The cause is the first finding above:
  the six dorsal plates cannot be worn by a consumer now that a plate fixes, and
  they were also the only parts absorbing the fractional loss that lets 1,611
  voxels floor to 1,284 mg. Holding the milligram was the choice, because every
  other row in the table reads off the ceiling and the spans and none reads off
  the voxel count. The plates' 294 voxels and 234 mg went back into `Mass`
  bulk — the chest and the broad leg-bearing segments — so the silhouette keeps
  a deep body without claiming armour it cannot have.

  **The instrument, both arms, ten seeds each.**

  | arm | breathes | thins | boil | collapse |
  | --- | ---: | ---: | ---: | ---: |
  | baseline (`Founding::Drawn`, DC1.5's founding) | 0 | 10 | 0 | 0 |
  | archetype (`Founding::BrowsingConsumer`) | **1** | 5 | 0 | **4** |
  | DC1.5's naive mobile grazer, for comparison | 1 | 6 | 0 | 3 |

  The baseline arm is **identical to DC1.5's receipt**, per seed and to the
  milligram (seed 8 ends at 1,131,018 mg, seed 9 at 864,135, seed 2 at 389,601 —
  DC1.5's own numbers), which is the proof that installing the archetype changed
  nothing for the tiers that still draw. Control all collapse, zero escapees,
  matter conserved in every sample of every run.

  The archetype arm is **one seed worse than DC1.5's naive grazer on collapse
  and one better on thins** — the same regime, and the third finding above says
  why it is a regime the body cannot leave: a tier is one interbreeding species,
  so founding the browser founds 230 mobile grazers with a 423 mg breeding gate,
  a working eye, the whole 610-strong stand as food, and nothing eating them
  back, in every seed. The baseline's consumer tier is a *predator* tier in nine
  of ten seeds and consumers die out anyway; the archetype's is the only
  founding in the series that ends with all three kingdoms alive (seed 8,
  453/6/5 — the only `breathes` the baseline instrument has ever read at this
  cohort). **The reading is that DC4 cannot install one consumer archetype: it
  has to install the pursuit form beside the browser, or the tier has to stop
  being one species.** That is §7's first open question and it is Mark's.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 86 s). Nine new tests: five in `axis::archetype` (the carving-B column
  measured off a developed body, palette admissibility, a speck seeing nothing,
  no individual developing out of Grazer over 256 seeds, and kin resembling
  without cloning) and three in `world::genesis` (the authored tier founds 230
  mobile grazers in seeds 1-10, a founded organism reads the carving-B column,
  and **the arm is isolable** — 687 producer and decomposer bodies per seed
  encode byte-identically under both foundings, with identical positions).
  `cargo test --workspace` green (545 passed, 1 pre-existing ignored;
  mesocosm-lens at `--test-threads=1`), `clippy --workspace --all-targets -D
  warnings` clean,
  `cargo fmt --all --check` clean, `cargo check -p paredros-room --features
  r1-proof` clean. **No `REFERENCE_*` or `*_BASE` constant was touched** — no
  file under `organism/` or `process.rs` is in the diff at all.

  **The fixtures did not move, and that is the receipt.** The demo trace
  re-records to the same 120 intents and the same hash `86af868ebb97e90b` DC1.5
  left it at, because the arm is off by default and no snapshot field changed;
  `--replay` at the default path exits 0 and reports the match, and a trace with
  one bit flipped in its recorded hash exits 1. This is the first slice in the
  series that did not break a hash. Instrument receipt at
  `Code/testing/mesocosm/dc2_browser.json`; DC1's and DC1.5's files are
  untouched. Capture at `Code/testing/mesocosm/dc2_browser.png`, from
  `mesocosm-lens`'s `menagerie` example at the same framing the catalogue's
  reference animals use, so the authored body and the generated ones are
  comparable at a glance.

  **What the capture actually shows, honestly.** Not a browsing hexapod. A
  segmented green bead-chain, closer to a caterpillar: uniform capsules with a
  slightly thicker cluster at the middle and small blobs at the head end. The
  legs are present and read as bumps rather than legs. Two causes, and the
  smaller one is the framing (the camera looks nearly along the lateral axis, so
  the mirrored legs foreshorten into the silhouette). The larger one is the
  fifth finding above: **the lens projects every part to a capsule with one
  radius, the smaller cross-axis half-extent floored at 1**, and all six of the
  archetype's shapes have a cross-section of 1 — so the shape vocabulary DC1
  widened is invisible, a one-voxel decorative speck draws the same size as a
  working eye, and a many-small-parts body reads *thinner* than the `[2,2,2]`
  blocks it replaced. One tint over the whole body (§3.3's open colour question)
  finishes the job. DC4 is the bar and this is not it; the good news is that the
  two things standing between here and there are already scheduled (DC3, DC5)
  and the radius rule is four lines.

  **Split, per the ceiling.** `world/genesis.rs` was 576 and is now 446, with
  its tests in `world/genesis/tests.rs` (308). `axis.rs` is 541 and
  `axis/archetype.rs` 286. `population_instrument.rs` is **598** — it grew a
  third batch and is now the file the next round has to split before adding.

  **Left for Mark.** Four. **Whether the consumer tier founds one archetype or
  three** — DC2's number says one is not viable and §4.2 already proposes three;
  this is §7's question 1 and it now has evidence. **Whether `Founding` stays a
  two-variant enum** or becomes a per-tier set once DC4 has a roster. **The
  capsule radius rule**, which is the cheapest legibility this plan has found
  and belongs somewhere between DC3 and DC5. And **whether the played critter
  should carry the archetype** — DC2's arm gives it to `SpeciesId(1)` along with
  the rest of the consumer tier, which is §5's open question answered
  provisionally rather than ruled.

- **2026-08-31 (DC1.5): kingdom unbinds from symmetry.** Landed against §6's
  DC1.5 done-conditions and both of §6.5's rulings.

  **The reading.** `Kingdom::from_symmetry` is gone. `Kingdom::of_body` and
  `FeedingMode::of_body` (`organism/kingdom.rs`) read four rules off the parts a
  body feeds with:

  | reading | anatomy |
  | --- | --- |
  | `Producer` | a living part performs `Process::Fix` — a plate, a frond, a leaf |
  | `Consumer` | no fixing part, and the head bears a mouth |
  | `Predator` | that mouth is `Limb`-classified: a jaw, which swings |
  | `Grazer` | that mouth is bulk: a crop, which does not |
  | `Decomposer` | neither organ — it absorbs across its own surface |

  **A mouth is a living part attached to the root and hung below it.** Both
  halves carry weight. Without *below*, the next segment along the axis reads as
  a mouth, since a spine is `Mass`-classified parts attached to `Mass`-classified
  parts; without *the root*, any bulk hanging anywhere off a body makes it a
  consumer. The root is the axis' front-most segment by construction of
  `develop_body`, so "at the head" is a fact of the pipeline rather than a
  convention this reading invents. Fixing is checked first, so a body carrying
  both organs reads `Producer` — the deferred mixotroph pinned by stated
  precedence rather than found by accident.

  **A fourth process.** `Process::Fix`, expressed by `Role::Plate`, wired in all
  four places the forms brief's §A listed — the variant, `NATIVE_DEFS`,
  `Role::processes()` and the parity test. `Process::ALL` is new and the parity
  test iterates it, so the trap that brief named (a fifth variant compiles clean
  and panics at runtime inside `Registry::of_native`) cannot recur. **The word
  is provisional**: `Fix` is biology's plain working verb and `CLAUDE.md` forbids
  coining mid-session, so the identifier and `ProcessId`'s `"fix"` are both
  placeholders awaiting a naming round. Renaming moves no rule.

  **`Symmetry` is geometry.** It keeps its three variants and its one job,
  deciding what `BodyPlan::mirrors`; `Kingdom::symmetry()` survives only as the
  silhouette a founding tier opens with, and nothing reads it back.

  **The transitional founding.** The pyramid no longer authors a `Kingdom` onto
  a founder for the body to wear — it picks which body that founder's line
  draws, and the world reads the kingdom back off the body. `axis::seed` takes a
  `Kingdom` instead of a `limbed: bool`: a producer draw gets a guaranteed
  fronded stretch and nothing that contracts, a consumer draw a mouth on its
  head, a decomposer draw neither. Consumers lost their plate arm, because a
  consumer that grew one would be the deferred mixotroph. **The mouth follows
  the legs** — a line that draws pursuit machinery draws the jaw to use it —
  which is both the natural authoring rule and the null change, since the old
  reading was "any part performs `Contract`". Drawing it independently instead
  is a different ecology and it is measured in the findings above.

  **The four bindings, each handled.** `LivingTarget.kingdom` keeps reading
  `o.kingdom()` and the grazer filter is unchanged in code — what a body earns
  by and what it is edible as are now one reading of one anatomy, and the
  register's §13 mixotroph case stays deferred because no body can honestly do
  both. `CohortKey` is unchanged mechanically and documented: it is an anatomy
  class now, so a body can leave it by growing or losing an organ. Hashes moved
  as priced: `FaunaTraits.feeding_mode` reaches `state_hash` through the fauna
  decision trace, and the demo fixture is re-recorded below. `guise` and
  `betrays_itself` keep every line of their logic and get sharper referents —
  the lie is a claim to carry a way of making a living's organs, and the tell is
  "a producer's claim over a body with no part performing `Fix`", which is why
  the tell exists at all rather than a coincidence of two enums lining up.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 53 s). **The census is a test, not a probe**
  (`world::genesis::tests::every_founding_body_reads_the_kingdom_its_tier_drew`):
  it replays genesis' own seeded tier draws and asserts every one of the 917
  founding bodies in each of seeds 1-10 reads back the kingdom its tier drew —
  9,170 of 9,170 — and a sibling test asserts ten seeds reach both consumer
  readings. The pyramid (610/229/77) and the TD2b kingdom floor still hold, now
  as readings rather than as fields. `population_instrument --release`: **0
  breathes / 10 thins / 0 boil / 0 collapse**, control all collapse, zero
  escapees, matter conserved and per-seed identical to DC1's; per-kingdom end
  states in the findings above. Demo fixture re-recorded — same 120 intents,
  hash `e2aca9bb40e7b0ea` -> `86af868ebb97e90b`; `--replay` at the default path
  exits 0 and reports the match, and a trace with one bit flipped in its
  recorded hash exits 1. `cargo test --workspace` green (537 tests,
  mesocosm-lens at `--test-threads=1`), `clippy --workspace --all-targets -D
  warnings` clean, `cargo fmt --all --check` clean, `cargo check -p
  paredros-room --features r1-proof` clean. Receipt at
  `Code/testing/mesocosm/dc15_kingdom.json`; DC1's file is untouched.

  **Split, per the ceiling.** `organism/kingdom.rs` is new (269 lines) and
  `organism.rs` came down to 556. `process.rs` is 569, `axis.rs` 540,
  `world/genesis.rs` 576 — that last one is close enough that DC2 should expect
  to split it rather than add to it.

  **Left for Mark.** Three things, none of them blocking DC2. **The process
  word** — `Fix` is a placeholder and wants a naming round. **Whether founding
  should reach a mobile grazer at all**, now that the number is measured: the
  draw here declines to, deliberately, and DC2 can take it up with the
  instrument watching. And **the armour collision**: `Role::Plate` is now the
  fixing geometry, so nothing can be armoured without also being a producer, and
  the roster will hit that on its first shelled archetype.

- **2026-08-30 (DC1): the palette learns more than four shapes.** Landed
  against §6's done-conditions.

  **What changed.** `PartPalette`'s four `PartTemplate` fields become four
  `RoleShapes`, each a default template plus `PALETTE_SHAPES - 1` optional
  extras — **four shapes per role**, sized on §2.4's carving B (three Mass
  shapes, one each of Limb, Plate and Sensor) with one spare. The default is a
  plain field rather than slot zero of an array, so "every role always has one
  shape" is a fact of the type instead of a rule `validate` enforces. `Tagma`
  gains `segment_shape` and `appendage_shape`, `u8` selectors where **0 is
  every role's default**; a selector a world does not admit falls back to the
  default rather than failing, which is what keeps the palette's stated
  cross-world promise (`development.rs:40`) true. `develop_body` resolves the
  segment template per tagma (the root from the first tagma's selector) and the
  appendage template from the tagma's own. `PartPalette::validate` now runs the
  classifier over *every* admitted shape, not just the default, and adds the
  §2.3 build-price refusal for Limb and Sensor, priced with the economy's own
  `part_ceiling_mg` rather than a copy of its formula. No archetype: the
  primitive palette is one shape per role, exactly what it was.

  **The null-change receipt.** The new `Tagma` fields and the widened palette
  move the snapshot format, so `state_hash` cannot witness body identity —
  it moved on all ten seeds, as §4.1 predicted. Identity was proven instead by
  digesting, per seed at genesis, every organism's encoded `BodyDocument` and,
  separately, every organism's genesis-drawn scalars (position, stage, age,
  since_offspring, signal, venom, guise, development seed, masses) — the second
  being a positive check that the **draw sequence** did not move, since every
  genesis `rng` draw lands in one of those fields. Both digests and the total
  living-part count are identical across all ten seeds before and after.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 26 s). Four new refusal tests: a Limb at `[3,1,0]` and a Sensor at
  `[1,1,0]` are refused with `Overpriced` while the primitive palette develops,
  a wrong-role shape in a *later* slot is caught like one in the default, and
  the two bounds are asserted equal to the primitive palette's own span and
  ceiling so guard and palette cannot drift apart. `population_instrument
  --release`: **0 breathes / 10 thins / 0 boil / 0 collapse**, control all
  collapse, matter conserved in every seed, and the curves byte-identical to
  TD11's (see Findings). Demo fixture re-recorded — same 120 intents, hash
  `8f6df49c63923be6` -> `e2aca9bb40e7b0ea`; `--replay` at the default path exits
  0 and reports the match, a falsified trace exits 1. `cargo test --workspace`
  green (mesocosm-lens at `--test-threads=1`, 38 tests), `clippy --workspace
  --all-targets -D warnings` clean, `cargo fmt --all --check` clean, `cargo
  check -p paredros-room --features r1-proof` clean.

  **Two small calls made here, both open to reversal.** The instrument's
  receipt filename moved `td11_chain.json` -> `dc1_palette.json`, per its own
  documented rule that no round overwrites another's; TD11's file is untouched.
  And `development.rs`'s tests moved to `development/tests.rs` under the
  600-LOC ceiling (528 / 310 / axis.rs 452).

  **Left for Mark.** `PALETTE_SHAPES` is four, and the palette is *world*
  state: DC4's eight archetypes share one. If the roster wants more than four
  Mass shapes across all eight, raising the constant is one line, but it is a
  budget question rather than a mechanical one and the number should be ruled
  before the roster is authored, not discovered during it.

- **2026-08-30:** founded. Assessment against the code complete: the body
  machinery (`axis.rs`, `plan.rs`, `development.rs`, `body.rs`, `species.rs`,
  `world/genesis.rs`), the economy coupling (`organism/ecology/rates.rs`,
  `organism.rs`), the render path (`mesocosm-lens/src/{lib,body}.rs`,
  `tracer/params.rs`, `mesocosm-genet/src/section.rs`), and the Isometry
  reference (`isometry/crates/isometry-voxel/src/{demo,recipe,body}.rs`). The
  part-scale arithmetic and the render-budget arithmetic are done and are §2 and
  §3. Nothing implemented; eight questions stand for Mark in §7, of which two
  (§4.3) block DC4.

## CP1: clearing and burrow camera prototype

**Status: implemented and native-verified 2026-09-05; visual review pending.**
Mark imagines mainly up/down/left/right play with shallow depth and favours
four deliberate quarter-turn views of a cutaway terrarium. The clearing and
burrow prototype follows VB3 body-part inspection in the proposed sequence;
this ordering interprets his request to do the scene "after". The accepted
direction reopens camera exploration beyond Q9's earlier three-angle result.
The current oblique default remains the comparison arm until a prototype is
reviewed. No exact isometric angle or final movement model is ruled here.

**Implementation note (2026-09-05):** `--scene ecology|terrarium` selects the
scene, with `ecology` as the backward-compatible default and a saved replay
scene taking precedence. The terrarium uses a fixed habitat volume and a
read-only Ground display filter. `Z` and `V` quarter-turn the presentation
around world axes; ordinary WASD movement is unchanged. Terrarium pitch and
slab size remain presentation settings. Cutaway policy is explicit:
`occupied`, `always` or `never`, with `occupied` as the default. Existing
traces without the scene tag continue to read as ecology. Native receipts are
recorded below.

The prototype explicitly converts anatomy to Ground units using the existing
locomotion contract (`BODY_VOXELS_PER_GROUND_VOXEL = 4`). Raster bodies,
capsule comparison and culling use scale 1/4, with anatomy grounded at its
lowest extent to match the walker stance; the existing ecology renderer
retains its previous scale for comparison. CP1 starts at slab half-height 18,
with configurable 0..45 degree pitch (default 12). The host fits admitted
anatomy at genesis and retains those world-aligned bounds during movement
and turns. Movement still uses a
turning cross-section proxy, rather than articulated voxel collision.
`occupied` opens the habitat cut when the controlled body enters its burrow
route and closes it on exit. This is an occupancy-triggered ground-truth
section of the authored habitat, not persistent exploration visibility.
`always` exposes it from the clearing; `never` keeps the terrain closed.

**Scene target:** one small habitat with a readable clearing, canopy/root
structure, surface entrance and connected underground chamber. Use actual
Ground geometry and the current voxel critters. Frame the habitat as a place,
with a visible section through soil and a legible route between surface and
burrow. This is a representative authored probe before procedural framing.

**Presentation target:** keep the same world-aligned habitat volume and
subject while turning between four orientations. Hold a stable view during
ordinary movement, allow shallow depth to clarify forms, and expose the
occupied/discovered interior through an explicit cutaway rule. Compare pitch,
framing and foreground suppression as view settings. The current view-aligned
slab is an implementation starting point, not the definition of the cube.

**Open gameplay question:** establish how screen-relative up/down/left/right
intent maps to real walkable/climbable surfaces and how a critter changes depth
at a passage. Rotating the view alone must not teleport a critter or connect
physically disconnected spaces. Cuts must have a stated information policy
for undiscovered chambers. These are prototype decisions, not settled rules.

**Done when:** the same habitat is captured from all four views; the controlled
critter, a nearby threat/food source and the burrow entrance remain readable;
entering the chamber demonstrates the chosen interior reveal; VB3 selection
still points to the displayed part after a turn; pure camera rotation leaves
the world hash unchanged; and the intended traversal has an ordinary input
receipt. Compare against the existing oblique view at the same world state.
Review the scene and controls with Mark before changing the shipping camera.


**CP1 receipt (2026-09-05):** 572 distinct library tests pass: core 401,
host 80, lens 49, render 13, runtime 29 (two existing ignored tests). The
release host was captured on Vulkan / RTX 4060 Laptop GPU. Evidence and
executable scenarios live under `Code/testing/mesocosm/cp1_terrarium/`.

- All four views plus return to east retain organism 0 / part 0 / revision
  `3c08feca18d1fcee`, the same coordinates and hash `65425aa79982d536` at
  every turn. The trace contains zero intents. Ordinary, non-dev camera
  controls also turn during replay without changing that hash.
- Four ordinary D inputs, each stepped once, descend from `[-14,6,-13]`
  to `[-10,2,-13]`. The occupancy cut opens on entry; the trace replays to
  `c90423c121afb7fb`. Open and closed interior comparisons preserve that hash.
- Seed 7 retains bounds `[-38,-4,-37] .. [10,18,13]` before and after travel.
  At genesis, four voxel bodies / 170 parts render at scale 1/4 with zero fallbacks,
  missing volumes or projection failures. Oblique and 0/24-degree pitch
  comparisons preserve the same world state.
- The historical 856-step trace still reaches `17ce02e24e152591`; its PNG
  byte-matches the retained pre-CP1 RG3 host. The default ecology projection
  and frame-master composition are preserved.

**Visual finding:** the side views now expose legs and body length clearly;
18 frames the whole habitat, while `--slab 12` gives a closer section. The
current producer reads as a spreading mat, so the canopy target is still
open. Terrain colour/contrast, composed vegetation and articulated foot
contact need further visual work. Locomotion still uses the turning-width
proxy, not full posed voxel collision. CP1 establishes the working comparison
and traversal; it does not promote a new shipping camera or claim final scene
composition.

### CP1 terrain readability follow-through

**Status: implemented and native-verified 2026-09-05.** The native prototype exposes a presentation
problem: heavily quantized terrain turns soil into olive/red bands and gives
open sky and exposed underground space too little distinction. Add an explicit
`--terrain-style auto|classic|habitat` setting. Auto retains classic in ecology
and chooses habitat in the opt-in CP1 scene. Host-owned appearance data supplies
soil, rock and backdrop colours to the existing lens tracer. The existing
Ground, cut policy, rays, body geometry, depth join and world record stay the
authority for geometry and interaction.

**Done when:** the same clearing and four-step burrow trace are captured under
both styles; all four turns preserve world hash and addressed selection; the
classic terrain/body pixels match CP1 and the historical ecology capture; sky, earth,
rock and exposed underground space are visibly distinct in the habitat style;
the receipt records the resolved style. Backdrop bands are illustration, not
lighting, sensing or another material field. Canopy generation and articulated
foot contact remain separate open anatomy work.

**Findings and progress (2026-09-05):** 130 focused library tests pass
(host 80, lens 50; one existing ignored). Native clearing and burrow replays
under classic/habitat retain `65425aa79982d536` / `c90423c121afb7fb`.
All four 12-degree views retain the selected part/revision and zero camera
intents. Historical ecology remains fully PNG-identical. Classic CP1's
terrain and body region is pixel-identical; whole-frame identity is not
claimed: tiny minimap-edge differences also occur between repeated captures
of the same binary and arguments. The exact differing pixels are recorded in
`Code/testing/mesocosm/cp1_readability/verification.json`, separately from the
zero-tolerance terrain/body check.

`section/terrain.rs` owns the preset; lens `TerrainAppearance` carries only
colours and the fixed backdrop plane. `Grade` still owns fog and quantisation.
The host chooses a smooth grade for habitat and retains the retro grade for
classic. Ground, body volumes, geometry, hit depths and saved trace data are
unchanged. The new picture removes the false red/olive soil bands and exposes
body/ground contrast more clearly. Canopy composition, contact and scene
dressing remain open; this pass does not declare the habitat visually complete.
