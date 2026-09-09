# Traits, Incorporation Cost, and Trait-Relative Perception — Design Brief (2026-08-29)

**Status: design brief recording Mark's direction, refreshed 2026-08-31 with
the acquisition-execution boundary added 2026-09-01. Two directions in it are
ruled: organism-to-organism edges are the carrier for
durable pair relations, and unlock conditions extend beyond a linear eating
tree. Their shape, key, cost, schedule, and NPC reach remain open.** Companion to
[the forms-of-life brief](2026-08-29_forms_of_life_brief.md), written the same
day. That brief asked what a *form of life* is; this one records what Mark said
about how a form of life is **acquired**, what it **costs**, and how one
organism **reads** another. It does not restate the companion — where the two
touch, this brief cites it and moves on.

**2026-08-31 acquisition ruling.** Eating may provide material, donor
provenance, and evidence, but a food category does not directly award its
matching trait category. Discovery may instead cite survival through stress,
repeated use or failure, environmental exposure, a relationship, an epoch
achievement, a particular donor part or process, or a combination under one
body and world. The existing `learn_from` path is therefore a migration input,
not the target model. Whether NPC lineages acquire through the same evidence
rules remains an open question. The
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns the first
condition-to-descendant proof; this brief retains the unresolved pricing and
perception choices.

**2026-09-01 execution boundary.** A discovery condition is an event-driven,
bounded program over accepted evidence and deterministic reading state. A meal,
injury, relationship change, threshold crossing, or epoch fact routes only to
conditions that declared that input. Survival through a sustained stress uses
an authoritative bounded accumulator or the same replay-derived reducer the
world rule consumes; a view-only trend cannot unlock anything. The evaluator
records the matched evidence, realized candidate component references,
parameters, generator provenance, and digest. It does not poll every organism
on every ecology tick, rerun world generation from a trait name, or become a
`ProcessDef`.

A condition that can fire for anonymous far subjects declares a cohort
reduction; otherwise it pins only the affected subjects to individual
execution. This boundary settles execution shape, not acquisition price, NPC
reach, or the player's information access. The
[ProcessDef plan](2026-08-01_processdef_plan.md#one-displayed-trait-three-compiled-programs)
owns the condition/development/process split.

**2026-09-01, PE2 landed and `learn_from` is subsumed.** The boundary above is
built. `crates/mesocosm-core/src/discovery.rs` holds the evaluator and the
records; `discovery/conditions.rs` holds the fixed condition table;
`world/discover.rs` holds the world's half — the authoritative accumulator and
what a landed discovery does to the line. **`World::learn_from` is gone as a
function and its one honest consequence was kept**: it taught every non-innate
appendage in the *donor's whole recipe* on every meal, and what a meal now
teaches is a word for the organ that was actually consumed, granted by a
condition rather than by the meal itself. `Event::Learned` is replaced by
`Event::Discovered { organism, species, condition }`, whose digest resolves to
the matched evidence, the route, the realized candidate and its parameters.
`Recipe::acquire` survives with a production caller for the first time since it
was written.

**§0's items 1-3 below are superseded by that landing** and are kept as the
record of what the mechanic was. Read them as history: the lesson is no longer
free (it is not a lesson at all — it is a condition), it no longer pays out on
every meal, and `Recipe::assign` still has no production caller, which is the
half PE3 owns. Items 4-6 are untouched and still open. Nothing here settles
acquisition **price**: the discovery costs nothing to come to, and what a body
pays is `Intent::Rearrange`'s development price when it takes the candidate up.
Mark's cost formula, its complexity and proximity terms, and NPC reach remain
this brief's open questions.

**Verification.** Code claims are checked against committed `HEAD 89d09aa` via
`git show HEAD:...`, not the working tree, because TD7 is live. Seven
subsystem readers were run against HEAD and then three adversarial challenges
against their findings. Where a reader and a challenger disagreed I re-checked
the code myself; four such disagreements are resolved in §6, and **two of them
overturned a claim that would otherwise have gone into this brief unchallenged.**

**TD7's dirty set is larger than the round's own file list.** As of writing,
`git status` shows modified: `examples/population_instrument.rs`, `src/axis.rs`,
`src/organism.rs`, `src/organism/ecology.rs`, `src/organism/ecology/rates.rs`,
`src/organism/ecology/tests.rs`, `src/places.rs`, `src/places/soil.rs`,
`src/world/genesis.rs`, `tests/control.rs`, `tests/instinct.rs`,
`tests/meal.rs`, and `crates/mesocosm-mesh/tests/attachment.rs`, plus an
untracked `src/axis/tests.rs`. **`axis.rs`, `world/genesis.rs`,
`tests/control.rs` and `tests/meal.rs` are central to this brief and are in
flight.** Every claim about them is HEAD-state and must be re-checked when TD7
lands. `world/act.rs`, `world/read.rs`, `species.rs`, `body.rs`, `history.rs`,
`snapshot.rs`, `behavior.rs`, `ecology/movement.rs` and
`movement/perception.rs` are **clean**, so the incorporation and perception
mechanics themselves are safe to reason about now.

---

## Mark's message, verbatim

> "Kleptoplasty should be possible as sort of a level up mechanic, with
> increasing metabolic costs to incorporate a consumed trait depending on trait
> complexity and proximity of the originator's lineage to the incorporator's.
> So when do traits originate? Essentially you draw from a rarity weighted bank
> of traits composed from elements weighted by the world's lifeline (like the
> lineage of the world's life... all the traits exposed through adaptation to
> world state); rarity is created through composition of effects (1 effect is
> common; 2, uncommon, 3, rare, 4, legendary, 5+ epic?). That way we can express
> traits as combinations of aesthetic attributes, body structure, capabilities,
> aspects, consequences, curses, magical dynamics, etc.
>
> Agreed on organism to organism edges. Also, edges should be derivable from the
> relation of an organism's trait composition to another's (if you read as flora
> from some apparent traits, an organism with requisite traits (eyes, compound
> eyes, etc.) would perceive it as a plant, but perhaps that's a strategy by the
> flora (predator in disguise?)). And a lot of those relations can exist between
> one organism on a trait by trait basis, plus memory. Also creates the
> situation where a player can be fooled because something appears to be
> something it isn't (deception)."

