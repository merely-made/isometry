# Playable Ecology Architecture Plan (2026-08-31)

**Status: active integration plan, refreshed 2026-09-05. PE0-PE3 are landed
(see §9); PE4-PE7 remain open. TG1 is the next mechanics slice.**
Mark ruled reproduction as an individual-scale micro-checkpoint and named
trophic visibility as the primary design challenge. He reaffirmed the other
load-bearing directions: body composition determines what a critter can do;
unlock conditions extend beyond a linear eating tree; world criteria shape
generated biology; and animal-, plant-, fungal-, and microbial-scale critters
belong in the playable roster.

This plan owns the **integration contract and full-game proof**: how the
existing body, ecology, history, lineage, generation, scale, and presentation
lanes compose into Mesocosm rather than remaining separate demonstrations. It
does not replace their detailed plans:

- [the played-slice plan](2026-08-28_played_slice_plan.md) owns live host
  wiring;
- [the ProcessDef plan](2026-08-01_processdef_plan.md) owns transformations,
  expression, packs, and validation;
- [the phenotype plan](2026-07-31_phenotype_plan.md) owns developmental
  programs, body realization, and capability;
- [the epoch-boundary plan](2026-08-01_epoch_boundary_plan.md) owns lineage
  review, significance, speciation, and authorship;
- [the traits and perception brief](2026-08-29_traits_and_perception_brief.md)
  owns the unresolved acquisition-cost and perception choices;
- [the forms-of-life brief](2026-08-29_forms_of_life_brief.md) owns research
  on composable forms until individual stages are ruled;
- [the elements and traits memo](2026-08-29_elements_and_traits_memo.md) owns
  the three candidate schemes for generated materials;
- [the scale plan](2026-08-29_scale_plan.md) owns cohort execution, residency,
  zoom, and stress receipts;
- [the dependency ledger](2026-08-07_dependency_ledger.md) remains the
  authority on global order and cross-plan blocking.

If this plan and an owning plan disagree about a domain detail, the owning plan
wins. If their execution order changes, the dependency ledger and this plan are
updated together.

---

## 1. The product rulings and their technical consequences

### Biomass is an intermediate goal; pursuits are readings (Mark, 2026-09-01)

> "Perhaps 'grow your share of biomass' is really an intermediate goal. If
> preventing bad outcomes that will result in the death of your lineage too
> is why you grow your biomass, to change into something with more control
> over/different position in the ecology, you're really trying to save the
> world and shape it to your desires!... there should be more goals than
> just biomass as a target; it's a guideline, but just one thing, kinda like
> colony wealth in rimworld... People should be able to play to end the
> world dramatically, deliberately! Or cultivate a particular world! Or
> really have one lineage with multiple clades and form a self-propagating
> superlineage!"

Consequences this plan adopts:

- **Biomass share is a guideline reading, not a win condition** — the
  RimWorld-wealth shape: a number the world responds to. The epoch plan's
  significance-as-abnormality is the responsive half and already exists.
- **Goals are readings, never quests.** Ending, cultivating, dynasty-
  building, and journey play are all served by one honest record that
  notices what happened, not by authored modes. The moment a pursuit
  becomes a system rather than a reading, the one-authority rule is
  violated.
- **Spatial heterogeneity is product, not decoration** ("only trees in one
  area, a dozen different critters at a lake nearby... migrations,
  temporary events, extreme conditions") — the place graph's product
  justification, promoting S3's region tier from performance work to
  product work; events route through PE4's environmental schedules.
- **Life changes the world at every scale** — algae to core-changing
  extremophiles. Honest terraforming through the conservation ledger;
  world-scale strangeness admitted through PE4's fields gate (named
  consumer, sources, sinks, conservation) and never around it.
- **Supercritters are the primordial-name hook's upstream** ("a big animal
  in the sky... would drastically change the ecology! If they name
  themselves or are named somehow, then they're borgs"). An unnamed
  supercritter is a body whose scale makes it an environmental condition
  for others — one substrate, no second system. Naming promotes it to a
  borg and hands it across the wing (vessel briefs: the primordial-name
  hook), with the naming ceremony plausibly a Paredros verb. Recorded as
  direction; nothing scheduled.

`PROJECT_DESCRIPTION.md` was updated by instruction 2026-09-01: biomass
share unlocks lineage play, and the record's changes ripple to the wing.

**Lineage switching and the gate to godhood (Mark, 2026-09-01).** The quick
criteria, recorded as ruled direction:

1. You may switch to any lineage you have unlocked.
2. Unlocking a lineage takes more biomass than that lineage holds, plus
   conditions that depend on the lineage.
3. Playing an unlocked lineage requires a living individual of it: a new
   one must be created for you to inhabit. Ageless entities rarely
   reproduce, but may carry a karmic cycle instituting rebirth after
   death, or other special divine rules. **TTRPG prior art should be
   surveyed before those rules are designed** (flagged as a research
   brief, not scheduled).
4. Killing a god does not make the corpse disappear. Weird things can
   happen with a god's corpse, especially for worlds built on a god's
   body.

Much of this gate already exists in code and should be extended rather
than duplicated: `World::eligibility` already gates `TakeControl` on the
complexity frontier and on the target being alive, and the `unlocked` set
already exists. The additions are the biomass comparison, per-lineage
conditions (the same evidence machinery as discovery, PE2), and the
divine-reproduction rules. Nothing here is scheduled; PE5 and the epoch
work reach it naturally.

### Reproduction and the epoch are different checkpoints

**Ruled direction, 2026-08-31.** Reproduction is the checkpoint at the scale
of one critter: descent becomes concrete, a parent pays for an offspring, and
the player may continue through a descendant. The epoch boundary is the
checkpoint at the scale of a lineage and its ecology: the world is reviewed,
evidence is weighed, and a developmental program is revised for future bodies.

The two may occur near each other, but one must not silently invoke the other.
Reproduction uses the ordinary breeding transaction and a recorded control
choice. The epoch uses an explicit, deterministic world rule whose exact
trigger remains open. The reproduction checkpoint does not grow a second
lineage editor, and the lineage editor does not manufacture a special body
outside ordinary development.

Here, "checkpoint" first means a bounded decision and a pointable descent
record. Whether it also writes a durable save automatically is a separate
persistence decision.

### Unlocks are evidence, not a diet tree

Eating may supply material, a donor, and an observation. It does not map
directly from food category to reward category. A discovery condition may cite:

- a particular donor part or process;
- survival through a quantified stress;
- repeated use, failure, injury, or recovery;
- environmental exposure or a world cycle;
- a relationship, dependency, or exchange;
- a lineage or epoch achievement;
- a combination of the above under one body and one world law set.

The durable record therefore needs the evidence and route, not only the thing
unlocked. The current `Event::Learned` and `World::learn_from` are migration
inputs, not the final acquisition model.

