# Scale Plan (2026-08-29)

**Status: active scale ladder, refreshed 2026-09-01. S1 landed 2026-08-29 on
top of TD7. The written ladder still places S2 next, while S1's measured
findings argue for S3 first; that order remains unruled. This lane is not the
next product slice. The [playable ecology plan](2026-08-31_playable_ecology_plan.md)
consumes cohort scale and zoom at PE6.**

The ruling, in Mark's words: "scale is remarkable to behold, a feature in
its own right! a large scale intelligible terrarium-esque world generator,
that scales from the immediate scale to huge scale through the magic of
zoom... would be super cool to have a big, dynamic local ecology. it is
also a good systems test, a fantastic one! all scales matter and this one
does too. we should definitely expand on those arbitrary limits to stress
the bounds of what is possible with this stack."

This supersedes the standing caution in the landscape doc's stop rules for
this repo: chunk/streaming machinery was deferred until real scale
pressure existed, and this ruling creates that pressure deliberately —
admission is still by trace (each rung lands with a measured receipt), but
the demand is no longer hypothetical. The 2026-08-05 rule that sim tier
and render LOD share facts, never mechanism, stands untouched.

## Where the arbitrary limits actually are (measured 2026-08-29)

- **Terrain is O(area), never O(volume):** `SURFACE_BAND = 24` caps height,
  bricks are 8³ at 1 byte/voxel. ±16 was 51 bricks, 25 KiB; **±64 measures
  589 bricks, 294 KiB** (S1).
- **±87 is the unwindowed wall:** `modulus::MAX_BRICKS = 2047` (the 1 MiB
  atlas) with the whole enclosure resident. The windowed mode
  (`with_capacity` + `retarget`) is already proven at ±256 — by Paredros.
  **S1 spent 28.8% of it and left 1,458 bricks of headroom.**
- **The place graph does not scale at all:** `PLACE_SIDE = 3` is fixed at
  nine regions whatever the enclosure; raising it reorders RNG draws and
  breaks every existing seed's replay (accepted — hashes break by design
  every round this week). **S1 measured what that costs at ±64**: a region
  is 43 voxels across rather than 11, the grown graph's diameter is **3**
  rather than the 2 `demote_hops` was tuned against, and 43% of the roster
  now sits in the far tier against 19% before.
- **The far tier is a pessimization today:** far organisms skip physics but
  scan the whole living/carrion roster with no distance cap — O(far × N).
  **Measured at ±64: 292,176 scanned pairs a tick at 916 founders, against
  110,576 for the same population at ±16 — 2.6x more, and the wider world's
  tick is still cheaper, because area thins every perception bucket.** Not
  yet binding; the only term growing quadratically.
- **`Cohort` exists and is unwired:** conserved-total bucketing with
  deterministic split, currently used only for tally reporting. Every far
  organism still runs the full individual pass, so tiering changes
  fidelity, not cost.
- **Population is the real ceiling:** measured N^1.6 organism loop — 300
  bodies ≈ 1.23 ms/tick, saturation ≈ 4,700 at the 10 t/s budget on the
  reference machine (Ryzen 9 7940HS).
- **The snapshot is monolithic:** `state_hash` re-serializes and re-scans
  the whole world every call; fine per receipt, O(E²) growth.

## The ladder