**What is ruled.** "Agreed on organism to organism edges" answers the
companion brief's §0.3 and Stage 3: **a durable organism-to-organism edge is
adopted as the direction.** That is a ruling on the carrier, not on its shape,
its key, its storage, or its schedule. The companion brief's constraints on it
still stand — it must be an instance of the general model plan's §4.1 relation
mark rather than a private subsystem, and its key has to survive the open
individual-to-cohort storage question.

**Everything else in the message is direction, not a ruling.** The level-up
framing, the cost formula, the trait bank, the rarity ladder, the composition
categories, trait-relative perception, and player-facing deception are all
recorded here so they can be argued with, and several of them collide with
things the repo has already written down. §6 says which, plainly.

**No names are coined in this brief.** Three things in Mark's message need
names and none get one here: the composed trait unit, the acquisition cost, and
the apparent-kind reading. Each needs a naming round with the usual
crates.io / game / studio / trademark checks, per `CLAUDE.md` Terminology. The
companion brief's Open Question 10 already has three unnamed things queued;
this adds to that list rather than jumping it.

---

## 0. The short version

Six things worth reacting to before the detail.

1. **The mechanic is already shipping, and it is free.** `World::learn_from`
   (`world/act.rs:306-336`) runs on every player meal, reads the *eaten
   lineage's recipe tagmata*, and calls `Recipe::acquire` on the eater's
   lineage for every non-innate appendage. Nothing is charged. So Mark's
   sentence is not "build a level-up mechanic" — it is **"price one that
   already runs."** That inverts the framing and makes the work much smaller
   than it reads.

2. **And it already pays out, in the one currency the game has.** `Recipe::
   complexity()` counts non-innate lexicon words at weight 4
   (`axis.rs:298-315`), `World::intricacy` reads it (`world/read.rs:93-98`),
   `world.rs:460` raises `self.frontier` from it every tick, and
   `World::eligibility` (`read.rs:113-136`) gates `TakeControl` on it. The
   complexity frontier is *already* a level: eating something new raises the
   ceiling on what you may next inhabit. `complexity`'s own doc comment says
   so — "eating something new teaches a word, and the ceiling lifts." **There
   is a reward with no price in the shipping build.** Mark's instinct is
   correcting a live imbalance.

3. **But the loop is a half-circuit: nothing expresses what it learns.**
   `Recipe::assign` — the only consumer of the lexicon gate, the only thing
   that turns a learned word into anatomy — has **zero production callers**
   (`axis.rs:242`; every call site is `axis.rs:486/500/510`, tests). And
   `develop_body` reads `tagma.appendage` directly, never the lexicon. So
   today a learned word changes exactly one number and no body, ever. Pricing
   a purchase that delivers nothing is the wrong order of work.

4. **The proximity term already exists, is correct, is tested, and is dead.**
   `Lineages::distance` (`species.rs:227`), wrapped as `World::kinship`
   (`world/read.rs:165`). Its only callers anywhere are
   `tests/control.rs:717/723`. Its own doc comment says it was built for
   exactly this: "one of the axes graft compatibility was ruled to scale with."
   **Mark's hardest-sounding input is a function waiting for its first caller.**

5. **And it returns `None` for essentially every meal.** `World::genesis`
   founds all lineages through `Lineages::found`, which sets `parent: None`
   (`genesis.rs:177`, `species.rs:123`); the only thing that ever creates a
   parent link is `Intent::Speciate` (`act.rs:117`), which is player-only and
   requires a name. The registry is a forest of unparented roots, so
   `common_ancestor` is `None` and `distance` is `None` for every cross-founder
   pair, forever. The epoch-boundary plan rules that `None` is *the honest
   answer* and explicitly forbids inventing a shared ancestor "to make the
   arithmetic work," because that would be "a lie the graft rule then acts on"
   ([epoch boundary plan](2026-08-01_epoch_boundary_plan.md) §, and
   `species.rs:224-226`). **What `None` costs is a ruling Mark owes before any
   formula can be written.** It is the single hardest blocker in the message.

6. **The deception half splits into a nearly-free part and an expensive part,
   and they are not the same work.** "A player can be fooled" is nearly done —
   `guise` is heritable, is set at genesis, and drives colour through
   `mesocosm-genet/src/app.rs:95`. "An organism perceives it as a plant" does
   not work at all, because the ecology reads `o.kingdom()` — the true
   symmetry reading — at `ecology.rs:279`, and **no ecological decision reads
   `guise` anywhere in `mesocosm-core`.** The whole deception subsystem is one
   line away from mattering and has been inert since it was written.

---

## 1. What the code can carry today

Verified against HEAD. TD7-dirty files flagged.

