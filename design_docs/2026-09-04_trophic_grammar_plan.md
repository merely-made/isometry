# Trophic Grammar Plan (2026-09-04)

**Status: accepted by Mark 2026-09-04; TG1 complete 2026-09-05. TG2-TG7 remain open.** The three rulings in section 4 are given. This is PE4's first
build: the material scheme ruled 2026-09-02 turned into a trophic grammar. It
owns typed intake, typed accounts, part composition, defenses, and selective
edibility. It does not own fields, generated worlds, or the second form of
life; see the [playable ecology plan](2026-08-31_playable_ecology_plan.md) section 6
ruling 4 and the [elements and traits memo](2026-08-29_elements_and_traits_memo.md)
sections 1, 2, 4, 5 and 7.

**The words, ruled by Mark 2026-09-04.** **nis**: the provenance-bearing living
substance, matter typed by the line it came from, kingdom first then lineage.
One form for singular and plural, from *nisus*, striving. Matter fully returned
to soil has lost its nis and is untyped stock. **scruple**: a part's stable
heterogeneous mix, a measured lot of nis, the part layer of composition. Do not
use "element" as a term. See repo `CLAUDE.md`.

---

## 1. Objective

Give the ecology a trophic grammar, so that herbivore, carnivore and omnivore
stop being labels on a body and become readings of what that body can take in;
so that a defended body is expensive to eat rather than merely unattractive;
and so that not everything in the stand is food for everything with a mouth.
The measure is the thirty-seed corridor from the
[default creatures plan](2026-08-30_default_creatures_plan.md) section 7 Q10, run
on both walls, with a roster that still holds a producer tier, a consumer tier
and a decomposer tier, and holds all three consumer readings inside the consumer
tier. Mark's framing, 2026-09-04, verbatim: "Sounds to me like a systematic
issue that cannot be fixed with a tweak, but it's missing the gameplay loops
that differentiate herbivore from carnivore and omnivore. Plus it's not like
herbivores should lack defenses that make them difficult to eat, and not all
flora should be edible to everything."

---

## 2. Phases

Each phase lands alone and is measurable alone.

