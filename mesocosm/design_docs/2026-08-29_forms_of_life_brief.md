# Composable Forms of Life — Research Brief (2026-08-29)

**Status: research brief, refreshed 2026-08-31. Not a plan, not a scheduled
round.** Mark has reaffirmed the product direction that animal-, plant-,
fungal-, and microbial-scale critters belong in the playable roster. The four
stages, their order, and their representation choices remain unruled.

**2026-08-31 integration boundary.** Different forms may receive different
controllers, but every controller proposes intentions to the same matter,
energy, space, body, relationship, and history authority. Before a true
mycelium, clonal stand, or biofilm is built, the game must rule whether the
played subject owns one connected body, several spatial bodies, or a colony
projection. The
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns that
architecture decision and the first second-form proof; this brief keeps the
biological axes and costs.
Written to be reacted to. Opened by Mark alongside TD7 and deliberately kept
out of that round (see
[terrarium dynamics plan](2026-08-29_terrarium_dynamics_plan.md) TD7's closing
paragraph). Nothing here is ruled; the last section is the list of things only
Mark can settle.

**Verification.** The first draft's code claims were checked by an adversarial
pass on 2026-08-29 — eight independent readers against HEAD `fbd49d4` plus
TD7's uncommitted working tree. Of roughly 48 claims, **35 needed correction**,
and the corrected text is what stands below. Three of those corrections changed
the argument rather than the wording: the combination the engine cannot express
is **mixotrophy**, not "motile + photosynthetic" (§0.2, §A, §6); Stage 1 costs
four bindings the draft did not list, one of which is a ruling only Mark can
make (§A, Open Question 1); and the two acquisition loops §4 leans on hardest
are **player-only** (§0.7, §4, §6). Two corrections made the case *stronger*:
`Kingdom`'s own doc comment already promises what §A proposes, and the geometry
a fixing process needs is already named in `plan::Role`. Read the count as
evidence the brief is worth checking, not as evidence it is unreliable.

**Two standing caveats.** Claims about `organism.rs` and `organism/ecology.rs`
are read against files TD7 is actively editing; re-check them when TD7 lands.
And §C is no longer an independent stage — TD7 has already built its harder
half.

## The prompt, verbatim

> "i think it would be cool to be a microbe/virus/germ/parasite/symbiote but on
> the scale of animals, and the playstyle would be more like watching animal you
> inhabit do its thing under your influence instead of being like the cell stage
> of spore. but yeah, we should scale each possible form of life (think of 'em as
> conditionally composable classes like in an rpg? maybe we should think through
> possible forms of life in terms of key traits or characteristics (or existing
> frameworks/prior art) and scope making combinations possible?)"

---

## 0. The short version

Seven things I'd want reacted to before anything else:

1. **The engine already has the right idiom, and the roster is four points —
   three of them reachable.** `Kingdom` is a *reading of anatomy*, not a flag:
   `Organism::kingdom()` reads `body.plan.symmetry`, a bijection. That idiom is
   exactly what "conditionally composable class" should mean. Three honest
   qualifications. The arithmetic is **3 kingdoms + 1 predator bit consulted
   only inside the Consumer arm = four forms of life** (not six; a Producer or
   Decomposer with limbs stays Producer or Scavenger). Only **three are
   reachable at founding** — `genesis.rs:190` sets `limbed = founder.kingdom !=
   Producer`, so every founding Consumer gets a limbed recipe and reads
   Predator; Grazer is a state a lineage falls into by losing limbs, not one it
   starts in. And only **three are economically distinct**: both `match
   feeding_mode()` sites are three-arm (`Producer` / `Grazer | Predator` /
   `Scavenger`), and Grazer and Predator share `feeding_rate_for_mass`.
   Decomposing that one enum into several independent readings is the actual
   work.

2. **The repo is named for a combination it cannot express — and it is not the
   obvious one.** `kleptoplasty` is *Elysia chlorotica*, the sea slug that eats
   algae and keeps the working chloroplasts (`CLAUDE.md` Terminology). Motile +
   photosynthetic is **already representable**: a Radial body carrying
   Limb-shaped parts reads `kingdom() == Producer`, draws soil income, *and*
   has `locomotion() > 1`. `Contract` only promotes Consumer → Predator; it
   never demotes a Producer, and neither `Recipe::acquire` nor `assign` has a
   symmetry gate. Nothing couples symmetry to contractile geometry. What is
   blocked is **mixotrophy** — one body that both fixes and takes a meal.
   Income (`ecology.rs:316`) and targeting (`movement.rs:258`) are both
   *exclusive* matches on `feeding_mode()`, so a body earns one way or the
   other, never both. It is doubly blocked: nothing reads income per part, and
   there is no fixing process for a per-part reading to find. Nor can you eat a
   *part* — `metabolize` swallows a whole organism and grafts it on as one part
   shaped like its root. **The strongest argument for fixing this is already in
   the code.** `Kingdom`'s doc comment reads: "Not a character class: these are
   the three ways of making a living, *and a lineage may combine them*."
   Nothing delivers that. §A is not a new proposal; it closes the gap between a
   shipped doc comment and the code under it.

3. **The missing machinery is one thing — a durable organism-to-organism edge —
   and the delta is smaller than "it does not exist".** Parasite, symbiote,
   germ and inhabitation all need two organisms in a relation that survives a
   tick. Inside the live hashed state the only durable composition today is
   *within* one body (`Attachment { parent }`) — but the pattern and its
   precedent both exist. `LastSeen { target, position, ticks_left }` is a
   cross-body field already serialized on the organism, already inside
   `state_hash`, already stably ordered, decaying over `MEMORY_TICKS = 8`, and
   its doc comment already rules the case §D needs to argue: perception state
   "must replay with the organism rather than live in a host-side perception
   cache." Durable individual-to-individual links exist as well, in the
   serialized `History` (`Event::Fed { eater, from }`, `Event::Born { parent }`,
   documented as the join that makes descent a graph rather than a set of
   chains) — beside the world, outside the hash, read by no tick rule. What
   exists nowhere in the crate is a **pair-keyed, non-decaying, two-sided,
   mass-routing relation that a tick rule reads.** That is the delta, and it is
   what §D has to argue. The general model plan names the carrier — **relation
   mark**, "state on a provenance, descent, contact, or trust edge", worked
   examples contagion and a debt
   ([general model plan](2026-08-06_general_model_plan.md) §4.1). That is a
   taxonomy entered in a 2026-08-07 review, not a ruling and not an
   implementation: the carrier is *named*, not chosen and not built.