| Piece | Where | State |
| --- | --- | --- |
| Acquisition (the eat→learn half) | `world/act.rs:306-336` `learn_from` | **Runs.** Player-only (early-returns without `controlled()`), unconditional, free. Teaches from the eaten *species' recipe*, not its body |
| The learned bank | `axis.rs:170` `Recipe.lexicon: BTreeSet<Appendage>` | **Runs.** Private, `BTree`-ordered "so iteration and serialization are deterministic". 6 possible values |
| Expression (the learn→grow half) | `axis.rs:242` `Recipe::assign` | **Dead.** Tests only. `develop_body` never consults the lexicon |
| The payoff | `axis.rs:298` → `read.rs:93` → `world.rs:460` → `read.rs:113` | **Runs.** Vocabulary at weight 4 raises the complexity frontier, which gates `TakeControl` |
| Lineage proximity | `species.rs:227` `distance`, `read.rs:165` `kinship` | **Runs since TD10 (2026-08-29):** `ecology/kinship.rs` spends it as a prey-score discount — predation's first production caller. `None` (unrelated founders) reads as full appetite there; note the sign flips for incorporation cost, where the unrelated donor is naturally the expensive one — that half stays open (register §15) |
| Donor identity, per part | `body.rs:87-102` `Origin::Incorporated { from_species, from_part }`, stamped `act.rs:451-459` | **Runs**, coarsely: `from_part` is hardcoded `PartId(0)` |
| Donor identity, per learned word | `history.rs:125-129` `Event::Learned { organism, species, appendage }` | **Absent.** Records the learner, never the donor |
| The graft itself | `act.rs:340-436` `land` | **Runs.** Creates *one* part (two mirrored) from the meal's **root** volume and half-extent carrying the meal's **whole** `biomass_mg`. A 40-part centipede grafts as one box |
| Incorporation cost | — | **None.** `Landed { budget_mg: 0, body_mg: eaten.biomass_mg() }`. The only debit in the path is the meal's own `venom_mg` (`act.rs:284`), player path only |
| Apparent kind | `organism.rs:189` `guise: Kingdom` | **Inert in the sim.** Heritable (`breeding.rs:122`), set at genesis, read only by `is_mimic`/`betrays_itself` (both test-only) and the renderer |
| Actual kind, as the ecology reads it | `ecology.rs:279` `kingdom: o.kingdom()` (**TD7-dirty file**) | **Runs.** True symmetry reading, one global value, same for every observer |
| The one trait-gated percept | `behavior.rs:98` `target_signal: (traits.sensory_parts > 0).then_some(signal)` | **Runs**, on the *movement* path only |
| Per-observer perception | `movement/perception.rs:89-125` `can_perceive` | **Runs**, but answers "can I see it at all", never "what does it look like to me". Short-circuits to `true` above `Tier::Near` |
| Memory | `organism.rs:44-48` `LastSeen`, `movement.rs:305-326` | **Runs.** One slot: one target id, one position, one `u8`. `MEMORY_TICKS = 8`, cleared on any tier change |
| Conservation | `world/read.rs:213-220`, `tests/matter.rs` | **Runs.** `total_matter_mg` = soil + Σ(`biomass_mg` + `energy_mg`); 4 seeds × 4,000 ticks, milligram-exact, with a proven positive control |

Three notes on the table.

**`energy_mg` is matter.** The budget is not a separate currency the world can
burn against — it is milligrams inside the conservation ledger. `tests/matter.rs`
states `# Exceptions / None`. Every milligram any cost model charges must
arrive somewhere with an address. §3 is entirely about that.

**There are two `Trait`s and neither is Mark's.** `epoch::lineage::Trait`
(`epoch/lineage.rs:33-48`) is a fixed 7-variant enum over a `traits: [i32; 7]`
array with escalating gain cost and superlinear upkeep — **structurally the
closest thing in the repo to Mark's level-up mechanic**, and `organism.rs:383-385`
names it "the provisional scaffolding the phenotype plan schedules for
deletion." Nothing but `examples/ecology_lab.rs` calls the whole `epoch`
adaptation round. `axis::Appendage` is the 6-variant vocabulary `learn_from`
actually acquires. Mark's composed trait would be a **third**. Building on the
first is a trap worth naming out loud.

**The graft transfers nothing structured.** `Part` (`body.rs:129-150`) has no
capability, appendage or trait field; what a grafted part can do is
`classify(half_extent)` → `Role` → `Process`, recomputed from geometry on every
call. So there is no per-trait *thing* anywhere between "the meal" and "the
body" for a per-trait cost to be levied on. That is the substrate gap under
Mark's whole first paragraph.

---

## 2. Trait origination and the bank

Mark asks: when do traits originate? His answer is a rarity-weighted bank of
composed elements, weighted by the world's lifeline.

**What exists that measures a lineage against the world's history.**
`WorldRecord` (`record.rs`) holds `Mark { high, holders }` per `(Feat, Scale)`,
joined by max — a genuine join-semilattice, deliberately so. `score::readings`
walks the whole event log once per epoch and notes Growth / Predation / Spread
/ Endurance. `World::end_epoch` (`world.rs:526-535`) is the only `note` caller
and the only method that holds both the world and its `History`.

**And here is the finding that most changes the approach: the epoch boundary
is not wired into the shipped game.** `Runtime::end_epoch`
(`mesocosm-runtime/src/runtime.rs:131`) has exactly one caller and it is a unit
test. Nothing in `mesocosm-genet`, `mesocosm-views`, `mesocosm-lens` or any
binary calls it. In an actual run **`world.epoch` never leaves 0 and
`WorldRecord` stays permanently empty.** A lifeline-weighted draw would today
read an empty record. That is not a reason to abandon the idea; it is a
prerequisite nobody has scheduled, and it should be known before the draw is
designed.

**What the world does not have.** There is no world-level set of "traits that
have ever appeared here." `Recipe::lexicon` is per-lineage. The nearest
world-scoped vocabulary is `development::PartPalette` — four fixed templates,
constant from genesis. And `git grep` for
`symmetric_difference|intersection(|difference(` across all crates returns
**zero hits**: nothing in the repo compares two `Recipe`s, two lexicons, or two
trait sets. So "proximity" today can only mean fork-count. If Mark wants trait
overlap, that is entirely new.

**Three constraints on any draw, all cheap to honour and expensive to miss.**

- **Its own RNG salt.** `world.rs:37-51` rules that any new draw gets a salted
  stream alongside `PLACE_SALT` / `RECIPE_SALT` / `DEVELOPMENT_SALT`, because
  drawing from `World::rng` shifts every subsequent draw and silently
  rearranges the ecology.
- **Deterministic draw *order*, not just value.** `Rng::below` is
  rejection-sampled, so one draw consumes a variable number of `next_u64`
  calls. Two organisms drawing in a different order produce a different world.
- **`BTree` ordering or hash-stable by construction.** `axis.rs:168-169`
  carries the rule explicitly for the existing lexicon.