**Visible-body integration, 2026-09-04.**
[Phenotype section 8](2026-07-31_phenotype_plan.md#8-visible-voxel-bodies-integration-2026-09-04)
now owns the immediate voxel-body draw and procedural-content join. Its VB5
measures geometry reuse, uploads and detail budgets before choosing body atlas
residency over the existing mesh route. This brings readable near bodies
forward without resolving S2/S3 order or making voxel anatomy wait for cohort
simulation. S4's far silhouettes remain a presentation reduction of real
bodies, separate from ecological execution detail.

Each rung pushes one limit, lands with a receipt, and stops if the trace
says the stack breaks — finding the break IS a deliverable ("a good
systems test"). And per Mark, "with optimization, of course": when a
rung's receipt shows a binding constraint, the rung optimizes it rather
than merely recording it — the already-named candidates being the
far-tier O(far × N) scan, `Places::at`'s linear lookup, the N^1.6
organism loop's bucketing, and (only once proven binding) the monolithic
snapshot hash. Optimization is admitted the same way everything else is:
by a before/after trace, never by suspicion. Population scales with area
at every rung, or the world gets diffuse instead of big.

**S1 — the wide terrarium (constants only). Landed 2026-08-29.**
`ENCLOSURE` 16 → 64, founders scaled with area, the instrument re-run at
size. No new machinery; proves the whole existing stack at 15× the area.
Receipt: instrument verdicts and tick/hash/memory costs at ±64 vs ±16; a
headed capture; the played host still holding frame at 10 t/s. See Progress.

**S2 — resident is not visible (adopt the windowed atlas).** Mesocosm
adopts `BrickMap::with_capacity`/`retarget` — the Paredros-proven paging —
keyed to the section slab, so the enclosure can exceed the 1 MiB atlas.
Receipt: ±128 or beyond playable; retargets without texture churn; replay
hash untouched by camera movement (residency is presentation).

**S3 — the region tier becomes real (the big one).** `PLACE_SIDE` grows
with the enclosure; `Places::at` gets a spatial index; far-tier perception
gets a distance cap and bucketing (kill the O(far × N) scan); and `Cohort`
becomes an actual execution path — far organisms simulated as cohorts with
conserved totals, split back deterministically on materialization. This is what
buys a big population instead of a big empty map.

S3 uses one rule model with two storage/execution forms. The aggregate carries
the sufficient state for every far-tier evaluator that actually runs: count,
biomass, energy and reserve, age and lifecycle distribution, developmental and
process distribution, relationship-flow totals, ecology-reading totals, and
ordered residual/entropy state. A generated process or relationship either
declares that reduction or pins its affected subjects to individual execution.
The global trophic graph remains a reading of those totals and accepted flows,
not a third population authority.

Played, named, injured, chronicled, descent-significant, and
relationship-bearing subjects retain pointable individual records even when
their mesh, collider, local mind, and detailed field sampling are dormant.
Ordinary equivalent subjects may aggregate. Crossing the boundary cannot
resample a retained subject. Aggregate → materialize → aggregate with no
intervening tick must restore byte-identical cohort state, including remainder
and entropy position.

That exact transition claim is separate from the fidelity of later cohort
execution. Every far-tier evaluator declares which counts, stocks, flows, and
lifecycle or relationship totals remain exact and which readings are reduced
approximations. At the same world size and initial state, an all-individual
reference arm and the hybrid arm run the same horizon; each approximated
reading has an explicit comparison envelope rather than one global “close
enough” tolerance. A tier transition itself emits no meal, birth, death,
movement, discovery, or resource flow. Repeated boundary crossing cannot earn
matter or advance lifecycle. A controlled subject’s position or another
replayed world fact may select simulation detail;
camera zoom and atlas residency remain presentation and cannot select it.

Receipt: the instrument at thousands of organisms; per-tick cost curves near
vs far; conservation (TD6's invariant) and bounded-reading equality across
aggregate/materialize; a boundary-oscillation trace proving persistent
identity and consequence silence; an all-individual reference versus hybrid
comparison over every reduced reading; and a deliberately unsupported process
proving the pin-to-individual fallback.
[Rain World](https://candlesign.github.io/Rain-World-Devlog/Pages/057)
is direct game precedent for realized, abstract, and super-abstract execution.
The [super-individual method](https://doi.org/10.1016/0304-3800(94)00055-M)
and [dynamic hybrid ecology](https://doi.org/10.1016/j.ecolmodel.2016.11.007)
are the relevant model precedent; the latter’s finding that added aggregation
need not improve performance is why the cost curve remains part of acceptance.

**S4 — the magic of zoom.** Continuous pull-back from critter scale to
whole-world scale: the section's orthographic extent as a zoom axis,
bodies handing off to silhouettes to region tint as the roster cap and
legibility demand — sim tier and render LOD sharing hop/region facts,
never mechanism, per the standing rule. Presentation only; never in the
trace. Receipt: captures across the zoom range; the far view legible
(territory, flows) rather than a smear.

**S5 — the stress receipt.** The systems test named as such: a matrix of
enclosure × population across the rungs — tick cost, hash cost, memory,
atlas behavior, where each ceiling actually stands after S1-S4 — written
down like the soil probe was, as the stack's measured envelope. Receipt:
the document itself, with every number reproducible from a cargo run.

## Not this ladder

The world tier above the enclosure — many regions as a world of records,
the overmap/chartulary shape — stays deferred: it is a graph-and-records
problem, not a voxel problem, and it should be designed against the wing's
fidelity contract with Mark rather than grown out of a stress test.
Incremental/merkle snapshot hashing is admitted only when a rung's receipt
shows the monolithic hash is the binding constraint, not before.

## The framing, ruled 2026-08-29 (Mark)

**Slab half-height 28**, closing the presentation question open since PS1.
The world is 5.2:1 (129 voxels wide against a 25-voxel band) inside a 1.78:1
window, so no half-height both frames the width and fills the height; 28
frames the content's height and lets width follow, showing 77% of the world
and reading a 9-voxel limb at 9% of frame. It ships as the default in place
of 20; `--slab` keeps every framing reproducible, and all of them replay to
one hash, so the ruling cannot reach a trace.

**Companion rule, ruled with it:** past roughly half-height 24 the frame dips
below bedrock and shows void, so the follow-centre is clamped so the view
stays on the world. The clamp is presentation, like the framing.

Superseded by this: PS1's note that "the right half-height is nearer 9-11".
That was measured on a 32-voxel enclosure S1 has since grown to 129.

## Findings

- **2026-09-01: the existing cohort is a conservation probe, not the S3
  record.** `Cohort` currently stores species/place/kingdom/mode/mass band,
  count, biomass, energy, and summed age. `split` distributes those totals by
  stable ids, but the type has no reserve, lifecycle distribution,
  developmental/process state, relationship state, reading totals, or entropy
  cursor. S3 must grow from its real consumers rather than treating the present
  tally shape as a future snapshot schema.

- **2026-08-29 (S1, and it argues S3 is needed sooner than planned): the
  near/far line's tuning premise is gone.** `TierLine::default` documents
  `demote_hops = 2` as chosen because "the standard enclosure is a 3x3 graph
  with diameter two". Measured at ±64 on the probe's seed the grown graph has
  **diameter 3** — the region count did not change, but links derive from a
  landscape sampled over four times the span, so the graph got *longer* rather
  than wider. Two consequences, both measured: a region is 43 voxels across
  rather than 11, so a body the tier demotes to the coarse mind can be standing
  43 voxels away — **inside the section's own frame at every half-height tried**
  (71 to 129 voxels wide); and the far tier now holds **43% of the roster**
  against 19% before, because a population spread across nine large regions no
  longer huddles inside one. Nearly half the enclosure is running the tier whose
  target scan is unbounded, and some of it is on screen. S3 owns the fix (a
  distance cap, a spatial index, cohorts as an execution path); S1 only reports
  it, but the report is that S3 is now the load-bearing rung rather than S2.

- **2026-08-29 (S1): the section's roster cap is saturated, and it is now the
  binding limit on what the ecology looks like.** `mesocosm_lens::MAX_ROSTER`
  is 40. At ±16 the TD7 receipt drew 18 bodies; every S1 capture, at every
  half-height from 20 to 48, reports exactly 40. The slab window now holds more
  organisms than the tracer can pose, so what the player sees is a truncation
  rather than the enclosure. It is presentation and it does not touch the
  trace — but S4's zoom cannot mean anything while the roster is clipped at 40,
  so growing the cap (or handing far bodies to silhouettes, which is S4's own
  plan) moved from "later" to "before zoom".

- **2026-08-29 (S1): the creature:world ratio the fix was ruled on is not what
  the player reads — the creature:*frame* ratio is.** The finding below prices
  a limb against the world's width (27% → 7%). What reaches a capture is the
  limb against the camera's frame, and the frame is the slab half-height's to
  set. At the shipped 20.0 the ±64 frame is 71 voxels wide, so a limb is 12.7%
  of it; the world's own 7% only appears at half-height 36.3, where the frame
  is exactly the enclosure's 129 voxels. **What S1 fixed without touching the
  camera** is subtler and matters more: at ±16 the frame was *wider* than the
  world, so both walls sat in shot and the eye had the whole enclosure to
  measure a body against. At ±64 the ground runs off both edges at the shipped
  framing, so there is no whole world in view to be a third of. A framing
  proposal with the arithmetic is in the Progress entry; the number is Mark's.
  **Ruled the same day at 28, with a clamp on the follow centre** — see "The
  framing, ruled 2026-08-29" above and the Progress entry that landed it.

- **2026-08-29 (S1): a generated nest entry almost never crosses a region
  boundary any more.** An entry route is 5 to 8 voxels long and a region is now
  43 voxels across, so the two facts a burrow test wanted to see together came
  apart: at ±16 seed 0 had a crossing, and at ±64 only 30 of the first 1,000
  seeds do — 26 of those between places the grown graph actually links. Harmless
  (the test re-pinned to seed 172) and worth knowing, because it is the same
  arithmetic as the tier-line finding wearing different clothes: everything
  sized in voxels stayed put while everything sized in regions grew 4x.

- **2026-08-29: the creatures are too big for the world, and S1 is the fix**
  (Mark, from the captures: "don't the creatures in the game seem super big
  to you?"). The measured ratio, from `PartPalette::primitive` against
  `ENCLOSURE = 16` (a 33-voxel span): a mass segment is `[2,2,2]` = 5 voxels
  (15% of the world's width), a limb `[4,1,1]` = 9 voxels long (**27%**), a
  plate `[4,4,1]` = 9x9. A played body carries ~61 parts, and 60 organisms
  at that size share a 33-voxel box — hence the overlapping pile in
  `td3_roster.png`. A creature is also larger than a terrain brick
  (`BRICK = 8`), so body features resolve finer than the world does, which
  is backwards.
  **Fix the world, not the bodies.** `part_ceiling_mg` prices voxel volume
  at 0.8 mg/voxel, so shrinking half-extents would cut mass ceilings by
  roughly eightfold and drag the economy TD6/TD7 just tuned; it would also
  spend shape legibility a 5-voxel body cannot spare. S1 costs neither. At
  `ENCLOSURE = 64` (129 voxels) a limb falls from 27% to 7%.
  Two consequences that argue the same way:
  - **Per-voxel soil granularity is currently moot.** The grain was chosen
    (2026-08-29, on measured evidence) so roots could hunt across
    neighbouring columns — but a 9-voxel producer already straddles nine.
    Its forage radius sits inside its own footprint until the world grows.
  - **The world is crossable in about two seconds** (1-2 voxel moves, 33
    voxels, 10 t/s ≈ twenty ticks). Scale is a felt property, not only a
    systems-test one.

## Progress

- **2026-09-01:** specified S3's materialize/aggregate identity, conservation,
  residual-state, unsupported-process fallback, and acceptance receipts from
  the playable-ecology refinement. The S2/S3 order remains open and no rung was
  dispatched.

- **2026-08-31:** clarified that S2-versus-S3 remains an open scale-order
  ruling and that the playable ecology plan consumes this lane at PE6. No rung
  was reordered or dispatched.

- **2026-08-29 (last today): the framing landed — half-height 28 is the
  default, and the follow centre is clamped to it.** Two lines of behaviour,
  both presentation. `section::SLAB_HALF_HEIGHT` is 28.0 (was 20.0), and
  `--slab H` still varies it; `section::centre_on` now takes the half-height and
  clamps the followed y to at least that, so the frame's floor is never below
  the world's lowest brick at y = 0. `section::half_height_or_default` is the
  one reading both the camera and the clamp ask, so they cannot disagree about
  how tall the frame is. Three tests state the rule (the clamp holds at every
  half-height tried, it is a floor rather than a lock — a body above it is still
  followed and the pan still pans, and an unset half-height falls back to the
  ruled default).

  **The clamp is measured, not asserted.** `td8_framed.png` against S1's
  unclamped `s1_slab_28.png`, same trace, same 1,920 x 1,080 capture: the
  unclamped frame's bottom **154 px — 14% of the frame — read background grey
  (191, 191, 191)**, which is the void under bedrock; the clamped frame reads
  ground (64, 64, 64) on every row to y = 1,079. Terrain and bodies occupy the
  bottom 41% of the clamped frame with sky above, so the band is held whole with
  headroom, which is what the ruling asked for. **The trade the clamp makes is
  worth stating:** because it is a floor on the centre rather than a re-framing,
  a critter standing at y ≈ 20 is followed from a centre of 28, so the content
  sits low in frame rather than filling the 62% the S1 proposal's arithmetic
  described for an unclamped 28. Void or a low horizon was the choice, and the
  ruling took the low horizon; a frame that centres the content *and* stays on
  the world would need the half-height and the centre ruled together, which is
  not this ruling.

  **It cannot reach a trace, and the receipt says so.** `ps1_played.json`
  records `slab_half_height: 28.0` and `state_hash_matches: true` against a demo
  recorded headlessly, where no camera exists at all. (The hash itself moved
  this round — `3b86d0ef9ebd7d33` — but for TD8's ecology rulings, not for the
  framing.) Re-proved with the clamp in: the same trace replayed at `--slab` 20,
  28, 36.3 and 48 lands that hash every time, exit 0, and none of the four shows
  a background-grey row at the bottom of the frame.

- **2026-08-29 (later): S1 landed — the enclosure is 129 voxels across, the
  stack held at 15x the area, and the one collapse in the receipt is gone.**
  Constants and scaling only: no windowed atlas (S2), no place-graph growth
  (S3), no zoom (S4).

  **The two scaling decisions, and their whys.**

  - **`ENCLOSURE` 16 → 64** — a 129-voxel span, **15.281x** the floor area
    (16,641 columns against 1,089). Terrain is O(area), so this costs bricks
    and soil quadratically and nothing cubically.
  - **`FOUNDERS`, new and derived: `side² × 60 / 33²` = 916** (917 with the
    played critter). *Density is what stays fixed* — the same 60-over-1,089 the
    world shipped with, so a wider terrarium is bigger rather than emptier,
    which is the failure the ladder names at every rung. Derived rather than
    typed, so the next widening carries the cohort with it, and read by the
    host and the instrument instead of their own literals.
  - **The pyramid survives exactly**: 916 founders compose **610 / 229 / 77**
    P/C/D at TD7's unchanged 2/3 : 1/4 : rest shares, asserted in every seed,
    and TD2b's kingdom floor is untouched (its test still passes at foundings
    of 3 to 12).
  - **Soil seeding was verified, not re-derived**: `Soil::seeded(ENCLOSURE,
    100)` gives 16,641 columns and **1,664,100 mg** at genesis — the instrument
    reads exactly that at tick 0 in every seed — still about three times what
    the founding cohort carries, because the cohort scaled with the area too.
  - **`PLACE_SIDE` stays 3, `SLAB_DEPTH` stays 16.** The first because growing
    it is S3's whole subject; the second because a section is a cut of fixed
    thickness, and thickening it to keep the same fraction of a bigger world
    would only stack more bodies into the same pixels — the pile this round
    started from. The minimap's `overhead()` extent and `minimap_score`'s
    bounds already derived from `ENCLOSURE` and were verified in a capture
    rather than changed.

  **Ceilings, with the arithmetic, all cleared.**

  | ceiling | bound | measured at ±64 | headroom |
  | --- | --- | --- | --- |
  | `modulus::MAX_BRICKS` | 2,047 | **589 bricks**, 294 KiB | 1,458 bricks (71.2%) |
  | brick upper bound | ceil(129/8)² × 4 = 17² × 4 = 1,156 | 589 (a brick column exists only where relief reaches it) | — |
  | tick at 10 t/s | 100,000 µs | 4,993 µs at genesis, **8,200 µs** mean over the instrument's busiest whole run | 92% |
  | soil store | — | 16,641 columns, 133,128 B (from 8,712 B) | — |
  | snapshot | — | 1.71 MiB at 917 bodies (from 1.40 MiB at 917; the *world's* fixed part went 29.7 KB → 323 KB) | — |
  | `state_hash` | — | 3.83 ms, called once per receipt and **not per tick** | — |

  It did not refuse, so S2 stays S2. The headed runs built their `BrickMap`
  from `Ground` without complaint, which is the runtime proof rather than the
  arithmetic one.

  **The cost table** (`Code/testing/mesocosm/s1_wide.json`, joined from
  `s1_cost_e16.json` and `s1_cost_e64.json`, both produced by a new
  `cargo run -p mesocosm-core --release --example scale_cost_probe`; the
  constant is compile-time, so the before/after is two runs of one binary).
  Every founder count is run at **both** enclosures, which is what separates a
  density effect from a world-size one. Percolation is timed directly on the
  world's own store — exact attribution, not a residue; `rest` is the tick
  minus it; the far-tier scan has no timer of its own (it lives inside the
  organism loop's borrow) so it is attributed by its exact work product,
  `far_members × living`, read from the tick's own `Tally`.

  | founders | ±16 tick | ±16 perc | ±16 rest | ±64 tick | ±64 perc | ±64 rest | ±16 far pairs | ±64 far pairs |
  | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | 0 | 49 µs | 46 | 3 | 710 µs | 698 | 12 | 0 | 0 |
  | 60 | 304 µs | 44 | 260 | 962 µs | 698 | 265 | 570 | 1,940 |
  | 240 | 1,250 µs | 41 | 1,209 | 1,692 µs | 679 | 1,012 | 5,832 | 20,354 |
  | 916 | 8,021 µs | 40 | 7,981 | 4,993 µs | 682 | 4,311 | 110,576 | 292,176 |

  Three readings, and the third is the surprise.

  1. **The percolation sweep is linear in columns and is not the problem.**
     15.3x the cost over 15.281x the columns, measured in one run on flat
     stores of both sizes. It is the *whole* cost of an empty world (698 of 710
     µs at zero founders) and **13.7% of the tick at the shipping cohort —
     0.68% of the 10 t/s budget**. Not a candidate for the optimization clause.
     The number to watch is per-channel: a typed-matter scheme would multiply
     this sweep by its field count, and 0.68% × N fields is where it bites.
  2. **The tick grew 16.4x for 15.3x the area and 14.8x the population**, and
     sits at 5% of the played budget at genesis. Over the population
     instrument's full 10,000-tick horizon the busiest seed averaged **8.2 ms a
     tick** while its population peaked at 2,207 organisms — 8% of the budget
     at the busiest moment the ecology reached.
  3. **Area is cheaper than density.** At the *same* 916 founders the wide
     world walks 2.6x the far-tier pairs and is still **40% cheaper per tick**
     (4,993 µs against 8,021 µs), because a population spread over 15x the area
     meets far fewer neighbours in every perception bucket. The O(far × N) scan
     is real and is the only term growing quadratically (1,940 → 20,354 →
     292,176, exponent 1.7 then 2.1 on N, against `rest`'s 1.5 then 1.1), but
     at this population area buys more than it costs. That trade reverses when
     the pair count outgrows the density saving — which is S3.

  **Verdicts, and this one moved in the good direction.** Baseline reproduced
  first at `ENCLOSURE = 16` and matched `td7_priced.json` in **every field
  except `elapsed_ms`** — 0 breathes / 9 thins / 0 boil / 1 collapse. At ±64:
  **0 breathes / 10 thins / 0 boil / 0 collapse**, control all collapse, max
  escapees 0, `total_matter_mg` identical across every sample of every run.
  Receipt: `s1_wide_instrument.json`.

  | seed | verdict | start | end | P/C/D end | end biomass | soil end | mean tick |
  | ---: | --- | ---: | ---: | --- | ---: | ---: | ---: |
  | 1 | thins | 917 | 1,411 | 1,378/0/33 | 622,514 mg | 1,294,298 | 8.2 ms |
  | 2 | thins | 917 | 1,140 | 1,050/**90**/0 | 611,396 mg | 1,105,776 | 2.4 ms |
  | 3 | thins | 917 | 1,255 | 1,255/0/0 | 540,297 mg | 1,212,689 | 6.6 ms |
  | 4 | thins | 917 | 1,481 | 1,414/0/**67** | 543,219 mg | 1,414,550 | 6.0 ms |
  | 5 | thins | 917 | 1,331 | 1,308/0/**23** | 661,602 mg | 1,174,730 | 6.6 ms |
  | 6 | thins | 917 | 1,295 | 1,295/0/0 | 258,167 mg | 1,696,218 | 3.9 ms |
  | 7 | thins | 917 | 483 | 386/**97**/0 | 474,646 mg | 1,356,730 | 7.2 ms |
  | 8 | thins | 917 | 889 | 878/0/**11** | 327,583 mg | 1,606,681 | 2.5 ms |
  | 9 | thins | 917 | 637 | 637/0/0 | 207,225 mg | 1,804,925 | 2.4 ms |
  | 10 | thins | 917 | 630 | 617/0/**13** | 176,158 mg | 1,865,104 | 2.5 ms |

  Every founding is 610/230/77 including the played consumer. `breathes` is
  still out of reach and the tally still reads ten thins — but underneath it
  the chain is materially better than TD7's, and S1 did not touch a single
  ecology constant: **decomposers survive to the horizon in 6 of 10 seeds
  against 0 of 10**, consumers in 2 of 10 against 0 of 10, and **the collapse
  is gone** (seed 2, which had survived three constant regimes as the one
  collapse, now ends holding 90 consumers). Room is what the failing kingdoms
  wanted. Every seed still ends producer-dominated, so the TD7 finding that
  consumers and decomposers are recruitment-limited stands: area relieved the
  symptom and did not answer the ruling.

  **The framing, proposed with its arithmetic — the number remains Mark's.**
  The window is 960x540, so the frame is `2H × aspect` = 3.556H voxels wide and
  2H tall. The enclosure is 129 wide against a 25-voxel terrain band
  (`SURFACE_BAND` caps height and S1 did not move it), so **the world's own
  aspect is 5.2:1 against the window's 1.78:1** — no half-height frames the
  whole width *and* fills the height, and that tension is the whole question.

  | half-height | frame, voxels | world width shown | a 5-voxel segment | a 9-voxel limb |
  | ---: | --- | ---: | ---: | ---: |
  | 20.0 (shipped) | 71 x 40 | 55% | 7.0% | 12.7% |
  | **28.0 (proposed)** | **100 x 56** | **77%** | **5.0%** | **9.0%** |
  | 36.3 | 129 x 73 | 100% | 3.9% | 7.0% |
  | 48.0 | 171 x 96 | 100%, and then some | 2.9% | 5.3% |

  Captures at all four are beside the receipt (`s1_slab_20.png`,
  `s1_wide.png` — the proposal, also written as `s1_slab_28.png` so the strip
  reads in order — `s1_slab_36.3.png`, `s1_slab_48.png`), each with its own
  host receipt recording the half-height it framed, and **all four replayed the
  same trace to the same hash**, which is the proof that framing is
  presentation and cannot reach a trace.

  **The proposal is 28.0, and the principle behind it is: frame the content's
  height and let the width follow.** The section's content is the 25-voxel
  terrain band plus a body's headroom above and a burrow below — about 35
  voxels — and at H = 28 that fills 62% of the frame, leaving sky above and
  ground below in the proportions a terrarium section wants. The width then
  falls out of the aspect at 100 voxels, 77% of the enclosure: enough that the
  ground runs off at least one edge from most standing positions, so the world
  reads as a place that continues rather than an object that ends. 36.3 is the
  only *other* principled number — it is exactly the enclosure's width, so the
  finding's own 7% becomes what the eye sees — but its capture shows both
  walls, the bedrock floor, and void under the world, which reads as a slab
  rather than a terrarium; 48.0 is past the useful range and included to show
  where the range ends.

  **My read of the capture, which is the motive for the whole rung.** At the
  shipped 20.0 the ±64 section is *already* right where it was wrong: the
  ground runs edge to edge, a continuous stand of green producers with salmon
  and orange consumers among them, and there is no longer a whole enclosure in
  frame for a body to be a third of. That is S1's actual answer to "don't the
  creatures seem super big" — the world got big, and it did so without moving
  the camera. At the proposed 28.0 the same frame reads as a landscape: bodies
  are ~85 px in a 1,920-px capture, individually legible and clearly small
  against the terrain they stand on, and the near-far relief of the ground has
  room to show. The ratio reads right at both, and better at 28. **One caveat
  belongs to the proposal**: past about 24 the frame's floor dips below
  bedrock and shows void along the bottom, so the half-height wants a companion
  ruling on the vertical centre — clamping the follow centre to at least `H` so
  the frame never frames below the world. That is a second question and a
  behaviour rather than a number, and it is Mark's too.

  Because the number is unruled, **the default did not move**: `SLAB_HALF_HEIGHT`
  is still 20.0 and the proposal is reachable as `--slab 28`, a new
  presentation-only host knob (`HostConfig::slab_half_height`) that the run's
  receipt now records, so a capture of either is reproducible from the tree and
  one line adopts whichever Mark rules.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`e2f037da2b2407e0`** (was
  `33ffc5b46789be9d`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 56 body parts, 40 roster members (the cap; see
  Findings), ground revision 2. Instrument proven: one bit flipped in the
  recorded hash exits **1** with `MISMATCH`, the unflipped trace exits 0.
  Default paths (`ps1_played.trace.json` / `.json` / `.png`). **The played host
  holds frame in the wider world**: a headed run drew 600 frames while the
  world stepped 65 times at exactly 10 t/s — about 92 fps with 916 founders
  resident, the tracer posing its full 40-body roster.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo test -p mesocosm-core --test matter`: green — conservation is the gate
  at any size, and it now runs at both: the four-seed 4,000-tick run keeps 60
  founders because conservation is a property of the *seams* and that run buys
  seam coverage by length, and a new
  `matter_is_conserved_at_the_shipping_cohort` proves the cycle closes over 917
  bodies and 16,641 columns in 200 ticks. Both broken controls still trip. **It
  costs**: that file went 84 s to 570 s in the debug profile the test harness
  uses, almost all of it the percolation sweep over 16,641 columns for 16,000
  ticks of the long run. Disclosed rather than traded away — shortening the
  4-seed 4,000-tick run is a coverage decision, not a speed one, and it is
  Mark's if the friction proves real.
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds, with the same one pre-existing `dead_code` warning TD6 and
  TD7 recorded. Three tests were retuned, one line of why on each, and two of
  the three are findings in their own right:
  `hunter_and_player_cross_one_generated_entry_and_place_boundary` re-pinned
  seed 0 → 172 because a 5-to-8-voxel nest entry rarely crosses a 43-voxel
  region any more (and the new seed's crossing joins two places the graph
  actually links, or the hunter answers by teleporting a region);
  `venom_is_charged_whatever_the_meal_becomes` now sets the venom on *both*
  sides of its comparison, because its control was "the fixture's prey,
  untouched" and the wider enclosure put a naturally venomous founder under the
  played critter's nose — 74 mg of it, so the "clean" world was the poisoned
  one and the subtraction underflowed; and `soil`'s own tests read
  `world::ENCLOSURE` instead of a copied 16, which is the mirror that would
  otherwise have gone on testing the world that used to ship. The population
  instrument lost its copied `ENCLOSURE` and its literal 60 for the same
  reason.

- **2026-08-29:** founded on the day's scaling sweep; ruled by Mark
  (scale as a feature, stress the bounds). Execution queued behind TD6.