4. **Conservation gives most of the composability rule for free — as a
   gradient, not a wall.** TD6 made matter milligram-exact (`tests/matter.rs`)
   and TD7 prices rent on what a body *does*. So most incoherent combinations
   need no authored exclusion table; they lose. Be precise about how much work
   that does. TD7's rent is `UPKEEP_BASE_MG + m^0.75 × (ceiling + span ×
   REFERENCE_SEGMENT_MG) / (UPKEEP_SCALE × ceiling)` — an anatomy factor
   normalized by the body's *own* mass ceiling, roughly 1x for a sessile plan
   to 6.25x for an all-limb one. A large body with token limbs is barely
   punished for anatomy it does not use. The economy sorts strategies; it does
   not refuse them. The sharp exclusions come from elsewhere: readings that
   cannot disagree with themselves (one number cannot say both "rooted" and
   "pursuing"), and reach. Reserve hard refusals for combinations that are
   *representationally* impossible, not merely bad.

5. **The microbe playstyle is the instincts-under-idleness dial — but turning
   it further means reversing it, not lowering it.** `World::held()` already
   runs the arrangement: at an idle run below `INSTINCT_IDLE_TICKS = 30` the
   ecology leaves the critter's dispersal to you; at 30 or more it disperses
   the same body itself. Control never moves either way — your keys steer
   through `controlled()` at any idle run, and `held()` gates only whether the
   ecology *also* disperses that body. The catch: `FaunaPolicy`'s drive scoring
   lives *inside* the call the ecology skips. While a hand is on the body there
   are no drive scores to bias, because the branch never executes. An influence
   channel therefore makes the held body **run** its own `disperse` under a
   biased policy — the keys and the instincts working one body on one tick,
   which is exactly what TD4 arranged them not to do ("holding a key moves the
   critter with no instinct fighting the hand"). That is a bigger and more
   interesting change than "the same dial at lower authority", and it should be
   ruled on in those terms. §5 names the bias surface, which already exists.

6. **A virus is the one form that does not fit the ruled economy, and that is
   Mark's call.** TD5 ruled one economy for all life; TD6 ruled matter
   conserved and growth determinate; **TD7** rules rent derived from the body,
   and TD7 is in flight, not landed. A thing with no metabolism pays no rent
   and reproduces out of someone else's synthesis. Three honest answers exist
   and they are not equivalent — see Open Questions.

7. **Two of the loops this brief leans on hardest run for exactly one organism
   in the world.** `World::learn_from` returns early unless `controlled()` is
   `Some`, and the only incorporation path is `Intent::Metabolize` on the
   controlled body. **No NPC lineage ever learns a word or keeps a part.** NPC
   meals move milligrams and nothing else. Both halves of kleptoplasty —
   `land()` grafting a working part onto *this* body now, and `learn_from`
   teaching the *lineage recipe* a word expressed in later bodies — are
   player-only. That is fine for a played mechanic and fatal for §4's synthesis
   as written, which describes an ecology of lineages acquiring each other's
   parts. Whether to lift it is Open Question 2.

---

## 1. What a form of life IS today

Verified against the code, not against the docs, and re-verified adversarially
(see the Verification note above).

| Reading | Where | Derived from |
| --- | --- | --- |
| `Kingdom` {Producer, Consumer, Decomposer} | `organism.rs`, `Organism::kingdom()` | `body.plan.symmetry` {Radial, Bilateral, None} — a bijection. No committed path mutates symmetry after construction |
| `FeedingMode` {Producer, Grazer, Predator, Scavenger} | enum in `process.rs`; `Organism::feeding_mode()` in `organism.rs` | kingdom, plus `body.performs(Process::Contract)` promoting Consumer → Predator. The "predator bit" is not an independent flag: `Contract` is emitted only by `Role::Limb`, `classify()`'s verdict for a part with one long axis, so the bit reads "has a living long-thin part" |
| `locomotion(): u32` | `organism.rs` | per unsevered part that `Contract`s, its longest half-extent (the max of its three half-extent magnitudes), summed, floored at 1. Read by `dispersal_for` (`rates.rs`) and by the fauna drive selector via `FaunaTraits` (`behavior.rs`). TD7 (in flight) splits out `actuator_span()` — the same sum, unfloored — and prices rent off that |
| `mass_ceiling_mg()` | `organism.rs`, `rates::part_ceiling_mg` | per part, its voxel volume priced at the palette reference segment (100 mg per 125 voxels = 0.8 mg/voxel) and floored at 1 mg, summed over living parts. A [2,2,2] segment is 100 mg; a [4,1,1] limb is 81 voxels, 64 mg. Determinate growth (TD6); TD7 makes it do double duty as rent's normalizer |
| Life-history tempo | `ecology/rates.rs` | maturity/lifespan/gestation on `life_history_mass^0.25` — the parent's brood cost, fixed at birth and never updated by growth; producer income/feeding/decay on live `biomass^0.75`; upkeep is **affine**, `1 mg + biomass^0.75 / 62`. A body that grows to its ceiling gets a bigger metabolism and keeps a juvenile's clock |
| Reproduction | `ecology/breeding.rs`, `Organism::can_reproduce` | mature + gestation elapsed + biomass above `STARVATION_MG * OFFSPRING_COST` (80 mg); brood costs `biomass/4`, and the child's opening budget is provisioned from the parent's reserve **up to that same cost** — zero if the parent has nothing banked. A ready parent's birth still waits silently if `lineage.realize(cost)` fails |
| Information tricks | `organism.rs`; `Signal` read in `ecology/movement.rs`, `venom_mg` in `world/act.rs` (player path only), `guise` in no decision at all | all three inherited verbatim at birth (`breeding.rs`), with no mutation operator |
| Acquisition prerequisite | `axis.rs` `Recipe::lexicon` | `Recipe::assign` refuses a word outside the lineage's lexicon; `World::learn_from` teaches words by eating — for the player-controlled lineage only |
| Part provenance | `body.rs` `Origin::Incorporated { from_species, from_part }` | every taken part remembers whose *species* it was; `from_part` is hardcoded to `PartId(0)` |

Five honest notes on that table.

**What the trophic reading actually forbids.** Not "a radial consumer" and not
"a motile plant" — a radial body with limbs already draws producer income and
already moves. What is forbidden is **one body earning two ways**: income and
prey targeting are exclusive matches on the same enum. Every axis below is, in
effect, a proposal to unbind one dimension from that match.

**Kingdom-as-reading is thinner precedent than one line of code makes it look.**
The bijection is real, but **no committed code path mutates `body.plan.symmetry`
after construction**. Every writer is construction- or inheritance-time
(`Organism::new`, `genesis`, `Species::realize`, `breeding` copying the parent
verbatim); the only other write is inside a `#[test]`. Growth, severing and the
adaptation phase change *parts*, never symmetry, so nothing can move an organism
between kingdoms in a live world. The doc comment's promise — "a reshaped body
therefore changes the role the ecology sees" — is exercised by nothing. Today
kingdom behaves as an inherited species constant with a bijective anatomical
alias. That matters for §4, which pitches "class as a reading" as settled
precedent.

**Mimicry is half-live, and the live half is narrower than it reads.** Only
`Signal` does work in the ecology, and it does two different things: for a
**predator** a warning is a hard **veto** (`choose_living_target` skips any
non-`Plain` target outright), while for a **grazer** it is a +4 danger weight
against a ×16 distance term — a quarter of one Chebyshev step, exactly
cancelling the maximum mass bonus. **Erratum (2026-08-29, found by the
traits-and-perception grounding pass):** the sentence that stood here said a
warning is sensed only at `Tier::Near`, with ground, and only by a body with
a `Sense` part. That gate is real but it is on the **movement/drive** path
only (`behavior.rs:98`). The **feeding** path (`movement.rs:65`, `:70`) has
no tier, ground, or anatomy gate at all. The live consequence is worth
stating, because it is the kind of thing this brief exists to find: **an
eyeless grazer walks toward a warning-coloured target and then declines to
bite it** — its drives cannot see the warning, its jaws can. `venom_mg` is
charged in `world/act.rs:284` and **only on the player path** (the charge also
saturates at zero). `guise` **is** read — `breeding.rs:122` and
`genesis.rs:237`, so it is heritable and flows through birth — but no
ecological *decision* consumes it; on the render half,
`mesocosm-genet/src/app.rs` is the only thing that reads `Organism::guise`,
and `mesocosm-render`'s `kingdom_colour(guise: u8)` is a palette function that
never sees an organism. So the false-kingdom claim is a lie told to *the
player*; the signal claim is a lie told to *other critters*; and the venom
claim is only ever collected from the player. That split is defensible and
worth stating out loud, because a symbiont's whole life is a lie told to a
host. §G carries the consequences.

**Two different enums are named `Role`.** `plan::Role` {Mass, Limb, Plate,
Sensor} is the geometry class produced by `classify(half_extent)` and it drives
processes; a trophic `Role` {Producer, Consumer, Decomposer} is used throughout
`epoch/`. Where this brief says "role" near a body reading — §A and §4's option
(3) — it means the **geometry** one.