**Where the bank would live is a real fork.** `WorldRecord` is in `state_hash`
and joins by max; a count, a frequency or a running total **does not join by
max**, so putting one in `Mark` breaks the property the type exists for
(`record.rs:12-31`, and the epoch plan's stop rule "do not break the
semilattice"). `History` is unbounded and deliberately outside the hash, and
`World::apply` does not take one — so no tick rule can read the lifeline at
all without a signature change.

The rarity ladder itself is argued in §6. It is the part of this section that
collides.

---

## 3. Incorporation as a costed level-up

This is the strongest part of Mark's message and it has the cheapest seam in
the subsystem.

### Where the milligrams come from

Under TD6 there is no combustion. Three honest destinations, not equivalent:

**(a) Skim the tax out of the meal; it falls to the ground.** `metabolize`
already computes

```rust
let column = self.soil.column_at(eaten.position);
let unkept = eaten.biomass_mg() - landed.budget_mg - landed.body_mg;
self.soil.deposit(column, unkept + eaten.energy_mg + spilled);
```

(`act.rs:291-294`). `Landed`'s own doc comment says it exists because "the
closed cycle needs one more thing from them than the outcome says — their
**sum**, because whatever the meal weighed and neither of them took has to go
back into the world. (TD6)". Reduce `landed.body_mg`, attach that much less,
and the existing arithmetic carries the tax into the soil with **no new code
and no new sink.** The reading is diegetic: a complex, distantly-related trait
is messy to graft — you waste more of the carcass getting it. This is the
recommended shape.

**(b) Charge the eater's reserve.** Precedent one block up: the venom debit at
`act.rs:281-286` computes `spilled` from the saturating floor and routes it
into the same deposit. Same three lines. But it inherits the unresolved
question in that comment — "a debt or damage model is a later decision" — and
it makes incorporation lethal at the margin.

**(c) A recurring surcharge on upkeep.** Best match for "metabolic cost" in
the ordinary sense. Attaches at `Organism::upkeep_mg` and settles honestly at
`ecology.rs:307-309`. **This collides head-on with TD7**, which is rewriting
exactly that function to price rent off `actuator_span`. It also breaks a
property upkeep currently has: rent today is derived purely from what the body
*is*. A surcharge keyed on where a trait *came from* means two byte-identical
bodies pay different rent. That may be wanted; it is not free, and it is the
first place this design starts to look like a second authority (§6).

### Two mechanical problems that must be solved either way

**The ledger is settled before the world knows what was learned.**
`self.soil.deposit(...)` runs at `act.rs:292-294`; `self.learn_from(&eaten)`
runs at `act.rs:295`. Any cost that depends on *what was learned* — which
appendages were new, how far the donor lineage is — needs `learn_from` moved
above the deposit, or returning a tax the deposit then reads. Small, real, and
it is in a clean file.

**Learning is not tied to incorporation at all.** `learn_from` is called after
the route has already been taken, so **burning a meal teaches the same words as
grafting it**. A starved player who burns everything levels the frontier
exactly as fast as one who builds. That decouples Mark's level-up from the
tradeoff `Route` was created to express (`world.rs:96-115`) and is almost
certainly not intended. It is a bug-shaped design question and it is cheap to
answer.

### The refusal channel exists

`Rejection` (`world.rs:177-207`) already carries `InsufficientMass` and
`NoRoom`, and `metabolize` is deliberately structured so that everything which
can refuse is checked **before** `self.organisms.remove(index)` at `act.rs:249`.
A graft too costly to pay for refuses through that existing gate. But note the
TD4 ruling this must not reverse: the burn/build choice was made **diegetic**
on 2026-08-29 — the body decides from budget state, never the player's fingers,
"which is why replays cannot disagree about it" (`world.rs:98-115`). A cost the
body pays automatically is fine. A cost/benefit prompt at meal time is a
reversal of that ruling and should be argued as one.

### One live inconsistency this would sit on top of

`BodyDocument::attach` enforces **no** mass ceiling, while `Organism::gain_mass`
clamps to `mass_ceiling_mg()`. Since `land()` gives the new part the meal's
whole `biomass_mg` but only its **root's** half-extent, incorporation can
create a part far above its own ceiling — a heavy multi-part meal collapsed
onto a small root. `organism.rs:327-330` names eating-adds-parts as the
intended escape hatch from determinate growth, and that is true for a part
attached at its own scale; it is not true for `land()`'s collapsed part. A
graft cost expressed as a fraction of mass is unaffected. One expressed against
the ceiling must resolve this first. (`organism.rs` and `rates.rs` are
TD7-dirty; re-check `part_ceiling_mg` when TD7 lands.)

---

## 4. Trait-relative perception and derived edges

Mark's second paragraph. This is the half with the cleanest architectural fit
and the worst worst-case cost, and the difference between them is entirely a
matter of formulation.

### The one binding structural constraint

`ecology.rs:295` holds `organisms.iter_mut()` across the whole feeding pass, so
a targeting function **cannot** reach into the target's `Organism` to read its
body. That is exactly why `LivingTarget` exists;
`movement/perception.rs:6-9` states it: "Positions and live body shapes are
copied before any organism changes, so every decision sees the same enclosure.
None of this is stored world state."

Any per-observer appearance model therefore has the signature
`fn(observer: &Organism, target: &LivingTarget) -> _`. Whatever the rule needs
about the target must be lifted into `LivingTarget` at its build site
(`ecology.rs:271-284`, **TD7-dirty**). This is not optional.

### The good news, and it is genuinely good

`LivingTarget` derives only `Clone, Copy, Debug` — **not `Serialize`**
(verified, `perception.rs:23`). It is rebuilt once per tick, before anything
mutates, inside a loop that already does two O(parts) body walks per organism
(`biomass_mg()` and `walker_shape()`). So:

- **It already is the cache, invalidated at exactly the right granularity.**
- **Widening it is hash-neutral.** Zero replay-hash movement from the struct.
- **Memory is free.** Its seven fields pack to 42 bytes of payload at align 8,
  which pads to 48; a fixed-width apparent-trait word lands in the existing tail
  padding. At 4,700 organisms the whole vector is ~226 KB either way. It stays
  `Copy`, which the derive and the by-value returns at `movement.rs:152-155` and
  `:237-247` require.

The observer side has the same property if the mask is computed **on the
stack** in `policy_living`, where `FaunaTraits::read` already walks the body —
and **not** stored in `FaunaTraits` or `FaunaSenses`. Both of those derive
`Serialize` and reach `state_hash` through `Organism::last_fauna_decision` →
`FaunaDecisionTrace`. Worse, a new *sensor* forces `SENSOR_COUNT` up from 5,
which reshapes `FaunaPolicy::sensor_weights: [[i8; SENSOR_COUNT]; DRIVE_COUNT]`,
the default genotype, and the gene indexing in `inherited()`. That last one
**re-indexes every organism's evolved policy** — a semantic break in the
inheritance operator, not a hash you re-record. That is the line not to cross.

### Cheap formulation versus expensive one

The literal reading of "a lot of those relations can exist between one organism
on a trait by trait basis" is an S×T loop per (observer, target) pair per tick.
That does not survive the scale plan. The measured P6 receipt on this host
([place graph engine plan](2026-08-05_place_graph_engine_plan.md)) is 75 bodies
→ 133 µs/tick, 600 → 3747, which is where the scale plan's `N^1.6` and its
~4,700 saturation figure come from. Fitting `T = aN + bN²` to those four points
puts the pairwise term at roughly 65% of the tick at N=300 and 83% at N=600 — so
the per-pair term *is* the noise floor, and in a quadratic-dominated regime the
saturation population falls as 1/√(per-pair cost). Multiply per-pair work by 4
and the ceiling drops from ~4,700 to ~2,350. **(Analytic, not measured. The
instrument to settle it is committed: `examples/sight_cost_receipt.rs` sweeps
75/150/300/600 and reports per-body as well as per-tick.)**

The cheap formulation: **one fixed-width bitset per target per tick, one
channel mask per observer, and a mask intersect per pair.** What the observer
cannot sense is masked off; the remaining bits index the reading. Deception
falls out for free — a disguised body sets the flora bits and clears the
predator bits, and an observer with the right channel bit sees through it. No
S×T loop. This is the same trick `behavior.rs:98` already plays at one bit;
it needs widening from one channel to a word, not a new mechanism.

### Two per-pair costs are already being paid for nothing

If cost becomes the blocker, the budget is sitting there:

- **The raycast is eager on the movement path and lazy everywhere else.**
  `movement.rs:176` calls `can_perceive` inside the `filter_map`, so every
  in-band candidate gets a `Ground::sees` walk; `choose_living_target`,
  `preferred_living` and `preferred_carrion` all sort-then-`find_map` and pay
  1-2. The rank tuple at `movement.rs:190-195` does not depend on perception and
  `Reverse(order)` makes it a total order with no ties, so max-over-perceivable
  equals sort-then-first-perceivable: **same target, same hash, ~50 raycasts
  down to ~1-2.**
- **`FaunaSenses::read` recomputes `organism.biomass_mg()` per candidate**
  (`behavior.rs:95`) — an O(parts) walk for an observer-only quantity that is
  constant across the loop.

Both are pure redundancy removals. Reclaim them and appearance is net free.
That is the honest answer to "does this survive the scale ambitions": **yes,
if it is paid for out of existing waste rather than added on top**, with a
before/after `sight_cost_receipt` to prove it, which is what the scale plan's
optimization clause demands anyway.

### Far tier is out of scope, and should be said so

`preferred_target` branches to `preferred_living(organism, living.iter()
.enumerate(), ...)` at `movement.rs:272` — the entire living roster, no
distance cap — and `can_perceive_position` short-circuits to `true` above
`Tier::Near`. At 4,700 all-far that is ~22M tuple writes and ~270M comparisons
per tick before any appearance work exists. The scale plan already calls the
far tier "a pessimization today". **Trait-relative appearance is a near-tier
refinement**, and since a far-tier cohort has no individual bodies to be seen
as anything, that is a natural boundary rather than a compromise. It should be
a stated commitment, not a discovery.

### Memory is the part that does not get cheap

`LastSeen` is one `Option` per organism holding a position, not a valence,
decaying in eight ticks and cleared on any tier change. Mark's "plus memory"
wants a pair-keyed, non-decaying, valenced relation. `LastSeen`'s doc comment
rules the shape of any such thing: it "must replay with the organism rather
than live in a host-side perception cache" — so it is world state, inside
`state_hash`, per organism. A pair-keyed store is O(N²) serialized state. If
memory of deception is wanted it has to be **lineage-keyed with bounded
cardinality and a decay rule**, not organism-pair-keyed.

---

## 5. Deception, and what the player is allowed to be fooled about

The companion brief's §G and §1 establish the split: the `guise` lie is told to
*the player*, the `Signal` lie is told to *other critters*, and venom is only
ever collected from the player. This brief adds what follows from that for
Mark's proposal.

**The hinge is one line.** `ecology.rs:279` builds `LivingTarget.kingdom` from
`o.kingdom()`. Route it through an apparent-kind reading and "an organism with
requisite traits would perceive it as a plant" becomes real. The consumer is
already there: `movement.rs:63` is literally
`(mode == FeedingMode::Grazer && target.kingdom != Kingdom::Producer)` — the
"does this read as a plant to me" test.

**The feeding path and the movement path disagree about who can see a
warning, and this is a correction to the companion brief.** The companion
brief §1 says of `Signal`: "It is sensed only at `Tier::Near`, with ground, and
only by a body with at least one `Sense` part." **That is true of the movement
path only.** `choose_living_target` (`movement.rs:46-82`) reads `target.signal`
at `:65` (a hard veto for predators) and `:70` (the +4 grazer danger weight)
with **no tier gate, no ground gate and no `Sense`-part gate**, and it is called
for every grazer and predator at `ecology.rs:345`. The `Sense` gate lives only
at `behavior.rs:98`, inside `FaunaSenses::read`, whose sole caller is
`policy_living`. Verified directly against HEAD. **Consequence today: an eyeless
grazer walks toward a warning-coloured target indifferently and then declines
to bite it.** That inconsistency is exactly the seam a per-observer appearance
model sits on, and it should be resolved deliberately rather than absorbed.
The companion brief owes an erratum.

**Nothing punishes a mimic's victim except the player.** `venom_mg` is charged
at `act.rs:284` only; the NPC meal resolution (`ecology.rs:384-401`) moves
milligrams and never reads it. And `breeding.rs:120-122` copies `signal`,
`venom_mg` and `guise` **verbatim** — no mutation operator — while
`fauna_policy` is `parent.fauna_policy.inherited(seed)`, which mutates one gene
per birth including the warning sensor. **So the response to a signal evolves;
the signal, the venom and the guise do not.** The whole mimicry composition is
authored once at `genesis.rs:151-156` and only differential survival moves the
mix. If deception is meant to be *earned* rather than authored, venom on the
NPC meal path plus a mutation operator in `breeding.rs` are prerequisites, not
polish. The companion brief already flags this as "the highest story-per-line
item in the brief."

**`betrays_itself` is a physically real consequence with no observer.** A fake
producer genuinely takes the Grazer/Predator arm and genuinely earns nothing
from soil, so the tell exists in the world. `betrays_itself` (`organism.rs:449`)
computes it and is called only from `ecology/tests.rs:639`. The cheapest first
step toward deception the player can see through is an *observer*, not a
mechanism.

**One thing TD7 quietly changes here.** TD7's `pyramid()` founding makes the
population roughly 2/3 Producer. Since only Grazers and Predators ever read
`signal`, the population that a warning can deter shrinks; and since genesis's
aggressive mimic always sets `guise = Producer`, most of them will now
*genuinely be* Producers, so `betrays_itself` matches far fewer of them and the
visual lie weakens the same way. Nobody has priced that. Worth a line in TD7's
findings.

---

## 6. The challenges, and what they got right and wrong

Three adversarial passes were run against the readers. Where they found a real
problem it is stated here undiluted.

### Real problems

**The rarity ladder collides with three written rulings.** Not taste — text.

- `2026-07-30_mesocosm_founding_plan.md:466`, the Tone section, about *this
  exact mechanic*: incorporation "carries real ethical weight once critters are
  not necessarily unintelligent — play it with ritual seriousness (Qud's water
  ritual), **never as a loot economy**." Verified.
- `epoch/adapt.rs:172-178`, on the candidate proposer: "Draws blind to the
  lineage on purpose: this is hill-climbing, and the selection pressure lives
  entirely in the scoring. A proposer that already knew which traits were good
  would be doing the search twice, and would quietly stop the world from
  surprising anybody." A rarity-weighted draw **is** a proposer that already
  knows which traits are good. Verified.
- `PROJECT_DESCRIPTION.md` pillar 5: "Everything costs upkeep. The metabolic
  budget is the scarcity that makes each generation a real tradeoff, and it is
  where a point budget belongs in a game about bodies." Verified. Rarity is a
  second scarcity axis alongside the one the pillars already named.

**Effect-count is derived; the tier boundaries are authored, and the tiers are
what people see.** Counting effects is a genuine reading — and the repo already
does this arithmetic in `Recipe::complexity()`, which weights distinct
appendage kinds at 8, regions at 4, vocabulary at 4 and raw length at 1/8, and
which the frontier already reads. So "composition creates richness" is
**already Mark's own live rule under a different name**. What is authored is
that 3 is where "rare" begins and 5 opens a new tier. Once a UI prints
"legendary", players optimise to the threshold rather than to the composition —
the flag-bag failure the companion brief's Niche refusal names, as a label you
farm rather than a slot you pick.

**Rarity-by-effect-count is on the wrong side of an explicit stop rule.**
`2026-08-06_general_model_plan.md:632`: "**Sample constraints, not powers.**
Sanderson's Second Law: a generator over powers produces noise, a generator
over limitations, weaknesses, and costs produces character." A 1/2/3/4/5+
ladder ranks by how much a thing does. **Mark's own message already supplies
the sanctioned lever** — a cost — so reframing rarity as constraint-count
rather than effect-count would move the proposal from "cuts against a stop
rule" to "is what the stop rule asked for," at no loss to what he described.

**The composition grammar is the near-inverse of the ruled one, and nobody had
flagged it.** `general_model_plan.md:422`: "**Fix the Technique axis; generate
the Form axis from the world's own ontology.**" Mark's proposal has no fixed
verb axis — it generates one flat bank and composes within it. And
`general_model_plan.md:272` requires an **exclusion relation** over any
generated combination space; Mark's design names none. The companion brief's
proposed answer (let the conservation economy be the exclusion relation) is
offered for rejection, not ruled, and has a stated limit: the economy sorts by
gradient, so derived exclusion means "loses over time", not "cannot exist".

**Sequencing: a bank is three gates downstream of anything ruled.** F0
(`general_model_plan.md:588-598`) demands one vertical slice — one carrier
state, one cost, one application route, one discoverable consequence — with
"**No registry, no shared type.**" F1 extracts the envelope *only if* F0 and a
second effect repeat the same shape. F5's generated Form axis is gated on F0-F2
existing to generate from. A rarity-weighted bank of composed traits **is** F5.

**Second-authority risk, with a shipped example of the failure.** `CLAUDE.md`:
"Refuse any shift that needs a second simulation **authority**... parallel
authorities are the multiplier that actually hollowed Spore." `process.rs:20-26`
states the positive form: "A part does not carry a list of what it does. Its
processes are derived from its geometry through `classify`... **So a part cannot
be given an ability it has no shape for**." A trait made of aesthetic
attributes, aspects and curses has no half-extent, so it must be *stored*, and
then there are two answers to what a body can do. **`guise` is this exact
mistake at small scale and it is decision-dead** — a stored claim about
appearance that no tick rule could honestly consume, and so none does.

**The version that survives.** A trait as a **reading over the body plan plus
provenance**, not a record beside it. `Origin::Incorporated { from_species }` is
already durable per-part state saying whose lineage a part came from, and
`body.rs:434` already filters parts on it. If "carrying a foreign trait" is
computed from the parts you actually carry and where they came from, geometry
stays the authority and provenance is just an attribute of matter already in
the ledger.

**And one process-layer stop rule quoted in code.** `process.rs:12-18`: "do not
add a broad process catalog before one path is played... A process vocabulary
authored ahead of any consumer becomes a catalog, which is the Spore failure at
a smaller scale. Three processes exist here because one capability needs them;
the fourth arrives when something asks." The PD1b registry
(`Registry`/`ProcessDef`/`ProcessId`, all three `digest()`s) is a fully-built,
fully-tested, entirely unconsumed vocabulary layer — evidence that this repo has
already made this mistake once at the process layer.

### What the challenges got wrong

Four disagreements between readers and challengers; I re-checked each against
HEAD and two of them would have put a false claim in this brief.

1. **"`Recipe::complexity` has no production caller."** **False.**
   `world/read.rs:96` calls it inside `intricacy`, which is read by
   `world.rs:460` (frontier raise, every tick), `act.rs:149` (`TakeControl`),
   `read.rs:127` (`eligibility`) and `genesis.rs:307`. It is the live payoff of
   the whole acquisition loop, which is exactly why §0.2 matters. Do not build
   on the "it's dead" version of this claim.

2. **"Adding a field to `Event::Learned` breaks the `.chronicle` fixtures."**
   **False.** `played.chronicle`, `returned.chronicle` and `rng.chronicle` are
   `Chronicle`/`Deed` records for the cross-game Isometry pipeline
   (`chronicle.rs`), loaded by `tests/homecoming.rs:24/27`. They have nothing
   to do with `Event` or with world snapshots.

3. **"`History` is outside the snapshot, so adding a field to `Event` does not
   move replay hashes."** **False in detail.** `History` is indeed outside, but
   `World.pending: Vec<crate::history::Event>` **is** a serialized `World`
   field (`world.rs:352`) — and its doc comment records that it is deliberately
   *not* `skip_serializing_if`, because "postcard is positional, so a field
   written conditionally cannot be read back. That trap already cost one decode
   failure here." So `Event`'s encoding is inside `state_hash` on any tick a
   `Learned` event is buffered. Cheaper than "every fixture re-records", not
   free.

4. **The companion brief's `Signal` gating sentence.** Corrected in §5 above;
   the gate is on the movement path only, and the feeding path has none.
   Verified directly.

Two further honest qualifications the challenges themselves raised and I agree
with:

- **`adapt.rs`'s blind-proposer ruling governs an unwired lab.** The whole
  `epoch` adaptation round is called only from `examples/ecology_lab.rs:58`.
  Weighting the *player's* acquisition draw would not touch it. That is a
  legitimate scoping move — but the cheaper-looking branch is the more
  expensive one, because a player-only rarity table deepens the player/NPC
  acquisition split the companion brief already calls fatal to its own §4.
- **Mark's ladder inverts the genre convention it borrows** (legendary at 4,
  epic at 5+; standard loot ordering runs the other way). He wrote "5+ epic?"
  with a question mark, so the ordering is not the point — but a borrowed word
  that does not even do its borrowed job is carrying tone, not information.