*(Built 2026-09-01. `World::learn_from` is gone and `Event::Learned` is
replaced by `Event::Discovered`, whose condition digest resolves to the
evidence, the route, the realized candidate and its parameters. `Recipe::
acquire` survives, called by the evaluator for the word a matched condition
grants rather than by every meal for the donor's whole recipe. See §9.)*

### Generated effects use authored verbs

The generator may compose surprising biology, but it does so inside bounded,
audited mechanical languages. It can generate materials, parameters,
conditions, placements, costs, developmental bundles, and combinations. A new
simulation verb still requires an authored evaluator and a real consumer.

`ProcessDef` remains a transformation rather than a universal trait object. A
player-facing generated trait may combine process allocation, anatomy,
material properties, regulation, and lifecycle constraints, with each piece
lowered through its owning validator. This plan does not introduce a universal
`TraitDef` or a callback that runs arbitrary generated code on every tick.

### The body and world remain the authority

Capabilities, trophic roles, danger, and ecological strategies are readings of
realized bodies acting under current world conditions and costs. A generated
label cannot confer flight, photosynthesis, immunity, predation, or symbiosis.
Severing the responsible anatomy, starving its inputs, breaking its route, or
moving it into an incompatible world changes the reading without editing an
ability score.

### Different forms get different controllers over one world

Direct animal movement, plant growth allocation, fungal routing, and microbial
colony bias need not share an interaction design. They do share the same
matter, energy, spatial authority, process evaluators, history, and intent
commit path. Controllers propose bounded intentions; the world resolves them.

One representation decision is deliberately open. A connected body at one
anchor can cover many plant and animal bodies. A true mycelium, clonal stand,
or biofilm may require one played subject to own several spatial bodies or a
multi-anchor body graph. That choice changes identity, occupancy, damage,
resource routing, cohort storage, and projection, so it must be ruled before a
distributed form is implemented.

---

## 2. One authority, several derived records

The target data flow is:

```text
realized world rules + seed + ordered intentions
                         |
                deterministic World mutation
                 /             |             \
          canonical state   causal events   flow events
                 |             |             |
              bodies        History      bounded readings
                 |             |             |
               AI senses   epoch review   warnings / overlays
```

`World` stays the only live simulation authority. `Runtime` stays the
fixed-step driver and owner of replay-derived history. Rendering, UI, physics
advisors, caches, and forecasts remain projections.

### Sparse causal events and dense flow events have different jobs

The existing `history::Event` is biographical: birth, feeding, growth, death,
incorporation, learning, carving, inhabitation, and speciation form causal
lines. Trophic visibility also needs exact, frequent resource movement. Putting
every unit of upkeep, soil draw, and return into the permanent causal log would
make the wrong record carry the wrong frequency.

The working split, with names illustrative rather than ruled, is:

```text
RecordedEvent { tick, optional place, event }

FlowEvent {
    tick, place, process,
    source account, destination account,
    carrier, amount,
    organism and lineage subjects
}
```

One accepted world transaction emits the applicable records at the same commit
point. A resource mutation cannot be visible to the state while absent from
the flow record. Refused transactions emit neither resource movement nor a
false ecological consequence.

The world buffers at most one tick of both record types. Runtime drains them,
records sparse events in `History`, and reduces flows into bounded integer
windows. Replay regenerates the same event sequence and the same windows, so
the raw flow stream does not enter the world snapshot merely to serve UI.

If a future AI rule or unlock condition acts on a trend, the facts it acts on
must become deterministic state or be recomputed through the same bounded
reducer. A view-only warning never becomes hidden simulation authority.

### Realized world rules are saved facts

A durable world records its admitted process definitions, material and field
definitions, environmental schedules, generator version, and digests. A seed
alone is insufficient because generator code changes. Ordinary replay remains
seed plus ordered intentions against those realized rules; it does not rerun a
new generator and hope to recover the old world.

`WorldRules` is a working label for this immutable record, not a settled Rust
type name. The existing `process::Registry` and its digest are its first proven
component. **Built at PD3, 2026-09-01** (`mesocosm-core::rules`): a world
carries a `WorldRules { processes: RulesetDigest }`, serialized and hashed with
everything else, and `snapshot::restore_under` refuses a save whose ruleset is
not the one offered — `SnapshotError::Ruleset`, both digests named — rather
than continuing against whatever biology the build happens to hold. It carries
the identity, not the definitions, the same way `ProcessRef` does one scale
down. Material and field definitions, environmental schedules and the generator
version join it as their own gates land; P3's graft affinity is the first
candidate and is still open (see the processdef plan's PD3 residues).

### Simulation detail changes; authority does not

Micro, meso, and macro are useful viewing and execution scales, not three
competing simulations. Near play may realize bodies, local interaction, and
tactile projections explicitly. Far play may execute equivalent anonymous
subjects as cohorts over coarser spatial fields. The global trophic web and
epoch nutrient graph are derived readings of those same stocks and accepted
flows. A macro summary does not own an extra population counter or settle a
resource transfer.

Use the existing terms **materialize** and **aggregate** for individual/cohort
transitions. “Hydration” already has an ecological meaning in this game and
would make moisture and simulation detail unnecessarily ambiguous. Both
representations remain canonical world state; rendering a body is a further
projection from the materialized record.

---

## 3. The ecology readings contract

Trophic visibility begins in the simulation record and ends in presentation.
It is not a health bar computed from private tuning constants.

### Facts retained in bounded windows

At minimum, by place and lineage where the data supports it:

- living, stored, soil, carrion, and detrital matter;
- captured energy, upkeep, growth, feeding, and return flows;
- births, maturation, reproductive reserve, deaths, and migration;
- matter transferred through a dependency or organism relationship;
- which processes and trophic sources supplied each flow.

Windows are fixed-size and deterministic. Several resolutions may coexist,
such as recent ticks, the current life stage, and the current epoch, but every
retention length is explicit and tested. Cohort materialization and aggregation
must preserve the sufficient totals these readings need. Near/far tier
promotion may trigger that transition, but it is not the transition itself.

### Interpretable leading indicators

The first indicators are ratios and trends whose evidence can be shown:

- **support ratio:** prey or producer growth against consumer withdrawal;
- **replacement ratio:** maturation against mortality;
- **resource runway:** available stock against net depletion;
- **return ratio:** decomposition and soil return against detritus production;
- **recruitment failure:** births that do not reach reproductive mass;
- **dependency concentration:** reliance on one lineage, carrier, or return
  path.

A warning says what moved and over what window, for example: "grazer demand
has exceeded producer regrowth for 240 ticks." It does not present an
unexplained collapse percentage.

### Four presentation distances

1. **World cues:** thinning stands, missing juveniles, accumulating remains,
   altered coloration, motion, or behaviour.
2. **Immediate reading:** a compact trend or refusal near the played critter.
3. **Inspect view:** stocks, flows, dependency edges, place, and uncertainty.
4. **Epoch review:** the causal chain from choices through intermediate
   lineages to collapse, adaptation, or recovery.

The authoritative reading may be omniscient internally. What the player sees
is filtered by the played body's senses, learned evidence, attention budget,
and the review phase's ruled access. Exact postmortem knowledge does not leak
into live play by accident.

The existing population instrument's `breathes`, `thins`, `boils`, and
`collapses` verdicts remain test classifications. They do not become player
language without a separate interaction ruling.

### Forecasts come after measured warnings

The first implementation uses current stocks, observed flows, and
interpretable leading indicators. A later forecast may fork deterministic
snapshots over a bounded horizon and label its assumptions. It is a projection
and never a cheaper simulation authority.

Generic early-warning research makes the same sequencing argument. Critical
slowing, variance, and autocorrelation can precede some transitions, but their
false-alarm and missed-alarm rates can remain severe even under favourable
conditions. Mesocosm therefore starts with mechanistic stocks and flows, keeps
an induced-stress arm and a neutral control, and labels uncertainty before it
experiments with generic tipping-point indicators. See
[Scheffer et al. 2009](https://doi.org/10.1038/nature08227) and
[Boettiger and Hastings 2012](https://doi.org/10.1098/rsif.2012.0125).

---

## 4. Integration order and done-conditions

The dependency shape is:

```text
PE0 flow record and first reading -> PE1 reproduction and succession

PD1b allocation -> PD2 one embodied process -> PE2 embodied discovery

PE0 + PE1 + PE2
  -> P3 branch transfer -> PD3/PD4 authoring parity
  -> PE3 lineage review + P4/PD5 filial expression
  -> PE4 world-generated biology
  -> PE5 another form of life and durable relationships
  -> PE6 cohort scale and ecological zoom
  -> PE7 collapse-and-recovery proof
```

The dependency ledger decides actual dispatch. Each phase below must deliver a
visible game change as well as a technical receipt.

### PE0: one flow record, one useful warning

Add the smallest general flow event and bounded reducer that can reconcile the
current matter cycle. Cover current soil draw and return, feeding, body growth,
upkeep, birth, death, and carrion return. Extend sparse events with tick and
place through an envelope rather than copying those fields into every variant.

The first view should expose one useful, evidenced trend. Replacement ratio is
the smallest current candidate because births, maturation, and deaths already
exist as events; support ratio follows once production and withdrawal are flow
facts.

**Done when:** the sum of recorded transfers reconciles with compartment
changes in a controlled run; accepted and refused transactions cannot disagree
with the stream; replay produces byte-identical readings; draining readings
does not change the world hash; a headed capture shows the first trend; and a
neutral control does not raise the warning that an induced stress arm raises.

### PE1: reproduction and succession as the individual checkpoint

Use the existing adult-mass gate, filial realization, matter debit, parent
link, and `Event::Born`. Add the host experience around a birth involving the
controlled critter, together with the already-planned death, witnessing, and
`Intent::TakeControl` path. The exact choice among continuing the parent,
taking the offspring, or applying a world default is an open interaction
ruling. Whichever is chosen enters the trace.

The lineage program does not change here. A descendant realizes the current
program under its own seed and conditions. Siblings remain in the ecology.

**Done when:** reproduction opens a bounded individual checkpoint; parent,
offspring, cost, and descent are pointable; one recorded choice resumes play;
death can continue through an eligible descendant; siblings persist without
becoming menu inventory; birth flows reconcile to the milligram; replay lands
the same body, control holder, history, readings, and state hash.

### PE2: discovery becomes an embodied option

Complete PD1b's allocation half and PD2's one native, visible process. Replace
the current all-appendages meal lesson with a condition evaluator and an
evidence-bearing discovery record. Distinguish observation, somatic
incorporation, developmental availability, expression, and inheritance.

Part-level eating starts with one bounded proof: consuming a severed or corpse
part settles that part's exact matter and donor evidence and cannot teach
unrelated parts from the donor's recipe. It does not attach a functioning
source branch; live subtree transfer remains phenotype P3. Full live
dismemberment is not a prerequisite for this proof.

**Done when:** one condition unrelated to food unlocks a candidate; one meal
supplies evidence without unlocking an incompatible candidate; one consumed
part settles only its own matter and provenance; the PD2 process is located
on anatomy, paid for, useful under one condition, dormant under another, and
lost when its dependency is severed; direct and automatic fixtures use the
same validator even if NPC acquisition itself remains an open ruling.

**Prerequisites met, 2026-09-01.** PD1b's allocation half and PD2's one
native, visible process both landed the same day (processdef plan §9,
2026-09-01 Progress entries), so this gate's own located/paid/useful/
dormant/severed clause is already satisfied by `Process::Secrete` and does
not need separate PE2 work — `world.gland()` and the two vitals lanes are the
reading surface PE2's inspector can build on rather than invent. What PE2
still owns and this landing does not touch: the condition evaluator, the
evidence-bearing discovery record, the observation/incorporation/
availability/expression/inheritance distinction, and the bounded part-level
eating proof (a severed or corpse part settling its own matter and donor
evidence). `Intent::Rearrange` remains PD2's temporary editor door — direct
and automatic fixtures already share `BodyPhenotype::develop`'s one
validator (PD1b), so PE2's NPC-acquisition ruling has a proposal source to
plug into whenever it is made, but that ruling itself is still open. This
note dispatches no PE2 work.

**Complete, 2026-09-01.** All five done-conditions are receipted; see §9. The
gate's own located/paid/useful/dormant/severed clause is re-receipted through
the discovery route rather than through PD2's temporary fixture:
`the_discovered_candidate_is_located_paid_for_useful_dormant_and_lost_with_its_branch`
starts from a condition and ends with a severed branch that still explains
itself.

### PE3: the lineage checkpoint turns discovery into a descendant

**P3, PD3 and PD4 have all landed (2026-09-01), so this gate's owning path is
clear and PE3 is the next link.** A discovered candidate has a bounded door, a
packed definition, and now an authored one, and all three walk the same
validator — what is still missing is the *review*: choosing among several
candidates, previewing what each would cost, and doing it at a lineage
checkpoint rather than mid-tick. PE2's residue that live subtree transfer was
P3's is discharged for the corpse case, and what a *live* one would still need
is written down in the phenotype plan's P3 entry.

Follow the owning phenotype and ProcessDef order through P3 branch transfer,
PD3 static pack admission, PD4 authoring parity, P4's adaptation bridge, and
PD5 filial expression. Give `Runtime::end_epoch` a production caller and open
the route-B review over the same world. (**Superseded 2026-09-02 by DT4's
reconciliation**: `Runtime::end_epoch` and `World::end_epoch` are deleted rather
than given a caller. The boundary block in `World::apply` is the one door —
reached by the world's own epoch rule or by a hand's `Intent::EndEpoch` — and
`World::reckon` is the separate read-the-past half. See the dev tools plan, DT0.) Replace the provisional scalar
adaptation result with a validated developmental-program revision. The player
reviews ecology readings and discovery evidence, spends a finite lineage
budget, previews a founder, and commits a program for future descendants. At
least one unplayed lineage takes a turn through the same proposal and validator
path.

**PE3a landed 2026-09-02: the boundary, the scorer and the round.** What PE3
still owns is the *review screen* (PE3b).

- **The Timed epoch rule is realized** (`rules::EpochRule`), a versioned world
  rule beside the ruleset: serialized, folded into `WorldRules::digest`, and
  refused by name on a restore that offers a different one. `Gated` and
  `PlayerTriggered` are named as data and answer `built() == false`; a world
  holding either never ends an epoch on its own. The default budget is 1,000
  ticks.
- **The world ends its own epoch**, in `World::apply`, because the rule that
  ends it is a world rule and a headless enclosure has to obey it too. The
  *reckoning* stays a separate call (`World::reckon`) because it reads the past,
  which lives beside a world; the driver does that half and a replay does it
  through the same function, which is what keeps a run through a boundary
  replaying to the same hash.
- **A candidate is scored by growing it** (`World::score`, P4b): the world is
  copied, the revision is committed on the candidate's line in the copy, the
  copy runs a bounded window with nobody at the keyboard, and the flow record is
  read — income against rent. No static formula, no fitness term. The copy is
  discarded and the real world's hash is unmoved.
- **Every unplayed line takes a turn** (`World::adapt_round`), in initiative
  order (descending recipe complexity, ties by species id), committing
  immediately through `World::revise` so a later line answers a world the
  earlier ones have already changed. The played line is skipped: its turn is the
  review.
- **`World::revision_admitted_now` is no longer a placeholder.** It reads
  `World::at_boundary`, so `Intent::Revise` is admitted only while the world
  stands at the lineage checkpoint and is refused `Unrevised::NotYet` otherwise.
  Revision cost stays flat (epoch boundary plan §8 q4, ruled 2026-09-01).
- **The driver holds** at `Occasion::Epoch`, with the PE1 hold machinery and the
  same key: `Intent::Resume` answers it, `Intent::Revise` also answers it and is
  the one answer that does *not* close it. A hand is required, exactly as for a
  birth, so an idle terrarium crosses a boundary without being asked.

`Runtime::end_epoch` kept its production role in the changed shape: the world
ends the epoch, the driver reckons it in the same tick it absorbs, and
`Runtime::end_epoch` is now the manual door for ending one early rather than the
missing caller.

**What the demo's boundaries do** (seed 7, 916 founders, 3,100 steps, budget
1,000). The played line comes to the gland at tick 219, and the boundaries land
at 1,000, 2,000 and 3,000:

| tick | lines that weighed | committed | what moved |
| --- | --- | --- | --- |
| 1,000 | 4 | 3 | three lines take the gland; e.g. one scores 165,978 mg net without it against 174,932 with |
| 2,000 | 1 | 1 | the fourth takes it once its own window holds a birth |
| 3,000 | 0 | 0 | every line that can carry one already does, so nobody has anything to weigh |

Two of the three opened a checkpoint the recording answered with `Resume`. The
third did not, and the reason is the ordering §0 asks for: the played critter
died on tick 3,000, and *who you are now* is the question that tick is asked.
The round had nothing to weigh at that boundary anyway. Those two answers are the first
thing in this chain to move the demo's **intent stream** rather than only its
hash: the script is shifted one step from tick 1,000 on, and the verbs it
exercises are unchanged. The census over the recorded trace carries 1 discovery,
1 graft, 1 succession, 2,605 births, **4 committed revisions** and 1,189 filial
expressions.

**The instrument did not move, and that is the finding.** All 55 seeds of the
six batches read the same verdict, the same decided tick, the same reason and
the same sample curve as `dc4_roster.json` — `0 moved`, and the file was
restored untouched rather than rewritten with new timings. The reason is
structural rather than lucky: **discovery is played-only** (PE2), so a headless
idle enclosure holds no candidate for any line, every `World::candidates` answers
a list of one, and every round is empty. The instrument therefore sees the epoch
end and nothing else. It will start seeing the loop when an unplayed line can
*acquire* — ruling 5's open half — and not before.

**PE3b landed 2026-09-02: the review, the board, and the two keys.** The
player's turn is now a screen. `World::offers` is the table — the status quo
first, then every discovery the line does not already hold, each with its
`World::score`, the price `program::express` would charge the next descendant,
the founder preview's digest, and the reason it cannot be taken when it cannot.
`mesocosm-runtime::Review` assembles that with the reckoning, the trend, the
lineage budget and a **second proposal source**: where the shipped pack declares
an expression script for a candidate's process, `Runner::propose` runs it with
host-owned entropy off the world's own stream and the row lists both proposals
by name. `mesocosm-views::Board` is the fourth chrome lane, Tab moves the
selection, R commits, Enter resumes. See the Progress entry below.

**Done when:** a player finishes an epoch, can explain why each offered change
is available and what it costs, commits one body-program revision, watches one
rival lineage respond, and returns through ordinary development to the live
terrarium in a descendant that can express the admitted option; somatic
incorporation, discovery, lineage commitment, and filial expression remain
distinct records; the old scalar trait array is either removed under its
existing deletion gate or marked explicitly as non-authoritative; replay and
the world record agree.

**Landed 2026-09-02; fully landed 2026-09-04.** Every condition above holds and
is receipted in the Progress entry. The last one — *the old scalar trait array
is either removed under its existing deletion gate or marked explicitly as
non-authoritative* — was **ruled by Mark, 2026-09-02: deletion**, and the
deletion happened on 2026-09-04: `epoch::Trait`, `fitness`, `standing`, the old
round and `examples/ecology_lab.rs` are gone, not merely marked
non-authoritative; the seven authored pressures and the three authored world
profiles are kept as data in `mesocosm-core/src/pressure.rs`, since they seed
PE4's world criteria. See the [phenotype plan](2026-07-31_phenotype_plan.md)
§D4 for the retirement conditions and the file-by-file receipt. The founder
preview's ground ruling landed the same day (Progress, below). **Nothing in PE3
is outstanding.**

### PE4: world criteria generate mechanically distinct biology

**PE4's first build (2026-09-04)** is the [trophic grammar plan](2026-09-04_trophic_grammar_plan.md), which lands typed intake, typed accounts, scruple, defenses, selective edibility, the re-declared roster and the lexicon in that order, and closes the unrestricted predator prey set as a consequence of typed ports.

**The visible integration runs alongside it (2026-09-04):**
[phenotype section 8](2026-07-31_phenotype_plan.md#8-visible-voxel-bodies-integration-2026-09-04)
owns VB0-VB5, connecting inherited development and allocation to procedural
voxel bodies, the played scene, part inspection and visible life-history
changes. Its first geometry/existing-mechanics proof does not wait for typed
matter. Diet-composition appearance waits on TG2/TG3. Visual acceptance and the
TG6 corridor supplement, rather than replace, PE4's generated-world criteria.

**CP1 integration finding (2026-09-05):** the fixed clearing/burrow now has
four hash-neutral views, addressed inspection and ordinary grounded traversal
(`377d774`). This provides a small acceptance habitat, not a generated-world
or trophic-viability result. Terrain readability is now native-verified there;
next join TG1's shared pursuit/bite admission answer to the existing inspector.
Follow with VB4a's visible intake, body change and descendant proof using
existing mechanics. Canopy form remains real developed anatomy; diet-derived
tissue waits for TG2/TG3. PE5's controller and PE6's scale work retain their
own gates. CP1's ground-truth cutaway is not an organism's sensory knowledge.

The material scheme is ruled in section 6 item 4. Generate an immutable
world-law record first, then admitted material
and process parameters, viable founding programs, and candidate weights. Run
reachability and headless ecology checks before exposing the world.

Every generated candidate states its causes, inputs, outputs, costs, counters,
observable cues, and inheritance path. Mechanical fingerprint and rank tests
reject vocabularies whose extra nouns do not reach independent formulas.

The affinity-table pack door moved here from PD4 ([ProcessDef plan](2026-08-01_processdef_plan.md)
§14 residue) waiting on a policy default. **Ruled by Mark, 2026-09-02: a
pack-declared affinity overrides `Founding`; `Founding` is the fallback the
world ships with.** Wiring `World::found` and `WorldRules` to take the
pack-declared component is PE4's to build.

**Done when:** one ordinary world and one impossible world use the same
evaluators but demand visibly different strategies; each has a reachable
energy-capture path, matter-return path, reproductive route, and counterplay;
the generated definitions and digests survive save and replay; changing a
rule-bearing byte changes the digest; material ids resolve through the saved
world-local definition table; every admitted scalar field names its consumer,
dimension, cadence, and conservation rule; and a deliberately mechanically
duplicate vocabulary is rejected by the anti-affix receipt.

### PE5: prove a second form before expanding the roster

Choose one non-animal control model with a real ecological consequence. A
plant growth-allocation proof may fit the current connected body. A fungal or
microbial proof first rules whether its played subject is connected,
multi-anchor, or a colony projection. Add a durable organism relationship only
when the chosen form consumes it; parasite, mutualist, and commensal remain
readings of net flow rather than stored classes.

**Done when:** two forms play differently through different controllers while
using the same world transactions; generated processes remain usable by their
NPC policies; any relationship routes exact matter and survives replay; its
net-flow reading can change as conditions change; and neither controller owns
a private movement, matter, or history engine.

### PE6: make the roster and readings survive scale

Resume the scale ladder against the product load. S3's spatial index,
distance-capped far tier, and cohort execution are the load-bearing simulation
work; S2's windowed atlas remains the presentation path beyond the current
resident wall. Which runs first remains a scale-plan ruling. S4 zoom adds
silhouettes, region tint, and flow presentation over the same place and cohort
facts.

**Done when:** near individuals and far cohorts conserve matter and the
sufficient ecology-reading totals across materialization and aggregation;
thousands of critters hold the configured tick budget; generated processes and
relationships have explicit far-tier reductions or keep their subjects near
for a measured reason; zoom remains out of the trace; and the far view reveals
territories and flows rather than a clipped roster. A materialized subject does
not reroll when it crosses the boundary twice; an aggregate/materialize round
trip with no intervening tick restores the same cohort bytes; and named, played,
relationship-bearing, injured, or chronicled subjects retain pointable
identity. Each far-tier evaluator passes its declared all-individual comparison
envelope, and repeated boundary crossing produces no biological consequence.

### PE7: the collapse-and-recovery proof

Compose the preceding phases in one seeded scenario. The control arm remains
viable. A stress arm deliberately overdraws one trophic support path. The game
surfaces a concrete warning before the chosen recovery point is lost, the
player changes strategy, and a recovery arm restores the flow.

**Done when:** identical rules, seed, and intentions reproduce state, history,
flows, warnings, and lineage revisions; severing or adding anatomy changes a
real capability and cost; a non-feeding discovery and an incompatible meal are
both visible; reproduction and the epoch boundary each perform their distinct
checkpoint job; an ordinary and an impossible world produce different viable
strategies; induced collapse is warned, causally explained, and recoverable in
the control; and the configured population, tick, memory, and render budgets
have measured receipts.

---

## 5. Stop rules

- Do not add a warning counter beside the simulation when a transition can
  emit the fact it needs.
- Do not put dense per-tick flow in permanent causal history or world snapshots
  merely for presentation.
- Do not let a view, forecast, controller, script, renderer, or physics advisor
  mutate authoritative ecology.
- Do not treat every generated trait component as `ProcessDef`; lower each
  component through its sovereign evaluator.
- Do not run unbounded authored or generated callbacks on ecology ticks.
- Do not save only a seed when realized generated definitions affect the
  world's meaning.
- Do not grant a capability, trophic role, or relationship by label when it can
  be read from anatomy, transactions, and current conditions.
- Do not make reproduction a duplicate epoch editor or make the epoch boundary
  a special reproduction route.
- Do not hide omniscient ecological truth behind a body that has not perceived
  or learned it.
- Do not compress a critter into a cohort unless matter, lineage, lifecycle,
  process, relationship, and reading totals have named reductions.

---

## 6. Open rulings

1. At the reproduction checkpoint, does the player continue the parent, take
   the offspring, or choose between them? Is there a world-configurable
   default? **Still open.** PE1 implements the *choice* — both answers are
   recorded intents (`Intent::Resume` and `Intent::TakeControl`) — and stands a
   placeholder default in for the ruling: `Checkpoint::default_answer` continues
   the parent, because that is the only answer that can be taken back. It is one
   function, and ruling otherwise is a one-line change there.
2. ~~What deterministic condition ends an epoch? Timer, world condition,
   lineage event, configurable rule, or a composition of them?~~ **Ruled by
   Mark, 2026-09-01: three separate rules, each a versioned world rule, not a
   composition.** *Timed* ends the epoch when a fixed tick budget is spent and
   is built first. *Gated* ends it when named conditions are all met, and
   comes second. *Player-triggered* ends it on demand and is a dev tool (see
   the [dev tools plan](2026-09-01_dev_tools_plan.md) DT3), never play. PE3
   realizes Timed and replaces `World::revision_admitted_now` with it.
   **Realized 2026-09-02 (PE3a):** `rules::EpochRule::Timed { ticks }`, default
   1,000, serialized and folded into `WorldRules::digest`, refused by name on a
   restore that offers a different one. The other two are variants carrying no
   condition — `built()` answers `false` and `spent()` answers `false` at every
   tick — so a world holding one runs on in its first epoch rather than ending
   it on a guess. `World::revision_admitted_now` now reads `at_boundary`.
   **Player-triggered built 2026-09-02 (dev tools DT3):** `Intent::EndEpoch`
   runs the same boundary a spent budget runs, admitted where
   `EpochRule::admits_demand` says so — under *Player-triggered*, which now
   ends only on the demand and answers `built() == true`, and under *Timed*,
   which takes it as an early end and restarts its budget from that tick, the
   way `World::end_epoch` always has. *Gated* refuses it, because a demand
   standing in for conditions nobody has named would make the two rules the
   same rule. It is a dev tool and never play.
3. Are fungal networks, clonal stands, and microbial colonies genuine
   multi-anchor subjects or connected local critters at the first proof?
4. ~~Which generated-material scheme from the elements memo is the first one
   built?~~ **Ruled by Mark, 2026-09-02: matter is typed by provenance.** A
   milligram carries where it came from, kingdom first (flora, fauna, myco,
   micro, and the meso/macro scale words the world already uses) and lineage
   under it. Storage is scheme A's (typed stock in soil and bodies,
   per-channel conservation, the matter test rewritten first) with the type
   vocabulary world-derived from the roster rather than an authored element
   table; payloads are scheme C's and fire on provenance at the three
   transfer sites. Payloads are part of the generative pipeline: by the time
   a world has a roster it has its payloads. Parts are budgets of typed
   milligrams. No fields in PE4's first world. Composition is two layers:
   the lineage layer (recipe and program declare the provenance of a line's
   tissue and which processes it expresses) and the part layer (the mosaic,
   which records the provenance mix the part was actually built from,
   differing after a graft or an odd diet); a payload's kind is the
   lineage's, its strength the part's mix; no per-organism vectors.
   Disfavoured element pair: a graft with conditioned limits, a small
   milligram allowance with penalties that trait conditions raise (composes
   with PE2's condition table). **Words, ruled by Mark 2026-09-04:** the
   provenance-bearing living substance is **nis** (one form, singular and
   plural; from nisus, striving), and a part's stable mix of it is a
   **scruple**; see the repo `CLAUDE.md` terminology. The lexicon (a term
   table in the packs, read only by views) is folded into PE4. Do not write
   "element" as a term. Mark's 2026-09-04 framing of the roster diagnosis
   sets PE4's first world: the ecology is missing the loops that
   differentiate herbivore, carnivore and omnivore, herbivores should carry
   defenses that make them costly to eat, and not all flora should be
   edible to everything. See the [elements and traits memo](2026-08-29_elements_and_traits_memo.md)
   §7 for the full question set.
5. Do NPC lineages acquire new developmental vocabulary through the same
   evidence rules, or only evaluate inherited candidates? **Still open**, and
   PE2 built so that ruling either way is small: `discovery::Evidence` names no
   player, the evaluator takes evidence and a set of known conditions and
   nothing else, and a `Candidate` lowers through the same validator whichever
   `Arrangement` proposed it. What the NPC path would need is two things PE2
   deliberately did not build — a per-body accumulator for each stress a
   condition reads (or a declared cohort reduction for it, per the execution
   boundary), and a proposal sink in the ecology's own step so an unplayed
   lineage can actually take a candidate up. Neither is a second evaluator.

   **Half answered 2026-09-02 (PE3a):** the proposal sink exists.
   `World::adapt_round` is where an unplayed line takes a candidate up, and it
   takes it through the identical `World::revise`. It offers *inherited or
   already-discovered* candidates only — `World::candidates` reads this world's
   `discoveries` and nothing there proposes a new one — so an enclosure nobody
   has played holds nothing for any line to weigh and its rounds are empty. The
   per-body accumulator is still not built, and acquisition is still open.
6. How much ecology truth is available during live play, and how much becomes
   available only during epoch review or postmortem?
7. What exact recoverability condition makes a trophic collapse terminal in
   the game, as distinct from the population instrument's test verdict?
8. When the scale lane resumes, does S3's correctness and cohort work precede
   S2's wider resident window?

None blocks founding PE0. Question 1 still blocks the reproduction default;
question 2 is ruled and realized, so what it blocked is now only PE3b's review;
3 blocks a distributed PE5 form; question 4 is ruled 2026-09-02, so what it
blocked is now only PE4's build; 5 blocks world-wide acquisition; 6 blocks
final warning presentation; 7 blocks the terminal run condition; and 8 blocks
scale dispatch.

---

## 7. Downstream architecture gates

These are ownership assignments and integration checks, not extra phases. The
detailed requirements and research live in the plan that owns each mechanism.

| Concern | Owning record | First integration gate | Required before admission |
| --- | --- | --- | --- |
| Individual/cohort execution | [Scale](2026-08-29_scale_plan.md) and [place graph](2026-08-05_place_graph_engine_plan.md) | PE6 | Exact zero-tick aggregate/materialize round trip; persistent pointable subjects; named reductions; per-evaluator all-individual comparison envelope; unsupported-process fallback. |
| Generated material vocabulary | [Elements and traits](2026-08-29_elements_and_traits_memo.md#storage-shape-shared-by-all-three) | PE4 | Saved world-local definitions and compact ids; exact mass reconciliation; measured local-palette versus wider-cell decision only when the one-byte baseline binds. |
| Sub-part body mutation | [Phenotype D3a](2026-07-31_phenotype_plan.md#d3a-when-do-voxel-cells-become-body-state) | First played case in PE2 or PE3 that cannot use whole-part loss | New immutable volume or explicit body patch; atomic body revision; bounded revision-safe mesh/collider work; truthful fallback. **P3 (2026-09-01) named the first candidate case and did not open it:** a live cut lands on the boundary between two parts, and whole-part loss cannot express it without creating or destroying matter. |
| Generated trait execution | [ProcessDef](2026-08-01_processdef_plan.md#one-displayed-trait-three-compiled-programs) and [acquisition](2026-08-29_traits_and_perception_brief.md) | PE2 then PE3 | Event-driven condition, discrete development program, and native repeated process remain separate; each is typed and bounded; the exact realized candidate and digest persist. |
| Environmental fields | [Resident views](2026-08-14_resident_views_composition_plan.md#field-admission-boundary-2026-09-01) and [elements](2026-08-29_elements_and_traits_memo.md#field-dimensionality-is-part-of-admission) | First PE4 world rule that needs a new field | Named consumer, honest domain, cadence, sources/sinks, boundaries, units/range, conservation, scale rule, cost, and control. |

**Generated trait execution, first half met (2026-09-01).** PE2 built the
condition program: event-driven, typed, bounded, with declared inputs, and it
persists the exact realized candidate reference and a digest. It is not a
`ProcessDef` and it is not a development program — those stay separate, and
the second of them is PE3's. See §9.

Three cross-lane rules stay here. Representation changes are world
transactions and cannot create events or resources. A derived mesh, collider,
field view, warning, or trophic graph never becomes authority through
convenience. New machinery enters through the first played consumer and its
receipt; a platform-shaped possibility does not reorder PE0-PE7.

---

## 8. Findings

- **2026-08-31, history seam:** `crates/mesocosm-core/src/history.rs` records
  causal subjects but carries no general tick, place, process, or resource-delta
  envelope. It is suited to sparse biography and insufficient by itself for
  trophic flow.
- **2026-08-31, runtime seam:** `crates/mesocosm-runtime/src/runtime.rs` already
  drains one tick of events beside the ordered trace and rebuilds history during
  replay. It is the existing owner for a replay-derived bounded flow reducer.
- **2026-08-31, reproduction seam:** `organism::can_reproduce` already reads
  adult mass and gestation; `ecology::breeding` debits parent body and reserve,
  realizes a filial body through the lineage recipe, records the parent, and
  conserves matter. The missing checkpoint is host, control, and presentation
  composition rather than a second breeding system.
- **2026-08-31, process seam** *(updated 2026-09-01)*: native `ProcessDef`
  identity, definition digests, and a registry have landed. **PD1b's
  `BodyPhenotype` allocation half landed 2026-09-01**, so PE2's first
  prerequisite is met; PD2's first additional played process is the remaining
  one. `crates/mesocosm-core/src/phenotype.rs` is the owner PE2 allocates
  through, and `phenotype/develop.rs` is the single validator its direct and
  automatic fixtures must both use.
- **2026-08-31, acquisition seam:** `World::learn_from` still runs after both
  burn and incorporation, teaches every non-innate appendage in the donor
  lineage recipe rather than the consumed part, and returns early for an
  unplayed eater.
- **2026-08-31, reading seam:** `score::readings` computes retrospective epoch
  feats from `World` and `History`; vitals and minimap expose immediate
  individual and place facts. There is no bounded stock-and-flow reading or
  early trophic warning in the live views.
- **2026-08-31, scale seam:** the measured scale plan finds a fixed nine-place
  graph, an unbounded far-tier target scan, unwired cohort execution, and a
  population ceiling before the target roster. Flow and relationship designs
  must state their cohort reductions rather than assume individual storage.
- **2026-09-01, downstream ownership audit:** each proposed subsystem already
  has an owning plan and an incumbent seam. Section 7 routes their admission
  gates without adding an engine layer or changing the product order.

- **2026-09-01, a support *ratio* is the wrong shape; a support *stock* is the
  right one.** PE0 built the flow reducer against the plan's own example
  ("grazer demand has exceeded producer regrowth") and measured three
  candidates for the supply term across eight seeds. Gross uptake fails because
  producers pay rent proportional to what they carry, so their draw out of the
  ground is mostly treadmill and outweighs every mouth in the enclosure four to
  eight times over; uptake net of rent fails because it sits on zero at any
  equilibrium and its sign is then decided by noise; uptake into *substance*
  fails once determinate growth binds, because a mature stand routes its income
  to reserve and reads as growing nothing. What a support path is actually short
  of is **standing matter**, so the shipped reading is the producer tier's net
  substance change over a bounded window, with the grazed share stated beside it
  rather than divided into it. Every later ratio in §3 inherits this: a flow
  ratio at equilibrium is one, and says nothing.

- **2026-09-01, the first honest reading agrees with the population
  instrument.** Eight untouched seeds of the shipping roster, two thousand
  ticks each: four never read short at all, and four read short and stayed
  short (83, 153, 169, 367 consecutive ticks). Those four are the enclosures
  whose stand really is declining, which is exactly the instrument's standing
  verdict for this world — `thins`, never `breathes`. The warning firing there
  is the reading being right, not misfiring, and it is independent evidence for
  the terrarium-dynamics work rather than against the reading. It also means the
  warning threshold cannot be tuned to keep a shrinking enclosure quiet: it is
  set so that an enclosure *holding* its stand never raises it, and
  `mesocosm-runtime/tests/readings.rs` asserts that of every quiet seed.

- **2026-09-01, `flow::Process` versus `process::Process`.** The plan's sketch
  names the field `process`, and the crate already has a `Process` — what a
  *part* does (`Fix`, `Contract`, `Sense`). The flow record keeps the plan's
  word behind the `flow::` qualifier rather than being renamed, because the two
  converge instead of colliding: when PE4 admits generated transformations,
  `flow::Process` is where a `ProcessDef` identity lands.

- **2026-09-01, the enclosure-wide reading averages away local trouble.** The
  place stamp is on every record and is not read by the reducer yet. A stand
  being eaten out in one region is invisible in a nine-region aggregate, which
  is why an induced overdraw had to be enclosure-wide to separate at all. §3's
  "by place and lineage where the data supports it" is therefore not a
  refinement of PE0's reading but a correction to it, and belongs in the first
  phase that has a player standing somewhere specific — PE1 or PE2.

- **2026-09-01, the flow record costs 13.5% of the tick.** The population
  instrument's full sweep — six batches, 55 runs, ten thousand ticks apiece at
  the shipping cohort — took 3,498 s against the pre-PE0 receipt's 3,082 s, and
  every verdict, curve, sample and matter figure came back **identical**. So the
  price of the record is one number and it is on the tick, not on the reading:
  the reducer is a fixed-size integer fold and the presentation is a per-frame
  read. If a later round needs that back, the lever is the per-flow `Subject`,
  whose kingdom is an anatomy read — already taken once per organism per tick in
  the hot loop, and cacheable on the body if the ecology ever wants it there.

- **2026-09-01, a pause is a question about *when to step*, so it belongs to the
  driver.** Reading §8's reproduction-seam finding literally — host, control and
  presentation composition, not a second breeding system — puts the whole
  checkpoint in `mesocosm-runtime` and leaves `World` with one added intent.
  Three consequences follow that a world-side `checkpoint` field would have
  cost. The state hash of an unchanged trace does not move, so nothing that
  merely *stopped to ask* has to be re-recorded. The population instrument
  drives `World::apply` directly and never builds a `Runtime`, so "verdicts
  unchanged" is structural rather than a measurement anybody has to trust. And
  "replay lands the same state hash" is true by construction: the world never
  learns that anybody stopped, so there is nothing for a replay to reproduce.

- **2026-09-01, `held()` is the gate, and it is why nothing else moved.** TD4
  already separated *control* from *holding*, and `idle_run` makes the
  difference world state rather than a host timer. A checkpoint therefore opens
  only for a hand actually on the critter, so an idle terrarium runs on and
  every all-idle fixture in the workspace keeps its exact timing. The window is
  not even reachable by accident: `held()` lapses after 30 idle ticks and the
  played critter's gestation is at least 480, so no headless run can meet a
  birth checkpoint at all.

- **2026-09-01, nobody is observably ready to breed between ticks.** The aging
  pass carries an organism over its gestation and the breed pass in the *same*
  tick spends it, so `Organism::can_reproduce` is only ever true inside
  `World::apply`. Anything that wants to *find* a breeder — a test, a tool, a
  future scheduler — has to ask the gate about a clone with its clock wound
  forward, and bisect if it wants the tick. Worth knowing before something tries
  to plan around reproduction.

- **2026-09-01, the recorded demo's own locomotion was killing it.** The demo's
  fallback move turned every three steps, which walks a critter in a circle over
  ground it has already eaten; it starved between ticks 75 and 320 on **every**
  seed measured, which is why the old 120-step demo could never have reached a
  birth however long it ran. Holding a heading instead makes a 3,100-step
  recording survivable. This is a fixture finding, not an ecology one — but it
  means the played-slice plan's "a held critter dies fast" reading was partly a
  script artifact and should not be spent as evidence about the terrarium.

- **2026-09-01, a line usually does not outlive its founder.** A newborn is food
  like anything else, and in 60-founder enclosures the first offspring was
  eaten before its parent's death in several seeds; the runtime's death test has
  to try a handful. At the shipping cohort, seed 7's third offspring survived.
  The checkpoint says "none living" honestly when it happens, which is right —
  but it means succession is an *opportunity* the ecology grants, not a
  guarantee the checkpoint makes.

- **2026-09-01, a severed part's milligrams have already left the conservation
  account.** `BodyPhenotype::sever` tombstones a branch and
  `BodyDocument::total_mass_mg` skips severed parts, so severing *destroys*
  mass as far as `World::total_matter_mg` is concerned. Nothing has noticed
  because `sever` has no production caller — it is reached only by fixtures and
  PD2's receipt — but it is why PE2's part-level meal takes an organ off a
  **corpse** rather than off the severed branch the plan's sentence also allows:
  eating tombstoned tissue would create matter out of nothing. Whoever opens
  dismemberment (phenotype D3a) owns the ruling this needs first — a severed
  branch either stays in the account as a detached body, returns to the soil
  column under it, or is a carcass in its own right. It is a one-line bug today
  and a conservation hole the moment an `Intent` can sever.

- **2026-09-01, not eating is not the same as going hungry.** The demo's
  critter grows a canopy from its first ten meals, and a canopy earns from the
  ground: with the script's meal branch simply switched off its budget went
  *flat* at 1,820 mg for two hundred ticks and never crossed the starved line
  once. The stress had to be induced with `Deposit` — spending the reserve into
  the ground — which is diegetic and needs no new verb, but it is a finding
  about the shipping body rather than about the script. A played critter that
  has grown a producer's anatomy is much harder to starve than the played-slice
  plan's reading assumes, and any later condition that reads hunger should
  expect to be reached by a *consumer*, not by whatever the demo happens to
  have grown into.

- **2026-09-01, eating one organ can change what the eater is.** Taking a plate
  off a carcass and attaching it in a canopy position makes the eater read as a
  producer to `Kingdom::of`, and the ecology grazes it accordingly — observed
  in `tests/flows.rs`, where the played critter appears on the *source* side of
  a `Feeding` record one tick after consuming a plate. This is the
  body-is-authority rule working exactly as ruled, arriving somewhere new: a
  single incorporation can move a critter between trophic roles, with all the
  income and all the predation that implies, and nothing warned it. Worth
  knowing before PE3 offers a candidate that changes a lineage's kingdom.

- **2026-09-01, availability outruns expression, and that is the interesting
  state.** The body that comes through the starvation horizon is a bulk
  consumer, and the gland it earns needs a plate. So `Candidate::propose`
  returns `None` on the very body that did the enduring, and stays `None` until
  something grows it a plate. That is not a gap: it is the plan's
  observation/availability/expression distinction showing up as a state a
  player will actually sit in, and PE3's review has to be able to *show* a
  candidate that cannot yet be taken, with the reason, rather than hiding it or
  offering it and refusing.

- **2026-09-01, PE0's place stamp is still unread, and PE2 did not take it
  either.** §8 routed the enclosure-wide reading to "the first phase with a
  player standing somewhere specific"; PE1 passed it to PE2 and PE2 has a
  player standing somewhere specific but spent its whole surface on acquisition.
  The reading remains enclosure-wide. It falls to PE3 or PE4; recording it here
  so it is passed deliberately rather than by inheritance a third time.

- **2026-09-01, the causal log had no tick until now.** PE0's envelope is what
  makes a bounded window over births, maturations and deaths possible at all:
  `History` stored bare `Event`s, so "how many died in the last two hundred
  ticks" was unanswerable from the record that knew who died. `RecordedEvent`
  and `RecordedFlow` are the same `Envelope<T>`, so the two records cannot drift
  apart on when or where.

---

## 9. Progress

- **2026-09-04, PE3's last two residues discharged: the preview's ground, and
  the trait array.** Two rulings of 2026-09-02 built, in one slice.

  **The founder preview quotes the poorest ground a birth can reach.**
  `World::prospect` read `soil.matter_mg(soil.column_at(parent.position))` —
  the single column the parent stood over — and a body that had endured a
  hundred ticks had returned its own upkeep into exactly that column. So the
  review's table quoted five cells of a site where the descendant, dispersing
  up to twelve voxels onto ordinary soil, could afford one. The dormancy rule
  was doing its job and the quote was still a number the game would not charge.

  It now reads the **minimum** over the dispersal neighbourhood — the same
  square `ecology::bear` scatters into, through the same `Soil::column_at`,
  which clamps at the wall exactly as the birth's own clamp does. The radius
  stopped being a literal `12` written twice: `ecology::BIRTH_SCATTER` is the
  constant `bear` scatters by and the constant the quote walks, so the two
  agreeing is structural rather than a coincidence somebody maintains. The read
  is a fixed square in a fixed order, no entropy, nothing moved; it draws no
  sample of where the birth will actually land, because a preview that guessed
  the scatter would move every time the world's stream did.

  **The invariant, and where each half is receipted.** *The quote never exceeds
  what a birth in reach pays*, and the two halves are asserted separately
  because they need different worlds. `on_uniform_ground_the_quote_is_the_column_under_the_parent`
  (`src/world/review/tests.rs`) is the equality: a world at tick zero has
  uniform soil by construction — `Soil::seeded` gives every column the same
  milligrams and no rent, decay or percolation has moved one yet — so the
  poorest column in reach *is* the one underfoot and the new reading and the old
  coincide. That is what keeps this a ceiling rather than a discount.

  The two embodied tests that receipted the old behaviour were rewritten.
  `the_price_is_the_filial_cost_the_birth_then_pays` still asserts quote equals
  charge, and its walk-away setup is now explained by what it actually does:
  eighteen voxels is past `BIRTH_SCATTER`, so the column the parent enriched is
  out of every birth's reach and out of the quote, and what is left in reach
  does not vary by enough to buy a further cell. It asserts the ceiling
  (`declared <= underfoot`) before it asserts the equality — see the surprise
  below for why it is that and not something stronger. Its companion was
  **renamed**, because its old
  name stated the opposite of the ruling:
  `richer_ground_under_the_parent_quotes_more_than_a_dispersed_birth_pays`
  became `richer_ground_under_the_parent_does_not_inflate_the_quote`, and where
  it asserted `paid < quoted` it now asserts `quoted <= paid`. It keeps a
  positive control so it cannot pass vacuously: the column underfoot must still
  be richer than the declared ground, which is precisely the case the old
  reading got wrong. Measured on seed 4,242 the gap it used to receipt is
  closed — 21 quoted against 21 paid, where the quote had been the larger.

  **A surprise, and it moved where the equality is pinned.** *No ground in this
  world is uniform once it has been ticked.* The first rewrite tried to assert
  that the walk-away left the whole neighbourhood holding one figure; it holds
  71 in the poorest column and 126 in the richest, because rent, decay and
  percolation move columns everywhere all the time. The second tried the weaker
  claim that the parent was left standing on the poorest column in reach; it
  stands on 81 against a poorest of 71.

  So `the_price_is_the_filial_cost_the_birth_then_pays` returns one number for
  a reason worth writing down: 71 and 81 buy the same number of cells, and cells
  are the grain `Conditions::affords` charges in, so a ten-milligram difference
  in the ground is invisible in the price. That is a real property of the
  pricing and not a coincidence of the seed, but it is not the same claim as
  *equality on uniform ground* — which is why that one is pinned at tick zero,
  where `Soil::seeded` makes uniformity true by construction, and why the
  embodied test now asserts only the ceiling (`declared <= underfoot`) before
  asserting the equality it is actually about.

  **Receipts.** The six standing gates in release are green (137 tests across
  `matter`, `flows`, `succession`, `embodied`, `control`, `reckoning`), and so
  are `mesocosm-runtime`, `-views`, `-genet` and `-phenotype` (183 more). Clippy
  is clean at `-D warnings` on both profiles, fmt is clean, and
  `cargo check -p paredros-room --features r1-proof` — the one downstream
  path-dependent consumer — is clean too. **The golden fixture is unmoved**:
  `--replay ps1_played.trace.json --scenario ps1_played.scenario` with explicit
  scratch receipt and capture waited 772 frames and exited 0 at
  `081b4ba4bdc46190`, the same hash DT4 recorded, and the same scenario with the
  literal falsified to `...91` exited 1 naming both the expected and the actual.
  The four golden artefacts are byte-identical to a pre-slice backup, which is
  the fixture-defaults change proving itself: the replay's own output went to
  `scratch_golden.*`.

  **The trait array is deleted**, which closes PE3's own last done-condition and
  the phenotype plan's §D4. Five files, 1,318 lines: `epoch.rs`,
  `epoch/{lineage,adapt,standing}.rs` and `examples/ecology_lab.rs`, which was
  the module's only caller anywhere. The seven authored pressures and the three
  authored world profiles moved whole to `mesocosm-core/src/pressure.rs` — a
  top-level module rather than a section of `rules.rs`, because `WorldRules` is
  hashed into the record's identity while a `WorldProfile` is authored data kept
  deliberately off the wire. Nothing outside the module imported anything from
  it but `lib.rs`'s re-export line, so no shim was needed and none was written.
  The phenotype plan §D4 carries the file-by-file receipt.

- **2026-09-02, four rulings recorded (doc only).** §6 ruling 4 (material
  scheme, typed by provenance, storage A plus payloads C, no fields in PE4's
  first world, two composition layers) is ruled; the affinity-table pack door
  moved here from the ProcessDef plan is ruled (pack overrides `Founding`,
  `Founding` is the fallback); PE3's last done-condition is ruled (the trait
  array is deleted, not marked non-authoritative, per the phenotype plan
  §D4); and the founder preview residue is ruled (it declares the poorest
  ground in the dispersal neighbourhood, not only the parent's cell). No code
  changed.

- **2026-09-02, PE3b landed: the lineage review.** The player's turn stops
  being the one nobody takes.

  **The reading is the world's, and it invents no number.**
  `mesocosm-core/src/world/review.rs` holds `Offer` — one row — and
  `World::offers(species)`, which puts the status quo first and then every
  discovery the line does not already hold, **including the ones it cannot
  take**. Each row carries `World::score` (the identical function an unplayed
  line's turn is decided by, over the same `score_ticks` window), the price
  `program::express` would charge, the founder preview's body digest, the
  program digest it was grown under, and `Untakeable` when it cannot be taken.
  Nothing here recomputes anything: the score is P4b's, the price is PD5's, the
  preview is `Species::preview`'s, and the refusal is `Unexpressed` carried
  whole. `World::offers` takes `&self`, and a receipt asserts the state hash is
  unmoved after building one.

  **A candidate that cannot be taken is a row**, which is PE2's own residue
  answered in the place a player acts on it: a bulk line's gland row says
  *nowhere on this body is a plate* and stays on the table. `World::candidates`
  drops those, deliberately, because a round should not spend a bounded run
  scoring one; a review shows them, because the difference between *this world
  has nothing for me* and *this body is the wrong shape for it, yet* is the
  whole of what the screen is for.

  **The budget is the founder's material, and it is spent for real.** §8 q4's
  flat-price ruling means committing costs nothing and the *descendant* pays the
  development price at its birth, out of its own reserve and into the ground
  under it. So `World::prospect` reads the founder the played line would bear
  next — the ecology's own provisioning arithmetic
  (`parent.energy_mg.min(parent.biomass_mg() / OFFSPRING_COST)`), the soil
  column the parent stands on, the world's palette, and the development seed the
  birth pass would hand the next id it allocates — and `World::lineage_budget`
  is that founder's `material_mg`. No currency was invented; the alternative
  (the line's living reserves summed) was rejected because no rule anywhere
  spends it.

  **The second proposal source.** `mesocosm-runtime/src/review.rs` assembles
  `Review` — the reckoning, the trend, the budget, the current revision and the
  rows — because three of its four halves live beside a world rather than in
  one, and the third of those reads a pack off a disk. `Authored::load` opens
  exactly the scripts the manifest declares, through the pack door's own
  `asset` path check; each row's candidate is offered to every declared script
  through `Runner::propose` with `Entropy::from_seed(World::draw())`, and a
  script that returns no site simply does not apply. **`World::draw` reads this
  world's own SplitMix64 stream on a copy of the state**, so the entropy is the
  game's, the world's sequence is untouched, and a review built twice makes the
  same call twice. A fresh `Runner` is loaded per call, because script
  determinism is per runner. This is `mesocosm-phenotype`'s first production
  consumer; the dependency still runs one way, and the core knows about neither
  crate.

  **A fourth lane, not the checkpoint widened.** PE1's panel is *four facts, two
  answers, out*, and a `mesocosm-views` test asserts in so many words that it
  never mentions a program, a trait, a budget, an epoch, a revision or a
  founder. Widening it would delete that claim and make the individual
  checkpoint the editor PE1's stop rule forbids. The two also change on
  different clocks — a checkpoint is fixed once shown, a board re-reads after
  every commit — so `mesocosm-views::Board` is its own surface and
  `mesocosm-genet::review` its own lane, and at a lineage checkpoint the board
  draws while the checkpoint panel stands down. `Succession::epoch` survives as
  the fallback for a boundary no review could be built for.

  **The keys.** Tab moves the selection over every row, untakeable ones
  included; **R** commits the selected candidate as `Intent::Revise`; Enter is
  the same Resume it is everywhere. Only R reaches the queue — a cursor move is
  presentation, so it never enters the trace — and `Review::commit` refuses the
  status quo and every untakeable row, so the key cannot send a revision the
  world would only reject. After a commit the driver rebuilds the review, so
  the table describes the program the line now has.

  **The evidence is narrowed, and the first capture is why.** A young enclosure
  reckons twenty-odd marks across six lines; the first headed capture had them
  push the candidate table and both answers off the bottom of the panel. So
  `evidence_words` keeps the played line's readings whole and states everyone
  else's as one line — *11 marks taken by 6 other lines* — which is
  significance-as-abnormality without a scrolling log.

  **Receipts.**
  - `mesocosm-core` lib: **385** green (+6, `src/world/review/tests.rs`): a
    review built twice is the same review; it moves no world; the status quo is
    always the first row; the preview digest is the one `Species::preview`
    answers; the budget is the ecology's own provisioning arithmetic; an extinct
    line has no prospect and no budget.
  - `tests/embodied.rs`: **70** green (+10, `embodied/review.rs`): the table
    offers the status quo and everything the line came to; an untakeable
    candidate stays on it with its reason; a line that grows the shape can take
    it; **the price is the filial cost the birth then pays**; richer ground
    under the parent quotes more than a dispersed birth pays; a commit is
    admitted at the boundary and refused outside it; after a commit the review
    shows the revision as current; **one rival lineage responds to the change**;
    and the gland the review prices is the one a descendant expresses.
  - `mesocosm-runtime`: **23 + 12 + 5 + 5** green (`tests/review.rs` is new):
    the review stands only while the world holds at a lineage checkpoint and a
    revision answers without closing it; a review built twice is the same
    review *with the scripts in it*; **a pack expression appears beside the
    discovered proposal and is marked**, and without the pack the same row has
    one source; a commit is offered only for a row the world would admit.
  - `mesocosm-views`: **31** green (+11): the board's four facts and the budget;
    the status quo as a row; an untakeable candidate's reason, including an
    unaffordable price that is still a price; every source named; the net with
    its window and its sign; a reading that says whether it was a first; the
    evidence narrowed; three answers and no menu.
  - `mesocosm-genet` lib: **19** green (+2): the board has two keys and neither
    is a verb.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --test embodied --release`: **6 + 13 + 7 + 70** green.
  - `cargo test --workspace --release`: **814** green, the lens crate's 45 also
    taken at `--test-threads=1`. Clippy `-D warnings` clean over `--all-targets`
    in both profiles, `cargo fmt --all --check` clean, and
    `cargo check -p paredros-room --features r1-proof` builds with its one
    pre-existing `dead_code` warning on `brick::retarget_from_ground`.

  **The rival's response, and how it is shown.** `one_rival_lineage_responds_to_
  the_change` is a counterfactual rather than an assertion about a number: the
  same seed twice, one run committing the player's revision at the boundary and
  one not, both carried to the next boundary, and the same unplayed line's
  `Turn::considered` differs between them. The round records it, because a round
  is a transcript.

  **The demo trace did not move.** The recorded stream is byte-identical and the
  hash is still **`081b4ba4bdc46190`** — the new keys change nothing a recording
  sends, and the demo still answers its boundaries with `Resume`. Headed
  `--replay` runs 3,100 steps over 775 frames and matches, exit 0; a hash
  falsified by one bit reports the mismatch and exits 1.

  **Captures.** `Code/testing/mesocosm/pe3b_review.png` is the board over the
  living terrarium at the demo's first boundary (tick 1,000): *the epoch is over
  / epoch 1 ended / your line line 1, born as it always was / a founder holds
  666 mg to develop with / the enclosure 88 matured, 47 died in 240 ticks*, then
  the reckoning — three marks of line 1, each *the most this world has seen*,
  and *11 marks taken by 6 other lines* — then the two rows (*the status quo
  (nothing to build)* and *endured hunger (discovered: nowhere on this body to
  put it)*, both *+72 mg over 480 ticks / nothing to develop / founder
  ebc38d7c6a5e08be*, the second carrying *nowhere on this body is a plate*), and
  the two answers. The vitals panel and the minimap read normally beside it and
  the terrarium is alive behind it. No `[R]` line, correctly: the cursor is on
  the status quo and the one candidate cannot be taken.

  **A finding the capture forced into the open, in `pe3b_unattended.png`.** An
  *unattended* clocked run cannot reach a boundary at all. `--auto-eat 40` lets
  `World::held` lapse between meals, so the epoch ends with no hand on the body
  and `succession::opened` — rightly — asks nobody anything; tighten it to
  `--auto-eat 20` and the run instead stalls forever at the **birth** checkpoint
  at step 800, because nothing unattended answers a question. Both are the
  TD4 ruling working as written. The consequence for the harness is that an
  epoch-boundary capture has to come from a run that answers its own
  checkpoints, which is what the recorded demo is for; `pe3b_review.png` is
  `--replay ... --frames 250`, which is a whole epoch of simulated time.

  **Splits at the ceiling**, per the workspace rule: `runtime/tests.rs` out of
  `runtime.rs` when the review pushed it to 607.

  **Residues, and what PE4 and DT2 inherit.**
  - **`Offer` is the row the next two consumers want.** It is
    `Serialize`/`Deserialize` and pure data — candidate, score, price, preview,
    program, reason — read entirely through core queries. The
    [dev tools plan](2026-09-01_dev_tools_plan.md)'s DT2 asks an inspector for
    a critter's discoveries and *its species' current program revision*, which
    is `Review::current` and `Offer::program` already; its principle 3 (the
    lane invents no readings) is satisfied for that half without adding one.
    PE4's generated candidates arrive as more `Offer`s, since nothing in the
    row names a hand-written condition.
  - **A founder preview declares the ground the parent is standing on**, and a
    descendant disperses up to twelve voxels away onto ordinary soil. A body
    that stood still for a hundred ticks returned its upkeep into the column
    under it, so the quote can be five cells where the birth affords one — the
    dormancy rule doing its job, receipted in
    `richer_ground_under_the_parent_quotes_more_than_a_dispersed_birth_pays`.
    ~~Whether a preview should instead declare the *neighbourhood* a birth can
    land in is a design question and is Mark's; nothing here guesses at it.~~
    **Ruled by Mark, 2026-09-02: the neighbourhood.** The preview declares the
    poorest ground within the dispersal neighbourhood a birth can land in, so
    the quote never exceeds what a birth affords. **Built 2026-09-04**; see the
    Progress entry of that date.
  - **The two proposal sources are read on the played body**, because that is
    what `Request::of` freezes and what a player can point at, while the price
    and the preview are the descendant's. Both facts belong on the screen and
    they are about different bodies; a row does not currently say so in words.
  - **Nothing is spent at the commit.** The budget is reported and the birth
    charges it. If §8 q4 is ever reopened — youth buying developmental change
    more cheaply — the one place a multiplier lands is still `program::express`.
  - **`epoch::Lineage`'s scalar trait array is untouched**, as instructed, and
    PE3's last done-condition turns on a reading only Mark can give: PE3a's own
    module note already says the module is provisional and that `World` reads
    none of it, which is the *marked non-authoritative* branch; the deletion
    branch is phenotype §D4's fifth retirement condition and has always been
    his. Flagged rather than decided here.

- **2026-09-01, PE2 landed: discovery becomes an embodied option.** A meal
  stops being a lesson.

  **The evaluator, and the boundary made structural.**
  `mesocosm-core/src/discovery.rs` holds the evidence, the rules and the
  routing; `discovery/conditions.rs` holds the fixed table. The traits brief's
  four requirements are each a property of a type rather than a discipline
  somebody keeps. *Event-driven*: `evaluate` runs once per accepted fact and
  nothing polls an organism. *Declared inputs*: a `Condition` names the `Input`
  lanes it accepts and routing checks the declaration **before** the rule, so
  a meal offered to an endurance condition is recorded as
  `Miss::UndeclaredInput` — a different and truer answer than "the rule went
  unmet". *Bounded*: two conditions, one integer compare each, and the only
  accumulator a rule may read is world state. *Recorded*: a `Discovery` carries
  the matched evidence with its quantities, the route, the **realized candidate
  reference** (a `ProcessRef`, never a name), its parameters, its `Source`, and
  a digest over all of it. `ConditionId` is itself a digest over the
  condition's rule-bearing bytes, exactly as `ProcessRef` is over a
  definition's, so two worlds that agree about a name and disagree about the
  rule cannot trade discoveries.

  **The record kept for evidence that unlocks nothing.** `Observation` is the
  other half and the one the done-condition actually needs: what was offered,
  which lane it came down, what took it, and — for every condition that did
  not — why. Without it "a meal supplies evidence without unlocking an
  incompatible candidate" would be a claim about an absence. `World` keeps one,
  not a log; a log of meals is what `History` is for.

  **The two conditions.** `mesocosm:endured-hunger` reads the endurance lane:
  `HUNGER_TICKS` consecutive ticks under the starved line with a hand on the
  body, and it grants the **gland** — `mesocosm:secrete` on a plate. No meal
  appears anywhere in it, and the reward is a chemical defence rather than a
  matching food category, which is §1's ruling in one table entry.
  `mesocosm:plate-eaten` reads the meal lane, narrowed to the **organ actually
  consumed**: a plate of at least `MEAL_EVIDENCE_MG` grants `mesocosm:fix` on a
  plate *plus* the lexicon word for one. **No number here was picked.**
  `HUNGER_TICKS` is `STARVED_UPKEEP_TICKS` itself — you endure the horizon you
  are inside; `MEAL_EVIDENCE_MG` is the ecology's own `STARVATION_MG`, its
  answer to how much substance is a body at all; the two cell counts are the
  ones PD2's own fixtures use for the same organ.

  **What a discovery *is*: availability, and three other things it is not.**
  A `Candidate` is a proposal the one validator can lower and nothing else.
  `Candidate::propose` builds an ordinary `AllocationProposal`, so
  `BodyPhenotype::develop` is still the only way allocation moves, and
  `World::candidate_intent` hands a player the `Intent::Rearrange` that would
  express it. `propose` returning `None` is a **real state**: a bulk consumer
  has the gland available and nowhere to put it until it grows a plate. That is
  the plan's requested distinction, in five places rather than one word —
  observation (`Observation`), somatic incorporation (`Intent::Consume`),
  developmental availability (`Discovery`), expression (`Intent::Rearrange`),
  and inheritance (`Candidate::word`, the lexicon entry a descendant is born
  with).

  **The accumulator is the world's, and played-only by the same gate PE1
  used.** `World::hunger_run` is one integer of world state, hashed,
  snapshotted and replayed. It advances only while `World::held` says a hand is
  on the body, and resets when the body is fed or stops being a body; a hand
  that lets go neither advances it nor throws it away, because the ecology is
  driving then and nobody is enduring anything. The crossing is an *event* —
  the evidence is offered at `== HUNGER_TICKS` and never again — so nothing
  sweeps the roster and a second crossing reads `Miss::AlreadyKnown`.

  **`learn_from` is subsumed, not merely deleted.** It ran on every meal, read
  the *eaten lineage's recipe*, and taught the eater's line every non-innate
  appendage in it. What replaced it is `World::observed_in`, which offers one
  piece of evidence about the organ that was consumed. The one honest
  consequence was kept and narrowed: `Recipe::acquire` is still called, by the
  evaluator, for a word the matched condition grants — so the complexity
  frontier stays connected to play and `acquire` finally has a production
  caller. `Event::Learned` is replaced by `Event::Discovered { organism,
  species, condition }`; the condition digest resolves to everything the old
  variant's `appendage` field could not say.

  **Part-level eating: `Intent::Consume`, and `from_part` becomes real.**
  `world/consume.rs` takes one organ off a **carcass**, settles exactly that
  part's milligrams, and writes `Origin::Incorporated { from_species,
  from_part }` naming the part it came off — a field written `PartId(0)` at
  every call site until now. The subtree under the organ stays on the corpse,
  because live subtree transfer is phenotype P3's. `BodyPhenotype::
  take_part_mass` is the named operation that makes "only its own matter"
  a property rather than an assembled call site.

  **Why a corpse and not a severed branch**, though the plan's sentence allows
  either: a severed part is tombstoned and `BodyDocument::total_mass_mg`
  already skips it, so its milligrams have left the conservation account and
  eating one would *create matter*. A corpse's living parts still weigh what
  they weigh. The severed half waits for the dismemberment gate (phenotype
  D3a) that would put those milligrams somewhere honest first. Three named
  refusals carry the boundary: `StillLiving`, `NoSuchPart`, `NothingLeft`.

  **Receipts.**
  - `mesocosm-core` lib: **346** green (+8, the evaluator's own routing claims
    in `discovery/tests.rs`).
  - `tests/embodied.rs`: **41** green (+13, in the new `embodied/discovery.rs`
    and `embodied/part_meal.rs`), one test per named claim: a stress unlocks a
    candidate with no meal in it; the accumulator is the world's and the
    crossing happens once; feeding ends the stress; **an idle terrarium
    discovers nothing** (1,200 ticks); a meal supplies evidence and cannot
    reach a condition that never asked for one; a meal no longer teaches the
    donor's whole recipe; the organ that teaches teaches the word for it; a
    consumed part settles its own matter and nothing else's; an organ can only
    be taken once and only off something that has stopped; the discovered
    candidate is located, paid for, useful, dormant and lost with its branch;
    direct and automatic fixtures lower the same candidate the same way; the
    discovery survives a snapshot and replays to the same hash; the causal
    record names the condition.
  - `tests/flows.rs` gains **a consumed part moves exactly its own milligrams
    and says so**: PE0's whole-compartment reconciliation over the tick, plus
    one `Feeding` record of exactly the organ's mass, substance to substance.
  - `mesocosm-views`: **20** green (+2): the discovery's three rows in the
    exact words a player reads, and the evidence row that appears only when
    nothing took it.
  - `mesocosm-genet` lib: **17** green (+2): the recorded demo reaches a
    non-food discovery, and a recorded meal is observed and unlocks nothing.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --test embodied --release`: **5 + 10 + 7 + 41** green.
  - **The instrument cannot observe this phase, and did not.** It drives
    `World::apply` with nothing but `Intent::Idle`, and `held()` lapses after
    thirty of those — so the hunger run never advances, no evidence is ever
    offered, and `discoveries` stays empty for the whole ten thousand ticks.
    That is structural rather than a measurement anyone has to trust, and
    `an_idle_terrarium_discovers_nothing` asserts it directly. Re-run anyway
    against `dc4_roster.json`, and **all ten** baseline seeds came back
    identical: verdict, start, peak, peak tick, end, cumulative births,
    cumulative deaths, end kingdom counts and end biomass, seed for seed and to
    the milligram. **0 breathes / 10 thins / 0 boil / 0 collapse** stands
    unchanged. The sweep was stopped after the baseline batch rather than run
    through the other five — the isolating mechanism is structural, not a
    per-seed coincidence — and stopped before it could overwrite
    `dc4_roster.json` with new timing on an unmoved result; the file is
    byte-identical to what DC4 recorded.
  - `cargo test --workspace` green — **680** tests across 27 suites in release,
    the lens crate's 45 also taken at `--test-threads=1`. Clippy `-D warnings`
    clean over `--all-targets`, `cargo fmt --all --check` clean,
    `cargo check -p paredros-room --features r1-proof` builds with its one
    pre-existing `dead_code` warning on `brick::retarget_from_ground`.

  **The demo exercises both loops.** Recorded at `DEMO_SEED = 7`,
  `DEMO_STEPS = 3_100`, 916 founders. Simply *not eating* was not enough — by
  step 120 the demo's critter has a canopy and earns about what it spends, so
  its budget sits flat and it never crosses the line at all. So the script
  spends its reserve where a player can, into the ground under it, and holds
  about half a horizon short: no new verb, and `Deposit` is a key the host
  already has. The recorded run comes through **one** discovery —
  `mesocosm:endured-hunger`, route `Endurance`, evidence *hunger for 100
  ticks*, at tick 219 — and its last observation is a meal at tick 801, *bulk
  part 0 of line 8, 316 mg*, matching nothing, with `endured-hunger` recorded
  as `UndeclaredInput` and `plate-eaten` as `RuleUnmet`. Both PE2 receipts, in
  the loop rather than in a fixture. It still reaches both PE1 checkpoints:
  `Resume` at 800, 1600 and 2400, `TakeControl { organism: 1692 }` at 3000, and
  it ends alive in that descendant at 1,282 mg.

  **Hash, replay and falsification.** The demo trace moved from
  `25a5a0096cef0af1` (PD2) to **`7e315db34c37baf7`**, and the intent stream
  moved with it — the script changed, so the run did. Headed `--replay` runs
  3,100 steps over 775 frames and matches, exit 0; a hash falsified by one bit
  reports the mismatch and exits 1.

  **Captures.** The one that matters is **`pe2_replay_end.png`**: the frame the
  recorded replay finishes on, in the real host, with both PE2 receipts legible
  side by side in one panel over a living terrarium — *discovered: endured
  hunger / by: endured: hunger for 100 ticks / grants: secrete on a plate*, and
  under them *last evidence: bulk part 0 of line 8, 316 mg — endured hunger:
  not a question this asks*. Energy 1,282 mg in the descendant the run
  succeeded into. Three panel-only captures carry the states ordinary play does
  not reach in one frame: `pe2_discovery.png` (the discovery on a body with no
  gland anywhere on it, because unlocking is not expressing),
  `pe2_meal_refused.png` (the meal's evidence and the condition it could not
  reach), and `pe2_candidate_taken.png` (the same candidate lowered through the
  one validator, with PD2's three gland rows underneath). The panel grew to
  300x320 to hold three more rows.

  **Splits at the ceiling**, per the workspace rule: `discovery/conditions.rs`
  out of `discovery.rs`; `mesocosm-views/src/vitals/tests.rs` out of
  `vitals.rs`; `tests/embodied/part_meal.rs` out of `embodied/discovery.rs`.

  **Residues, and what PE3's review inherits.**
  - **A candidate that cannot be taken is the ordinary case, not an edge one.**
    `World::candidate_intent` returns `None` on the body that earned the
    discovery, because a bulk consumer has nowhere to put a gland. PE3's review
    has to *show* an offered candidate with the reason it cannot be taken yet,
    and PE3 is also where the answer arrives: a development program that grows
    the shape a descendant needs. That is the second of the ProcessDef plan's
    three compiled programs, and PE2 deliberately built only the first.
  - **`Intent::Rearrange` is still the door**, and it is still PD3's to delete.
    What PE2 added on top of it — `candidate_proposal` and `candidate_intent` —
    is proposal *construction*, not a second authoring path, so it survives
    that deletion and points at whatever review replaces the door.
  - **NPC acquisition is still open** and §6 ruling 5 now says exactly what it
    would need.
  - **The condition table is a table, not a generator.** PE4's generated
    conditions arrive by being admitted into `discovery::conditions()` with the
    same declared inputs, bounded rules and digest; nothing here reruns a
    generator from a name, and `ConditionId` is what stops a generated
    condition's meaning drifting under its own label.
  - **Acquisition is still unpriced.** Coming to a discovery costs nothing; the
    only price a body pays is `Intent::Rearrange`'s development cost when it
    takes the candidate up. Mark's cost formula — complexity, and proximity of
    the donor's lineage — remains the traits brief's open question, and the
    proximity term's `None` for every cross-founder pair is still the blocker
    that brief named.
  - **The `Rearranged` outcome does not know it came from a discovery.** A
    development taken up from a candidate and one drawn by hand are the same
    event in the record. If PE3's review wants to say *this body expressed what
    its line came to*, it will need the discovery reference on the instruction
    or a join through the digests; PE2 did not invent one with no consumer.

- **2026-09-01, PE2's first prerequisite is met: PD1b landed whole.** The
  [ProcessDef plan](2026-08-01_processdef_plan.md)'s allocation half is
  complete, so §4's `PD1b allocation -> PD2 -> PE2` chain now waits only on
  PD2's one native played process.

  What PE2 inherits, concretely. `crates/mesocosm-core/src/phenotype.rs` owns
  `BodyPhenotype`: a private wrapper over anatomy plus one authoritative
  cell-graph mosaic per living part, seeded from geometry, conserving capacity,
  and impossible to split from anatomy by an attach or a sever.
  `phenotype/develop.rs` is the **single validator** PE2's direct and automatic
  fixtures must both use — its `Arrangement` is diagnostic metadata the
  validator never reads, which is why "the same candidate lowers the same way"
  is a property of the type rather than a test that has to keep being
  re-passed. Expression identity is `ProcessRef`, a definition digest resolved
  through the registry, so "the PD2 process is located on anatomy" has a place
  to be located and a name to be located under. `BodyPhenotype::explain` is the
  explanation path a headed receipt reads: capacity, free tissue, each site's
  qualified id, cell count, and the cause that placed it — and it still answers
  for a severed part, which is the "lost when its dependency is severed" state
  PE2's done-condition names.

  This changed no PE gate, no ecology number and no product order. The drawn
  baseline is unmoved and the demo trace's intent stream is byte-identical; the
  only fixture movement is the intentional snapshot format bump, which took the
  demo hash from `f90123db6f2a5ac5` to **`0ebe0655317a7392`**. PD1b's Progress
  entry carries the residues, including the one PE2 will feel first: the
  anatomy readings (`performs`, `reach`, `canopy`, `mouth`, `feeding_mode`)
  still read geometry rather than allocation, deliberately, because that
  rewrite is only a *different* answer once PD2 gives a site something to be
  dormant about.

- **2026-09-01, PE1 landed: reproduction and succession as the individual
  checkpoint.** Death stops being a wall.

  **Where it lives, and why that is the whole design.** §8's reproduction-seam
  finding says the missing checkpoint is *host, control and presentation
  composition rather than a second breeding system*. Taken literally that puts
  the pause in `Runtime` — which already owns the clock, the step cap and the
  queue, so **not** stepping is its job too — and leaves `World` with one new
  intent and nothing else. `mesocosm-runtime/src/succession.rs` holds
  `Checkpoint { tick, occasion, heirs }` over `Occasion::Birth(Birth)` and
  `Occasion::Loss(Loss)`; `Runtime::step_once` refuses to advance while one
  stands and nothing queued answers it. The breeding transaction is untouched:
  the adult-mass gate, filial realization, the matter debit, the parent link and
  `Event::Born` are exactly what they were, and this reads their records.

  **What opens one.** A birth whose parent is the critter **under your hand**,
  or that critter's body ending. `World::held` — TD4's already-ruled distinction
  between *control* (whose body a key would move) and *holding* (whether anyone
  has moved it lately) — is the gate, read before the tick because a critter
  that dies this tick is held by nobody after it. An ant farm nobody is touching
  is never interrupted, which is the ruling and also the reason no existing
  fixture's timing moved.

  **What answers one.** `Intent::TakeControl` or the new `Intent::Resume` —
  take, or carry on. Both are ordinary recorded intents through the ordinary
  eligibility gate, so the choice is in the trace, replays with everything else,
  and grows no second control path. `Resume` is `Idle` that admits to being a
  hand: nothing moves, but the idle run resets, because somebody answered.

  **The default, and the ruling it is standing in for.** §6 ruling 1 is open and
  is **not** answered here. `Checkpoint::default_answer` is `Resume` — continue
  the parent, stay disembodied — for one reason: **it is the only answer that
  can be taken back.** The offspring stays alive in the enclosure and
  `TakeControl` still reaches it, whereas a default that moved control would
  silently discard a body a run spent nine hundred ticks growing and nothing
  undoes that. Flagged for Mark; the implementation records the choice either
  way, so ruling it later is a one-line change to one function.

  **Descent needed no new link.** `History::descendants` walks the parent
  `Event::Born` has always carried, transitively and eldest-first;
  `World::heirs(&history, of)` filters it through `World::eligibility`, the same
  gate `TakeControl` uses. No parent field on `Organism`, no lineal table beside
  the world, nothing to keep in step. The past is a parameter for the reason
  `end_epoch` takes one.

  **Not a lineage editor.** Four facts, two answers, out. The panel offers **one**
  body — the offspring just born, or the eldest living descendant — never the
  brood as a numbered roster, and `mesocosm-views`' own test asserts the words
  never mention a program, a trait, a budget, an epoch, a revision or a founder.

  **Presentation.** `mesocosm-views/src/succession.rs` is the second cambium
  consumer and the first that appears because the world is *waiting*:
  centre-frame, 468x208, headline plus the pointable facts plus the two keys.
  `mesocosm-genet/src/succession.rs` is its lane, third over the same netrender
  instance and blend pass. At a checkpoint the keyboard narrows to Enter and T,
  because the world is stopped and a move would only go stale in the queue.

  **Receipts.**
  - `mesocosm-core/tests/succession.rs` (7): descent off the existing link, a
    grandchild is a descendant, **death continues through an eligible
    descendant**, siblings persist without becoming inventory, the choice is in
    the trace and a run that answered is distinguishable from one that never
    did, answering is a hand and not an idle, and a dead heir is not offered.
  - `mesocosm-runtime/tests/checkpoint.rs` (8): **an idle terrarium is never
    asked anything** (1,200 ticks, no checkpoint, every step ran); a birth under
    the hand opens a bounded checkpoint whose cost is the ledger's; the world
    holds — `step(10)` runs **0**, no tick, no trace entry, no intent consumed;
    one recorded choice resumes play; taking the offspring is the other answer
    and leaves the parent an organism; a paused run replays to the same hash,
    control holder, body, history and **byte-identical readings**; a death under
    the hand offers the line and the run continues in it; declining resumes
    disembodied.
  - `mesocosm-core/tests/flows.rs` gains **a birth reconciles to the milligram**:
    two `Process::Birth` records, parent to child, body for body and reserve for
    reserve, and the newborn's entire substance and entire budget are exactly
    those two numbers. PE0's tick-level reconciliation runs over the same tick.
  - `cargo test -p mesocosm-core --test matter --test flows --release`: 5 + 9
    green. `cargo test -p mesocosm-runtime`: green.
  - **The demo trace exercises both loops.** Recorded at `DEMO_SEED = 7`,
    `DEMO_STEPS = 3_100`, 916 founders: `Resume` at steps 800, 1600 and 2400
    (three births continued, the first costing 1,766 mg — 883 of body, 883 of
    reserve), **`TakeControl { organism: 2205 }` at step 3000** — the played
    critter died at the end of its natural life and the run continued in its
    surviving descendant — and `Resume` at 3040 for a fourth birth, from the new
    body. It ends alive. Hash `f90123db6f2a5ac5`; the headed `--replay` runs
    3,100 steps over 775 frames and matches, exit 0, and a hash falsified by one
    bit exits 1.
  - **Captures.** `Code/testing/mesocosm/pe1_succession.png` (frame 750, the
    death): *the body is gone / was critter 0, line 1 / descendants one living /
    [T] continue as critter 2205, your eldest / [Enter] let the line go*, with
    the vitals panel reading `state dead` and PE0's warning live beside it.
    `pe1_birth.png` (frame 200, the birth): *a birth / parent critter 0 /
    offspring critter 1253 / cost 1766 mg — 883 of body, 883 of reserve /
    descent child of critter 0, line 1*, over a living critter at 1,073 mg.
    `pe1_replay_end.png` is the frame the demo finishes on: no panel, the world
    running again, and the vitals reading `energy 1283 mg` on a full bar — the
    run alive in the descendant it succeeded into.
  - **The population instrument cannot observe this phase.** It drives
    `World::apply` directly and never constructs a `Runtime`, so the checkpoint
    is structurally invisible to it; `World`'s only change is an intent variant
    nothing headless sends. Re-run anyway, and the drawn baseline came back
    **identical** — all ten seeds, and not merely the verdict: start, peak, peak
    tick, end, cumulative births and cumulative deaths each equal to the DC4
    receipt's, seed for seed. **0 breathes / 10 thins / 0 boil / 0 collapse.**
    The other three batches were not re-run, since the argument for them is the
    same structural one; `dc4_roster.json` is byte-identical to what DC4
    recorded, because the run was stopped after the baseline rather than left to
    overwrite it.
  - `cargo test --workspace` green — **612** tests, the lens crate's 45 taken at
    `--test-threads=1`. Clippy `-D warnings` clean, `cargo fmt --all --check`
    clean, `cargo check -p paredros-room --features r1-proof` builds.

  **Split at the ceiling**, per the workspace rule: `app/setup.rs` out of
  `app.rs` (window, device, surface, section and chrome bring-up) when the third
  lane pushed it to 602.

  **Residues.** The host still has no general "inhabit that one" key — T at a
  checkpoint takes the *eldest* heir and nothing else, deliberately, so a player
  who wants a particular descendant has no input for it yet. A loss offers
  descendants only, not the wider lineage the played-slice plan's PS2 also
  allowed. And PE0's place stamp is still unread: the reading stays
  enclosure-wide, and §8 routes it to the first phase with a player standing
  somewhere specific — PE1 did not take it, so it falls to PE2.

- **2026-09-01, PE0 landed: one flow record, one useful warning.**

  **The records.** `mesocosm-core/src/flow.rs` adds `Envelope<T> { tick, place,
  record }` and the two aliases the plan named — `RecordedEvent` and
  `RecordedFlow`. `History` now stores the envelope, so the causal log finally
  carries a tick. A `FlowEvent` is `{ process, carrier, source, destination,
  amount_mg, from, to }`: `Account` is `Soil | Substance | Reserve`, exactly
  TD6's conserved sum split three ways; `Carrier` has the one variant matter
  needs and is the seam PE4's materials arrive through; `Subject` names an
  organism with its lineage and its **true** kingdom, read off anatomy rather
  than off `guise`. `Ledger` is the world's one-tick buffer and `Records` is the
  writing façade both streams share, which is what makes "one accepted
  transaction, one commit point" structural rather than a convention.

  **Where flows are emitted.** Every seam TD6 closed: producer uptake, upkeep
  (split by the account it was paid from — `Organism::pay_upkeep` now reports
  that), NPC feeding and its spill, venom, dispersal travel, birth (as a
  parent-to-child transfer, not a spawn), death, carrion decay, and every played
  verb — move, deposit, and the meal's five destinations. Nothing was found that
  moves matter and does not emit, and `tests/flows.rs` is what would find one.

  **The reducer.** `mesocosm-runtime/src/readings.rs`: one ring of
  `RETENTION_TICKS = 240` per-tick totals, read through two windows — the whole
  ring for replacement, `JUDGEMENT_TICKS = 60` for the stand — and a shortfall
  streak that warns at `WARN_AFTER_TICKS = 60`, one whole judgement window
  unbroken. `Runtime::replayed` rebuilds it during replay.

  **Receipts.**
  - `mesocosm-core/tests/flows.rs` reconciles the stream against **every
    account of every body plus the soil, tick by tick**, over three seeds x
    1,200 ticks and over every played verb, milligram-exact. It also carries a
    positive control: a doctored tick with an unrecorded milligram must be
    reported, and is.
  - An accepted deposit puts exactly one `Process::Deposit` of exactly its mass
    in the stream; a refused one puts nothing there, and a refused meal leaves
    the prey out of it entirely.
  - `mesocosm-runtime/tests/readings.rs`: a replay's windows encode **byte for
    byte** to the driven run's; a run that reduces every tick and a bare world
    that never looks land on the same state hash; and the windows do not grow
    with the run.
  - Draining does not change the world hash, because the ledger is
    `serde(skip)` and its `PartialEq` is unconditional — the `drain_ground_dirty`
    arrangement. A world holding a tick of flow encodes to the same bytes as one
    that drained, which is the stop rule against dense flow in snapshots made
    structural.
  - **Two arms.** Seed 7, 200 founders, 2,000 ticks. The untouched control never
    read short once; the induced overdraw — half the stand dies off and every
    surviving mouth starts on what is left, neither half moving a milligram —
    reached **586** consecutive short ticks. Three further seeds whose
    enclosures hold their stand are asserted quiet.
  - **Headed capture:** `Code/testing/mesocosm/pe0_reading.png`, seed 7, 307
    steps over 5,000 frames, reading `energy 7214 mg` and `replacement 0
    matured, 126 died in 240 ticks` live in the vitals panel.
  - The demo trace's hash is **unchanged** (`2295790889f3ccd5`) and the recorded
    trace is byte-identical, because the envelope only widened a buffer that is
    drained empty every tick. `--replay` exits 0; a falsified hash exits 1.
  - The population instrument's whole sweep returned **identical verdicts,
    curves and matter figures** — baseline 0 breathes / 10 thins / 0 boil / 0
    collapse, and every other batch unmoved — for 13.5% more wall time. Its
    `dc4_roster.json` was left as DC4 recorded it rather than overwritten with
    PE0-era timings; the comparison lives here.

  **Presentation.** `mesocosm-views` gains `replacement_words` and
  `warning_words` and the panel grew to 300x200 to hold them. The warning states
  what moved and over what window and never a bare percentage; a views test
  asserts it never spends `breathes`, `thins`, `boils` or `collapses`, which
  remain test classifications.

  **Splits at the ceiling**, per the workspace rule: `world/records.rs` out of
  `act.rs`, `ecology/flows.rs` out of `ecology.rs`, `ecology/tests/fixture.rs`
  out of `tests/mod.rs` (which was already at 601 lines before this round).

  **Not done, and named in §8:** the place stamp is written on every record and
  not yet read; the reading is enclosure-wide. That belongs in the first phase
  with a player standing somewhere specific.

- **2026-09-01:** refined the downstream architecture from technical prior art,
  routed multi-scale execution, materials, body mutation, generated-trait
  execution, and fields into their owning plans, and summarized their admission
  checks in §7. PE0-PE7 order is unchanged; documentation only.

- **2026-08-31:** founded from Mark's first-principles Mesocosm review. The
  architecture, integration order, trophic-readings contract, done-conditions,
  stop rules, live-code findings, and eight remaining rulings were recorded.
  Documentation only; no implementation dispatched.