**The lexicon is a softer rule than it reads.** Three qualifications, all of
which matter to §F and to Open Question 9. (a) `Recipe::of` — which worldgen's
`seed` uses — pre-loads the lexicon with every appendage the recipe's tagmata
already name, so generated lineages start able to express Limb, Feeler and
Plate having eaten nothing; only `Recipe::founding` starts innate-only. So the
rule constrains vocabulary *beyond the starting body*, not absolutely. (b) It
is a gate on one setter, not an invariant: `Recipe::tagmata` is a public `Vec`
of structs with a public `appendage` field, `catalogue::snake` writes it
directly, and development never consults the lexicon. `divide` copies
appendages without a check. (c) Only the player-controlled lineage ever learns
a word.

---

## 2. The axes

Six axes plus the one that already exists. For each: what the engine has, what
it would need, and what it costs against conservation (TD6's milligram-exact
test) and determinism (replay hash, stable iteration order).

### A. Energy source — photo / chemo / hetero / sapro

**Has.** Two of four, fused into `Kingdom`. `Producer` fixes (income from soil
plus light, the ruled open input); `Consumer` and `Decomposer` are both hetero,
separated by whether the meal is alive.

**Needs.** Income routed per-part rather than per-kingdom: a body earns fixing
income for the parts that perform a fixing process, and grazing income for the
parts that don't. That requires one new process alongside
`Contract`/`Intake`/`Sense`.