### What is strong

Said plainly, because the above is long.

- **The cost half is right, on-idiom, and fixes a live imbalance.** There is a
  reward with no price in the shipping build. Charging for it is correcting the
  code, not decorating it.
- **The cost formula's *shape* is the one the plan already asked for.**
  `general_model_plan.md:429-432` wants "a closed-form cost function over the
  parameter vector... keeps a generated space balanced without hand tuning",
  and TD7's own done-condition demands "derived from body-plan numbers, not
  tuned." Complexity × lineage distance is derived.
- **The seam is genuinely free.** `Landed` and `unkept` already exist for
  exactly this reason. Most designs this size have no free seam at all.
- **Lineage proximity is real biology, real code, and nearly a ruled
  direction.** Horizontal transfer really does work better between related
  organisms. And F0's own ruled candidate list — "an organism that metabolizes
  remembered events; **migration following kinship rather than distance**; a
  predator that consumes names or affinities" — already contains a
  kinship-over-distance mechanic as a sanctioned first fantastical slice.
  Nobody in the readers or challenges noticed that; it is the closest thing to
  a green light in the message.
- **Trait-derived appearance is a better idea than what ships**, is a one-line
  hinge, and lands in the one struct that is already the right cache and is
  already outside the hash.

---

## 7. Staged scope: cheap versus structural

