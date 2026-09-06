# Trophic Grammar Plan (2026-09-04)

**Status: accepted by Mark 2026-09-04; TG1 next.** The three rulings in section 4 are given. This is PE4's first
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
3. **Three nis kinds in the first world**, one per kingdom. Dead is a state of
   the body, not a kind of matter; a scavenger port reads that state.

Mark's framing for the work that follows, same day: the loops are not all in
place and that is alright; keep adding loops that compose well and surprise;
get granular with the voxels; proceed with the rest of the plan; make
beginning body types; start investigating a beginning set of traits.

---

## Findings

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

---

## Integration references

The index, playable ecology plan and dependency ledger now link this accepted
plan. Phenotype section 8 owns the visible voxel-body integration; TG6 closes
this first trophic build, while PE4 retains its generated-world acceptance.