**Visible-body integration (2026-09-04).** The
[phenotype plan section 8](2026-07-31_phenotype_plan.md#8-visible-voxel-bodies-integration-2026-09-04)
owns VB0-VB5: procedural voxel representation, the live body draw path,
addressed inspection and the body-change/descendant proof. TG1 can proceed
alongside its initial geometry work. TG2/TG3 gate diet-driven tissue appearance;
TG4 gates claims of functional defense. Initial visible anatomy uses existing
parts, allocation and provenance without pretending those TG mechanics have
landed. TG6 is acceptance for this first trophic build, not all of PE4's
ordinary/impossible-world and generated-vocabulary requirements.

### TG1: typed intake

**Implementation scope, 2026-09-05:** declarations live beside part allocation
and survive graft, expression, save and filial development. Geometry seeds
founding defaults, but never reinterprets an existing declaration. Active
ports derive the current feeding reading. The existing addressed inspector
shows the selected part's declaration and the body's aggregate reading; it
does not infer food from appearance. Native rules and played traces carry a
trophic grammar revision, so pre-TG1 recordings fail with a named incompatibility
instead of pretending their old intents still mean the same thing. The
thirty-seed instrument uses explicit output paths and both drawn/roster arms;
TG1 records the outcome without claiming TG6's corridor is already achieved.

A mouth becomes a port with a declared nis kind, read off the body the way
`plan::classify` already reads a role. `FeedingMode` stops being a shape
selector with an implied prey set and becomes a **reading of the ports a body
carries**: a body whose intake ports admit flora nis reads Grazer, fauna nis
reads Predator, both reads Omnivore, dead stock reads Scavenger, none reads
Producer. The grazer and predator filters in
`organism/ecology/movement.rs` and `movement/perception.rs` are replaced by one
question asked of the port set, so the unrestricted predator prey set diagnosed
at `perception.rs:206` disappears because there is no longer a branch that can
omit the test, not because a clause was added.

**Done when:** the edible set is computed in one function used by both the
gradient and the bite; a Predator with no flora port cannot graze the stand, and
a test says so by name; Omnivore exists as a reading with at least one roster
body producing it; the thirty-seed instrument runs and its numbers are recorded
against the 17/30 and 1/30 baseline whatever they say.

### TG2: nis in the accounts

`Soil.matter_mg` and a body's substance become typed stock. Start at the three
kingdom-level nis kinds ruled in section 4, inside the memo section 4 Tier 1 budget with no fields
added; `percolate` runs per channel and its cost is measured before anything
else lands on it. **The conservation receipt is rewritten first**: the
milligram-exact matter test becomes per-channel, and the rewrite ships with a
broken-control run proving the old total-only test passes a deliberate
cross-channel leak while the new one fails it. Decay returns nis to untyped
stock on one named rule: matter that has finished returning to a column loses
its type, so soil holds untyped stock plus whatever has not finished returning.

**Done when:** the per-channel receipt is green and the broken control fails as
predicted; `Account::Soil` and `Account::Substance` reconcile per channel over
a full run; the named decay rule is one function with one caller; the tick
budget receipt is inside the 10 t/s wall at the current population.

**TG2a accounting foundation, 2026-09-09.** `mesocosm-core::matter` supplies
fixed four-channel stocks, checked arithmetic, deterministic proportional
splits, typed account receipts and a candidate column-transport kernel.
The reconciliation law is per-account, per-channel balance change equals
transfers plus explicitly declared conversion deltas. Nis kinds record
provenance rather than indestructible chemical species: a named conversion
can change the channel while conserving total milligrams exactly.

The instrument's candidate rules are soil synthesis into producer tissue,
digestion into declared tissue or an untyped metabolized reserve, and completed
mineralization into soil. Reserve erasure is a working accounting contract to
review at live integration. A retained dietary contribution or graft uses an
unchanged typed transfer; digestion does not require converting the whole meal.
TG3 still owns the actual recipes and the part mosaic's retained mix. Death
alone does not change a material kind.

The standalone `typed_matter_receipt` example checks independently stated
balances, a serialized receipt replay, and a deliberately substituted channel
whose total mass still passes. It also compares the four-channel kernel with
the frozen pre-integration scalar Soil transport at 129 by 129 columns. The
kernel retains no world state. TG2a left live Soil, body mass, reserves,
snapshots and grammar revision 1 unchanged; it was neither the full-run TG2
receipt nor a full-tick budget measurement. Next, replace storage at the
existing owners and carry typed
stocks through accepted mutations, with scruple on the existing part mosaic.
Do not introduce a parallel composition authority.

**TG2b live soil, 2026-09-09.** Soil now owns fixed typed stocks per column and
uses the measured transport kernel. Scalar reads and draws expose only untyped
nutrients; typed draws preserve the selected mixture. Deposits reject column
or global overflow atomically. The scalar total is a derived cache, reconstructed
from stocks on decode and excluded from the wire. Ordinary roots cannot consume
typed matter awaiting its completed return. Existing body returns remain
untyped until body accounting is integrated; live ecology does not yet emit nis.
Tests seed mixed soil as an initial condition to exercise this boundary.

World grammar revision 2 admits this storage and uptake contract. Current
snapshots retain all channels and validate soil shape and amount bounds on
decode. Older trace rules are incompatible; historical raw postcard worlds
have no migration and may fail decoding before the rules check. The standalone
scalar reference is frozen from `ef4828b`, so transport parity remains an
independent comparison after Soil itself adopts the typed kernel.

The next body join must cover root growth, ordered body spending, named-part
consumption, attachment, subtree severing, reserves, meals, births and grafts
together. `Mosaic` owns the scruple, with part mass equal to its total;
harvest/receive carry the donor mix and rejected candidates restore both.
Founder and child realization need explicit initial composition. Conversion
receipts must come from the accepted mutation, never inferred afterward from
the body's current kingdom. Full TG2 and TG3 remain open.

### TG3: scruple per part

Composition gets its two ruled layers. The **lineage layer** declares what nis
a line's tissue is made of; the **part layer**, the scruple, records the mix a
part was actually built from, which differs after a graft or an odd diet. It
rides the existing part mosaic rather than a new store, and there are no
per-organism vectors. The disfavoured pair lands here as the ruled graft: a
small milligram allowance carrying penalties that trait conditions raise,
composing with PE2's condition table.

**Done when:** two bodies of one line differ in scruple only because they ate
differently, and the difference is visible in a dev reading; a graft past the
allowance is refused by name; the scruple survives save, restore and replay to
the milligram.

### TG4: defenses

`Role::Plate` gains a gate, read from the body the way DC1.5 reads kingdom off
anatomy: a plate presents a threshold, and a mouth takes a meal only when its
bite clears it. Payloads fire **on being eaten**, at the meal site, on the
provenance of what was taken, priced from the payload's own numbers per memo
section 1 C rather than from an authored per-payload row. `venom_mg` is the
existing single instance and is **generalized into that path**, not duplicated
beside it; the live inconsistency the memo names (only the played meal charges
venom) closes as part of the generalization, with the spill deposited to the
column as `act.rs` already does.

**Done when:** a plated body is measurably costlier to eat and the instrument
shows a bite distribution that respects the threshold; one payload row can be
deleted and recomputed from the kind's own numbers; the NPC meal and the played
meal charge identically, proved by one test running both routes over one
transfer; matter is conserved to the milligram across a payload firing.

### TG5: selective edibility

Not all flora is edible to everything with a crop. A crop declares which flora
nis kinds its port admits; a stand whose nis it does not admit is not food.
This is **port match**, the relation memo section 2 permits, and never a
pairwise table: the answer is computed from the port's declaration and the
target's provenance, both of which exist for other reasons.

**Done when:** at least two flora lines differ in who can eat them, and the
difference changes the run's outcome; the anti-affix check passes, so the
admission rule reads properties with three consumers and no property is read by
only one kernel; there is no table keyed by eater and eaten.

### TG6: roster and instrument

The eight archetypes in `world/genesis.rs` are re-declared under the grammar:
ports rather than implied prey sets, declared tissue nis, plates with
thresholds where the archetype is armoured. The corridor is measured on thirty
seeds, both walls, through `examples/population_instrument/`.

**Done when:** over thirty seeds the roster arm is inside the corridor with a
producer, consumer and decomposer tier all present at the end, and Grazer,
Predator and Omnivore all present in the consumer tier; the baseline arm is
re-measured on the same thirty; no `REFERENCE_*` or `*_BASE` constant moved to
get there.

### TG7: lexicon

A term table in the packs, id to display forms, read only by views (Mark's ask,
2026-09-04). Small, data-only, no rule-bearing content, so a rename never
touches core. It exists so a display form can be changed without a code change.

**Done when:** every displayed nis and payload name comes from the pack; core
holds no display string for them; changing a display form does not change the
world digest.

---

## 3. Stop rules

Carried, not restated as new law.

- No "draw from X / resist Y" affix. The memo's section 2 tests bind: three
  consumers, disjoint subsets, a world write path, fingerprint and reject
  collisions.
- Payloads act on bodies. Payloads do not act on payloads.
- If a row can be deleted and recomputed from the kind's own numbers, it is
  real. If it cannot, it is a lookup in costume.
- No per-organism composition vectors. If individuals must differ, differ them
  by the parts they grew.
- No fields in this world.
- No second authority. A view, forecast, controller or instrument may summarize
  the ecology and may not hold a parallel answer.
- No constants dressed as mechanics: a fix is a body or a rule, not a number
  nudged until the instrument agrees.
- Milligram exactness, now per channel.

---

## 4. Rulings, given by Mark 2026-09-04

All three answered with "the three recommendations are reasonable".

1. **The prey-set rule moves.** A mouth's edible set comes from its ports;
   the grazer and predator filters go.
2. **Omnivore is a reading**, a body carrying both port kinds, not a fourth
   `FeedingMode` variant.
   **Implementation clarification (2026-09-05):** `FeedingMode::Omnivore`
   names the computed result of that union. It is never stored on an organism
   or assigned as a character class; the retained part declarations remain
   authoritative. This changes the enum wording of the ruling while keeping
   its separation between anatomy and reading.
3. **Three nis kinds in the first world**, one per kingdom. Dead is a state of
   the body, not a kind of matter; a scavenger port reads that state.

Mark's framing for the work that follows, same day: the loops are not all in
place and that is alright; keep adding loops that compose well and surprise;
get granular with the voxels; proceed with the rest of the plan; make
beginning body types; start investigating a beginning set of traits.

---

## Findings

- **2026-09-09, TG2b live soil:** mixed initial soil retains producer, consumer
  and decomposer totals through 120 ordinary ticks; untyped changes reconcile
  against the existing flow stream. Restore at tick 60 replays to the same
  subsequent hashes. Equal-mass channel substitutions change the world hash,
  and revision-1 rules are refused by snapshot admission. The release
  `live_soil_receipt` instrument starts 917 organisms on each of seeds 1, 4 and 7,
  warms 20 ticks and measures 200 more serially. Median core tick times are
  7.34, 7.69 and 7.22 ms; p95 times are 11.98, 12.64 and 9.85 ms; the largest
  measured tick is 15.46 ms. Every tick conserves total matter, and all three
  final snapshots round-trip exactly. These are `World::apply` timings,
  excluding validation, snapshot and rendering cost, and do not establish
  long-run food-web viability or performance after typed-body integration.
  Validation: 507 release tests pass across the core library, flows, matter
  and replay targets, with one existing ignored test. This includes the
  four-seed 4,000-tick conservation run and new typed-soil rejection,
  extraction, foraging, cache reconstruction and world replay checks.
  Local receipt: `Code/testing/mesocosm/tg2b_live_soil.json`; reproduce with
  `cargo run -p mesocosm-core --release --example live_soil_receipt -- <output.json>`.
- **2026-09-09, TG2a accounting and transport:** the release instrument keeps
  101 mg across synthesis, graft, digestion to reserve and mineralization;
  independent expected balances reconcile and postcard receipt replay is exact.
  Replacing the graft's remaining 10 mg of producer nis with consumer nis
  preserves the scalar total and fails with `BalanceMismatch` at that part.
  The 129-by-129 transport probe warms both implementations, alternates order,
  and records nine samples of 100 passes. Median per-pass times are 0.471 ms
  for four channels and 0.733 ms for incumbent scalar Soil on this machine.
  Every channel total is exact; the untyped channel matches incumbent Soil
  after every sampled run. Stored column payload grows from 8 to 32 bytes
  (130 to 520 KiB for 16,641 columns), plus temporary transport scratch.
  This measures two different implementations, not a general claim that four
  channels are cheaper. It admits the kernel for live integration; full TG2,
  TG3, the population tick budget and visible dietary composition remain open.
  Validation: 497 tests pass across the core library and the existing flows,
  matter and replay targets, with one existing ignored test. This includes 16
  new accounting/transport tests, the four-seed 4,000-tick scalar conservation
  run and shipping-population conservation. Scoped formatting and diff checks
  pass; all added source files remain below 600 lines. The pre-existing
  `organism/kingdom.rs` unused-mut warning remains.
  Local receipt: `Code/testing/mesocosm/tg2a_receipt.json`. Reproduce with
  `cargo run -p mesocosm-core --release --example typed_matter_receipt -- <output.json>`.
- **2026-09-05, TG1 acceptance:** 956 tests pass across eight packages, with
  zero failures and two existing ignored tests, using the latest result for
  each target. The native inspector, four-turn habitat, ordinary burrow walk
  and revision-1 replays pass. All owned Rust files pass scoped formatting and
  stay at or below 600 lines. The thirty-seed, two-arm instrument is complete:

  | Arm | Breathes | Thins | Boils | Collapses | Historical Q10 collapses |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | Drawn | 0 | 29 | 0 | 1/30 | 1/30 |
  | Authored roster | 0 | 17 | 0 | 13/30 | 17/30 |

  Both arms use seeds 1-30, 916 ecology founders plus the played founder,
  10,000 ticks with the existing early-stop rule, and 100-tick samples. All
  5,252 samples conserve total matter exactly, have zero bodies outside the
  enclosure, and reconcile feeding-mode counts with the living census. The
  authored roster starts with Grazer, Predator and Omnivore readings. No
  ecological constants moved. **TG6 is not achieved:** every run either
  collapses or loses a founded tier. The roster's lower collapse count is an
  observed change from Q10, not proof of a viable food web. Elapsed times were
  collected with eight seed workers and overlapping builds/tests, so they
  are not a serial tick-budget receipt. The full output is
  `Code/testing/mesocosm/tg1_intake/corridor.json`; `verification.json` records
  test sources, sample checks, source and executable hashes. The next integrated
  visible-body slice is VB4a; typed stock and scruple remain TG2/TG3.
- **2026-09-05, TG1 integration:** typed declarations are retained beside each
  part's allocation. One `Organism::admits` answer serves NPC intake, food
  search, played meals and the inspector. A port requires living tissue at its
  declared supporting process; losing that support suspends admission while
  retaining the declaration. The authored browser carries producer and
  consumer intake; the drawn arm retains geometry-seeded defaults. The
  no-port fallback reading does not grant soil uptake without active fixation.
  The initial three-kind target tag lowers the body's current kingdom; it is
  not yet a typed substance account or a part's provenance-bearing scruple.
  Existing signal avoidance and fauna Avoid/Hold policy remain behavioral
  choices layered over port admission; TG4 still owns their defense integration.
- **2026-09-05, native TG1 receipt:** the selected crop reads `active: living
  producers`, and its body reads `omnivore; living producers, living consumers`.
  Four paused views retain world hash `8c655e425264e795` and selected structural
  revision `3c08feca18d1fcee`. Four ordinary moves enter the burrow and produce
  `2fc1d29e8ce0e4b0`; both new traces replay to their recorded hash. Captures
  show the full inspector with zero missing volumes or projection fallbacks.
  Revision-0 JSON recordings are refused before opening the host. Raw historical
  postcard snapshots have no migration receipt and may fail decoding before
  the rules gate; current-format snapshots round-trip the ports and reject
  incompatible world rules. Local evidence:
  `Code/testing/mesocosm/tg1_intake/verification.json`, scenarios, captures and
  replay receipts. Historical CP1 captures below describe their original build.

- **2026-09-05, CP1 review:** camera and addressed-body inspection now have a
  native four-view habitat receipt (`377d774`). Core still derives feeding
  from jaw/crop geometry in `organism/kingdom.rs`; pursuit and bite still
  carry separate Predator/Producer filters in `organism/ecology/movement.rs`
  and `movement/perception.rs`. TG1 remains unimplemented. Its first visible
  consumer should read the same port-admission answer in the existing body
  inspector. A displayed food/threat cue must not infer edibility from colour,
  silhouette or kingdom alone. CP1 cutaways expose Ground for observation,
  and do not grant the controlled organism perception through terrain.

- **2026-09-04.** Nothing verified by this plan yet. The diagnosis it is built
  on is the default creatures plan section 7 Q10 Findings entry of the same
  date: the stand is innocent, the producer tier's balance is the verdict, a
  `FeedingMode::Predator` has no kingdom restriction on its prey at
  `perception.rs:206`, and no body change measured holds the corridor.

## Progress

- **2026-09-04.** Drafted, awaiting Mark. No code touched.
- **2026-09-04, later.** Accepted; the three rulings given as recommended. TG1 dispatches next.
- **2026-09-05.** TG1 implemented, measured and accepted against its done
  conditions. Shared intake, support dormancy, graft/lineage retention, the
  inspector and replay-version refusal are verified. The historical demo's
  graft window no longer guarantees an encounter; its replacement test names
  a fixed carcass fixture and proves transfer/provenance replay from that
  initial state. Natural graft encounters and their visible descendant join
  remain VB4a evidence, separate from this core transaction receipt.

---

## Integration references

The index, playable ecology plan and dependency ledger now link this accepted
plan. Phenotype section 8 owns the visible voxel-body integration; TG6 closes
this first trophic build, while PE4 retains its generated-world acceptance.