Ordered by cost, not by appeal. Everything here inherits the companion brief's
gate: **the terrarium has to breathe first**, and TD6's finding records that
this is the third round running `breathes` has been out of reach. Adding
dimensions to an ecology that does not balance adds search, not answers.

**Cheap, and mostly repairs.**

1. **Close the acquisition loop before pricing it.** `Recipe::assign` needs a
   production caller, or a learned word will never change a body and the price
   buys nothing. This is a prerequisite, not a stage.
2. **Tie learning to the route.** `learn_from` currently fires on burn as well
   as graft. One conditional.
3. **Move `learn_from` above the soil deposit** (or have it return a tax), so a
   cost can depend on what was learned.
4. **Record the donor on `Event::Learned`.** One `from: SpeciesId` field. `Event`
   stays `Copy`. Moves `state_hash` on ticks that carry a `Learned` event; no
   fixture breakage. Without it, "what did this trait cost me and who did I take
   it from" is answerable for parts and unanswerable for vocabulary.
5. **Reclaim the two wasted per-pair costs** (`movement.rs:176` eager raycast;
   `behavior.rs:95` per-pair `biomass_mg`), with a before/after
   `sight_cost_receipt`. Hash-neutral, and it is the budget everything in §4
   spends.
6. **Route `ecology.rs:279` through an apparent-kind reading.** The one-line
   hinge that makes NPC deception real.