The mechanism for adding one is **a data structure nobody calls**, not a code
path. `ProcessId`, `ProcessDef` and `Registry` live in `process.rs` and have
zero callers outside that file; they are not re-exported from `lib.rs:85`. The
engine's real vocabulary is the hardcoded `Role::processes()` match. A pack
cannot mint a process either: `ProcessDef.native: Process` is a required
non-`Option` field and `ProcessId`'s fields are `&'static str` — owned strings
arrive with packs (PD3), not before. So adding a process means: a `Process`
variant, a `NATIVE_DEFS` entry, an arm in `Role::processes()`, and a decision
about which geometry expresses it. Note the trap: adding a variant **compiles
clean** — there is no exhaustive match and no `Process::ALL` — and then panics
at runtime inside `Registry::of_native`'s `.expect(...)`, and the parity test
iterates a hardcoded three-variant list that will not catch it.

**The geometry is already chosen and already named.** Processes are expressed
by `plan::Role`, which has exactly four variants out of `classify(half_extent)`.
Limb, Mass and Sensor are taken. **`Role::Plate` is the one role that expresses
nothing (`&[]`) — and its doc comment reads "Two long axes and one short. Fins,
plates, leaves."** A fixing process expressed by Plate needs no new role, no
change to `classify()`, `Role::ALL`, the development `Palette` or
`Appendage::role()`, and it lands "if nothing flat lives in your world you
cannot grow leaves" in the existing limb idiom, for free. The alternative — a
fifth role — touches all four of those. The *process* still needs a naming
round; the shape it attaches to does not.

**Costs.** The income switch itself is small: two matches. What it does not
unbind is four other things keyed off `Kingdom`.

1. **Target-side edibility — the harder half.** `LivingTarget.kingdom` drives
   `mode == Grazer && target.kingdom != Producer` (`movement.rs:63`) plus two
   prey-scoring gates (`movement.rs:139-140`, `171-172`). "What I earn from"
   and "what I am edible as" are the same enum. A mixotroph that fixes from the
   soil would be **invisible to every grazer** — fixing capacity added to the
   world without prey added to it, which quietly works against the trophic
   pyramid TD7 is founding. This is a ruling, not an implementation detail:
   **Open Question 1**, and it should be settled before Stage 1 is scoped.
2. **`CohortKey { kingdom, mode }`** is the far tier's equivalence class.
   Per-part income makes the key stop implying equal income. A readout break,
   not a determinism break — but the far tier is where most of the world lives.
3. **Determinism is not free.** `FaunaTraits.feeding_mode` is serialized inside
   `last_fauna_decision`, which is inside `state_hash`. Changing what
   `feeding_mode()` returns **moves replay hashes** and re-records fixtures.
4. **`betrays_itself`** (`organism.rs:449`) is literally "a producer guise that
   does not gain mass is not fixing". A per-part fixer falsifies it. Latent —
   no non-test callers — but it is a claim the code currently makes about the
   world.

Sequencing, one level down: `room = organism.intake_room_mg()` is read once at
`ecology.rs:314` and each arm clamps against it, so two income streams in one
tick would both clamp the same stale figure. Not a leak — the spill is
deposited — but it has to be sequenced deliberately.

Conservation: none. Producer income already draws from a specific soil column
via `Soil::draw`, so a mixotroph draws from the same place by the same call.
Genesis: **nothing.** Genesis never calls `feeding_mode()` — it reads an
authored `Kingdom` field and writes symmetry, and TD7's `pyramid(count) ->
Vec<Kingdom>` allocates `Kingdom` directly. (The first draft claimed a genesis
cost here; there is none.)

**Payoff.** This axis alone makes kleptoplasty pay out, and it composes with
every existing loop: the one verb (metabolize), the lexicon (you must have
eaten it), determinate growth (a fixing part brings its own ceiling), TD7's
rent (a fixing part is cheap, a limb is dear), and the soil (a fixing body must
stand where matter is). One second-order effect to state, because it is
invisible: under TD7 the mass ceiling is *also* rent's normalizer, so "a fixing
part brings its own ceiling" silently **reprices rent** for the whole body. Any
proposal that adds parts has to say what it does to the anatomy factor.

**Status caveat.** `feeding_mode()` is at `organism.rs:288`, inside a file TD7
is actively editing, and the "not trivial in `ecology.rs`" judgement above is
read against the same moving target. It is honest against HEAD; re-check it
when TD7 lands rather than treating it as settled.

### B. Matter acquisition — rooted forage / pursuit / filter / infiltration

**Has.** Pursuit, well (`preferred_target` splits seek-range from bite-range for
grazers, predators and, since TD2d, scavengers). Rooted uptake, as of TD6, but
only from the column the body stands on; TD7 rules the forage radius, and the
soil API is already shaped for it — `Soil::columns_within` exists and a test
asserts r=3 reaches 49 of 1,089 columns.

**Needs.** *Filter* needs something flowing to filter from. The enclosure has
exactly one current: soil percolation (each column sheds
`1/PERCOLATION_DIVISOR` into its four neighbours, TD6). A sessile filter feeder
on the soil current is close to buildable, but it needs §A as well as a
current: drawing from a column is a Producer-only income route today, so a
hetero body that filters is a body earning by a route its kingdom does not
have. A filter feeder in water or air needs a second medium, which is new world
state and a new conservation account. *Infiltration* — drawing matter across
another organism's boundary — is the edge from §D/§E, not a separate mechanism.

**Costs.** Conservation: a filter draw is a `draw` from a column, exactly like a
producer's, so it is free. A second medium is not free: any new store must join
`state_hash` and the matter test, and TD6's experience says the test will catch
a 1 mg discrepancy — it caught the meal-settlement bug at **tick 28 of seed 4,
in the *conjuring* direction**, and a separate deliberately-broken control
proves it catches leaks too. Good outcome, real work.

### C. Motility — sessile ↔ motile

**Has.** More than the draft credited, because TD7 has already done the harder
half. `Organism::locomotion()` is the summed contractile geometry, floored at 1.
TD7's in-flight split adds `Organism::actuator_span()` — the same sum,
**unfloored**, so it can return 0 — prices rent on *that*, and leaves
`locomotion()` as `actuator_span().max(1)` for its old readers. So the unfloored
sessility reading **already exists in the working tree**, and a sessile body
plan already pays no motility rent today.

**Needs.** Almost nothing new. What remains is dispersal. `dispersal_for` still
reads the floored `locomotion()`, and the formula at `rates.rs:174` is
`(locomotion()/4).max(1) + u32::from(is_hungry(organism))` — so driving
locomotion to zero is **not sufficient**: the TD2d hunger bonus still gives a
starving sessile body one step. And the draft's "a plant still gets one step of
wander per tick" was false in the other direction: a **fed** producer already
stands perfectly still, because `preferred_target` returns `None` for a producer
and `disperse` moves only toward a target or, targetless, when hungry. **The gap
is only the starving case.** So the change is: hand `dispersal_for` the
unfloored span, and rule what a starving sessile body does, since the hunger
term is the only thing that would still move it.

**Costs.** None to conservation. Determinism moves only through dispersal
changing, which re-records fixtures. The real cost is ecological — a truly
sessile lineage is one bad column from extinction, which is exactly the pressure
TD6's finding described and TD7's forage radius answers. The instrument
(`population_instrument.rs`) is where you find out whether that reads as drama
or as a dead terrarium.

**Scope.** This is no longer an independent stage; see §6.

### D. Scale relationship — free-living / attached / internal

**Has.** Free-living only — but the honest claim is a delta, not an absence
(§0.3). The one cross-body field, `LastSeen`, is one-directional and decays in
8 ticks; it is otherwise exactly the machinery this axis needs, and its doc
comment already rules the argument §D has to make. Durable
individual-to-individual links exist too, in `History` — beside the world,
outside the hash, read by no tick rule. What exists nowhere in the crate is a
**pair-keyed map or field holding a persistent organism-to-organism relation
that a tick rule reads.** That is confirmed, and it is the whole delta.

**Needs.** The edge. Concretely: a serialized, hash-included, stably-ordered
list of (host, guest, kind, state) that survives ticks — the same four
properties `LastSeen` already demonstrates in miniature, minus the decay, plus
a second direction. Attached (ecto-) needs nothing more than that: two bodies
at one position with a mark between them. Internal (endo-) additionally needs a
decision about whether the guest has a position at all, or whether its position
*is* the host's.

**Costs, and this is the section that matters.**

- **Conservation.** The rule that keeps TD6's test green: **the edge routes
  flow; it never merges accounts.** A guest is its own body with its own mass,
  its own `mass_ceiling_mg`, and its own rent paid into its own column. What the
  edge carries is a per-tick transfer with an explicit source and sink, settled
  the way TD6 settles meals — reach, then settle pairwise with both bodies in
  hand. Merging the two masses into one number would be the same class of bug
  that TD6's three-pass restructure was built to prevent.
- **Determinism.** Edges must iterate in a stable order and live inside
  `state_hash`. The precedent is right there: `Soil` is a `World` field,
  serialized, hashed, round-tripped by a test — and `LastSeen` is the same
  precedent at organism granularity.
- **The far tier is a live constraint on the edge's key.** The general model
  plan's own status header still flags an open "authoritative
  individual-to-cohort storage replacement", and `Cohort` carries **no
  `OrganismId`**. If that lands, an edge keyed on individual ids does not
  survive a demotion to Far. Either the edge is designed to survive
  cohort-ization, or holding an edge is itself a reason to keep both parties
  Near — which is a cost, not a free choice.
- **The duplication test, which the draft never ran.** The stop rule is not
  just "no second authority": it is "one simulation authority; projections
  plural and cheap; refuse a second authority, **and refuse duplicating
  functionality the stack already owns**" (place-graph plan §0.1; the same
  clause is in `CLAUDE.md`). The edge has to answer the second half. Built as
  an instance of the §4.1 **relation mark** carrier — state on a contact edge —
  it passes, because that is the stack's own named shape for this. Built as a
  private symbiosis subsystem with its own storage and its own transfer rules,
  it does not. Since §4.1 is a taxonomy and not an implementation, "use the
  existing carrier" today means "be the first instance of it", which is a
  design obligation rather than a free ride.
- **The tick.** Order matters and has to be ruled: does the guest eat before or
  after the host pays rent? TD5's implementation note records the analogous
  answer ("rent is paid before income, so a body is asked whether it is
  starving *after* the day has cost it something") and the same reasoning
  probably applies — but that sentence sits in TD5's **Progress** log as a
  recorded consequence, not in its ruled section, so it is precedent by
  practice, not by ruling. Given this brief's own position that the guest-edge
  tick order is a ruling rather than a derivation, it should not lean on an
  implementation note as if it were one.

### E. Host relationship — none / parasite / mutualist / commensal

**Has.** Nothing. Predation is the only inter-organism relation and it is
instantaneous.

**Needs.** Given §D's edge, possibly nothing else — and this is the most
interesting finding in the brief. In a milligram-exact economy, **the difference
between a parasite and a mutualist is the sign of the net flow across the edge
over a window.** A guest that takes more than it returns is a parasite; one that
returns more than it takes is a mutualist; one whose net is zero is a commensal.
No label, no enum, no authored classification — a reading of the ledger, in
exactly the idiom `Kingdom` already uses (with the caveat from §1 that the
idiom's precedent is thinner than it looks, since nothing today moves an
organism between kingdoms at runtime; a flow reading would be the *first* one
that actually changes at runtime, which is an argument for it, not against).

That also gets the biology right in a way a label cannot: the parasite-mutualist
boundary is famously conditional in real ecology (the same partner is mutualist
in a rich year and parasitic in a lean one), and a reading of flow reproduces
that for free. It makes the RimWorld bar too: "my symbiont line turned parasitic
when the enclosure thinned" is a sentence about a run, generated rather than
authored.

**Costs.** The window is a new piece of per-edge state (a running net), which is
cheap but must be hashed. The obligate/facultative distinction is a second
question and probably a *body* reading, not an edge one: a guest whose body can
pay its own rent alone is facultative, one whose cannot is obligate. That falls
out of §C and §A with no new state.

### F. Reproduction — brood / spores / budding / horizontal

**Has.** Brood only, and it is well-priced: gestation on `m^0.25`, cost
`biomass/4`, and since TD6 the child's opening budget comes out of the parent's
reserve rather than being conjured — capped at that cost, and zero if the parent
has nothing banked.

**Needs.** *Budding* is nearly free given determinate growth: a body at its
ceiling with income to spare currently routes the overflow to budget; routing it
to a bud instead is a third destination for the same overflow, and `Route`'s
doc comment already anticipates exactly this ("provisioning reproduction"). It
also gives determinate growth a second outlet, which is good — right now hitting
your ceiling is a dead end. *Spores* are the interesting one: reproduction
without locomotion, a child scattered further than a sessile parent could carry
it. That needs a scatter rule that is not the parent's step budget.
*Horizontal transfer* — acquiring a trait from something you did not eat —
conflicts with the lexicon rule, but **the rule it conflicts with is softer than
the draft claimed** (§1's fifth note: pre-loaded from the founding recipe, a gate
on one setter rather than an invariant, and player-only in the first place).
Either the edge counts as eating (defensible: a symbiont is metabolically inside
you) or horizontal transfer stays out. Mark's — and it is a smaller relaxation
than it looked.

**Costs.** Budding: conservation-neutral (overflow already had to go somewhere).
Spores: needs a bounded scatter, and TD2b's finding is the warning — birth
scatter was the instrument's actual escapee source, and threw offspring through
walls with no bound check.

### G. Information — honest / Batesian / aggressive

**Has.** The four points exist — `Signal` {Plain, Warning} × `venom_mg` {0, >0},
two of them lies — and `organism.rs`'s doc comment names them correctly
(Batesian mimic: warns without a bite; aggressive mimic: looks plain and bites
hard). But four qualifications matter more than the count, and together they
change what "most ready to be extended" means.

- **Only `Signal` does any work among NPCs**, and it does two different jobs:
  a hard veto for predators, a sub-unit danger weight for grazers (§1). It is
  sensed only at `Tier::Near`, with ground, by a body with a `Sense` part.
- **Venom is decorative among NPCs.** The charge lives on the player path only;
  NPC meals go through `Meal { eater, prey, mass_mg, kind }`, which carries no
  venom term. So a Batesian bluffer and a genuinely armed critter are
  *identically* costly to an AI predator: zero. **The selection pressure that
  would make Batesian mimicry interesting does not exist in the NPC
  population** — what deters AI predators is the `Signal` enum alone. That is
  arguably a bigger hole than the `guise` one.
- **Neither signal nor venom evolves — only their frequencies can.**
  `breeding.rs` copies both (and `guise`) verbatim, with no mutation operator;
  contrast `fauna_policy.inherited(seed)`, which does mutate. The whole mimicry
  space is fixed at world seeding — genesis draws 10% Batesian bluffer, 10%
  aggressive mimic, 20% honestly armed, 60% honest-plain — and only differential
  survival moves the mix.
- **The guise lie and the signal lie are welded, not independent.** Genesis sets
  a divergent guise on exactly one arm — the aggressive mimic — and always sets
  `Kingdom::Producer` there. So at world start every guise-liar is also a
  signal-liar, and `betrays_itself()` names precisely the aggressive-mimic
  population. Guise is not a free third axis today; it is one corner of the
  Signal × venom square. (`is_mimic()`, `signals_falsely()` and
  `betrays_itself()` have no consumers outside tests: `betrays_itself()`
  describes a tell that no observer in the engine and no UI ever checks.)

**Needs.** For symbiosis specifically: a claim aimed at a *host* rather than at
a predator. That is the same carrier — the general model plan's **claim** type,
"observer-relative information that may be false" — pointed at a different
observer. A guest that reads as self is not detected. And the carrier cost is
genuinely low, for a specific reason: **`guise` is already heritable and already
flows through birth**; it is simply never read by a decision. Giving a host a
detection check over the edge costs the check, not the carrier.

**Costs.** None structurally. But read "the axis most ready to be extended"
precisely: the machinery is here and **unfinished**, not here and evolving.
Making mimicry selectable at all needs venom charged on the NPC meal path and a
mutation operator on the inherited traits. Both are small; neither is free; and
both are prerequisites for any mimicry story that is not authored at seeding.

---

## 3. Prior art

### Biology's own frameworks

- **Trophic modes** (photo/chemo × auto/hetero × litho/organo). **Steal:** energy
  source and carbon source are two independent axes; the standard names are
  compounds because the axes cross — and mixotrophy is a *named point in that
  space*, not an exception to it. Mesocosm's `Kingdom` fuses the axes into one
  3-valued reading, which is exactly the collapse §A proposes to undo.
- **r/K selection → life-history theory.** **Steal:** nothing new — the engine
  already ships this. Allometric `m^0.25` life-history rates *are* the fast-slow
  continuum, derived from body size. **Refuse:** r/K as discrete class labels;
  the field itself abandoned them for the reason that applies here. **But do not
  overstate what ships:** `quarter_power` returns an **integer** fourth root, so
  tempo is a step function — every mass from 625 to 1,295 mg reads 5 and gets
  identical timings. In the occupied mass band that is a ladder with very few
  rungs, not a continuum. (`REFERENCE_MASS_QRT` is also 3 against a true 3.162,
  a standing ~5% normalization bias.) The continuum is the right *shape*; the
  resolution is coarse enough that "the engine already ships it" should not
  close the question.
- **Grime's CSR triangle** (Competitor / Stress-tolerator / Ruderal, for plants).
  **The closest existing framework to what Mark is asking for, done well.** Three
  strategies as a *simplex* — every plant is a position, not a class — and
  Grime's own method derives a plant's CSR coordinates from measured leaf traits.
  That is Mesocosm's kingdom-from-symmetry idiom already in the literature, and
  it is the strongest argument that "class" should be a continuous reading of the
  body rather than a slot you pick.
- **Parasite/symbiont ecology.** The standard classification is three orthogonal
  questions: *where* (ecto/endo), *net effect* (parasitic/commensal/mutualist),
  *dependence* (obligate/facultative). **Steal:** all three, and note that they
  map onto §D, §E and §A/§C respectively — location is a position question,
  effect is a flow question, dependence is a body question. **Steal harder:** the
  net effect is a rate, not a label, and the same partner flips with conditions.
- **Semelparity / iteroparity** for the reproduction axis: one brood then death,
  versus many. The engine is iteroparous by construction (`since_offspring`
  resets). Semelparity is cheap and dramatic and nobody has asked for it.

### Games

- **Spore.** Five stages, five genres, one corridor. **Refuse the stage** — but
  cite the law correctly, because the draft cited the retired version of it.
  Wing founding §2 restated the anti-Spore law at the right altitude: a vessel
  must not mint a private answer to *who this creature is, where it came from,
  and what happened to it*. It explicitly retires the older wording ("if any
  stage grows its own engine the wing hollows out") as "directionally right and
  technically wrong, because it forbade too much", and says plainly: **"A vessel
  may absolutely have its own renderer, event loop, ECS, camera, or physics
  dimensionality."** So a separate camera or control map is *permitted* by the
  identity law. What is forbidden is the private answer to identity and history,
  and — separately, from the place-graph plan §0.1 — a second simulation
  authority. Forms-as-classes is fine under both, as long as each is a reading
  over the one substrate. The cost argument against a second camera is real but
  it is a *different* argument, made in wing founding §1 on production grounds;
  see §5.

  **Flag for Mark:** `mesocosm/CLAUDE.md` still carries the retired phrasing —
  "Do not let a stage grow its own engine... the wing's single most load-bearing
  rule". The repo therefore holds both formulations, and this brief cites the
  narrow one while parts of the repo argue from the broad one. Worth resolving;
  it is Open Question 11.
- **Thrive.** The open-source Spore-alike, explicitly stage-structured, and a
  decade of visible difficulty getting past the first stage. **Evidence, not
  theory:** the stage-engine cost is observable in a live project.
- **Rain World.** An ecosystem that runs whether or not you are there, and being
  small and outmatched as a whole playstyle. Already the cited lodestar for the
  camera pull-back (place-graph plan, 2026-08-06). **Steal:** the ecology is the
  content. **Note the difference:** its creatures are hand-authored; Mesocosm's
  are derived, which is harder and is the whole bet.
- **Niche.** Traits as discrete allele slots on a card. **Refuse the card grid** —
  it is a flag bag, the exact opposite of readings-from-anatomy. **Steal:** the
  legibility. Mesocosm's epoch review board is already the good version of this.
- **Plague Inc.** Host-scale play where the player never acts directly; every
  verb is a modification of what a population does. **Steal:** the grammar —
  influence rather than control, and a spend-on-traits board as the whole
  interface, which Mesocosm's epoch review already is. **Refuse:** the
  abstraction. No bodies, no places, no ecology under it.
- **RimWorld.** The bar itself (TD's rulings), plus two mechanics: the
  `BodyPartRecord` tree already cited as Mesocosm's body precedent (wing founding
  §2), and pawn trait composition. **Steal the reason trait composition works
  there:** traits interact through shared systems (mood, work, health), not
  because the list is long. Ten axes that all feed the one matter economy will
  generate more stories than forty that each own a private subsystem.
- **Dwarf Fortress spheres** — cited in the general model plan §4 as the
  exclusion precedent: any generator over that space "needs an **exclusion
  relation**, the same way a deity may not hold precluded spheres." The word is
  *relation*, not *rule*, and it matters — a relation is closer to the derived,
  read-off-the-ledger exclusion this brief proposes than a rule is. See §4.

---

## 4. Forms as conditionally composable classes

### What "conditionally" should mean

Three candidate meanings; the repo has leaned between them twice, in the same
direction — though less firmly than the draft claimed.

1. **A flag you set.** Rejected by precedent. `Kingdom` was deliberately made a
   reading of the body rather than "a genesis decree" — the doc comment says so,
   including "a reshaped body therefore changes the role the ecology sees."
   **The precedent is thin:** nothing in committed code mutates symmetry after
   construction, so that sentence is exercised by nothing (§1). The repo has
   picked the *principle* twice; it has not yet built a case where the reading
   actually changes under a live body.
2. **A prerequisite you earn.** Shipped in `axis.rs`: a lineage cannot express
   an appendage it has never eaten, and `Recipe::assign` refuses assignments
   outside the lexicon. This is the RPG class-prerequisite in its Mesocosm form,
   and it is the acquisition half of kleptoplasty — **for the player only**, and
   with the three softenings in §1's fifth note.
3. **A position you occupy, read off the body.** Grime's CSR, and the engine's
   own idiom (the geometry `Role`, not the trophic one).

The synthesis, which I think is what Mark is describing: **the class is (3), the
condition is (2).** You do not pick "parasite"; you eat something that teaches
your line an infiltration part, the adaptation phase spends the bank on placing
it, and thereafter the ecology *reads* you as a parasite because that is what
your body does. That satisfies the anti-Spore law by construction — a form of
life is a reading over the one substrate, and there is nothing for a form to own
privately.

**The synthesis has one hard dependency the draft did not name.** Both halves of
(2) are player-only: `learn_from` returns early without `controlled()`, and
incorporation happens only through `Intent::Metabolize` on the controlled body.
So as shipped, the synthesis describes a loop that runs for **exactly one
organism in the world**. Either the claim is scoped to the played lineage — a
real mechanic, but a much smaller claim than "forms of life are earned" — or
lifting the restriction is part of the work. Open Question 2.

### Which combinations are coherent

Reading energy source (A) × acquisition (B) × motility (C) × scale (D) × host (E):

| Combination | Real thing | Status |
| --- | --- | --- |
| sessile + photo + rooted + free | a plant | ships (`Producer`) — and a *fed* producer already stands still |
| motile + hetero + pursuit + free | an animal | ships (`Predator`/`Grazer`) |
| motile + sapro + free | a scavenger | ships (`Decomposer`) |
| motile + photo + free, fixing only | a motile alga | **already representable**: radial body, limb-shaped parts |
| **photo + hetero in one body** | **kleptoplastic sea slug — a mixotroph** | **needs §A.** The repo's own metaphor, and the combination the draft mislabelled |
| sessile + hetero + filter + free | barnacle, coral, sponge | needs §A (a non-producer income route) + §C + a current; the soil current works |
| sessile + hetero + internal + host | a gall, an endoparasite | needs the edge (§D) |
| motile + hetero + attached + host | a tick, a lamprey | needs the edge; cheapest edge case |
| sessile + chemo + internal + mutualist | rhizobia, gut flora | needs the edge + §A |
| motile + hetero + free + Batesian | a hoverfly | ships, but inert (§G): fixed at seeding, and venom is uncharged among NPCs |

Incoherent, and worth distinguishing *why*:

- **Rooted + free-living + pursuit hunter.** Not refused by a rule — but the
  draft's receipt was wrong. It is not "it pays motile rent with no motility and
  starves": TD7's rent is normalized by the body's own ceiling (§0.4), so a
  large body with token limbs pays close to nothing extra. What actually refuses
  the combination is sharper: **one number reads both halves.** Rooted means
  `actuator_span() == 0`; pursuit needs contractile geometry, which is what the
  span sums. The body cannot say both. Reach finishes the job — a rooted hunter
  cannot get to prey, and a producer with no target does not move at all. Still
  derived, still no authored rule, but the exclusion comes from the reading and
  from geometry rather than from the rent bill.
- **Internal + pursuit.** Refused *representationally*. A host is one body at one
  position; there is no place graph inside it, so there is nothing to chase.
  Making one would be a second **simulation authority over space** — place-graph
  plan §0.1 and the general model plan's first stop rule. (Not "a second world
  model": wing founding §2 explicitly declines to forbid that — "'shared world
  model' is too strong", since the three vessels' live states will never be one
  in-memory model.)
- **No metabolism at all (a true virus).** Refused by TD5's one-economy ruling
  unless it is given a body and a rent source. See Open Questions.

### The ruling I would propose, for Mark to reject

**Let the conservation economy be the exclusion relation wherever it can be.**
The general model plan says a generator over a combination space needs an
**exclusion relation** (§4, the DF spheres lesson) — a relation, which is what
the ledger already is. Mesocosm has one nobody has to author: combinations that
cannot pay rent do not persist, and combinations whose readings contradict each
other cannot be built. Reserve an authored refusal for the small set that would
require *new world machinery* to even represent — which today is exactly one
item (an interior place graph) and possibly a second (a second medium to
filter).

Two honest limits on that. The economy sorts by gradient rather than refusing
(§0.4), so "derived exclusion" means "loses over time", not "cannot exist"; and
the sharpest exclusions in the table above come from the readings contradicting
themselves, not from the rent bill. Both still beat a hand-written coherence
table, which is a maintenance burden growing as the square of the axes and which
the epoch loop will immediately try to route around.

---

## 5. Inhabitation: playing something that influences rather than acts

### It is the dial that already exists — run backwards

`World::held()` (`world/read.rs`) is the mechanism, and reading it closely is the
load-bearing observation of this brief:

- `held()` names the controlled critter while its idle run is **below**
  `INSTINCT_IDLE_TICKS = 30`, and returns `None` at 30 or more — so the ecology
  takes the body **at** 30, not above it. It is also `None` when nobody is
  embodied or the controlled organism is dead.
- **`held()` gates nothing about steering.** Your keys steer through
  `controlled()` at any idle run; `held()` gates only whether the ecology *also*
  disperses that body this tick. The clean dichotomy holds in practice because
  any non-Idle intent zeroes `idle_run` before the ecology reads it.
- While a hand is on the body the ecology skips **only its `disperse` call** —
  it still ages, pays rent, feeds, breeds and dies (TD4). But `disperse` carries
  more than locomotion: the skipped call also means no `FaunaPolicy` scoring for
  that body, no advance of the policy's recurrent state
  (`fauna_policy.remember`), no `last_seen` refresh or decay, and no
  `last_fauna_decision` written. (What is spared is not cost — the player pays
  `MOVE_COST_MG` himself in `act.rs`.)
- The count is a function of the trace, so replay is unaffected.

So the played critter is **already** an autonomous animal whose steering the
player borrows part-time. But Mark's microbe playstyle is not that same
arrangement at lower authority — **it is a reversal of it.** `FaunaPolicy`
scoring lives *inside* the skipped call (`disperse` → `preferred_target` →
`policy_living`). While a hand is on the body there are no drive scores to bias,
because the branch never executes. An influence channel makes the held body
**run** `disperse` under a biased policy: the keys and the instincts working the
same body on the same tick, which is precisely the arrangement TD4 built to
avoid ("holding a key moves the critter with no instinct fighting the hand").
That is a larger and more interesting change than "the same dial, turned
further", and it should be put to Mark in those terms rather than as a dial
setting.

### Where the influence hook would go — there is no hook today

`FaunaPolicy` scores three `FaunaDrive`s — `Pursue`, `Avoid`, `Hold` — and
records a `FaunaDecisionTrace`. Four corrections to how the draft described it:

- **`FaunaTraits` is not a scoring input.** `FaunaPolicy::score(senses,
  own_mass_mg, sight)` consumes the five quantized sensor values plus own mass
  and sight range. `sensory_parts` gates whether a warning is sensed at all,
  `feeding_mode` filters the candidate list, and `reach`/`locomotion` reach the
  score only through sight range and the candidate set.
- **There is no hook, public or otherwise.** `score`, `selected` and `remember`
  are all `pub(crate)`, with one private caller five frames down.
- **The policy is narrower than the axis it would carry.** It runs only for
  `Grazer | Predator`, at `Tier::Near`, with ground. Producers, scavengers and
  every far-tier body bypass it entirely. That bounds what is
  inhabitable-by-influence, on a different axis from the complexity frontier.
- **Biasing drives cannot steer feeding.** Bite targets come from
  `choose_living_target`, an independent hand-written score
  (`distance*16 + danger - mass/64`) that never consults `FaunaPolicy`. For a
  parasite whose whole point is influencing what its host eats, influence is
  purely locomotor unless it also reaches `choose_living_target`.

**There is, however, a ready-made shape the draft missed.**
`FaunaPolicy.biases: [i16; 3]` is a per-drive additive term already summed into
the score and already inherited and mutated by evolution. A player influence is
arithmetically the same shape as a genotype bias — which means the bound on
player authority is already expressible in evolution's own units ("your nudge is
worth as much as a strong inherited disposition, and no more"). That is a strong
candidate answer to Open Question 7.

This keeps one simulation authority, in the shape the wing has already ruled
twice: "evolved controllers as bounded intent proposers"
([engine ecology rulings](2026-08-18_engine_ecology_rulings_and_review.md)), and
"Burn proposes, the record disposes" ([resident views
plan](2026-08-14_resident_views_composition_plan.md)). The influence proposes;
the ecology disposes.

**The readout is not free.** `last_fauna_decision` is exactly the "why did it do
that" surface a player steering by influence needs — the difference between "the
animal ignored me" and "the animal weighed my nudge against a predator forty
voxels away and the predator won". But it is written only inside `policy_living`,
inside the skipped call, so for an inhabited animal it is stale or `None` today.
Making it the readout is part of the same change, not a freebie, and it records
only the winning candidate.

### What the wing's rules constrain

- **Care granularity does not pass cleanly.** Mesocosm is care for a *species*
  (wing founding §1), and a played symbiont's species is the symbiont line, so
  the granularity is not obviously broken. But it is not clean either. The
  founding record is explicit that the relaxation does not skip the guardrails:
  each thing it permits "still has to pass the three guardrails." And the
  guest's best strategy is to grow a host whose biomass belongs to *another*
  species — which is Open Question 5, and which is a **care-granularity**
  question about whom the player is really growing, not only a win-condition
  one.
- **The three guardrails are the real test.** (1) Home person: unchanged. (2)
  Bounded and diegetic: an influence channel is diegetic by construction — it is
  what a symbiont *does*. (3) **Removal test: this is where it could fail.** The
  argument that bites is the production one, not the identity law: "every person
  is a camera, a control map, and a rendering need... two persons per vessel is a
  production multiplier" (wing founding §1). Note the distinction §3 draws — the
  identity law (§2) *permits* a vessel its own camera and control map; §1
  refuses a person-shift that needs them on cost grounds. Inhabitation passes
  only if it is the same `Intent` stream resolved differently, in the same
  terrarium section, at the same scale.
- **No second simulation authority — and no duplicating functionality the stack
  already owns. Ever.** The full ruling is "one simulation authority; projections
  plural and cheap; refuse a second authority, and refuse duplicating
  functionality the stack already owns" (place-graph plan §0.1; also
  `CLAUDE.md`). The draft quoted only the first half, and the second is the one
  that bites this brief. Applied here: an influence channel that adds a term to
  `FaunaPolicy.biases` passes both halves; a parallel scoring path beside
  `FaunaPolicy`, or a "host AI" running next to the ecology, fails both. Applied
  to §D: an edge built as the §4.1 relation-mark carrier passes; a private
  symbiosis subsystem with its own transfer rules duplicates the typed carriers
  and `Intent`, and fails.
- **Not designed here:** PS2's succession and epoch wiring, and the trust plane.
  Both are adjacent and both are out of scope for this brief by intent.

### The one thing that needs a name and does not have one

The channel itself — the thing that is gentler than `TakeControl` and stronger
than nothing. `Intent` currently has one inhabitation verb and it is total.
**This needs a naming round** per the repo's naming discipline; I am flagging
it, not filling it. Same for the fixing process (§A) and the
organism-to-organism edge (§D) — though for the fixing process the *geometry*
is already chosen and named (`Role::Plate`), so only the process word is open.
Three naming needs, zero coinages.

---

## 6. Scope, staged

Judged by the three tests the brief was asked to judge by: (a) composes with
loops that exist today, (b) the conservation economy carries it with no new
authority, (c) the RimWorld bar — does it generate a sentence about a run.

**Two stop rules the draft did not test itself against, argued rather than
skipped.**

- **Admission by trace.** The place-graph plan rules: "no machinery adopted
  because prior art ships it; acceleration structures are admitted by trace."
  Stages 1 and 3 below are admitted by *design argument* — "pays out the mechanic
  the repo is named for", "highest story yield" — not by a receipt showing the
  current machinery falls short. None of the three tests above is a trace. The
  rule was written about acceleration structures and adopted dependencies, so
  applying it to an in-crate mechanic is an extension of it — but the honest
  form is a gate, and it is cheap: **each stage names the instrument or trace
  reading that shows the gap it closes**, before it is scheduled. For Stage 1
  that is a run where lineages sit at a trophic dead end the enclosure has
  capacity for. For Stage 3, a run where every interesting relation in the
  record is instantaneous.
- **Consumer pull, inverted.** Stage 3 adds new hashed world state whose only
  named consumer — Stage 4 — is "deliberately last". That is the inversion the
  repo corrected elsewhere this week: route B sat deferred on a consumer pull
  nobody ever scheduled and "operated in practice as a text ban" ([views
  founding plan](2026-08-02_views_founding_plan.md) §6, amended 2026-08-29).
  Consumer pull as ruled is about dependency and lane adoption, not about
  ordering mechanics inside one crate, so the counter-argument is available and
  Mark may take it. It should be *made*, not assumed.

**Stage 1 — unbind energy source from symmetry (§A): make mixotrophy
expressible.** Income read per-part instead of per-kingdom, plus one fixing
process, expressed by the already-named `Role::Plate` geometry.
(a) Composes with metabolize, the lexicon, determinate growth, TD7's rent, and
the soil. (b) Conservation: free — the same `Soil::draw`, the same account.
**Not otherwise free**, and the draft said it was: replay hashes move
(`feeding_mode` is serialized inside `last_fauna_decision`, inside
`state_hash`), `CohortKey` stops implying equal income, `betrays_itself` becomes
false for a per-part fixer, and **target-side edibility needs a ruling before
the scope can be fixed** (Open Question 1) — a mixotroph that fixes from soil is
invisible to grazers, which adds fixing capacity to the world without adding
prey. (c) "My hunter's line ate a plant and stopped needing to hunt" — **but
only the played lineage can tell that story today**, because both halves of
kleptoplasty are player-only (§0.7). Either the payoff is scoped to the player's
own line, or lifting the restriction is part of this stage (Open Question 2).
**Still recommended first, and by a wide margin**: it is the smallest change
that pays out the mechanic the repo is named for, and it closes a gap
`Kingdom`'s doc comment has been advertising since it was written.

**Stage 2 — finish sessility (§C). Not an independent stage.** TD7's in-flight
split already does the load-bearing half: `actuator_span()` is the unfloored
reading, rent is already priced on it, and a sessile body plan already pays no
motility rent in the working tree. What remains is dispersal — hand
`dispersal_for` the unfloored span, and rule what a *starving* sessile body
does, since `+ is_hungry` is the only term that would still move it (a fed
producer already stands still). (a) Composes with TD7 and the forage radius.
(b) Free. (c) "The stand couldn't move off the ground it had exhausted" is
already the terrarium's most-told story per TD6's findings; this makes it a
*choice* a lineage made rather than a constant. **Re-scope this once TD7 lands.**
Presenting it to Mark as a standalone stage would be describing work that is
already half done.

**Stage 3 — the attached edge (§D, ecto only), then the internal one.**
Two organisms, one position, a persistent mark, separate accounts, flow settled
pairwise. (a) Composes with predation's targeting and with the meal settlement
TD6 already restructured the tick for. (b) Not free, and the matter test is the
gate — but the test exists and is proven falsifiable, which is the best possible
starting position. Two live constraints beyond it: the edge's key has to survive
the open individual-to-cohort storage question, and the edge has to be an
instance of the §4.1 relation-mark carrier rather than a private subsystem (§D
Costs). (c) Highest story yield of anything in the brief, because §E comes with
it: the parasite/mutualist flip is generated, not authored. Owes the two stop
rules above an answer before scheduling.

**Stage 4 — inhabitation as play (§5).** The influence channel, on top of stage
3's edge. **Deliberately last.** Building the playstyle before the ecology can
express the relationship is the Spore mistake in miniature: a mode with nothing
underneath it. Note that this stage is a *reversal* of TD4's skip rather than an
extension of it (§0.5), and that its shape has a ready-made candidate in
`FaunaPolicy.biases` (§5).

**Not recommended now.** Horizontal transfer (conflicts with the lexicon rule —
needs a ruling first, though the rule is softer than it looks, §1). A second
medium for filter feeding (new conservation account; the soil current is enough
to prove the mode). Spores (bounded-scatter work, and TD2b's escapee finding is
the warning). True viruses (see below).

**Also not free, and worth its own line:** making mimicry *selectable* among
NPCs (§G) — venom charged on the NPC meal path, plus a mutation operator on the
inherited traits. Small, cheap, and a prerequisite for any story about mimicry
that is not authored at world seeding. It is not one of the four stages, but it
is the highest story-per-line item in the brief.

**The honest constraint on all of it:** TD6's finding records that this is "the
third round running that `breathes` has been out of reach," and names the reason
— a consumer:producer *ceiling* ratio of ~4:1 written into the body plans rather
than into the constants. Adding axes to an ecology that does not yet balance adds
dimensions to the search, not answers. Every stage above should be gated on the
terrarium breathing first — which is exactly what TD7 is for, and exactly why
Mark kept this out of that round.

---

## 7. Open questions — Mark's to rule

Ordered by what blocks what, not by topic. The first two block Stage 1, which is
the stage recommended first.

1. **Does a mixotroph read as prey?** `LivingTarget.kingdom` makes "what I earn
   from" and "what I am edible as" the same enum: a grazer only bites a
   `Producer`. So a body that fixes from the soil *and* eats would be invisible
   to every grazer — fixing capacity added to the world with no prey added,
   working against the trophic pyramid TD7 is founding. Three answers, none
   obviously right: edibility follows any fixing part (a mixotroph is grazeable,
   and mixotrophy costs you safety); edibility stays with kingdom (mixotrophs
   are free riders, and the pyramid tilts); or edibility becomes its own reading
   off the body, which is a second unbinding and a bigger job. **This has to be
   ruled before Stage 1 is scoped.**

2. **Does kleptoplasty stay player-only?** `learn_from` returns early without
   `controlled()`, and incorporation runs only through `Intent::Metabolize` on
   the controlled body. No NPC lineage learns a word or keeps a part. §4's
   synthesis and Stage 1's payoff sentence both describe loops that today run
   for exactly one organism in the world. Either that is the design — acquisition
   is the *player's* verb, and the ecology around you is a fixed cast — or NPC
   acquisition is a stage of its own, with its own costs (every NPC meal touching
   a lineage recipe, and recipes changing under bodies that are mid-life).

3. **Is a virus an organism or a condition?** Three answers, not equivalent.
   (i) An organism with a tiny body whose rent is paid *through the edge* out of
   the host's budget — stays inside TD5's one economy, and is the only one of the
   three you can inhabit. (ii) A **condition** carrier on the host (general model
   §4.1's typed carriers) — coherent, cheap, and *not* a critter, so not
   playable. (iii) Out of scope; germ and parasite are enough. Which?

4. **Parasite vs mutualist: reading or label?** I argue for a reading of net flow
   across the edge over a window (§E) — no enum, and the real-ecology conditional
   flip comes free. Is that right, and if so what is the window?

5. **Whose biomass counts when you play a guest?** Mesocosm's goal is your
   lineage's share of the world's biomass. A symbiont's best strategy is to grow
   a host whose biomass is *not yours*. That is either the most interesting
   tension in the idea or a broken win condition, and I cannot tell which from
   here. It is a **care-granularity** question as much as a win-condition one —
   the invariant is care for a species, and this asks which species the play
   actually grows.

6. **Does the complexity frontier gate the host, the guest, or both?**
   `Ineligible::AboveTheFrontier` refuses inhabiting something more elaborate
   than you have earned, permitting a step *down* into a newly viable niche.
   Playing a simple guest inside a complex host is a step down by that rule — but
   you are steering the complex thing. Does the frontier read the guest, the host,
   or the pair? (Related: `FaunaPolicy` only runs for `Grazer | Predator` at
   `Tier::Near`, which bounds what is inhabitable-by-influence on a different
   axis entirely.)

7. **What shape does the influence channel take?** A bias on `FaunaPolicy`'s
   drive scores, or a veto/nudge on the drive it already chose? Both keep one
   authority; they feel completely different to play. A concrete candidate for
   the first: `FaunaPolicy.biases: [i16; 3]` already exists as a per-drive
   additive term that evolution inherits and mutates, so a player nudge could be
   bounded in evolution's own units. And note the prior question this raises —
   influence over drives does not touch feeding, because `choose_living_target`
   never consults the policy.

8. **Reproduction: budding, spores, both, neither?** Budding is nearly free and
   gives determinate growth a second outlet (right now hitting your ceiling is a
   dead end). Spores need a bounded scatter that is not the parent's step budget.

9. **Does the edge count as eating, for the lexicon?** `axis.rs` rules that a
   lineage cannot express what it has never eaten. If a symbiont is metabolically
   inside you, horizontal acquisition is defensible. Worth knowing before ruling:
   the shipped rule is softer than it reads — the lexicon is pre-loaded from the
   founding recipe, it gates one setter rather than holding an invariant, and it
   only ever teaches the player's lineage (§1).

10. **Three things need names** and none are coined here: the fixing/absorbing
    process (§A), the organism-to-organism edge (§D), and the influence channel
    (§5). Each needs a naming round with the usual crates.io / game / studio /
    trademark checks. The fixing process is the narrowest of the three — its
    *geometry* is already chosen and named (`Role::Plate`, "Fins, plates,
    leaves"), so only the process word is open.

11. **The repo holds two anti-Spore formulations.** Wing founding §2 retired "if
    any stage grows its own engine the wing hollows out" as forbidding too much,
    and explicitly permits a vessel its own renderer, event loop, ECS, camera or
    physics dimensionality; but `mesocosm/CLAUDE.md` still carries the retired
    phrasing as "the wing's single most load-bearing rule". This brief cites the
    narrow one. Which stands, and does `CLAUDE.md` get corrected?