**Structural, and each needs a ruling first.**

7. **Price incorporation** (§3). Blocked on the `None`-distance ruling, and on
   TD7 landing if the cost is recurring.
8. **Trait-relative perception as a mask intersect** (§4), near-tier only.
   Blocked on the LOC split — `movement.rs` is 560 against a 600 ceiling and
   cannot absorb it; `perception.rs` at 125 is the obvious home.
9. **NPC acquisition**, i.e. lifting the `controlled()` guard. Companion brief
   Open Question 2, still open, and everything about a world-lifeline trait bank
   presupposes it.
10. **Deception made selectable** — venom charged on the NPC meal path plus a
    mutation operator in `breeding.rs`.
11. **The trait bank itself.** Three gates downstream (F0 → F1 → F5), needs an
    exclusion relation it does not have, and needs the epoch boundary actually
    wired before "the world's lifeline" has any data in it.

**LOC pressure is a real prerequisite, not a cleanup.** At HEAD: `body.rs` 622
and `axis.rs` 608 are **already over** the 600 ceiling; `movement.rs` 560,
`world.rs` 541, `process.rs` 525, `ecology.rs` 507. The two files this design
most wants to extend are the two already over, and TD7 is mid-split on one of
them.

---

## 8. Open questions — Mark's to rule

Ordered by what blocks what.

1. **What does `None` lineage distance cost?** For every meal in a fresh world
   the proximity term is undefined, because genesis founds only unparented
   roots and only the player can create a parent link. The standing ruling
   (`species.rs:224-226`, epoch boundary plan) says `None` is the honest answer
   and forbids substituting a large number. Max cost? Outright refusal? A
   separate "unrelated" tier? Something that makes founders related at genesis,
   which reverses the ruling? **This blocks the cost formula entirely.**

2. **Where does the cost land: the meal, the reserve, or the rent?** §3's (a),
   (b), (c). (a) is free and diegetic; (c) is the ordinary meaning of
   "metabolic cost" and collides with TD7. Not interchangeable.

3. **Rarity tiers, or the repo's own idiom?** Both steelmen, fairly:

   *For tiers.* The companion brief's Niche refusal says explicitly "**Steal:
   the legibility**", and `2026-07-30_games_wing_founding.md:352` lists
   "sortie and return, with the loot reveal ritualized at home" as a wing design
   spine. A player needs to know fast whether a trait is a big deal; effect
   count is an honest proxy; tiers are the standard grammar for making a reveal
   land, and "you learned Vane" is flat. And `adapt.rs`'s blind-proposer rule
   governs an unwired lab, so weighting the player's draw need not touch it.

   *Against.* The founding plan bars the loot economy **by name for this exact
   mechanic**; pillar 5 already names where scarcity lives; "sample
   constraints, not powers" rules against ranking by effect count; and
   `common/uncommon/rare/legendary/epic` is five uncleared coinages arriving in
   one sentence into a repo that killed *zoophyte* on taste and rules "do not
   coin new names for these concepts mid-session." Legibility does not require a
   ladder — it requires *a* legible quantity, and `Recipe::complexity()` is
   already one, already a reading off anatomy, already wired to the frontier.
   And a tier is the most context-free number you can attach to a trait: you can
   compute it without knowing the body, the world, or what else it composes
   with — which fails the companion brief's own criterion for good trait
   composition, taken from RimWorld: "traits interact through shared systems
   (mood, work, health), not because the list is long." A tier travels with the
   card, not with the organism. Contrast `Trait::answers()`
   (`epoch/lineage.rs:68-77`), where a trait
   has no intrinsic worth at all — Frame is worthless in a warm crowded world
   and decisive under gravity and predation. **There is no tier you could print
   on Frame that would be true in two worlds.**

   A note rather than an argument: the repo already has a live metaphor on this
   exact ground. A recipe has a `lexicon`; `acquire` "learns an appendage kind
   by having eaten something that had it"; `assign` fails with
   `Unspeakable::NotInLexicon`; `complexity` counts "vocabulary". Traits are
   already **words a lineage can say**. Richness in a language metaphor is not
   rarity. Whatever is chosen needs a naming round; nothing is coined here.

4. **"Bank" is taken, and the collision is mechanical.** `Lineage::bank: i32`
   (`epoch/lineage.rs:108`) is "what the epoch banked, and what this phase has
   to spend", spent by `Mutation::cost` — and the founding plan uses the word
   the same way ("your **bank of possible filial changes**"). Mark's bank is a
   *pool you draw from*; the repo's is a *budget you spend*, in the same phase of
   play. Which word moves?

5. **Does the level-up stay diegetic?** TD4 ruled the burn/build choice is made
   by the body from budget state, never by the player's fingers, "which is why
   replays cannot disagree about it." A costed incorporation the body pays
   automatically is inside that ruling. A cost/benefit prompt at meal time
   reverses it. Which?

6. **Is the trait a reading or a record?** If a lineage can hold a trait no part
   expresses, there are two answers to what a body can do, and `guise` is the
   shipped demonstration of where that ends up. Reading-over-parts-and-provenance
   survives; a stored bank does not, without an argument this brief cannot make
   for Mark.

7. **Is the fixed-verb grammar retained or replaced?** `general_model_plan.md:422`
   rules a fixed Technique axis and a generated Form axis. Mark's proposal is one
   generated flat bank. If the ruling stands, the bank's shape changes; if Mark
   is replacing it, that should be recorded as a replacement.

8. **What is the exclusion relation?** Required by
   `general_model_plan.md:272` for any generated combination space. The
   companion brief's conservation-economy answer is a gradient, not a wall.

9. **Apparent kind: reading, or per-observer function?** Related to but not the
   same as companion Open Question 1 (does a mixotroph read as prey). That
   question asks whether edibility unbinds from kingdom; this one asks whether
   what a body *appears* to be is one global value or a function of who is
   looking. `guise` exists and answers neither.

10. **Deception memory: what is the key and what is the decay?** `LastSeen` is
    one slot per organism with an 8-tick decay, cleared on tier change, and it
    holds a position rather than a valence. Organism-pair-keyed is O(N²)
    serialized state. Lineage-keyed with a cap and a decay is affordable. Is
    "remembering that a lineage lies" the mechanic, or is it per-individual?

11. **Does the epoch boundary get wired?** `Runtime::end_epoch` has one caller
    and it is a test. Until something calls it, `world.epoch` never leaves 0 and
    `WorldRecord` stays empty — so "weighted by the world's lifeline" has no
    data source in shipped play. This is a prerequisite for the bank that no
    plan currently owns.

12. **"Reads as flora" is a spent word.** `CLAUDE.md`: "the bare word *flora*
    is spoken for platform-side (a moot's accumulated engrams). Game vocabulary
    must not reuse it." The concept in code is `Kingdom::Producer`; there is no
    plant noun and there must not be one without a round.
