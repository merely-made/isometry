# Terrarium Dynamics Plan (2026-08-29)

**Status: landed and closed 2026-08-30; refreshed 2026-08-31.** TD1 through
TD11 are implemented and receipted. The instrument remains a diagnostic, but
`breathes` no longer spawns tuning rounds for systems the game has not built.
The [playable ecology plan](2026-08-31_playable_ecology_plan.md) now owns the
integration work this series exposed.
Make the terrarium compelling on its own — an ant farm worth watching — and
let the player's considerations step into that. Mark's words, ruling the
priority: "slices that shape the world and ecosystem dynamics... that's my
priority" and "let's make the terrarium/ant farm compelling on its own and
then let the player considerations step into that."

The diagnosis this plan answers (played slice plan, 2026-08-29 findings):
the first playtest's whole arc — budget gone in 3 seconds, death of old age
at 17, population 61 → 8,155 — was the ecology's tick-tuned life-history
constants driven at 60 ticks per wall second. In tick-space nothing was
broken; played wall-time mapped 60:1 onto it.

## The rulings (Mark, 2026-08-29)

- **Tempo: both, deliberately.** A canonical played tempo is picked and the
  life-history constants are retuned for feel at it. Working value proposed
  here: **10 ticks/second** (100ms input granularity; a starter lifetime
  lands at minutes once constants follow). `Runtime::new` already takes
  ticks-per-second, replay is per-intent, so pacing never touches the hash.
- **The hand: instincts under idleness.** A controlled critter obeys the
  keys while the player acts and reverts to its own drives when the player
  has been idle a while. The idle terrarium is a feature, not a failure —
  it is how the game's dynamics are observed.
- **Income: the body routes it.** A meal eaten while the budget is starved
  burns; eaten while provisioned it incorporates. Hunger decides — no menu,
  no second key. D5's burn-or-incorporate choice becomes a state you play.
- **Priority: world and ecosystem dynamics first.** Succession/epoch wiring
  (PS2) deliberately waits.
- **The bar (Mark, 2026-08-29): the RimWorld standard.** "We're trying to
  reach that rimworld bar: a composition of loops so compelling it could be
  called a story generator." Earlier the same day: "rimworld wouldn't make
  sense without the confluence of many loops and mechanics... it's hard to
  be surprised when there's nothing to learn." No single loop is the game;
  the game is loops interacting until the world produces events worth
  retelling. This bar judges every slice here: a mechanic earns its place
  by composing with the others into legible surprise, and a dynamic nobody
  can witness or remember generates nothing. The witnessing half — history
  surfaced, death seen, succession felt, the epoch retold — is where the
  loops become stories, which is why PS2 and the record-reading surfaces
  are story machinery, not plumbing.

## TD1 — the population instrument

Headless, before any tuning: run the ecology across seeds for several
lifespans and write per-kingdom population and biomass curves plus a
verdict (stabilizes within a band, oscillates, boils, collapses). The
baseline receipt documents today's boil at today's constants. Prove the
instrument before believing anything it says: a constant set known to
collapse must read as collapse.

**Done when:** a `cargo run` producible receipt (JSON curves + summary)
exists under `Code/testing/mesocosm/`, with a baseline at current constants
and at least one deliberately-broken control run proving the verdict can
fail.

## TD2 — tempo and the retune

Set the played host to the canonical tempo, then retune life-history
constants against TD1 until the terrarium breathes at watchable speed: a
starter lifetime of minutes, populations that rise and fall without boiling
or flatlining across the instrument's seeds. Headless labs keep their own
tick rates; the constants are the world's and change for everyone, which is
why TD1 comes first.

**Done when:** the instrument's receipt at the new constants shows breathing
(not boil, not collapse) over several lifespans across seeds; the played
host runs the canonical tempo; existing tests pass or are retuned with the
change documented.

## TD3 — see the ecology

The brick tracer takes exactly one pose, so the section shows one organism
in an enclosure holding dozens. An ant farm needs the ants: the lens grows
a pose roster — every organism in the slab window drawn, culled to the
section, with a documented cap chosen by uniform-size arithmetic rather
than hope. This is a mesocosm-lens change; the paredros room consumes the
same tracer, so the roster must not break the single-pose path it uses.

**Done when:** a headed capture shows multiple organisms in the section at
once; the cap and its arithmetic are documented; the replay receipt still
lands its hash; paredros-room still builds against the lens.

## TD4 — the hand and the meal

The two core-rule changes, after TD1 lands (they share files with nothing
above but sit in the same crate):

- Instincts under idleness: the world counts consecutive idle intents;
  the ecology drives a controlled critter's locomotion only past the idle
  threshold. A function of the trace, so replay is unaffected.
- Hunger-routed metabolize: the intent drops its route; the body burns when
  starved, incorporates when provisioned, and the threshold is a documented
  constant. The vitals surface already shows the state that decides.

**Done when:** holding a key moves the critter with no instinct fighting
the hand; walking away returns it to the ecology (visible in an idle run);
a starved critter's meal refills the budget on screen; replay receipts
land their hashes; the played slice plan's D5 note records that the choice
became diegetic.

## TD5 — one economy for all life (ruled 2026-08-29, Mark)

NPCs earn energy the way the player does. Today every non-played gain —
producer fixing, grazing, predation, decay — builds biomass only, and
`energy_mg` is a birth endowment that never refills (the TD2d finding), so
every hunger threshold trips early and decomposers cannot bank a corpse
against the gap to the next one. TD5 routes every organism's feeding
income through the body rule TD4 landed for the played meal: starved
(`budget_below(STARVED_UPKEEP_TICKS)`) → the gain credits the budget;
provisioned → it builds the body, as today. One predicate, one economy,
every kingdom. The `dispersal_for` zero-energy gate aligns to the same
hunger predicate while its ledger becomes real.

**Done when:** the instrument shows founded kingdoms persisting to the
horizon in a majority of seeds — `breathes` reached, zero boils, the
collapse control still collapsing; a death-cause probe shows decomposers
banking corpses and surviving the gaps; a mechanics-only receipt is
recorded before any constants follow-up, and any follow-up nudge is
documented constant by constant; fixtures re-record at the new economy.

## TD5b — the founding cohort arrives mid-life (ruled 2026-08-29, Mark)

TD5's own finding: the corpse drought is set at founding, not in `rates.rs`.
Founders stagger age over `rng.below(200)` against a lifespan in the
thousands, so the enclosure holds no real carrion until the founders
themselves start dying of old age around tick 1,800 — decomposers, banking
correctly since TD5, are all dead by tick ~430 with nothing left to bank.
Widen the stagger so a founder's age is drawn proportional to its own
`lifespan_for_mass`, so the cohort arrives distributed across its whole
life — mid-life on average, with a near-death tail that seeds carrion from
the first ticks — rather than every founder a newborn.

**Done when:** the carrion-window probe shows corpses present in the
formerly-empty tick 20-1,400 span; the instrument shows a gain (more
`breathes`, decomposers alive at the horizon) over TD5's mechanics-only
receipt; collapse control still collapses and escapees stay zero; fixtures
re-record at the new genesis.

## TD6 — the closed matter cycle and determinate growth (ruled 2026-08-29, Mark)

The mass fixed point, answered structurally. Income, upkeep and the
reproduction tax all scale as `m^0.75`, so the sign of net growth never
depends on size and no body ever arrives at an adult mass; crowding, which
counts bodies rather than mass, is the only regulator and it never bites
(TD5: producer stands reaching 40 billion mg on ~150 bodies). Mark ruled
**both** answers in one round, one hash break rather than two:

**The matter cycle.** The enclosure gets a finite matter budget held in a
per-voxel-column soil store. Producers draw matter from the soil; **light
stays the one open input** (energy arrives from outside, matter does not);
decay returns bodies to the soil where they fell; the player's `Deposit`
enriches it. Conservation is then the fixed point — mass cannot run away
because it has to be somewhere — and the same store is the **detritus
pool** Mark chose for decomposer persistence, so one system answers both
rulings.

**Determinate growth.** The body plan caps mass: a recipe-derived ceiling
per part, with income past the ceiling routed to budget or brood rather
than to more substance. With matter conserved this stops being load-bearing
stability machinery and becomes what it should be — an evolvable trait the
adaptation phase can push, gigantism as a lineage strategy.

**Granularity ruled per-voxel column, on measured evidence**
(`Code/testing/mesocosm/soil_granularity_probe.md`, 96 configs). At the
shipping enclosure per-voxel is *fastest* for point uptake (1.75us vs
5.66us per place — a direct index into 4KB beats a nearest-site scan) and
is the **only** grain that can express a forage radius at all: per-place
r=3 reaches 12 of 16 sites at any size, and per-crowd-cell r=3 reads every
cell that exists at today's 4x4 grid. Mark's reason for wanting the depth
is roots hunting minerals through soil, which coarse grains cannot
represent at any price. Cost is 0.015% of a tick at 10 t/s; minimum system
requirements are untouched (GPU-bound, as before).

**Done when:** total matter (soil + biomass + carrion) is conserved across
a long headless run — the load-bearing invariant, asserted as a test with a
deliberately-broken control proving it can fail; producer stands no longer
reach runaway biomass and report a bounded end mass; the instrument shows
more seeds breathing with founded kingdoms persisting, zero boils, the
collapse control still collapsing; crowding is either retired as redundant
or its remaining job is stated; fixtures re-record.

## TD7 — life priced by how it lives (ruled 2026-08-29, Mark)

TD6's two flagged residues, answered together by one principle Mark
stated: "moving has metabolic costs. locomotion doesn't come free for
animals. we consume more nutrients than plants. so for flora, roots
seeking water and nutrients and minerals from/in soil is doing so at the
speed of growth on low rent metabolism." Rent prices what a body does,
not only what it weighs.

- **Soil flow: percolation and root forage, both.** Percolation stays as
  shipped — diffusion is the medium's property. Producers additionally
  gain the forage-radius uptake the per-voxel grain was chosen for: roots
  reading neighbouring columns and drawing from the richest — slowly, at
  the speed of growth, on low-rent metabolism.
- **Locomotion-derived rent.** Upkeep gains a motility term derived from
  the body plan (the locomotion anatomy already computes), so sessile
  producers run cheap and motile consumers pay for their speed. The
  trophic asymmetry becomes physics, not an authored constant.
- **Pyramid-shaped founding.** Counts make the pyramid: many producers,
  fewer consumers, few decomposers, replacing the equal-thirds kingdom
  floor. Individual sizes stay what bodies honestly say.
- **Producers grow bigger.** Producer recipes gain the parts/volume for
  real stands — plants are big — raising their derived ceilings.

Alongside, not inside, this round: Mark opened the composable-forms
question — microbe / virus / parasite / symbiote as inhabitable classes
at animal scale ("watching an animal you inhabit do its thing under your
influence"), forms of life as "conditionally composable classes." That
gets its own research brief (traits, prior-art frameworks, and which
combinations the substrate can honestly carry), not a slot in this round.

**Done when:** the instrument reaches breathes in a majority of seeds with
zero boils, founded kingdoms at the horizon, the collapse control still
collapsing, matter still conserved to the milligram (the TD6 test is the
gate every change must pass); the rent asymmetry is derived from body-plan
numbers, not tuned; fixtures re-record.

## TD8 — the chain under `breathes` (ruled 2026-08-29, Mark)

Three rulings, taken together from the open rulings register's blocking set
(entries 1-3), which are the whole remaining chain under `breathes`. TD7
proved the constraint is recruitment rather than mortality, and that no
`rates.rs` constant reaches any of these — each is a rule about what a body
may do, not a number.

- **Reproduction gates on adult mass.** Breeding becomes eligible at a
  fraction of the ceiling the body's own plan implies, replacing the
  absolute 80 mg floor that knows nothing about determinate growth. This is
  the missing half of TD6: a big-plan body must grow up before it breeds,
  which both throttles the founding boom and stops a body stalled at a fifth
  of its ceiling shedding broods it cannot afford. The gestation clock
  stays; the mass floor is what changes. Pick and document the fraction
  against the instrument.
- **Corpses persist longer.** Decomposers starve beside 12-15 standing
  corpses because carrion is a rare event rather than a standing resource;
  the yield lever is ruled out by measurement (quadrupling `DECAYS_BASE_MG`
  changed nothing) and the search was already fixed in TD2d. Slowing carrion
  decay turns the event into a resource, and it lands on the side the code
  already worries about — "the dead return whether or not a decomposer is
  present... locked matter is a real failure mode."
- **No actuator, no travel.** `dispersal_for` floors locomotion at 1, so a
  body drawn with no `Limb` tagma (22 of 160 consumers, 20 of 50
  decomposers) pays a plant's rent under TD7 and moves anyway — grazing at a
  plant's price, which is what seed 2's long-standing collapse was. Remove
  the floor: a body with no contractile parts is sessile. A sessile consumer
  that cannot reach food starves, and that is the correct outcome, not a
  regression. Whether `axis::seed` should draw such a line at all is a
  separate question and is not ruled here.

**Done when:** the instrument reaches `breathes` in a majority of seeds with
zero boils and founded kingdoms at the horizon; the collapse control still
collapses; matter is still conserved to the milligram; each ruling is shown
to have moved what it was aimed at (recruitment, decomposer persistence, the
free-lunch species) rather than only moving the total; fixtures re-record.

## TD9 — income reads the body too (ruled 2026-08-29, Mark)

TD8's fourth structural finding, answered: **TD7 made rent scale with mass
*and* build and left income scaling with mass alone**, so a body that pays
for moving earns no more for having moved. A limbed consumer's rent went
from 1.5-1.8 to 2.3-6.4 mg/tick while its bite stayed where TD2c tuned it
against the cheaper body — and consumers now clear TD2c's ~75% prey hit-rate
bar and starve anyway, on mouthfuls of 5-11 mg. Confirmed structural, not a
number: `GRAZES_BASE_MG` swept to 12 and every seed still ended a pure
producer stand.

- **The bite scales with build.** Feeding income reads the same anatomy that
  rent reads — the machinery a body actually built to feed with. Symmetric
  with TD7 by construction: the body that pays for its machinery is the body
  that gets to use it, and limbs become a strategy rather than a tax.
  Derive it from what the body plan already computes, as TD7 did; no new
  authored constant if one can be avoided, and if one is needed, say why.
- **Producers creep.** TD8's no-actuator ruling made producers sessile as a
  side-effect (129,534 / 2,615 / 292,361 movement events to zero), because a
  producer is unlimbed by construction. Mark ruled they should creep: a
  small movement budget that is **not** actuator-derived — root creep,
  runners — so a stand can still spread without growing legs. It must not
  reopen TD8's free lunch: an unlimbed *consumer* stays sessile, so the
  budget belongs to the producer's own way of living rather than to
  bodies-without-limbs generally.

**Done when:** the instrument reaches `breathes` in a majority of seeds with
zero boils and founded kingdoms at the horizon; the collapse control still
collapses; matter is still conserved to the milligram; the income change is
shown to close the specific gap TD8 measured (consumers clearing the hit-rate
bar and starving anyway) rather than only moving totals; producer creep is
shown to restore spread without restoring the free lunch; fixtures re-record.

## TD10 — kinship tempers the appetite (ruled 2026-08-29, Mark)

TD9's fifth structural finding, answered: the consumer kingdom is eaten by
itself, 90-94% of it by the eater's own species, and it is extinct before its
first possible birth. Of the four candidates TD9 put to Mark — a size ratio, a
species wall, `axis::seed` founding a tier as one interbreeding species, or a
thinner founding cohort — **kinship alone** was ruled.

- **Prey scoring discounts by relatedness.** The closer the target's lineage to
  the eater's, the less appetizing. Derived from `Lineages::distance`, which is
  built, tested and had zero production callers; integer-exact and
  deterministic. Cannibalism becomes **rare, not impossible**: a starving
  predator may still take kin, distant kin more readily than siblings.
- **What an undefined distance costs, decided.** Genesis founds unparented
  roots, so most cross-species pairs have no common ancestor and `distance` is
  `None`. For **predation** that reads as no relation, therefore full appetite.
  The incorporation half of the traits brief's Q1 is untouched and stays open.
- **Hunger still overrides**, through TD2d's existing `is_hungry` horizon.
- **No size gate and no species wall.** Kinship alone.

This is the kinship machinery's first production caller, arriving under a
concept already on the general model plan's F0 sanctioned list ("migration
following kinship rather than distance").

**Done when:** the instrument reaches `breathes` in a majority of seeds with
zero boils and founded kingdoms at the horizon; the collapse control still
collapses; matter is still conserved to the milligram; the same-species share
of consumer predation falls and the consumer kingdom's extinction crosses its
own first-brood interval; seed 2's unlimbed grazers — the natural control,
which never cannibalized — do not move; fixtures re-record.

## TD11 — sight reads the body, hunger follows a gradient (ruled 2026-08-29, Mark)

TD10's sixth cause, answered with both levers. The pattern's completion:
rent reads the body (TD7), income reads the body (TD9), and now the search
horizon does too.

- **Sight reads Sense anatomy.** Near-tier sight range derives from the
  body's `Role::Sensor` parts the way bite derives from actuators —
  TD7-style, from what the plan already computes, no new authored constant
  where avoidable (the flat 8 may survive as floor or reference). A body
  with no sensor parts stays nearsighted; the raycast gate stays, so a
  wider horizon is never wallhack.
- **Hunger follows a gradient.** The one-random-voxel hungry wander becomes
  movement toward denser pasture, by the pattern producers already use
  (`draw_richest_within` over a perceivable radius). Must compose with
  TD10's kinship discount and TD4's held-critter skip.

**Status:** verified 2026-08-30 with the full receipt discipline run — see the
TD11 Progress entry below for both formulas, the four-arm attribution, the
cost table and the verdict. The work sits in `main`'s tree, squashed off
`wip/td11-sight`; the branch is left untouched as history.

**Done when:** the standing TD discipline — breathes or the seventh cause
named with evidence; attribution shows the in-window alternatives table
staying populated and hungry bodies closing distance to pasture; seed
comparisons against TD10's table; conservation exact; fixtures re-recorded.

## After TD11: the playable ecology chain is next

The [playable ecology plan](2026-08-31_playable_ecology_plan.md) owns the
successor product chain. **Refreshed 2026-09-05:** PE0-PE3 now compose flow
readings, reproduction/succession, embodied discovery and lineage review;
the former missing epoch caller and review path are implemented. NPC
speciation remains open. CP1 adds a fixed habitat for observing those bodies,
with ordinary burrow traversal and four presentation-only views (`377d774`).
TG1 typed intake is the next mechanics slice; this does not reopen TD for
another constants-only retune. Its corridor result must be measured after the
shared edibility rule exists, not inferred from the better picture.

The loop-composition correction from the second playtest stands: single-loop
polish is done until the loops can compose.

## The series closes here (ruled 2026-08-30, Mark)

TD11 is the last TD round. Mark's correction, on being shown "the seventh
cause" (the founding draw hands out almost no sense organs):

> "seems to me like there are none of the pressures to ensure that they
> would acquire senses or anything. there is no update stage yet, for one,
> where each lineage can evaluate and refine their bodyplan/trait
> selection/evolutionary trajectory. so there's no loop... hardwiring
> things isn't making mechanics; the hardwiring is like an addition because
> we're unsure of how to proceed. so stop framing the incomplete state of
> an incomplete game as unexpected issues that need squirrely chasing. we
> should do the plan and fix what is functionally broken with the awareness
> that later steps and architecting will be necessary to truly get to a
> functioning prototype mesocosm."

The reading this plan adopts: the eleven rounds fixed real defects (matter
conjured at birth, missing walls, silent refusals, untempered predation)
and those stand — but several "causes" were absences of systems the design
already plans (no update stage, so no selection pressure; one species per
tier, so no kinship structure; no epoch, so no world record), and treating
each absence as a fresh surprise was the wrong frame. The instrument and
its receipt discipline stay; `breathes` stops being a gate that spawns
rounds. The work goes back through the plans.

Two directions ruled with the correction, in Mark's words:

- **Default, well-tuned creatures.** "making creatures just sorta voxel
  chunks is pretty weird. let's start from the presumption that things
  have senses, limbs, and set roles. let's start with default, well tuned
  creatures with bodies, yes, made of voxels, but a bunch of small ones,
  recognizably flora and fauna like how the isometry tokens aren't just
  sorta cylinders... we're shooting for imagined creatures made of voxels
  here. the game is not beginning at the start of evolution. we're not
  dealing with single cells inventing life's basics. there should be some
  built in expectations of capacities." Founding stops rolling bodies from
  nothing and starts from authored, capable archetypes — recognizable
  voxel flora and fauna at the Isometry-token bar, many small voxels per
  body, senses/limbs/roles presumed. Owns the second playtest's
  part-scale finding and the founding-senses observation in one designed
  move. Gets its own plan.
- **The update stage.** The missing loop named above: each lineage
  evaluating and refining its bodyplan, trait selection, and trajectory —
  the epoch boundary and adaptation machinery the epoch boundary and
  phenotype plans already design, wired into the played loop (PS2). This
  is where capacities become subject to selection and evolution becomes
  actual rather than presumed.

## Findings

- **2026-08-30 (TD11, and it is the seventh structural thing in the way,
  measured rather than guessed): the founding cohort has no sense organs to
  read, so a rule that reads them is inert — and in the one seed that grew
  them, it grew them on bodies that cannot walk.** The census, over the
  instrument's own ten seeds, at genesis, before anything moves. Consumers and
  decomposers with **no** part performing `Process::Sense`, out of 307, and the
  mean near horizon the new rule gives that cohort:

  | seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | blind of 307 | **307** | 78 | 230 | 230 | 230 | **307** | 306 | **307** | **307** | **307** |
  | mean horizon | 8.0 | **10.7** | 8.7 | 8.9 | 8.6 | 8.0 | 8.0 | 8.0 | 8.0 | 8.0 |

  In five of the ten seeds **not one body in the world has a sensing part** —
  `max sensor_span` is literally 0 — and a sixth has exactly one. The derived
  horizon in those six is 8.0 voxels, which is the flat constant it replaced, to
  the voxel. Across all ten seeds **307 of 3,070 founding fauna can see**, which
  is one in ten.

  **And the draw is per tier, not per body**, which is why the number is so
  lumpy. A sense organ here is a *geometry*, not an appendage: `plan::classify`
  calls any part whose half-extents are all within 1 a `Role::Sensor`, and
  `axis::seed` draws one recipe per species, so a whole tier is sighted or blind
  together. The cross-tab at genesis, limbed against sighted, says it exactly —
  every sighted count is 77 (the decomposer founding) or 229 (the consumer one):

  | seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | limbed + sighted | 0 | 0 | **77** | 0 | **77** | 0 | 0 | 0 | 0 | 0 |
  | unlimbed + sighted | 0 | **229** | 0 | **77** | 0 | 0 | 1 | 0 | 0 | 0 |
  | max sensor span | 0 | 12 | 10 | 12 | 10 | 0 | 12 | 0 | 0 | 0 |

  Read down the two sighted rows and the whole attribution falls out. **Seed 2's
  229 sighted bodies are, to the body, its unlimbed ones** — the species that
  drew eyes drew no limbs, so its bite `reach` binds before its sight does and
  its world is **bit-identical** with the rule on or off (Progress §3). Seeds 3
  and 5 are the only seeds whose sighted tier can walk, they are the *decomposer*
  tier in both, and they are the only seeds where the sight arm moves anything:
  seed 5's decomposers go 18 → 26 alive and scavenge 456,606 → 602,273 mg on
  sight alone, and seed 3 gains a decomposer tail it did not have. **A
  body-derived rule can only be as good as the bodies the world hands it**, and
  `axis::seed` hands out sense organs to one tier in ten.

  **Mark's call, and it is a rulings-level one, not a constant.** Whether
  `axis::seed` should found sensory tagmata the way it founds contractile ones;
  whether a founding tier drawing *no* sense organ at all should be legal, the
  way TD8 ruled about actuators; whether the sensory palette is simply too thin
  for a per-species draw to hit; or whether an NPC lineage should be able to
  *acquire* what it was not founded with — `World::learn_from` returns early for
  anything but the controlled critter, so today an NPC line is frozen at genesis
  no matter how long it lives or what it eats, which is the traits brief's
  incorporation half and reads `None` lineage distance with the opposite sign
  (see the TD10 finding below). No `rates.rs` constant reaches any of them, and
  the seventh round in a row declines to sweep one.

- **2026-08-30 (TD11): seed 2 has stopped being the control the world supplied,
  and the leave-one-out arm has replaced it.** TD10 leaned on seed 2 staying
  bit-identical, which it could, because TD10 ruled a *predation* rule and seed
  2's consumers are grazers with no consumer targets. TD11 rules a *movement*
  rule, and seed 2 moves — consumers 23 → 8, producers 48 → 80, `cannibal_mg`
  0 → 2,060 — while its unlimbed bodies still record **zero** moves, so TD8's
  ruling is intact and it is the seed's *other* bodies that changed. A control
  is only a control against the class of rule it is blind to, and no seed is
  blind to movement. What replaced it is stronger anyway: arm `neither`
  reproduces `td10_attribution.json` number for number, which proves the whole
  harness rather than one seed's immunity. Later rounds should build the arm
  first and read the seed second.

- **2026-08-29 (TD10, and it is the sixth structural thing in the way, measured
  rather than guessed): a body forages at eight voxels and bites at fifty, so
  the stand walks out of its sight inside fifty ticks and the only thing left to
  eat is its own line.** Kinship is obeyed exactly (Progress §2 above: `chose
  kin` equals, to the decision, the count of decisions with no non-kin candidate
  at all). It nevertheless barely moves the cannibalism number, and the probe's
  new **prey pool** reading says why. Per living consumer, plain non-consumer
  bodies standing inside the window the tick actually scans, and the share of
  consumers with **any**:

  | seed | tick 0 | 50 | 100 | 200 |
  | --- | --- | --- | --- | --- |
  | 1 | **7.85 @ 100%**, 2.6v | 0.55 @ 44%, 7.8v | 0.46 @ 38%, 8.4v | 0.25 @ 25%, 13.3v |
  | 2 | **7.92 @ 100%**, 2.6v | **4.76 @ 95%**, 4.4v | **2.28 @ 71%**, 6.6v | 0.48 @ 31%, 11.3v |
  | 5 | **7.50 @ 99%**, 2.6v | 0.12 @ **6%**, 14.4v | 0.06 @ 4%, 15.6v | 0.04 @ 2%, 17.7v |

  The third figure is voxels to the nearest such body **whether or not the window
  reaches it**, and it is the one that names the cause. At genesis every consumer
  in every seed has something else in sight — the alternative is 2.6 voxels away
  — and the discount works. Fifty ticks later, in the two seeds whose consumers
  go extinct, **94% and 56% of them have no alternative in the window at all**,
  and the nearest one has receded to 7.8 and 14.4 voxels. It has not gone: it is
  **inside the bite and outside the sight**. Seed 2 is the only seed that keeps a
  populated window through the founding transient (4.76 at tick 50, 2.28 at tick
  100, against 0.55 and 0.12), and it is the only seed whose consumers survive.
  Its pool empties too, by tick 200 — by which point its cohort is through the
  transient TD9 named as the fatal part, and is made of grazers that could not
  have eaten each other anyway.

  The mechanism is a **rule, not a number**, and it is the same asymmetry TD9
  closed on the income side, one layer up. `movement::choose_living_target`
  computes `reach = GRAZE_RANGE + body.reach()` — 15 to 62 voxels across the
  founding cohort, and the thing TD7 charges rent for and TD9 pays income on.
  It then scans `sight_range`, which for a near body is
  `reach.min(NEAR_SIGHT_RANGE)` and `NEAR_SIGHT_RANGE` is **8**. Nearly every
  body is near. **So `body.reach()` buys a bigger bite and no wider search
  whatever a body grows**: rent reads the body, income reads the body, and the
  horizon a body looks for that income across does not.

  What the body does instead of searching is TD2d's hungry wander: one random
  grounded voxel per tick, no gradient, no memory beyond eight ticks of a target
  it has already lost. That is how a founding cohort eats its own neighbourhood
  clean and then stands in it.

  **Mark's call**, and it is the same *kind* of decision TD7, TD8 and TD9's were
  — what a body plan is allowed to do — not a rate: whether sight should scale
  with the body the way reach and rent do, whether `NEAR_SIGHT_RANGE` is a
  terrain bound that should stop capping the *affordance* search, whether a
  hungry body should get a gradient rather than a coin flip, or whether the
  founding stand should simply be dense enough that eight voxels is enough. No
  `rates.rs` constant reaches it; `NEAR_SIGHT_RANGE` is not in `rates.rs` and
  raising it alone would be a retune wearing a ruling's clothes.

- **2026-08-29 (TD10): `None` lineage distance means the opposite thing on the
  two sides of the traits brief's Q1, which is why answering it for predation
  does not answer it at all.** The brief asks what an undefined distance costs
  and offers "max cost? outright refusal? a separate unrelated tier?"
  ([traits and perception brief](2026-08-29_traits_and_perception_brief.md) §8
  Q1) — every option phrased as a *penalty*, because the brief was written
  around **incorporation**, where a graft from an unrelated line is the
  expensive one. Predation reads the same `None` the other way round: an
  unrelated body is the *ordinary* meal, and the discount belongs to kin. So
  TD10's answer is "`None` costs nothing", and it is not transferable. The
  standing ruling both readings obey is the one `species.rs:224-226` and the
  epoch boundary plan already made: `None` is the honest answer and no shared
  ancestor may be invented to make the arithmetic work. **Q1 stays open for
  incorporation**, and whoever closes it should expect the opposite sign.

- **2026-08-29 (TD9, and it is the fifth structural thing in the way, measured
  rather than guessed): the consumer kingdom is eaten by itself, and the seed
  where it cannot be is the only seed where it holds.** `Event::Fed` names both
  sides of every meal and no receipt before this one read the *prey's* side.
  Reading it, over 3,000 ticks with both TD9 changes in, seeds 1 / 2 / 5:

  | | taken out of consumers | of that, by consumers | of that, same species | consumers at 3,000 ticks |
  | --- | ---: | ---: | ---: | ---: |
  | seed 1 | 83,211 mg | **80,996 (97%)** | **78,522 (94%)** | 0 |
  | seed 2 | 21,563 mg | **3,250 (15%)** | **0 (0%)** | **23, and flat** |
  | seed 5 | 79,910 mg | **73,244 (92%)** | **71,618 (90%)** | 0 |

  The founding consumer cohort is 230 bodies whose survivors still read ~570 mg
  apiece at tick 50, so it is of order 130,000 mg of flesh; seeds 1 and 5 take
  79-83,000 mg of that back out through consumer mouths,
  **more than half the kingdom's founding mass eaten by the kingdom itself, and
  nearly all of it by its own species.** It is invisible in every earlier
  receipt because a body eaten from 590 mg down to 20 mg dies reading `starved`,
  exactly like one that found nothing — which is why four rounds have read this
  as a food-supply problem.

  Seed 2 is the control the world supplied for free: its consumer species draws
  an unlimbed recipe, so it is a `Grazer` rather than a `Predator`, so
  `choose_living_target` will only let it take `Kingdom::Producer`. It is the
  only one of the three whose consumers survive the probe window (23 alive, the
  curve flattening 36 → 24 → 23 at ticks 1,500 / 2,250 / 3,000), and the only
  one whose cannibalism reads zero. That correlation holds across all four arms
  of TD9's leave-one-out.

  The mechanism is in `movement::choose_living_target`, and it is a rule rather
  than a number: a `Predator`'s candidates are filtered by reach, by signal
  (a `Warning` body is skipped), and **by nothing else** — no size ratio, no
  species check, no kin check. The score then *prefers* mass
  (`.saturating_sub(target.mass_mg.min(256) / 64)`), and at genesis the richest
  plain body within reach of a founding consumer is another founding consumer.
  The cohort is dense, it is its own best meal, and it is gone inside 300 ticks:
  seed 5 reads 230 → 94 by tick 50 and → 20 by tick 300, against a first brood
  interval its own plan sets at ~580 ticks. **The kingdom is extinct before its
  first possible birth**, which is why no income change reaches it.

  It is not mortality-versus-recruitment in the old sense either. Mean age at
  death is 1,114-1,252 ticks against a brood interval of 561-580, so the bodies
  that *survive the founding* do live long enough to breed twice. It is the
  transient that is fatal, and the transient is intraguild predation.

  TD9's own income change makes it sharper, which is the honest reading of the
  leave-one-out: consumer-on-consumer went **63,137 → 80,996 mg** in seed 1 and
  **40,117 → 73,244 mg** in seed 5 with the build-scaled bite in, because the
  bodies with the most build are both the best predators and the richest prey.
  **Mark's call**, and it is the same *kind* of decision TD8's three were —
  what a body plan is allowed to do — not a rate: whether a predator needs a
  size ratio over its prey, whether conspecifics are off the menu, whether
  `axis::seed` should stop founding a whole tier as one interbreeding species,
  or whether the founding cohort should simply not be dense enough to be its own
  pasture. No `rates.rs` constant reaches it; the four arms above are the
  evidence.

- **2026-08-29 (TD9): the played critter no longer survives its own demo.** The
  120-intent playtest trace ends with `state dead` in the vitals panel and
  `body_parts` 0, where TD8 recorded 56 parts and a `burn` notice. Attributed,
  not assumed: the pre-TD9 core re-records the demo at TD8's exact hash
  `3b86d0ef9ebd7d33` with 56 parts, and the same run with TD9 in reads
  `a892c9cf398f08a3` with 0. It is the finding above wearing a hand: the played
  critter is a `Consumer` in a founding cohort that now eats itself with a
  build-scaled bite, and it is the size of body that ranks best as prey. It is
  flagged rather than fixed because the fixture is doing its job — it recorded
  the change — and because the cause is the ruling question above.

- **2026-08-29 (TD8, and it is the fourth structural thing in the way):
  consumers reach food on most of their ticks and starve anyway, because TD7
  priced motility into rent and nothing priced it into income.** Measured over
  3,000 ticks with all three TD8 rulings in, seeds 1 / 2 / 5, counting every
  `Fed` event against every body-tick lived:

  | | prey hit rate | mouthful | deaths starved / aged |
  | --- | ---: | ---: | ---: |
  | consumers | **69% / 22% / 95%** | **7 / 11 / 5 mg** | 158/25, 314/75, 146/74 |
  | decomposers | 54% / 36% / 67% | 15 / 5 / 15 mg | 46/6, 72/13, 63/19 |

  In seeds 1 and 5 a consumer eats on **two thirds to nineteen twentieths** of
  the ticks it is alive for, in an enclosure holding 1,700-2,000 producers, and
  the kingdom still ends the run at 3 and 0. It is not search — TD2c's "~75%
  prey hit rate" bar is being cleared and the bodies die hungry anyway (158 of
  183 and 146 of 220 deaths are starvation, not age). What it *is*, in the
  ledger: a grazer's bite is `GRAZES_BASE_MG` scaled by its own mass and nothing
  else, while TD7 made its rent scale by its mass **and its build** — the same
  round raised a limbed consumer's rent from 1.5-1.8 to 2.3-6.4 mg/tick and left
  the income side exactly where TD2c tuned it against the cheaper body. **A body
  that pays for moving earns no more for having moved.** That asymmetry is
  structural rather than a constant: the fix is not a bigger
  `GRAZES_BASE_MG` (TD7's own sweep raised it to 12 and every seed still ended
  `P/0/0`), it is that a hunt should be worth something a plant's tick is not —
  income that reads the body plan the way TD7's rent does, or a mouthful that
  reads the *prey* rather than the eater. **Mark's call**, because it is the
  same kind of decision TD7 made on the cost side and it belongs beside it.

- **2026-08-29 (TD8): the reproduction gate and the free-lunch ruling both
  reached further than the kingdom they were aimed at, in opposite
  directions.** Two consequences worth having on the record before the next
  round builds on them. First, gating breeding on adult mass throttles
  *producers* hardest, because they are the kingdom whose plans most outran the
  old 80 mg floor (a 3,500 mg plan cleared it at 2% of adult size): producer
  births fell 3,598 → 1,929 in seed 1 while consumer births fell 127 → 33, so
  the gate's largest single effect is a producer throttle rather than the
  consumer recruitment it was reasoned about. Second, "no actuator, no travel"
  makes the *stand* sessile too (129,534 → 0 `Moved` events for seed 1's
  producers), which retired the one way a shaded plant had of leaving its own
  shade. Neither reads as a regression in the receipt — five seeds hold a second
  kingdom against three, and end biomass rose in nine of ten seeds — but both
  are rules about producers arrived at through rulings about animals, and if
  either wants a different answer it wants it explicitly.

- **2026-08-29 (TD7, and it wants a ruling): consumers and decomposers are
  recruitment-limited, not mortality-limited, and reproduction never learned
  about determinate growth.** Measured over 3,000 idle ticks, seeds 1 and 5,
  counting `Born` and `Died` events by kingdom and sampling every living body
  against its own `mass_ceiling_mg`:

  | seed 1 / seed 5 | births | deaths | mean body / adult mass |
  | --- | ---: | ---: | ---: |
  | producers | 252 / 253 | 132 / 104 | 0.32 / 0.37 |
  | consumers | **13 / 6** | 26 / 21 | **0.23 / 0.17** |
  | decomposers | **0 / 2** | 5 / 7 | **0.00 / 0.03** |

  Producers replace themselves twice over; consumers breed at half their death
  rate and decomposers essentially never breed at all. They are not being
  killed faster than producers — they are failing to recruit, because they live
  at a fifth of the adult mass their own body plan describes and a decomposer
  lives at its starvation floor. TD6 derived an adult mass from the body plan
  and made growth determinate; `Organism::can_reproduce` still gates on an
  absolute `STARVATION_MG * OFFSPRING_COST` floor of 80 mg and a gestation
  clock, so it has no idea what "adult" means, and a body stalled at 17-23% of
  its ceiling sheds a quarter of itself per brood into a child born smaller
  still. **Mark's call:** whether breeding should be gated on the plan's own
  adult mass (life history's own answer, and the missing half of TD6), on the
  reserve rather than the body, or left alone with the shortfall fixed
  elsewhere. No `rates.rs` constant reaches it — the sweep above is the
  evidence.

- **2026-08-29 (TD7): decomposers still starve inside a full larder, which is
  TD2c's finding re-measured with the food actually present.** The same probe
  read a mean of **12-15 carrion bodies standing in the enclosure** in seeds 1
  and 5, while the decomposers dying in those runs had gone 26-106 ticks since
  their last meal and were dying at 15-20 mg. Five founding decomposers took
  602 mg of scavenging in 3,000 ticks with a dozen corpses on the ground. The
  binding constraint is `DECOMPOSE_RANGE` and the search, not the yield: TD6
  said raising `DECAYS_BASE_MG` would not rescue them, and quadrupling it in
  this round's sweep did not. It is the one kingdom whose failure is a
  *movement* problem, and it is the reason `breathes` needs a reach ruling
  rather than a rate.

- **2026-08-29 (TD7): a body with no actuator still walks, and now walks for
  free.** `axis::seed` gives a limbed line a chance of drawing no `Limb` tagma
  at all, so a seed can found consumers or decomposers with `actuator_span` 0 —
  across the ten measured seeds at the shipped founding, 22 of 160 consumers
  and 20 of 50 decomposers.
  Those bodies pay a plant's rent, and `dispersal_for` floors `locomotion()` at
  1, so they move anyway. Seed 2 is the visible consequence: its consumer
  species draws an unlimbed recipe, grazes at a producer's price, and is the
  one collapse in the receipt. TD7 priced the machinery honestly; nothing yet
  says a body without the machinery cannot travel. **Mark's call**, since it is
  a rule about what a body plan is allowed to do rather than a number.

- **2026-08-29 (TD6, and it wants a ruling): point uptake at per-voxel grain
  starves the enclosure, because a producer can reach 1 column of 1,089.** The
  granularity ruling was made on performance and expressiveness evidence, which
  it still passes; this is new evidence of a different kind and it invalidates
  nothing the probe measured. Measured, with a sealed store: a producer drains
  the column it stands on within tens of ticks and thereafter earns **exactly
  the rent it just paid** — net zero, permanently, at any constants, because
  the only matter returning to its column is its own. A probe at soil 300
  mg/column read 17 producers standing on **0 mg** with **340,000 mg** lying in
  the enclosure around them; consumers dead by tick 1,100, decomposers by 800,
  every seed collapsing. Raising the seed does not fix it — it converts the
  collapse into producers *mining outward* column by column (610-825 bodies,
  consumers still dead), because reachability rather than supply is what binds.
  **What shipped is soil percolation**, added under protest and flagged here:
  once per tick every column sheds `1/PERCOLATION_DIVISOR` of what it holds
  into its four neighbours, exact in integers, so a column's loss is its
  neighbours' gain and conservation is untouched. It is the medium's own
  property (dissolved minerals move through soil, which is *why* a root that
  searches a radius finds more) rather than the foraging behaviour this round
  deliberately left unbuilt, and it is what makes "the enclosure gets a finite
  matter budget" mean *one* budget. It is decisively load-bearing: with it off
  and everything else identical, every baseline seed ends at **~1 organism**;
  with it on at divisor 8, 46-100. **Mark's call:** whether the real answer is
  the forage radius the grain was chosen for (TD7), percolation as shipped, a
  coarser grain for uptake only, or producers that relocate off spent ground.
  This is the one decision in TD6 taken without him, and it was taken because
  the alternative was shipping a dead terrarium.
- **2026-08-29 (TD6): consumers, not producers, are what still fails.** With
  the cycle closed and percolation supplying the stand, the chain now dies at
  the same place it died in TD5 and TD5b: the founding cohort's ~20 consumers
  over-graze ~17 producers in the first 200 ticks, grow to 700-1,500 mg on the
  proceeds (their body plans allow 2,000-2,500), and then starve in a thinned
  pasture; decomposers follow. The consumer:producer *ceiling* ratio is ~4:1
  because a consumer recipe realizes ~24 parts against a producer's ~5, so
  determinate growth as derived lets a grazer outgrow its own food's adult size
  by four times — an inverted pyramid written into the body plans, not into the
  constants. That is the next round's target, and it is the third round running
  that `breathes` has been out of reach for this reason.
- **2026-08-29 (TD1):** the ecology at 61 founders is **bimodal, not
  reliably explosive**: across seeds 1-5, three boil and two collapse. The
  boils are pure producer blooms — producers 60 → 600+, consumers flat at
  1-3 (in one seed the played lineage dies while plants overrun the
  enclosure), decomposers gone early — so crowding's self-thinning is not
  enough to cap a bloom. The collapses are the mirror: a founder pool one
  bad species draw from zero producers starves the whole chain (seed 3
  founded with none). The retune target is therefore trophic balance —
  producer throttle, consumer viability, founder composition — not "less
  growth". Verdicts early-exit (boil and collapse are terminal; only
  breathes needs the full horizon), which took the receipt from unbounded
  to under two seconds.
- **2026-08-29 (TD2 consequence, noted in advance):** any constant change
  alters world dynamics from genesis, so every recorded trace's replay
  hash — including the 2026-08-28 playtest fixture — stops matching the
  moment TD2 lands. The old files stay as history with their claim
  retired, and a fresh demo trace is recorded at the new constants.

## Progress

- **2026-08-31:** marked the TD series closed and handed its product residues
  to the playable ecology plan. No constants, code, receipts, or TD rulings
  changed.

- **2026-08-30: TD11 verified — the gradient is the half that works, sight
  reads a body the world almost never grows, and the seventh cause is that the
  founding draw hands out no sense organs.** Both mechanisms are in and both
  are correct; leave-one-out over four builds says the gradient carries every
  behavioural gain and sight carries the whole cost. The verdict tally is
  unmoved for the **seventh** round at **0 breathes / 10 thins / 0 boil /
  0 collapse**, control all collapse, escapees 0, and `total_matter_mg` is
  identical seed-for-seed to TD10's ten totals, to the milligram. Receipts:
  `td11_chain.json` (instrument), `td11_attribution.json` (probe, landed
  configuration), `td11_arms.json` (the four-arm leave-one-out),
  `td11_sight.png` (capture). No `rates.rs` sweep was run, deliberately and for
  the seventh round running.

  **1. Sight reads the body, in one line of arithmetic.** `sight_for_body` in
  `rates.rs`, the same `build_multiple` TD7's rent and TD9's bite divide by,
  handed a *sensory* span instead of a contractile one:

  ```text
  sight = NEAR_SIGHT_RANGE * (ceiling + sensor_span * REFERENCE_SEGMENT_MG) / ceiling
  ```

  `sensor_span` is `Organism::sensor_span()` — each living part performing
  `Process::Sense`, its longest half-extent, summed — the exact shape of
  `actuator_span`. **No new authored constant**: the base is the old flat
  `NEAR_SIGHT_RANGE`, which survives as the reference *and the floor*. A blind
  body reads span 0, the multiple is `ceiling / ceiling`, and its horizon is
  **exactly** the eight it always had. Normalized against the plan's own adult
  mass for TD7's reason, so it reads build and not size. Bounded by
  construction: the palette's sensor is half-extent `[1, 1, 1]`, so a body made
  of nothing but sense organs reads `121 / 21` and tops out at 46 voxels — no
  anatomy can see the enclosure. The near cap also stopped clamping by `reach`,
  which is the second half of the ruling rather than a tidy-up: sight is a
  sensory reading and clamping it by an actuator span is reading the wrong
  tissue. **The raycast gate is untouched** — the widened `sight` is passed to
  `can_perceive` exactly as the narrow one was, so a wider horizon is a longer
  ray and never a wallhack.

  **2. Hunger follows a gradient, and it is a heading rather than a sight.**
  `perception::forage_gradient` reads the tick's own sensory buckets the way a
  producer's `draw_richest_within` reads soil columns, over the span the body
  was already searching in with the near cap off (`GRAZE_RANGE + reach`, or the
  decomposer's `DECOMPOSE_RANGE + reach`) — so a body that bites at fifty also
  *smells* at fifty, which is exactly the asymmetry TD10's sixth finding named.
  The answer is a **bucket centre, never a body**: nothing it returns can be
  pursued, bitten or remembered, `can_perceive` still decides every one of
  those, and a body that walks up a gradient into an occluded stand still sees
  nothing when it arrives. **Nearest ring first, richest inside it**, ties to
  the lowest `BTreeMap` key — the same lexicographic shape `preferred_living`
  ranks by, and deterministic on every replay. It composes as ruled: a bucket's
  weight counts only what this body would actually eat and `Kin::remove` scores
  the eater's own line to a hard zero (TD10), and `disperse` is still never
  called for the held body (TD4) — `unlimbed_moves` is 0 in every seed of every
  arm. The step is **one grounded voxel**, the same size the random wander
  took; only the direction changes, and a heading the ground refuses falls back
  to the wander.

  **3. Leave-one-out, four builds, and the positive control holds.** Both
  mechanisms were reverted in place and the probe rebuilt per arm.
  **Arm `neither` reproduces `td10_attribution.json` exactly — every number in
  all three seeds** — so the harness is proven before any arm is read.

  | seed | arm | P/C/D at 3,000 | consumers first zero | cannibal mg | scavenged mg |
  | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | neither (= TD10) | 1810/**0**/16 | 2,153 | 71,143 | 197,347 |
  | 1 | sight only | 1810/**0**/16 | 2,153 | 71,143 | 197,347 |
  | 1 | gradient only | 1890/**5**/36 | **never** | 66,826 | 768,563 |
  | 1 | both | 1890/**5**/36 | **never** | 66,826 | 768,563 |
  | 2 | neither (= TD10) | 48/**23**/0 | never | 0 | 39,275 |
  | 2 | sight only | 48/**23**/0 | never | 0 | 39,275 |
  | 2 | gradient only | 80/**8**/0 | never | 2,060 | 78,715 |
  | 2 | both | 80/**8**/0 | never | 2,060 | 78,715 |
  | 5 | neither (= TD10) | 1794/0/18 | 1,139 | 98,154 | 456,606 |
  | 5 | sight only | 1798/0/26 | **674** | 100,242 | 602,273 |
  | 5 | gradient only | 1853/0/30 | 858 | 107,072 | 722,815 |
  | 5 | both | 1815/0/23 | 862 | 95,954 | 686,637 |

  **In seeds 1 and 2 the sight arm is bit-identical to the pre-TD11 core.** In
  seed 1 that is because the whole cohort is blind. In seed 2 it is stranger and
  worth the finding below: its consumers *do* carry sense organs and their
  measured window really does go 8.0 → 11.1 voxels, and the world still does not
  move a body, because the same species draw that grew them eyes grew them no
  limbs — all 229 of them, measured, not inferred — so their bite `reach` binds
  before their sight does. Only seed 5 moves under sight alone, and there the
  sighted tier is the **decomposers**, who gain (18 → 26 alive, 456,606 →
  602,273 mg scavenged) while the consumers lose: extinction from 1,139 to 674.

  **4. The in-window alternatives table, which is what TD10 asked for.** Per
  living consumer, plain non-consumer bodies inside the window the tick actually
  scans, the share of consumers with any, and voxels to the nearest such body
  whether or not the window reaches it.

  | seed | arm | tick 0 | 50 | 200 | 600 | 1,500 | 3,000 |
  | ---: | --- | --- | --- | --- | --- | --- | --- |
  | 1 | neither | 7.85 @ 100%, 2.6v | 0.55 @ 45%, 7.8v | 0.25 @ 25%, 13.3v | 2.67 @ 67%, 10.6v | 0.17 @ 17%, 9.1v | **0 @ 0%** (extinct) |
  | 1 | both | 7.85 @ 100%, 2.6v | 0.55 @ 45%, 7.8v | 0.40 @ 35%, 10.8v | 2.80 @ 100%, 7.0v | 2.22 @ 67%, 7.2v | 0.20 @ 20%, 8.8v |
  | 2 | neither | 7.92 @ 100%, 2.6v | 4.75 @ 96%, 4.4v | 0.48 @ 32%, 11.3v | 0.17 @ 12%, 18.0v | 0.30 @ 17%, 26.5v | 0.39 @ 13%, 25.5v |
  | 2 | both | **14.42 @ 100%**, 2.6v | **9.14 @ 100%**, 4.3v | **1.40 @ 54%**, 11.8v | **0.82 @ 46%**, 12.9v | **0.90 @ 60%**, 10.9v | **2.50 @ 62%**, 12.0v |
  | 5 | neither | 7.50 @ 100%, 2.6v | 0.12 @ 6%, 14.4v | 0.04 @ 3%, 17.7v | 1.00 @ 100%, 8.0v | 0 (extinct) | 0 (extinct) |
  | 5 | both | 7.50 @ 100%, 2.6v | 0.08 @ 4%, 17.7v | 0.43 @ 4%, 21.0v | 0.33 @ 33%, 28.0v | 0 (extinct) | 0 (extinct) |

  **Yes, it keeps the window populated where a consumer kingdom survives to be
  measured, and no, that is not sight doing it.** Seed 2's doubling at tick 0 is
  the sight arm's arithmetic and nothing else — it is the probe reading a wider
  window over an unchanged world. Everything after tick 50 in seeds 1 and 2 is
  the gradient: it is the only arm that moves the trajectory.

  **Hungry bodies do close distance.** Voxels to the nearest edible body, which
  is the reading the gradient is aimed at, seed 2 (the only seed with consumers
  alive in both arms all the way out):

  | tick | 200 | 300 | 400 | 600 | 750 | 1,500 | 2,250 | 3,000 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
  | gradient off | 11.3 | 16.2 | 16.9 | 18.0 | 20.0 | 26.5 | 29.2 | 25.5 |
  | gradient on | 11.8 | **13.4** | **12.9** | **12.9** | **13.8** | **10.9** | **14.2** | **12.0** |

  Off, the pasture recedes monotonically to 29 voxels and stays there. On, it is
  held between 11 and 14 for three thousand ticks. Seed 1 reads the same shape
  (15.4 → 10.7 at tick 300, and 8.8 rather than a dead kingdom at 3,000), and
  the decomposers say it loudest: seed 1 scavenges **768,563 mg against 197,347**
  and ends with 36 decomposers rather than 16.

  **Consumer survival past the founding transient, in the two seeds that lost
  them.** Seed 1's consumer curve goes 64 → 40 at tick 200 (the gradient costs
  it bodies early — a walking body burns), then 3 → 5 at 600, 2 → 10 at 750,
  0 → 6 at 2,250 and **0 → 5 at 3,000, never reading zero at all**. That is the
  first time in eleven rounds a consumer kingdom in seed 1 has been alive at the
  probe horizon. Seed 5 is not saved: 1,139 → 862. Seed 2, which never lost
  them, ends 23 → 8.

  **5. Seed 2, the standing control, moves — and it is not the unlimbed
  grazers that moved.** TD10 recorded seed 2 bit-identical because its
  consumers are unlimbed `Grazer`s whose only legal targets are producers. Under
  TD11 the seed changes: consumers 23 → 8, producers 48 → 80, and
  `cannibal_mg` goes 0 → 2,060 for the first time. `unlimbed_moves` is still
  **0** — the free lunch stays withdrawn and TD8's ruling is intact. What moved
  is everything *around* them: producer `Moved` 5 → 44, its limbed consumers
  44 → 303, decomposers 9,054 → 11,376. The control was a control against a
  *predation* rule, and the gradient is a *movement* rule, so it was never
  entitled to hold here. Stated rather than explained away: the seed the world
  supplied to keep TD10 honest cannot keep TD11 honest, and arm `neither`
  replaces it.

  **6. Cost, and it is the round's cheapest surprise.** Sight range feeds the
  bucketed scan, so a wider horizon means more buckets — but almost no body has
  a wider horizon. Measured two ways. The attribution probe, same three seeds
  and 3,000 ticks per arm, wall clock normalized by the run's own body-ticks
  (`alive_ticks`, which the probe already records, so populations that diverge
  between arms do not confound it):

  | arm | wall ms | body-ticks | us per body-tick |
  | --- | ---: | ---: | ---: |
  | neither | 33,711 | 6,072,919 | 5.551 |
  | sight only | 34,042 | 6,063,914 | **5.614** |
  | gradient only | 34,027 | 6,192,088 | 5.495 |
  | both | 34,009 | 6,058,173 | **5.614** |

  **+1.1%, and every milligram of it is sight**; the gradient is free to the
  noise floor, because `densest_cell` only runs for a hungry body that resolved
  no target at all and it walks buckets the tick already built. `sight_cost_receipt`
  agrees from the other end — a 300-body Near tick goes **2,013.75 us → 2,061.00 us**
  (+2.3%, 12.1% → 12.4% of a 16.7 ms frame), the per-body shape is unchanged, and
  its four state hashes are **identical across all four arms**, which is itself the
  finding: eight ticks from genesis, nobody is hungry and nobody has an eye.

  **Fixtures, and the demo hash did not move.** Re-recorded: 120 intents, hash
  **`8f6df49c63923be6`** — *the same hash TD10 recorded*. Headed `--replay`
  lands it exactly, exit 0, 30 frames on the RTX 4060 (Vulkan), ground revision
  2, `slab_half_height` 28, 35 roster members, `body_parts` 60. The instrument
  is proven the usual way: one bit flipped in the recorded hash exits **1** with
  `MISMATCH`. This is the first round whose fixture did not break, and the
  reason is measured rather than assumed — the probe run at the demo's own seed
  (10975940, the same `FOUNDERS` roster) reads a **8.0-voxel window and 0%
  sighted consumers at every sample**, so sight is inert in that world by
  construction, and across the demo's 120 ticks no hungry body's gradient step
  differed from the random one it replaced. Default paths
  (`ps1_played.trace.json` / `.json` / `.png`), plus `td11_sight.png`, which is
  byte-identical to `ps1_played.png`.

  **The capture, read.** A flat grey sky over the top half; a stepped dark-red
  soil section rising left to right onto a plateau, over a dark grey interior.
  Roughly twenty-five bodies strung along the surface — orange-red capsules,
  green hemispheres, four or five lavender ones. Four black notches sit in the
  soil surface where the section shows no voxel, in the same places TD10
  recorded them and still unexplained. Minimap top right: a purple field with
  scattered pink cells, a pale blue view wedge and the white dot of the
  controlled critter at its apex. Vitals bottom left: `energy 2934 mg` on a
  nearly full bar with the ordinary `burned` notice — the played critter is
  alive with all 60 parts, as TD10 left it.

  **Verdicts.** Baseline is TD10's own ten seeds at the same ±64 horizon
  (`td10_chain.json`, which is the pre-TD11 core).

  | seed | verdict | start | end (TD10) | end (TD11) | P/C/D end (TD10) | P/C/D end (TD11) | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 917 | 1,530 | 1,629 | 1471/0/59 | 1569/0/**60** | 1,241,396 mg | 488,432 | 2,206,906 |
  | 2 | thins | 917 | 887 | 1,117 | 813/74/0 | 1094/**23**/0 | 691,333 mg | 672,825 | 2,220,206 |
  | 3 | thins | 917 | 1,425 | 1,480 | 1425/0/0 | 1431/0/**49** | 1,087,614 mg | 645,999 | 2,217,028 |
  | 4 | thins | 917 | 1,415 | 1,430 | 1415/0/0 | 1430/0/0 | 847,273 mg | 71,814 | 2,202,302 |
  | 5 | thins | 917 | 1,605 | 1,571 | 1560/0/45 | 1521/0/**50** | 1,235,217 mg | 529,745 | 2,214,890 |
  | 6 | thins | 917 | 1,445 | 1,428 | 1445/0/0 | 1408/0/**20** | 762,454 mg | 954,104 | 2,214,018 |
  | 7 | thins | 917 | 1,410 | 1,495 | 1410/0/0 | 1461/0/**34** | 1,157,160 mg | 515,757 | 2,222,946 |
  | 8 | thins | 917 | 1,339 | 1,258 | 1339/0/0 | 1258/0/0 | 370,074 mg | 1,361,412 | 2,212,850 |
  | 9 | thins | 917 | 1,199 | 1,159 | 1199/0/0 | 1159/0/0 | 381,867 mg | 1,317,317 | 2,209,340 |
  | 10 | thins | 917 | 1,159 | 1,295 | 1159/0/0 | 1295/0/0 | 395,097 mg | 1,310,729 | 2,204,656 |

  **Six seeds hold a second kingdom to the horizon against TD10's three**, which
  is the best that number has read in the whole plan — and every one of the
  three new ones is a *decomposer* tail (3, 6, 7), bought by the gradient
  walking scavengers onto carrion. **It still does not breathe**, and it is not
  close: nine of ten seeds end as a pure producer stand, the consumer kingdom is
  extinct at the ten-thousand-tick horizon in nine of ten, and seed 2's 23
  consumers are a decaying remainder rather than a population. See the first
  Finding below for the seventh cause, which this round measured directly.

  **Tests.** Two behaviours in `movement/tests.rs` (a blind plan reading exactly
  the old eight, sensory anatomy buying horizon, the far tier untouched; and the
  gradient walking past a nearer, fatter sibling to a stranger's bucket, then
  answering `None` when only kin are in the horizon) and one rule in
  `rates.rs`'s own tests (the blind case exact, monotonic in span, scale-free in
  ceiling, and the 46-voxel ceiling). `cargo test -p mesocosm-core --test matter
  --release` green — conservation is the gate and it holds.

  **Residues.** `rates.rs` came off the branch at **611 lines**, over the
  repo's six-hundred ceiling; it is back at exactly 600 with comment volume
  trimmed and the long-form reasoning left here instead, which is the remedy
  `CLAUDE.md` names, but the file has no room left and the next rate to land
  there splits it. Two doc errors came off the branch with it and are corrected:
  `sight_for_body` claimed its caller still clamped by reach (it does not), and
  the probe's census claimed the sensory draw was gated by `Appendage::Feeler`
  acquisition (it is not — `plan::classify` reads part *geometry*, and
  `axis::seed` draws it at genesis). The demo's four black soil notches are
  still recorded as seen rather than explained, for the second round.

  `cargo test --workspace` green, `mesocosm-lens` separately at
  `--test-threads=1` (38 passed). `cargo clippy --workspace --all-targets --
  -D warnings`: clean. `cargo fmt --all --check`: clean. `cargo check -p
  paredros-room --features r1-proof`: builds, from inside the `paredros`
  checkout, with the same one pre-existing `dead_code` warning TD6 through TD10
  recorded.

- **2026-08-29 (last today): TD10 landed — kinship is spent, it is obeyed
  exactly, and the cohort still eats itself, because after fifty ticks there is
  nothing else in sight.** Conservation is milligram-exact and identical
  seed-for-seed to TD9's ten totals, the control still collapses, escapees are
  zero, and the verdict tally is unmoved for the sixth round at **0 breathes /
  10 thins / 0 boil / 0 collapse**. The ruled change is in, it does at the
  decision level precisely what it was ruled to do, and the cannibalism number
  it was aimed at barely moves. **What this round bought is the sixth cause, and
  it is a rule rather than a rate** — see the first Finding below. Receipts:
  `td10_chain.json` (the instrument) and `td10_attribution.json` (the probe,
  extended with TD10's target and a prey-pool reading).

  **1. The discount rule, in one line of arithmetic.** `Lineages::distance` gets
  its first production caller, in a new `organism/ecology/kinship.rs`. It is
  spent as a **remove** — how much further away a body reads for being kin, in
  the same voxels the score already measures distance in:

  ```text
  remove = (span + 1) >> (forks + hungry)      relation known
  remove = 0                                   no common ancestor
  ```

  `span` is the far edge of whatever the eater is choosing among: bite reach in
  `choose_living_target`, sight in the two paths that decide where to walk. A
  body of the eater's own line therefore reads as though it stood one voxel past
  that edge and **ranks behind everything the eater could actually get to**,
  while each fork of divergence halves the remove, so a cousin is taken more
  readily than a sibling and a distant cousin is barely noticed. **No new
  authored constant**: the base is the score's own distance span and the gate is
  TD5's existing hunger horizon. Nothing is ever forbidden — a discounted body
  is still a candidate — which is what "rare, not impossible" means in code.

  **The undefined distance costs nothing, and only for predation.** Genesis
  founds unparented roots, so `distance` is `None` for every cross-founder pair.
  For predation the natural reading of "not related" is full appetite, so `None`
  is zero remove and a producer or a stranger's line is eaten exactly as before
  this round. That is the traits brief's Q1 answered **for predation only**; the
  incorporation half stays open and has a Finding of its own below, because the
  same `None` reads with the opposite sign there.

  **Hunger reads as one more fork.** A body inside `is_hungry` (TD2d's horizon,
  `energy_mg` under eight ticks of rent) shifts one place further and halves its
  remove: a starving predator takes a sibling as readily as a fed one takes a
  cousin. No size gate and no species wall, as ruled.

  **2. It is obeyed exactly — the decision-level receipt.** Instrumented
  temporarily inside `choose_living_target` (counters removed before landing),
  every predator decision was classified. **`chose kin` equalled, to the
  decision, the number of non-empty decisions with no non-kin candidate at all**
  — seed 1 to tick 50: 9,201 decisions, 392 with nothing in range at all, 4,879
  with a non-kin option, **3,930 chose kin = 9,201 − 392 − 4,879**. It held to
  the unit at every sample of both seeds (seed 1 to tick 200: 13,668 − 3,611 −
  5,053 = 5,004; seed 5 to tick 200: 27,411 − 9,419 − 9,921 = 8,071). A predator
  now takes its own line **only** when there is nothing else it can see. There is
  no headroom left in the rule.

  **3. And the number it was aimed at barely moves,** because the pool it
  chooses from empties. Probe at 3,000 ticks, seeds 1 / 2 / 5:

  | | consumer-on-consumer (TD9 → TD10) | same-species of that | taken out of consumers | consumers at 3,000 |
  | --- | --- | ---: | --- | ---: |
  | seed 1 | 80,996 → **73,635 mg** | 78,522 → **71,143 (96%)** | 83,211 → 75,796 | 0 |
  | seed 2 | 3,250 → **3,250 mg** | 0 → **0 (0%)** | 21,563 → 21,563 | **23** |
  | seed 5 | 73,244 → **99,780 mg** | 71,618 → **98,154 (98%)** | 79,910 → 103,299 | 0 |

  Seed 5 goes *up*, and honestly so: its consumers now live to tick 1,139
  instead of ~700, and a longer-lived cohort with no alternative eats more of
  itself. **Seed 2, the control the world supplied, is bit-identical** — every
  number in its whole run, not only the cannibalism ones. That is not luck: its
  consumers draw unlimbed recipes, so they are `Grazer`s, so their only legal
  targets are `Kingdom::Producer`, so `distance` is `None` for every candidate
  and the remove is zero by construction. The control could not have moved and
  did not.

  **Extinction against the brood interval, now measured rather than inferred.**
  The probe reads the first tick the kingdom is empty and the interval the
  *founders'* own plans ask for (TD9 read the interval off bodies that died,
  which a seed where nothing dies cannot report):

  | seed | first zero | founding brood interval | trough |
  | ---: | ---: | ---: | --- |
  | 1 | 2,153 | 575 | 0 at 2,153 |
  | 2 | **never** | 573 | 18 at 2,805 |
  | 5 | 1,139 | 582 | 0 at 1,139 |

  The crossing TD9 named is gone as an *arithmetic* fact — survivors do outlive
  a first brood interval now — and it bought nothing, because recruitment is 17
  and 6 births over three thousand ticks. Extinction moved from a transient to a
  slow bleed; it did not stop.

  **Verdicts.** Baseline is TD9's own ten seeds at the same horizon.

  | seed | verdict | start | end | P/C/D end (TD9) | P/C/D end (TD10) | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 917 | 1,530 | 1,506/0/51 | 1,471/0/**59** | 1,266,757 mg | 393,320 | 2,206,906 |
  | 2 | thins | 917 | 887 | 878/55/0 | 813/**74**/0 | 508,276 mg | 867,447 | 2,220,206 |
  | 3 | thins | 917 | 1,425 | 1,435/0/38 | 1,425/0/0 | 842,267 mg | 94,069 | 2,217,028 |
  | 4 | thins | 917 | 1,415 | 1,430/0/0 | 1,415/0/0 | 849,879 mg | 77,238 | 2,202,302 |
  | 5 | thins | 917 | 1,605 | 1,458/0/0 | 1,560/0/**45** | 1,286,169 mg | 410,024 | 2,214,890 |
  | 6 | thins | 917 | 1,445 | 1,484/0/0 | 1,445/0/0 | 771,126 mg | 313,017 | 2,214,018 |
  | 7 | thins | 917 | 1,410 | 1,503/0/47 | 1,410/0/0 | 886,436 mg | 54,682 | 2,222,946 |
  | 8 | thins | 917 | 1,339 | 1,311/0/0 | 1,339/0/0 | 370,023 mg | 1,370,162 | 2,212,850 |
  | 9 | thins | 917 | 1,199 | 1,197/0/0 | 1,199/0/0 | 349,465 mg | 1,377,597 | 2,209,340 |
  | 10 | thins | 917 | 1,159 | 1,184/0/0 | 1,159/0/0 | 362,011 mg | 1,364,477 | 2,204,656 |

  Three seeds hold a second kingdom to the horizon against TD9's four — seed 5
  gains a decomposer tail and seed 2's consumers go 55 → 74, while seeds 3 and 7
  lose theirs. `total_matter_mg` is flat across every sample of every run and
  **identical seed-for-seed to TD9's ten totals, to the milligram**. Control all
  collapse, max escapees 0. **A lateral move in the verdict, and the round says
  so.** No `rates.rs` sweep was run, deliberately and for the sixth round
  running.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`8f6df49c63923be6`** (was
  `a892c9cf398f08a3`), headed `--replay` landing it exactly, exit 0, 30 frames on
  the RTX 4060 (Vulkan), ground revision 2, `slab_half_height` 28, 35 roster
  members. Instrument proven twice, as TD9 did it: one bit flipped in the
  recorded hash exits **1** with `MISMATCH`, and the *pre-TD10* core — this
  round's `src/` stashed, nothing else touched — re-records TD9's own
  `a892c9cf398f08a3` exactly, which is what makes the played critter's survival
  below an attribution rather than a guess. Default paths
  (`ps1_played.trace.json` / `.json` / `.png`), plus `td10_kinship.png`, which is
  byte-identical to `ps1_played.png`.

  **TD9's residue is answered, and the answer is the calmer cohort rather than
  kinship.** TD9 recorded the played critter dying in its own demo —
  `state dead`, `body_parts` 0. It now reads **`body_parts` 60** and a vitals
  panel showing `energy 2934 mg` on a nearly full bar with the ordinary `burn`
  notice. It is *not* the discount protecting it: the played critter is its own
  `SpeciesId`, so every founding line is unrelated to it and its own appetite and
  its attackers' appetite for it are both untouched by TD10. What changed is the
  world around it — the founding cohort spends fewer of its early ticks eating
  and more of them walking, and the body that was being stripped at the demo's
  scale is no longer standing in the middle of that. The capture reads a stepped
  dark-red soil section under a flat grey sky, with roughly twenty-five bodies
  strung along the surface — orange-red capsules, green hemispheres, four or
  five lavender ones — and the minimap top right. Four black notches sit in the
  soil surface where the section shows no voxel; the demo script does press dig,
  so that is the likely reading, but it was not verified and is recorded as seen
  rather than explained.

  **Tests.** Two new files rather than two new blocks: `ecology/kinship.rs`
  carries the rule's own tests (the undefined-distance decision, each fork
  halving the remove, hunger reading as one more fork, and that the memo changes
  no answer), and `movement.rs` was split at 585 lines into `movement/tests.rs`,
  which carries the behaviour — a predator passing over a *nearer and fatter*
  body of its own line for a stranger, then taking the sibling when the stranger
  is removed. No existing test was retuned.

- **2026-08-29: TD9 landed — income reads the body, producers
  creep, and `breathes` is still out of reach. What this round bought is not a
  verdict, it is the cause: the consumer kingdom is eaten by itself.**
  Conservation is milligram-exact and identical seed-for-seed to the pre-round
  baseline, the control still collapses, escapees are zero, and the verdict
  tally is unmoved at **0 breathes / 10 thins / 0 boil / 0 collapse**. Both
  ruled changes are in and both moved their own targets. Receipts:
  `td9_chain.json` (the instrument) and `td9_attribution.json` (the probe, now
  `crates/mesocosm-core/examples/td8_attribution/`, extended with TD9's two
  targets and run in all four arms of a leave-one-out).

  **1. The bite scales with build, by exactly the multiple TD7's rent divides
  by.** TD7 lifted rent's build term out of the arithmetic; TD9 lifts it out of
  TD7 as `rates::build_multiple` so there is **one** such term and both halves
  of a body's ledger read it:

  ```text
  bite = GRAZES_BASE_MG * m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
                        / (m_ref^0.75 * ceiling)
  rent = UPKEEP_BASE_MG  + m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
                        / (UPKEEP_SCALE * ceiling)
  ```

  Three body-plan numbers, all already here — `biomass_mg`, `actuator_span`,
  `mass_ceiling_mg` — and **no new authored constant**. `GRAZES_BASE_MG` and
  `DECAYS_BASE_MG` are untouched at TD2c's 3 and 4; the sweep of the first to 12
  is on record as not reaching this, and a base that had to move would have
  meant the round was a retune wearing a ruling's clothes. `feeding_rate_for_mass`
  and `decay_rate_for_mass` became `..._for_body`, taking span and ceiling; the
  scavenger arm gets the same multiple, because a decomposer that grew something
  to tear with should tear more off for the same reason a predator should.

  **The symmetry check, exact rather than directional.** A sessile body reads
  span 0, the multiple is `ceiling / ceiling`, and the bite is **bit-identical**
  to the old `allometric_rate(GRAZES_BASE_MG, m)` — one floor over a fraction
  that reduces, not two roundings — which is TD7's own collapse-to-the-old-
  formula test written on the income side. A test asserts both that and the stated
  formula. What it does **not** assert is a ratio of the two rates: rent divides
  by `UPKEEP_SCALE` and income does not, so the two floors round at different
  scales and the shared thing to state is the multiple, which is one function.

  **Its target moved, and the specific gap TD8 measured closed.** Leave-one-out
  at 3,000 ticks, seeds 1 / 2 / 5, mouthful and hit rate:

  | arm | mouthful C | hit rate C | intake per body-tick | consumer adult% mean |
  | --- | ---: | ---: | ---: | ---: |
  | neither change (round start) | 7 / 11 / 5 mg | 69 / 22 / 95% | 4.8 / 2.4 / 4.8 | 34 / 37 / 9 |
  | creep only (**bite left out**) | 8 / 10 / 5 mg | 71 / 27 / 95% | 5.7 / 2.7 / 4.8 | 48 / 44 / 9 |
  | bite only (creep left out) | 23 / 11 / 34 mg | 27 / 25 / 27% | 6.2 / 2.8 / 9.2 | 43 / 43 / 14 |
  | **both, as shipped** | **25 / 11 / 35 mg** | 28 / 25 / 27% | **7.0 / 2.8 / 9.5** | **50 / 43 / 17** |

  The row that matters is the third against the second: leaving the bite out
  leaves the mouthful exactly where TD2c tuned it, and putting it in is the only
  thing in this round that moves it. The 5-11 mg mouthful TD8 named is gone:
  23-35 mg where the body is limbed, and unchanged at 11 in seed 2 — whose
  consumer species is unlimbed, so its bite is multiplied by one, which is the
  symmetry visible in the wild rather than in a test. Intake per body-tick rises
  by half to double and consumers live at half their adult mass instead of a
  third. **The falling hit rate is not a regression**: a bigger bite fills
  `intake_room_mg` sooner and a full body does not reach for a meal, so
  `fed_events / alive_ticks` now conflates "cannot find food" with "does not
  need any". Intake per body-tick is the honest number and it went up.

  **And it did not reach `breathes`**, for a reason this round measured — see
  the first Finding below. Consumers end at 0 / 23 / 0 against 3 / 7 / 0.

  **2. Producers creep, and the creep is the smallest budget in the file.**
  `rates::travels` replaces the bare `actuator_span() == 0` test in
  `movement::disperse`: a body travels if it has an actuator **or** if it is a
  `Producer`. The exception is written against the **feeding mode**, never
  against the absence of limbs, which is the whole of the care — an unlimbed
  *consumer* still reads false and stays exactly as sessile as TD8 left it.

  **The size, and why.** `preferred_target` gives a producer no target, so the
  hungry-wander branch is its entire travel budget: **one grounded voxel per
  tick, only while its reserve is under `HUNGRY_UPKEEP_TICKS` (8 ticks) of
  rent, paid for in substance like any other step.** Only a plant being shaded
  or drained out of its own column moves, and it moves at one voxel. A creeping
  body is additionally barred from the place-graph hop the far tier and the
  groundless fixtures use — a stand spreads out of its own shade, it does not
  relocate to the next place — which is the one thing here that is smaller than
  what TD8 removed rather than equal to it. **No new constant**: the rate is the
  wander's own one-step-per-tick and the gate is TD5's existing hunger horizon.

  **Its target moved, and the free lunch stayed shut.** Seeds 1 / 2 / 5, producer
  `Moved` events **0 / 0 / 0 → 48,065 / 5 / 122,974**, against TD8's pre-ruling
  129,534 / 2,615 / 292,361 — restored, and at a third to a half the volume,
  which is the hunger gate doing the limiting. Unlimbed consumer and decomposer
  `Moved` events are **0 in every seed and every arm**, which is what TD8's
  ruling means in receipt terms and is the line this change had to not cross.

  **The honest half: the spread it buys is small.** Producer occupancy at the
  horizon, in 8-voxel cells, goes 285 → 289 (seed 1) and 288 → 288 (seed 5).
  Creep restores *movement*; it barely moves *occupancy*, because breeding
  already scatters offspring ±12 voxels and the stand had therefore not actually
  lost the ability to spread when TD8 made it sessile — it had lost the ability
  of an individual plant to leave. That is worth having and is what was ruled,
  but the "129,534 → 0" number TD8 flagged overstated what was lost, and this
  round can say so with the occupancy column.

  **Verdicts.** Baseline reproduced at HEAD before anything was touched and
  matched TD8's table seed for seed, verdict for verdict. Receipt:
  `td9_chain.json`.

  | seed | verdict | start | end | P/C/D end (TD8) | P/C/D end (TD9) | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 917 | 1,557 | 1,637/0/54 | 1,506/0/**51** | 1,285,193 mg | 346,865 | 2,206,906 |
  | 2 | thins | 917 | 933 | 1,036/45/0 | 878/**55**/0 | 549,904 mg | 830,332 | 2,220,206 |
  | 3 | thins | 917 | 1,473 | 1,472/0/0 | 1,435/0/**38** | 1,092,153 mg | 598,746 | 2,217,028 |
  | 4 | thins | 917 | 1,430 | 1,492/0/27 | 1,430/0/0 | 855,336 mg | 76,948 | 2,202,302 |
  | 5 | thins | 917 | 1,458 | 1,636/0/53 | 1,458/0/0 | 865,262 mg | 72,079 | 2,214,890 |
  | 6 | thins | 917 | 1,484 | 1,386/0/0 | 1,484/0/0 | 781,143 mg | 315,594 | 2,214,018 |
  | 7 | thins | 917 | 1,550 | 947/52/0 | 1,503/0/**47** | 1,317,265 mg | 358,280 | 2,222,946 |
  | 8 | thins | 917 | 1,311 | 1,193/0/0 | 1,311/0/0 | 380,819 mg | 1,336,331 | 2,212,850 |
  | 9 | thins | 917 | 1,197 | 1,043/0/0 | 1,197/0/0 | 354,588 mg | 1,387,178 | 2,209,340 |
  | 10 | thins | 917 | 1,184 | 1,151/0/0 | 1,184/0/0 | 359,938 mg | 1,363,773 | 2,204,656 |

  **What the curve actually does, since `thins` is the verdict either way.** It
  is not a flatline: seed 1 runs 917 → 531 (tick 100) → 760 → 1,311 → **1,874
  (tick 3,100)** → 1,362 (tick 5,100) → 1,798 (tick 8,100) → 1,557, a founding
  crash then a roughly 3,000-tick oscillation of ±25% about ~1,600. That is a
  producer stand rising and self-thinning against the soil, with a decomposer
  tail riding the carrion it drops. It is a **living** curve and it is still not
  `breathes`, because the verdict asks for the *founded kingdoms* at the
  horizon and the consumer line is flat at zero in nine of ten seeds — every
  one but seed 2, the seed whose consumers cannot eat each other. A stand
  that oscillates is not a chain that breathes, and TD1's `Thins` verdict exists
  precisely to refuse the relabelling.

  Control all collapse, max escapees 0, `total_matter_mg` flat across every
  sample of every run and **identical seed-for-seed to the pre-round baseline**
  — the same ten totals TD8 recorded, to the milligram. Four seeds hold a second
  kingdom to the horizon against TD8's five: seed 3 gains decomposers and seed 7
  swaps consumers for them, while seeds 4 and 5 lose theirs. End biomass rose in
  seven of ten. **This is a lateral move in the verdict and the round says so.**

  **No `rates.rs` sweep was run**, deliberately and for the fifth round running.
  What this round has to say about the constants is that they are not where the
  answer is, and the Findings below say where it is instead.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`a892c9cf398f08a3`** (was
  `3b86d0ef9ebd7d33`), `--replay` headed landing it exactly, exit 0, 30 frames on
  the RTX 4060 (Vulkan), ground revision 2, `slab_half_height` 28, 33 roster
  members. Instrument proven twice: one bit flipped in the recorded hash exits
  **1** with `MISMATCH`, and the *pre-TD9* core re-records TD8's own hash
  exactly, which is what makes the played critter's death below an attribution
  rather than a guess. Default paths (`ps1_played.trace.json` / `.json` /
  `.png`), plus `td9_income.png`. The capture reads a stepped dark-red soil
  section with roughly twenty-five bodies strung along the surface — orange-red
  capsules among green hemispheres, two or three lavender ones, the minimap top
  right — and a vitals panel that says **`state dead`**. That is new, it is
  TD9's doing, and it has a Finding of its own. `td9_income.png` and
  `ps1_played.png` are byte-identical, which is worth saying because it was
  checked: three consecutive replays of one trace produce the same PNG, so the
  capture is as deterministic as the hash.

  **Hazard, found the hard way and recorded rather than fixed:** a live headed
  session writes the *same* default paths the fixture uses, so one interactive
  run of `cargo run -p mesocosm-genet` silently replaces
  `ps1_played.{trace.json,json,png}` with its own. It happened mid-round — the
  receipt read back `"mode": "played"`, 5,003 frames, 1,819 steps — and the
  fixture had to be re-recorded and re-verified afterwards. The verification
  order that survives it is: record, replay, and check `"mode": "replay"` and
  `trace_len` in `ps1_played.json` **last**. Whether a played session should
  default to a different filename is a genet question and is not ruled here.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo test -p mesocosm-core --test matter --release`: 5 passed, 26 s.
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds, with the same one pre-existing `dead_code` warning TD6, TD7
  and TD8 recorded — it has to be invoked from inside the `paredros` checkout,
  since `--features` for a package outside this workspace is refused here.
  Two new tests state the two rules —
  `the_bite_reads_the_same_build_the_rent_does` and
  `producers_creep_and_unlimbed_consumers_do_not`, the second seeded at 2
  because that is the founding that has both an unlimbed consumer and a producer
  to tell apart. One test was retuned, one line of why:
  `allometric_rates_cross_three_orders_without_flat_steps` asks for the sessile
  bite explicitly, the way TD7 made it ask for the sessile rent. The probe went
  past the ceiling and split into `td8_attribution/{main,report}.rs` (411 + 287)
  — the same split-before-adding move `ecology/tests` and `axis` made. It keeps
  its TD8 name because it is the same probe extended rather than a second one;
  receipts are per round, so `td8_attribution.json` stays as TD8 wrote it.

- **2026-08-29: TD8 landed — all three rulings are in, each moved
  its own target, and `breathes` is still out of reach for a reason this round
  measured rather than guessed.** Conservation is milligram-exact, the control
  still collapses, escapees are zero, and the verdict tally is unmoved at **0
  breathes / 10 thins / 0 boil / 0 collapse**. What moved is underneath it, and
  the round's discipline was to show each ruling moving the thing it was aimed
  at rather than only moving the total. New receipts: `td8_chain.json` (the
  instrument) and `td8_attribution.json` (the probe,
  `crates/mesocosm-core/examples/td8_attribution.rs`, which reports all three
  targets from one run and can be run with any ruling switched off).

  **1. Reproduction gates on adult mass, at a third of the ceiling.**
  `Organism::can_reproduce`'s mass clause became
  `biomass_mg >= ecology::breeding_mass_mg(mass_ceiling_mg())`, a share of TD6's
  own derived adult mass. The gestation clock and the 80 mg
  `STARVATION_MG * OFFSPRING_COST` floor are both kept, and they do not
  conflict: a small plan's third-of-ceiling falls under 80 mg and the floor is
  then the binding one, which is the guarantee it was always making — a brood
  costing a quarter of the parent is born above starvation rather than into it.

  **The fraction is `BREEDING_SHARE_PCT = 33`, picked against the instrument.**
  Three shares, ten seeds each, everything else identical:

  | mass gate | breathes | thins | boil | collapse | seeds holding a second kingdom |
  | --- | ---: | ---: | ---: | ---: | --- |
  | 80 mg absolute (before) | 0 | 10 | 0 | 0 | **3** (2, 4, 7) |
  | 25% of ceiling | 0 | 10 | 0 | 0 | **4** (1, 2, 3, 5) |
  | **33% of ceiling** | 0 | 10 | 0 | 0 | **5** (1, 2, 4, 5, 7) |
  | 50% of ceiling | 0 | 10 | 0 | 0 | **3** (2, 5, 7) |

  No share reaches `breathes`, so the pick is made on the ruled done-condition
  that *was* reachable — founded kingdoms at the horizon — and a third of adult
  mass keeps a second kingdom alive in half the seeds against three in ten
  before. 25% under-throttles (3,620 producers alive at 3,000 ticks against
  1,692 at a third) and 50% over-throttles the two kingdoms it was meant to
  help.

  **Its target moved.** Leave-one-out at 3,000 ticks, seeds 1 / 2 / 5: producers
  live at **22 / 91 / 27%** of their own adult mass, against **8 / 100 / 14%**
  with the gate off and **16 / 51 / 18%** before the round; producer births
  **1,929 / 156 / 2,229** against **4,402 / 117 / 3,767** off and
  **3,598 / 707 / 3,503** before. Bodies grow up now, and the founding boom is
  throttled at its source instead of by starving afterwards. The honest other
  half: consumer and decomposer *birth counts fell with everyone else's*
  (consumers 127 → 33 in seed 1). The gate raised how grown a body is, not how
  often it recruits, so TD7's recruitment shortfall is **not** closed by it.

  **2. Corpses persist longer, and the lever is duration.** The `Stage::Carrion`
  arm returned a milligram every tick; it returns one every
  `CARRION_DECAY_TICKS = 4` now, phased by the corpse's own `age` so the
  enclosure does not decay in lockstep and a replay lands the same way.
  `decay_rate_for_mass` — what a scavenger draws per bite — is **untouched**,
  because the yield lever was measured out in TD6 and again in TD7; this is the
  only number in that arm that is a rate rather than a share. Conservation is
  unchanged: the milligram still goes into the column the body lies on, just
  less often.

  **Its target moved, and it is the round's clearest movement.** With this
  ruling off and the other two on, seeds 1 / 2 / 5: standing carrion
  **193 / 24 / 166**, scavenged **112,399 / 28,323 / 204,525 mg**, decomposers
  alive at 3,000 ticks **3 / 0 / 9**. With it on: **269 / 44 / 176**,
  **127,721 / 45,685 / 587,503 mg**, **8 / 0 / 36**. Against the round's start
  (0 / 0 / 0 decomposers alive, 6,512 / 23,411 / 19,197 mg scavenged) the
  kingdom that "essentially never breeds" is now the one that reaches the
  horizon: **decomposers survive to tick 10,000 in seeds 1, 4 and 5**, where
  they survived in one seed before.

  **3. No actuator, no travel — and it reaches further than the floor did.**
  `dispersal_for` read `locomotion()`, which floors the actuator span at one for
  the drive selector's arithmetic; it reads `actuator_span()` now, so a body
  with nothing contractile gets a budget of zero. **That alone was not the
  ruling**: the hungry wander and the far tier's graph hop never asked for a
  budget, and a measurement showed an unlimbed body still walking with the floor
  gone. `movement::disperse` answers the question once now, on every path — a
  body whose plan carries no actuator stays where it is. It still reads its
  drives and its memory; what it cannot do is act on them by going somewhere.

  **Its target moved, and the reach is the finding.** Seed 2's consumer species
  is the free-lunch draw (233 of that founding's 307 consumers and decomposers
  have no actuator). Its `Moved` events went from **95,738** before the round —
  100,989 with only this ruling off — to **0**. It did not vanish: standing
  where it was founded it still grazed 413,122 mg of what grew beside it, and it
  still ends extinct, which is the ruled outcome rather than a regression (with
  the ruling off, seed 2's probe window ends with 56 living consumers against
  7). **The larger consequence was not in the ruling's text and is measured
  here: producers were walking.** A producer is unlimbed by construction, so the
  same rule makes the stand sessile — **129,534 / 2,615 / 292,361 producer
  `Moved` events in seeds 1 / 2 / 5 before, 0 / 0 / 0 after.** That is what a
  plant should do, and TD7 already priced them as sessile; the instrument says
  the stand is better for it (end biomass 811,061 → 1,263,296 mg in seed 1). But
  it is a rule about producers arrived at through a ruling about consumers, so
  it is flagged rather than assumed.

  **Verdicts.** Baseline was reproduced at HEAD before anything was touched and
  matched S1's table seed for seed. Receipt: `td8_chain.json`.

  | seed | verdict | start | end | P/C/D start | P/C/D end | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 917 | 1,691 | 610/230/77 | 1,637/0/**54** | 1,263,296 mg | 331,362 | 2,206,906 |
  | 2 | thins | 917 | 1,081 | 610/230/77 | 1,036/**45**/0 | 570,115 mg | 925,939 | 2,220,206 |
  | 3 | thins | 917 | 1,472 | 610/230/77 | 1,472/0/0 | 881,929 mg | 85,567 | 2,217,028 |
  | 4 | thins | 917 | 1,519 | 610/230/77 | 1,492/0/**27** | 917,953 mg | 134,738 | 2,202,302 |
  | 5 | thins | 917 | 1,689 | 610/230/77 | 1,636/0/**53** | 1,048,697 mg | 604,541 | 2,214,890 |
  | 6 | thins | 917 | 1,386 | 610/230/77 | 1,386/0/0 | 635,267 mg | 525,179 | 2,214,018 |
  | 7 | thins | 917 | 999 | 610/230/77 | 947/**52**/0 | 816,555 mg | 139,676 | 2,222,946 |
  | 8 | thins | 917 | 1,193 | 610/230/77 | 1,193/0/0 | 343,212 mg | 1,405,936 | 2,212,850 |
  | 9 | thins | 917 | 1,043 | 610/230/77 | 1,043/0/0 | 304,448 mg | 1,457,896 | 2,209,340 |
  | 10 | thins | 917 | 1,151 | 610/230/77 | 1,151/0/0 | 329,791 mg | 1,419,565 | 2,204,656 |

  Control all collapse, max escapees 0, `total_matter_mg` flat across every
  sample of every run and identical seed-for-seed to the pre-round baseline.
  Against that baseline: **five seeds hold a second kingdom to the horizon
  against three**, end biomass rose in nine of ten seeds (811,061 → 1,263,296 mg
  in seed 1), and the soil is drawn much further down (798,188 → 331,362 mg) —
  the enclosure is being eaten rather than sitting in the floor. The tenth is
  seed 2, down 705,702 → 570,115 mg, and that is the third ruling doing its job:
  seed 2 is the free-lunch founding, and its consumer species no longer grazes
  the whole enclosure at a plant's price.

  **No `rates.rs` sweep was run**, deliberately: TD5, TD6 and TD7 each ran one
  and each recorded that nothing in it reached `breathes`. What this round has
  to say about the constants is in the Findings below instead, and it is a
  structural claim rather than a number.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`3b86d0ef9ebd7d33`** (was
  `e2f037da2b2407e0`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 56 body parts, 40 roster members (the cap), ground
  revision 2, `slab_half_height` 28. Instrument proven: one bit flipped in the
  recorded hash exits **1** with `MISMATCH`, the unflipped trace exits 0.
  Default paths (`ps1_played.trace.json` / `.json` / `.png`), plus
  `td8_framed.png` at the newly ruled default framing (see the scale plan).

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue). `cargo
  clippy --workspace --all-targets -- -D warnings`: clean. `cargo fmt --all
  --check`: clean. `cargo check -p paredros-room --features r1-proof`: builds,
  with the same one pre-existing `dead_code` warning TD6 and TD7 recorded.
  `ecology/tests.rs` was already 101 lines over the ceiling and the carrion test
  would have taken it further, so it split into
  `ecology/tests/{mod,carrion,signals}.rs` (597 + 97 + 95) — the same
  split-before-adding move `axis.rs` made in TD7. Three new tests state the
  three rules. Six were retuned, one line of why on each, and every one is the
  new rule showing up in a fixture built before it existed:
  `an_exhausted_body_disperses_through_the_place_graph` and the "slow" half of
  `drive_selection_makes_fast_and_slow_bodies_different` now carry limbs,
  because a fixture made of bulk cannot travel any more;
  `the_same_pursuit_selects_between_bodies_at_a_generated_threshold` gives both
  hunters a vertical limb so it still measures footprint against a one-voxel gap
  rather than legs against no legs;
  `producers_alone_spread_until_something_eats_them` gives its pasture one plant
  per crowding cell, because a stand shading itself down to the income floor
  never reaches breeding mass now (which is the ruling working); and TD4's two
  instinct fixtures moved from seed 4,242 to 4,244, because 4,242 founds its
  *played* critter from an unlimbed recipe and every claim about who is walking
  that body would have been vacuously true.

- **2026-08-29 (later still): TD7 landed — rent is priced by how a body lives,
  and the base of the pyramid is fixed. The tiers above it are not, and the
  round measured exactly why.** All four ruled changes are in, conservation is
  still milligram-exact, and the verdict tally is unmoved at **0 breathes / 9
  thins / 0 boil / 1 collapse**. What moved is everything under that number:
  the producer stand went from a thinned 3,267-27,616 mg to 14,192-65,541 mg of
  standing biomass, and the two failing kingdoms turn out to be
  **recruitment-limited, not mortality-limited**.

  **1. Root forage** (`places/soil.rs`, 363 lines). `Soil::draw_richest_within`
  reads `columns_within` and then takes the ordinary income out of the richest
  column it found: **wide reach, one tick's draw**, never the radius' worth of
  columns at once. `FORAGE_RADIUS = 3` — the reach the per-voxel grain was ruled
  for, 49 of 1,089 columns here, where every coarser grain's r=3 already covered
  the whole world. Ties go to the lowest column index so a stand on flat ground
  forages the same way in every replay. Percolation is exactly as TD6 shipped
  it. Conservation is untouched because this is the same `draw`: the addressing
  and the transfer were kept apart for precisely this, and the forage read was
  an addition rather than a redesign, as the TD6 entry predicted.

  **2. Locomotion-derived rent, and the formula is the body plan's own.**
  `Organism::locomotion()` was a sum of each living contractile part's longest
  half-extent, floored at 1 for the drive selector. The floor is now split off
  as `Organism::actuator_span()`, which reads **0** for a body with no actuator
  — what a sessile plan honestly says — and `locomotion()` is that `.max(1)`.
  `ecology::upkeep_for_body` prices it:

  ```text
  rent = UPKEEP_BASE_MG + m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
                        / (UPKEEP_SCALE * ceiling)
  ```

  Three numbers, all already here: `biomass_mg`, `actuator_span`, and TD6's
  `mass_ceiling_mg`. The motile multiple is `span * 100 / ceiling` — **the
  actuator swing a body carries per reference segment of body it carries it
  on**. Dividing by the plan's own adult mass is what makes it scale-free: both
  halves grow with the body, so the term reads a body's *build* and not its
  size, and a long plant does not pay for being long. **No new constant**, and
  `REFERENCE_SEGMENT_MG` is `REFERENCE_MASS_MG` renamed for the job.

  The consequences are readings. A sessile body reads span 0 and the formula
  collapses to `m^0.75 / UPKEEP_SCALE` **exactly**, to the milligram — a test
  asserts it. A palette limb is half-extent `[4,1,1]`, so it swings 4 and holds
  a 64 mg ceiling against an axial segment's 100 mg, which bounds the whole
  surcharge by construction at `1 + 4*100/64` = 7.25x for a body of nothing but
  limbs. Measured at genesis across seeds 1-10, mean rent per founder:

  | kingdom | before TD7 | after TD7 |
  | --- | --- | --- |
  | producers (span 0, always) | 1.4-1.9 mg/tick | **1.4-1.9, unchanged** |
  | consumers, limbed recipe (span 16-108) | 1.5-1.8 | **2.3-6.4** |
  | decomposers, limbed recipe (span 32-64) | 1.4-1.7 | **2.6-4.6** |
  | either, unlimbed recipe (span 0) | 1.5-1.8 | **1.2-1.8, unchanged** |

  Before this round every kingdom paid the same 1.4-1.9 mg/tick for the same
  mass. That is the trophic asymmetry TD7 was for, and it is anatomy.

  **3. Pyramid founding** (`world/genesis.rs`). `pyramid(count)` composes the
  tiers exactly rather than drawing them: `PRODUCER_SHARE` 2/3, `CONSUMER_SHARE`
  1/4, the rest decomposers, then a Fisher-Yates shuffle over the seeded stream
  so the pyramid is the world's shape rather than a distribution it usually
  lands near. At the shipping 60 non-played founders that is **40 / 15 / 5**,
  and 40/16/5 with the played consumer — in every seed, asserted. TD2b's kingdom
  floor is kept: a tier that rounds to nothing takes one founder from the widest
  rather than leaving a rung out, tested over foundings of 3 to 12. The species
  draw inverted with it — the tier a founder drew now names its species, since a
  species has one inherited silhouette.

  **4. Producers grow bigger** (`axis.rs`). Only the unlimbed branch of
  `axis::seed` changed, and only its two existing draws: stretches
  `1 + below(2)` -> `1 + below(3)`, segments per stretch `1 + below(6)` ->
  `4 + below(8)`. No new part type, no new template, no palette change — the
  same axial rules asked for a larger stand. Measured at genesis, seeds 1-10:

  | | parts per body | derived ceiling | mean ceiling per seed |
  | --- | --- | --- | --- |
  | producers before | 2-16 | 200-1,600 mg | 477-1,069 mg |
  | producers after | **5-35** | **500-3,500 mg** | **747-3,175 mg** |
  | consumers (unchanged) | 3-48 | 300-3,576 mg | 1,392-2,799 mg |

  TD6's inverted pyramid — a grazer able to outgrow its own food's adult size by
  four times — is gone from the body plans.

  **Verdicts. The tally is unmoved and the world underneath it is not.**
  Baseline reproduced once before touching anything and it matched TD6's table
  milligram for milligram. Receipt: `td7_priced.json` (mechanics only; no
  `rates.rs` constant was changed, so there is no `td7_priced_tuned.json`).

  | seed | verdict | start | end | P/C/D start | P/C/D end | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 61 | 117 | 40/16/5 | 117/0/0 | 63,473 mg | 33,305 | 148,390 |
  | 2 | collapse | 61 | 0 | 40/16/5 | 0/0/0 | 0 mg | 141,677 | 144,646 |
  | 3 | thins | 61 | 89 | 40/16/5 | 89/0/0 | 58,974 mg | 52,557 | 148,386 |
  | 4 | thins | 61 | 120 | 40/16/5 | 120/0/0 | 53,932 mg | 43,626 | 143,412 |
  | 5 | thins | 61 | 119 | 40/16/5 | 119/0/0 | 65,541 mg | 35,990 | 147,150 |
  | 6 | thins | 61 | 114 | 40/16/5 | 114/0/0 | 24,234 mg | 100,840 | 147,810 |
  | 7 | thins | 61 | 155 | 40/16/5 | 155/0/0 | 43,670 mg | 62,493 | 144,338 |
  | 8 | thins | 61 | 83 | 40/16/5 | 83/0/0 | 26,450 mg | 97,742 | 147,702 |
  | 9 | thins | 61 | 76 | 40/16/5 | 76/0/0 | 14,192 mg | 121,018 | 149,070 |
  | 10 | thins | 61 | 91 | 40/16/5 | 91/0/0 | 32,528 mg | 88,288 | 145,330 |

  Control all collapse, max escapees 0, `total_matter_mg` identical across every
  sample of every run. Against TD6: end biomass 3,267-27,616 -> 14,192-65,541
  mg, and the soil is drawn down to 33,305-121,018 mg from a flat
  108,900 — the stand is now eating the enclosure the way a stand should. The
  one collapse moved from seed 10 to seed 2 (whose consumer species draws an
  unlimbed recipe, so it grazes at a plant's rent).

  **The bounded `rates.rs` pass was run and is recorded here rather than
  shipped** — the same call TD5 and TD6 made, for the same reason: nothing
  reached `breathes`. Six configurations, ten seeds each:

  | GRAZES / DECAYS / COMFORT / HUNGRY | breathes | thins | boil | collapse |
  | --- | ---: | ---: | ---: | ---: |
  | 3 / 4 / 1 / 8 (shipped) | 0 | 9 | 0 | 1 |
  | 6 / 4 / 1 / 8 | 0 | 9 | 0 | 1 |
  | 6 / 4 / 2 / 8 | 0 | 9 | 0 | 1 |
  | 6 / 8 / 2 / 8 | 0 | 8 | 0 | 2 |
  | 6 / 8 / 2 / 24 | 0 | 8 | 0 | 2 |
  | 9 / 16 / 2 / 24 | 0 | 9 | 0 | 1 |
  | 12 / 16 / 3 / 24 | 0 | 9 | 0 | 1 |

  Every one of them ends every seed at `P/0/0`. Raising the graze and decay
  rates and loosening crowding buys **bigger producer stands** (up to 315 at the
  horizon) and nothing else, which is itself the evidence that the binding
  constraint is not a rate.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`33ffc5b46789be9d`** (was
  `c8713ce9a82f5d6f`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 55 body parts, 18 roster members, ground revision
  2. Instrument proven: one bit flipped in the recorded hash exits 1 with
  `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` / `.png`). The
  capture reads a section that is visibly a different ecology from TD6's: a
  chain of green producer segments strung along the whole surface with salmon
  and red bodies among them, the minimap, and vitals at `energy 1364 mg` — with
  a **burn** notice this time, which is TD7's rent showing on the played body.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds (the same one pre-existing `dead_code` warning in that repo
  TD6 recorded, not this round's). `axis.rs` went past the ceiling, so its
  test module moved to `axis/tests.rs` (428 + 202) — the same split the ecology
  already uses. Six tests were retuned, one line of why on each:
  `allometric_rates_cross_three_orders_without_flat_steps` now asks for the
  sessile rent explicitly; `serialization_does_not_distinguish_the_played_critter`
  compares every field of an organism that a tick does not move rather than
  bytes, because a wider forage read lets a different hand reach a plant's
  mouthful inside one tick; `a_held_critter_still_ages_pays_rent_and_can_die`
  measures rent against what the critter ate, because forty producers mean a
  held grazer now out-earns its rent; both venom tests subtract the ecology's
  own bite from the played meal's receipt; and mesocosm-mesh's two attachment
  tests provision the critter, because TD7's rent moved the starved horizon
  further from empty than the walk to a neighbour leaves it.

- **2026-08-29 (later): TD6 landed — the matter cycle closes, growth becomes
  determinate, and the enclosure stops being able to make mass.** Conservation
  is exact and proven; the runaway is gone; the verdict tally regressed, and
  the reason is a structural finding that wants a ruling.

  **The store.** `places::soil` (`crates/mesocosm-core/src/places/soil.rs`,
  306 lines) holds one `u64` of milligrams per voxel column, row-major in z
  then x over `-extent..=extent`, sized from `ENCLOSURE` — 33x33 = 1,089
  columns, 8.7 KB. World state: a `soil` field on `World`, serialized, inside
  `state_hash`, round-tripped by a test. A `Column` is an opaque index only
  `Soil` mints, and `column_at` **clamps** rather than refusing, so nothing
  deposited can fall off the edge of the ledger. Addressing (`column_at`,
  `columns_within`) is deliberately separate from transfer (`matter_mg`,
  `draw`, `deposit`), so the ruled forage radius is a read over
  `columns_within` and then the same `draw` — a test asserts r=3 reaches 49 of
  1,089 columns here, which is the reach the granularity ruling was for. The
  foraging *behaviour* is not built, as scoped. `Soil` is not `Ground`: bricks
  are what is solid, soil is what can be eaten out of the floor, and a carve
  moves none of the second.

  **The cycle, closed at nine seams.** Producers `draw` from their own column
  and can take nothing more; rent goes into the column the body stands on
  (both halves — budget-paid and the self-consumption when the budget is
  empty); carrion decay puts its milligram in the column it is lying on; a
  death releases the reserve the body was still carrying; travel is paid in
  substance into the ground it was covered over, for NPCs and for the played
  `Intent::Move` alike; `Intent::Deposit` now enriches the column instead of
  minting a carcass; the played meal returns what the eater did not keep —
  the meal's own reserve, an odd milligram a mirrored split could not halve,
  the half of a pair that would not attach, and what a bite of venom cost;
  and **a birth is provisioned rather than conjured** — until now a parent
  paid `cost` once in body mass and the child was handed a body worth `cost`
  *and* a budget worth `cost`, so every birth in the enclosure minted matter.
  The child's opening budget now comes out of the parent's own reserve.

  Genesis seeds `SOIL_SEED_MG_PER_COLUMN = 100` per column — the ecology's own
  `REFERENCE_MASS_MG`, so the rule reads *one reference body's worth of
  substance under every voxel column*. At the shipping enclosure that is
  108,900 mg, about three times what a 61-founder cohort carries in bodies and
  reserves. Sized from `ENCLOSURE`, never hardcoded.

  **The tick had to be restructured, and this is why.** Crediting an eater its
  full mouthful and reconciling afterwards conjures matter, because two grazers
  reach the same small producer and the second finds less than it bit for —
  and there is no reconciliation that is always payable (the eater may have
  died and released its reserve before the drain pass ran). The conservation
  test caught it at **tick 28 of seed 4, 1 mg**. So the tick is now three
  passes: rent and the bite each body *reaches for*; the meals settled one at
  a time with both bodies in hand, crediting exactly what came out of the prey;
  then dispersal, maturity, decay and death. The birth pass moved to
  `ecology/breeding.rs` at the 600-line ceiling, following the crate's own
  sibling-file precedent.

  **Determinate growth, derived, no new number.** `Organism::mass_ceiling_mg`
  sums a per-part ceiling over the living body, and a part's ceiling is its own
  voxel volume priced so that a reference segment holds `REFERENCE_MASS_MG` —
  both numbers were already here. `gain_mass` caps at it and returns what would
  not fit; `earn` routes the overflow to the budget (capped the same way) and
  hands the remainder back to the world; a feeding body clamps its bite to
  `intake_room_mg`, so **a full body does not feed**. Measured at genesis: mean
  producer ceiling 477-870 mg against a mean drawn mass of ~290, mean consumer
  ceiling 2,044-2,456, the played critter 1,568-2,522; 0-3 founders of 61 open
  at or over their ceiling. The way past the ceiling is the game's own verb —
  eating adds *parts*, and every part brings its ceiling with it.

  **Crowding is not redundant, and the round measured why.** The soil bounds a
  stand's **mass**; nothing in a closed matter budget bounds its **number**. A
  run with crowding removed answered a finite enclosure by subdividing: 620
  producers and still climbing at the horizon at every soil seed swept (100,
  300, 500, 600, 1,000 mg/column) and every minimum-body size swept
  (`STARVATION_MG` 20/60/120). `CROWD_CELL`/`CROWD_COMFORT` are therefore kept
  at TD2c's 8/1, unchanged, and the rent floor under them is kept too — but it
  is no longer a hand-out: it is a *request*, and the column answers it or does
  not. Density is the job crowding still does; it is simply no longer the only
  regulator.

  **Conservation, and the proof it can fail.** `tests/matter.rs`: four seeds x
  4,000 idle ticks, checked **milligram-exact every tick** (births, deaths,
  grazing, predation, scavenging, decay, dispersal and the founding cohort
  dying of old age all fall inside that window), plus a played-verb run
  covering deposit, movement, carve and metabolize. **Zero exceptions** — no
  sink, no source, no tolerance. Two things are outside the account because
  they are not matter: light (the ruled open input, which powers uptake and
  never enters the ledger) and ground bricks. The same `conserved()` the long
  runs use is handed a conjured milligram and a leaked one and must report
  both; the conjured control replays the pre-TD6 producer income exactly.
  Flipping that control's assertion to prove it really trips:

  ```text
  thread 'the_check_catches_income_conjured_the_way_it_used_to_be' panicked at
  crates\mesocosm-core\tests\matter.rs:146:10:
  PROOF RUN: the deliberately-broken control must trip the check: "matter is not
  conserved after conjuring a producer's old income: 145549 mg against 145538 mg
  at genesis (11 mg conjured); soil 109002 mg, 61 bodies holding 36547 mg"
  ```

  The instrument carries it too: every seed's receipt records `total_matter_mg`
  per sample, and it is identical across every sample of every run.

  **Verdicts, reported honestly: this is a regression.** Baseline was 2
  breathes / 7 thins / 0 boil / 1 collapse (TD5b). TD6 reads **0 breathes / 9
  thins / 0 boil / 1 collapse**, control all collapse, max escapees 0. Receipt:
  `td6_matter.json`.

  | seed | verdict | start | end | P/C/D start | P/C/D end | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 61 | 36 | 22/25/14 | 36/0/0 | 3,267 mg | 139,784 | 145,538 |
  | 2 | thins | 61 | 60 | 20/21/20 | 60/0/0 | 27,616 mg | 96,778 | 146,602 |
  | 3 | thins | 61 | 40 | 17/27/17 | 40/0/0 | 7,400 mg | 130,604 | 145,180 |
  | 4 | thins | 61 | 83 | 17/21/23 | 83/0/0 | 9,522 mg | 125,069 | 141,652 |
  | 5 | thins | 61 | 100 | 16/25/20 | 100/0/0 | 9,106 mg | 129,782 | 147,248 |
  | 6 | thins | 61 | 69 | 16/23/22 | 69/0/0 | 7,346 mg | 133,943 | 146,890 |
  | 7 | thins | 61 | 78 | 22/20/19 | 78/0/0 | 8,425 mg | 131,345 | 147,070 |
  | 8 | thins | 61 | 56 | 24/19/18 | 56/0/0 | 7,720 mg | 132,195 | 146,032 |
  | 9 | thins | 61 | 64 | 24/20/17 | 64/0/0 | 21,052 mg | 109,033 | 145,148 |
  | 10 | collapse | 61 | 0 | 23/20/18 | 0/0/0 | 0 mg | 147,375 | 147,904 |

  **The runaway is gone and the numbers say so.** TD5's producer stands reached
  ~4 x 10^10 mg on ~150 bodies. End biomass is now **3,267-27,616 mg**, six
  orders of magnitude down, and it is bounded by construction rather than by
  tuning: the rest of the world's matter is in the ground, where it can be
  pointed at.

  **No `rates.rs` constant was changed and no `td6_matter_tuned.json` was
  written** — the same call TD5 made, for the same reason. The bounded pass was
  run and is recorded here rather than shipped: soil seed 100/300/500/600/1,000
  mg per column, percolation divisor 8/16/32/64, `CROWD_COMFORT` 1/2/3,
  `STARVATION_MG` 20/60/120. None reached `breathes`. Below ~300 mg/column
  without percolation everything collapses; at 300-1,000 producers boil in
  *count* (610-825 bodies, still rising) while consumers and decomposers are
  already dead; `CROWD_COMFORT` 3 buys bigger stands (237-299 producers) and
  one seed each keeping consumers or decomposers, but no seed keeping both.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`c8713ce9a82f5d6f`** (was
  `e8ba7206ac96834f`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 61 body parts, 12 roster members, ground revision
  2. Instrument proven: one bit flipped in the recorded hash exits 1 with
  `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` / `.png`). The
  capture reads the section with the played critter's wide-armed silhouette at
  centre, two green roster bodies to its left and three lavender ones to its
  right, the minimap, and vitals at `energy 615 mg` with no burn notice.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds (one pre-existing `dead_code` warning in that repo, not
  this round's). Four tests were retuned with a one-line why on each:
  `ecology::tests`'s fixture half-extent went `[1,1,1]` to `[5,5,5]` (27 voxels
  carrying 300 mg was invisible until a body plan had an adult mass);
  `a_contractile_consumer_can_take_live_consumer_prey` widened its long-and-thin
  shapes to `[8,2,2]`/`[4,4,4]` so a 300 mg body sits under its own ceiling and
  can still bite; and `deposit_returns_matter_to_the_enclosure` now asserts the
  ground got richer and no carcass was minted.

- **2026-08-29 (later): TD5b landed — founders arrive mid-life, and the
  corpse drought closes.** `genesis.rs`'s age draw was `rng.below(200)`
  regardless of the founder's own mass; it is now
  `rng.below(lifespan_for_mass(mass))` — proportional to the individual's
  own life-history rate, uniform across its **whole** life. Mean age lands at
  the founder's own midpoint (mid-life, the title's word, not a knob picked
  to sound right), and the draw's near-death tail seeds carrion from the
  first ticks without any founder's age exceeding its own lifespan. The
  played critter (organism 0) keeps `age: 0` — its own line, separate from
  the loop, untouched — because the player's life should start near its
  beginning, not drawn from the same whole-life distribution as the ecology
  around it. One-line why lives on the draw itself; genesis is otherwise
  unchanged, `since_offspring`'s own stagger included.

  **The carrion window (probe, seeds 1 and 7 — TD5's own probe seeds,
  extended to 2,000 ticks, sampled every 10).** Before: the [20,1400] span
  held *some* corpse on 360/1381 and 316/1381 sampled ticks respectively,
  but every one was a trickle — 1 to ~30 mg, a starvation death decaying
  at 1 mg/tick before the next one replaced it — so TD5's "no real carrion"
  reading holds even though the window was not literally always at zero.
  Real accumulation (hundreds of thousands of mg, from old-age deaths)
  only began at tick 1,267-1,330. After: seed 1 holds a corpse on
  **1,381/1,381** sampled ticks in the window — full coverage — at
  hundreds to low thousands of mg from tick 30 on; seed 7 holds one on
  1,092/1,381, with the last empty tick at 409 and the window continuously
  occupied after. The drought's far edge is gone; its near edge (the first
  ~20-30 ticks, before any founder can plausibly have died) is structural
  and untouched by this change.

  **Verdicts.** Baseline 0 breathes / 9 thins / 0 boil / 1 collapse (of 10,
  reproduced before touching genesis, matching TD5's own receipt exactly)
  → **2 breathes / 7 thins / 0 boil / 1 collapse**. Decomposers alive at the
  10,000-tick horizon: 0 of 10 seeds before, **3 of 10 after** (seed 3: 0 →
  37; seed 7: 0 → 14; seed 9: 0 → 31). Seeds 3 and 7 convert fully —
  founded P/C/D end at 90/3/37 and 54/70/14, every kingdom the world
  started with still standing. Collapse control still all-collapse (the
  control founds no extra founders, so it never touches the changed draw);
  max escapees still 0 across every sample. Receipt: `td5b_midlife.json`,
  both batches, same schema as every earlier round's.

  **Reported honestly: this is not an unmixed win.** Total end-population
  fell in most seeds that were already thinning (seed 1: 364 → 143; seed 4:
  369 → 195; seed 6: 125 → 48; seed 8: 308 → 90; seed 10: 168 → 83) — the
  wider stagger's near-death tail is real attrition, not just useful
  corpses, so a founding cohort that arrives partway through dying trades
  some peak population for the carrion that population's death provides.
  Seed 9 is the sharpest trade: consumers held 35 at the horizon before and
  **0 after** — the predicted risk (an early die-off wave thinning live
  prey) landed in this one seed, even as its decomposers went 0 → 31. Seed
  5's consumers fell 2 → 0, already marginal either side. TD5's own
  majority-breathing bar is still unmet (2 of 10, not a majority) — TD5b
  closes the structural finding TD5 could not reach from `rates.rs`, it
  does not finish TD5's own done-condition.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`e8ba7206ac96834f`**
  (was `02b072a5cfe7b10b`), `--replay` headed landing it exactly, exit 0, 30
  frames on the RTX 4060 (Vulkan) — 61 body parts, 14 roster members, ground
  revision 2. Instrument re-proven: one bit flipped in the recorded hash
  exits 1 with `MISMATCH`. Written to the default paths (`ps1_played.trace.json`
  / `.json` / `.png`).

  **Tests.** `genesis.rs` gained one test, `ages_are_staggered_across_the_founders_own_lifespan`
  (max founder age exceeds the old flat 200 cap; the played critter stays
  `age: 0`), with a one-line why on both the test and the draw it checks.
  The two existing genesis tests (`every_seed_founds_all_three_kingdoms`,
  `since_offspring_is_staggered_like_age`) needed no change — neither reads
  age's distribution — and pass unmodified. `cargo test --workspace`: green,
  with one environment residue below. `cargo clippy --workspace --all-targets
  -- -D warnings`: clean. `cargo fmt --check`: clean.

  **No constant in `rates.rs` was touched.** The corpse drought was
  genuinely a genesis question, as TD5's finding said; nothing here argues
  for a retune, though seed 9's consumer trade-off is worth watching if a
  future round touches predation pressure.

  Two residues. **`mesocosm-lens`'s GPU-adapter tests deadlock under the
  default parallel test harness in this environment** — reproduced on the
  unmodified crate (nothing this round touched it), confirmed by isolating:
  `cargo test -p mesocosm-lens` hangs indefinitely with several
  `tracer_tests` cases stuck past 60 seconds regardless of thread count
  contention, but `cargo test -p mesocosm-lens -- --test-threads=1` passes
  all 38 clean in 14 seconds. Pre-existing, out of this round's scope
  (rendering, not genesis), and not touched; `cargo test --workspace` here
  means that plus the rest of the workspace at default threading, both
  green. Second, the wider stagger widens the founding population's own
  variance — a seed's early die-off is now part of its founder draw the way
  its kingdom composition already was, so a future retune reading "seed 9
  collapsed" should check whether that seed's draw changed before assuming
  a constant did.

- **2026-08-29 (later): TD5's mechanics landed; its done-condition did not,
  and the reason is structural.** The economy is one rule now, and the
  instrument says so without saying `breathes`.

  **The routing.** `ecology::step` gained one private `earn(organism, mg)`,
  and all four NPC income paths call it instead of `gain_mass`: producer
  fixing (the crowd-divided, rent-floored share), grazing and predation (the
  meal's biomass, one shared call), and decay. Inside it is TD4's question
  verbatim — `budget_below(STARVED_UPKEEP_TICKS)` credits `energy_mg`,
  otherwise the body grows as before. The played critter's path in
  `world::act` is untouched and unchanged in behavior; it was already this
  rule, which is the point. `dispersal_for`'s extra step moved off literal
  `energy_mg == 0` onto `is_hungry`, and `HUNGRY_UPKEEP_TICKS` plus the
  predicate moved from `movement` into `rates` so the tick's one hunger
  horizon sits with the tick's numbers. **Tick order is unchanged and had to
  be**: rent is paid before income, so `earn` reads this tick's post-rent
  reserve — a body is asked whether it is starving *after* the day has cost
  it something, which is the only reading that means anything.

  **Verdicts (mechanics only, no constant touched).** 0 breathes / 9 thins /
  0 boil / 1 collapse, control all collapse, escapees 0 — the TD2d tally to
  the seed. The tally is unmoved; underneath it is not. Consumers alive at
  the horizon went from 3 seeds to 5, and far larger where they survive
  (seed 7: 39 → 108, seed 8: 35 → 42, seed 9: 12 → 35); births and deaths
  roughly doubled across the board (seed 1: 205/138 → 899/542), which is a
  chain turning over rather than a stand sitting still. Seed 2's lone
  collapse is the same founder draw that survived three constant regimes.
  Receipt: `td5_economy.json`. **Decomposers are still zero at the horizon
  in every seed**, so `thins` is honest and `breathes` is unreached.

  **The probe (seeds 1, 7, 9).** Decomposers do bank now — 1,320-2,908
  organism-ticks holding a reserve — and mean life rose where it could
  (seed 7: 215 → 257 ticks). But the last decomposer in every seed dies by
  tick 365-430, and the probe's carrion curve says why: **the enclosure holds
  no corpses at all between roughly tick 20 and tick 1,400.** Founders are
  age-staggered over `rng.below(200)` against a 2,000-3,000-tick lifespan, so
  nothing dies of old age until ~1,800; early starvation corpses are ≤ 20 mg
  (`STARVATION_MG`) and decay at 1 mg/tick, so they are gone in twenty ticks.
  After 1,400 carrion is abundant — 20+ bodies, tens of millions of mg — and
  there is no decomposer left to eat it. That is locked matter, and it is a
  *timing* failure, not a yield or a search one.

  **The bounded constants pass found nothing worth shipping, and that is the
  finding.** `UPKEEP_SCALE` 62 → 186 (rent to a third, stretching the birth
  endowment) moved last-decomposer-death 365/385/430 → 487/705/646 and made
  the producer stand's biomass ~5x worse; `STARVATION_MG` 20 → 100 (bigger
  corpses from starvation deaths) put real bodies in the early enclosure and
  moved the deaths to 656/534/719. Neither comes within 1,000 ticks of the
  drought's far edge, because no rates.rs constant sets when the founders
  die. Both were reverted; **no constant changed and no
  `td5_economy_tuned.json` was written**, so the mechanics-only picture is
  the whole picture.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`02b072a5cfe7b10b`**
  (was `18615c6b8309f821`), `--replay` headed landing it exactly, exit 0,
  30 frames on the RTX 4060 (Vulkan) — 61 body parts, 17 roster members,
  ground revision 2. Instrument proven: one bit flipped in the recorded hash
  exits 1 with `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` /
  `.png`). The capture reads roster silhouettes across the section, the
  minimap, and vitals at `energy 586 mg` with no burn notice — the last meal
  was more than the 25-tick notice window before the final frame.

  One test was retired by the change and says so: `instinct.rs`'s
  `walking_away_gives_the_body_back_to_the_ecology` set the played critter's
  budget to zero and watched it wander, but an NPC now eats its way out of
  hunger on the first tick and correctly stands still, so the wander claim
  moved to a `stranded()` fixture — empty budget, empty enclosure.

- **2026-08-29 (later): TD4 landed, with the tempo and fresh fixtures.**
  Both core rules, the canonical 10 t/s host, and a re-recorded receipt set.

  **The hand.** `World` counts consecutive `Intent::Idle` applications in a
  plain hashed field; any other intent resets it. Below
  `INSTINCT_IDLE_TICKS = 30` (three seconds at the tempo) `World::held()`
  names the controlled critter and the ecology skips **only its dispersal** —
  it still ages, pays rent, feeds, breeds and dies. Past the threshold `held`
  is `None` and its own drives resume, with control itself never moving: the
  next keypress lands mid-stride, nothing is handed over or reclaimed. The
  count is a function of the trace, so every host and every replay reaches the
  same answer at every frame rate; a host-side wall clock would have made the
  ecology depend on how fast the machine drew.

  **The meal.** `Intent::Metabolize` dropped its `Route` and carries only
  `Placement` — which was always the other question (*where a kept part goes*,
  a growth policy an editor needs) rather than the one D5 asked. Burn-or-build
  is now the body's, decided in `World::apply` at
  `STARVED_UPKEEP_TICKS = 100`: ten seconds of standing still, about a third of
  a 1,000 mg starter's 333-tick budget. Wide on purpose — the ecology's own
  hunger horizon is eight ticks, the point a body starts eating itself, and
  routing there would have meant every meal grew you until the tick before you
  died. Both thresholds ask the same question through one predicate,
  `Organism::budget_below(ticks)`: hunger is never a milligram count, because
  a large body burns through the same number faster. `Route` survives as what
  the body concluded. The F key is gone; E/Space is the whole verb.

  **Tempo.** Host `ticks_per_second` 60 → 10. Two presentation constants were
  ticks-at-60 and were retimed with it: the vitals notice window (150 → 25,
  the same 2.5 s) and the minimap backdrop cadence (10 → 2, restoring both the
  wall cadence and the per-frame cost). Nothing else in the host counts ticks.

  **Receipts.** Demo re-recorded at the new world and tempo: 120 intents, hash
  **`18615c6b8309f821`**, `--replay` headed landing it exactly, exit 0, over 30
  frames on the RTX 4060 (Vulkan) — 61 body parts, 16 roster members, ground
  revision 2. Instrument proven once: a trace with one bit flipped in its
  recorded hash exits 1 with `MISMATCH`. Written to the default paths
  (`ps1_played.trace.json` / `.json` / `.png`), superseding the 2026-08-28
  fixtures. The capture shows the section with a dozen-plus bodies in it, the
  minimap, and the vitals panel reading `energy 793 mg` with a live **burned**
  notice — the meal's destination, said on screen, which is the whole of the
  feedback now that the player is not the one choosing it. The trace's own
  verbs now include a forty-tick hands-off stretch, so the fixture covers both
  halves of TD4.

  **The idle run, by hand (109 steps, no keys touched).** Every recorded
  intent was `Idle`. The played critter held station for exactly the
  documented window and **first moved on step 29** — the apply at which the
  count reaches 30 — then wandered [0,21,0] → [3,21,1] on its own drives. By
  the end 27 of the surviving founders had moved, the population had run
  61 → 41, the section roster was 24, and the budget had fallen 941 → 488 mg
  paying rent with nobody earning it. The trace replays to its hash exactly.

  Three residues. **A held critter cannot flee, and a fat one is the best meal
  in the enclosure**: the 200-step demo died every time around tick 134 with
  up to six predators feeding on it at once, which is why `DEMO_STEPS` is 120.
  That is the movement-economy seam from the playtest findings, flipped —
  before TD4 the ecology fled for you. Second, growth raises rent, so a critter
  that grows fast crosses `STARVED_UPKEEP_TICKS` on its own success and its
  next meals burn; whether that negative feedback is the design or an accident
  of the m^0.75 scaling is Mark's (it touches the "individual mass has no fixed
  point" finding below). Third, `population_instrument.rs` drives worlds with
  `Intent::Idle`, so its controlled critter is now held for the first 30 of
  ~10,000 ticks — negligible, and the TD1-TD2d receipts were not re-run.

- **2026-08-29 (later): TD2d's scavenger sight landed.** Scavengers seek
  carrion out to sight range and bite at `DECOMPOSE_RANGE`, mirroring the
  grazer split; hunger wanders below eight upkeep-ticks of budget instead
  of at literal zero. The death-cause probe proves the mechanism:
  starvations beside in-range carrion fell to ~0. The verdict tally did
  not move (9 thins / 1 collapse) — decomposers now die traveling, or
  waiting for anything nearby to die: the binding constraint is corpse
  throughput, not search. Receipt: `td2d_scavengers.json`.
- **2026-08-29 (TD2d finding): NPC energy never refills.** `energy_mg` is
  endowed at birth and topped up only for the controlled critter (the
  metabolize gain in `world/act.rs`); every other organism's feeding
  builds biomass only, so all NPCs cross any hunger threshold early and
  live off their bodies thereafter. The NPC economy runs on biomass;
  energy is effectively a played-only ledger. Whether that is the design
  (and decomposer persistence instead wants, e.g., continuous detritus
  income or carrion-timing constants) or NPCs should earn energy like the
  player does is a Mark-level fork, recorded here unruled. Related loose
  end: `dispersal_for`'s bonus still gates on literal zero energy,
  inconsistent with the new hunger threshold.
- **2026-08-29 (later): TD2c's persistence retune landed.** The verdict
  gained `thins` — count held but a founded kingdom is gone at the
  horizon — which honestly re-read TD2b's six "breathes" as producer-only
  stands. Producer supply got headroom over grazing demand (FIXES 2 → 5,
  GRAZES 2 → 3), crowding re-sized for the walled enclosure (CELL back to
  8, COMFORT 1: TD2's doubling only ever compensated for escapees on an
  unbounded plain). Verdicts: 0 breathes / 6 thins / 4 collapse → 0
  breathes / **9 thins / 1 collapse**, zero boils throughout, consumers
  alive at the horizon in 3 seeds (10-23 individuals) where before none.
  Control still collapses; escapees still zero. Receipt:
  `td2c_persistence.json`. The lone collapse (seed 2, consumer-heavy
  draw stripping its base before the first brood) survived every variant
  under three constant regimes — likely legitimate founder variance
  rather than a tuning failure.
- **2026-08-29 (TD2c finding, proven by death-cause probe): decomposers
  cannot search.** Across probed seeds, most decomposer starvations
  happened with carrion lying in the enclosure but outside
  `DECOMPOSE_RANGE 6` — the only radius a scavenger can act on, because
  `preferred_target` caps scavenger *seeking* at bite range while grazers
  and predators seek out to `sight_range`, and `disperse` only wanders at
  exactly zero energy, so a scavenger stands still until half dead.
  `DECAYS_BASE_MG` swept to 6 with no effect — yield is not the binding
  constraint. Fix (TD2d): mirror the grazer's seek/reach split for
  carrion, and let a hungry-but-not-empty body wander. No decomposer
  survives to the horizon in any seed until then, so `breathes` stays
  unreachable by constants.
- **2026-08-29 (later): TD2b's walls and founding floor landed.** The
  enclosure edge refuses a step (wall, not cliff) for every walker — and
  the fix needed three doors closed, not one: `step_for`'s candidates,
  nest routes (burrow mouths and depth drifted up to 12 voxels past the
  bound — a half-unreachable trap once walls exist), and **birth scatter**,
  which threw offspring through the wall with no bound check and was the
  instrument's actual escapee source. Proof across every sample of every
  run: occupied span exactly 16, zero bodies outside; seed 1's end biomass
  fell 4,375x (4.4e9 → 1.0e6 mg) with crowding finally engaged. Genesis
  now founds all three kingdoms every seed (Fisher-Yates floor over the
  three non-played species) and staggers `since_offspring` like `age`.
  Verdicts: 6 breathes / 0 boil / 4 collapse; control still collapses.
  Receipt: `td2b_walls.json`.
- **2026-08-29 (TD2b finding, the next retune's target):** balanced
  founding changed the problem the constants face. In every breathing run
  consumers and decomposers now die to zero by tick 10,000 (pure producer
  stands), and two formerly-breathing seeds collapse from healthy
  ~20/20/18 starts within ~1,000 ticks: a balanced consumer cohort exerts
  predation pressure the TD2 constants — tuned against mostly-producer or
  mostly-empty draws — never saw. TD2c: retune against the new genesis,
  targeting all-kingdom persistence.
- **2026-08-29 (later): TD3's roster landed** — the section shows the
  ants. The tracer gains a second uniform (`@binding(3)`) of up to **40
  roster members at 10 capsules each** (352 B/member, 14,096 B — 86% of
  the 16 KiB downlevel-WebGL2 binding limit the lens's own GL probe holds
  it to; a storage buffer or the 64 KiB desktop limit buys ~4x if that
  reach is ever dropped). Members are silhouettes — no eyes; the
  controlled critter keeps the full single-pose path, which is additive-
  compatible and pixel-identical when the roster is empty (asserted in a
  test). Host feeds every alive organism in the slab window through the
  shared projection. Capture `td3_roster.png`: ~22 visible bodies plus
  the controlled critter (37 submitted; the rest behind terrain). Replay
  hash proven at the pre-retune commit in an isolated worktree
  (`49c7a47f`, exit 0); the live fixture is stale from TD2 by design.
  Measured occupancy that sized the cap: 26-34 of 60 genesis organisms
  in-slab; median body 8-19 living parts. Residues: cap headroom is thin
  (37 of 40 at genesis — resize when TD2b changes occupancy);
  `BodyLensProjection::project` hashes more than a pose needs
  (`project_pose` variant would trim it); `tracer_tests.rs` sits over the
  600-line ceiling from before this work.
- **2026-08-29 (later): TD2's constants landed** — ten seeds go from 0
  breathes / 7 boils to **7 breathes / 0 boils** (remaining 3 collapses are
  founder draws, see the structural findings). Lifespan base 600 → 1800 (a
  1,000mg starter lives 3,000 ticks — five minutes at 10 t/s), gestation
  4x against lifespan's 3x (the knob that decided boil vs breathe),
  producer/grazer income halved-ish, upkeep scale halved (budget 166 → 333
  ticks), crowding cells widened and comfort halved. Collapse control
  still reads collapse. Receipt: `td2_retune.json` beside TD1's
  before-history. Every prior replay hash is now broken by design, per the
  Findings note.
- **2026-08-29:** founded; tempo (both-deliberately, working value 10 t/s),
  instincts-under-idleness, body-routed income, and dynamics-first priority
  all ruled by Mark the same day.

## Findings (structural, from TD2 — constants cannot reach these)

- **The terrarium has no walls.** `Ground::grow` lays bricks over ±16 but
  `Ground::solid` calls everything below y=0 floor forever, so
  `near::step_for`'s doorway/forced-drop branches walk bodies off the edge
  onto an infinite plain (seed 1: 71 escapees by tick 10,000; occupied
  span 16 → ~220). Off the map, crowding — the ecology's only density
  regulator — never engages: producers grow to 10^7 mg, consumers and
  decomposers cannot find food within range. The enclosure is the vessel's
  ruled identity; the edge should refuse a step, wall not cliff. (TD2b)
- **Founding has no kingdom floor.** Each of the 3 non-played species
  draws its kingdom at 4/6 producer, 1/6 consumer, 1/6 decomposer: over
  seeds 1-10 that yields 2 worlds with zero producer species (both
  collapse under any constants), 5 with no consumer species beyond the
  played lineage, 7 with no decomposer. An ant farm is stocked with a
  working web: genesis should guarantee each kingdom at least one species.
  Founders also all start `since_offspring: 0`, gating the first brood in
  any world behind a full gestation; staggering it as `age` already is
  converts at least one collapse seed. (TD2b)
- **The enclosure has a corpse drought, and it is set at founding.** (TD5)
  Founders are staggered over `rng.below(200)` ticks of age against a
  2,000-3,000-tick lifespan, so no founder dies of old age until ~1,800;
  starvation corpses before then hold ≤ `STARVATION_MG` (20 mg) and decay at
  1 mg/tick, so they last twenty ticks. The measured result is an enclosure
  holding **zero carrion from about tick 20 to tick 1,400** and then
  20+ bodies and tens of millions of milligrams forever after — a larder
  that opens long after its only customers are dead (last decomposer: tick
  365-430 across probed seeds). Decomposers cannot be rescued from the
  income side, whatever the routing or the yield, because there is nothing
  to route: `DECAYS_BASE_MG` was swept in TD2c, search was fixed in TD2d,
  and TD5's banking is real but has nothing to bank. Confirmed by
  diagnostic, in a throwaway worktree, deliberately not shipped: widening
  the founding stagger to `rng.below(2000)` puts **42 decomposers alive at
  the 10,000-tick horizon** in seed 7 and multiplies mean decomposer life
  by ~5 in the other two probed seeds. Whether the fix is the stagger, a
  slower carrion decay, a detritus pool that exists before anything dies,
  or a founding cohort that arrives already mid-life is Mark's — it is a
  genesis and matter-cycle question, not a rates.rs one, and against the
  RimWorld bar it is the loop that currently fails to compose: decomposers
  are a kingdom nobody ever sees do their job.
  **Ruled 2026-08-29: the founding cohort arrives mid-life (TD5b).** Age now
  draws proportional to the founder's own `lifespan_for_mass`, not a flat
  200; see TD5b's Progress entry for the carrion-window and verdict
  receipts. Not a full close — seed 9 traded its consumers for its
  decomposers, and the window's first ~20-30 ticks are still structurally
  empty — but the far edge (tick 20-1,400 holding no real carrion) is gone.
- **Individual mass has no fixed point.** Income, upkeep, and the
  reproduction tax all scale as m^0.75, so net growth's sign is
  mass-independent: bodies either grow without bound or shrink to stall;
  no constant choice creates an adult size — only crowding (which counts
  bodies, not mass) can bite. A real fix is a body-plan-derived mass
  ceiling or substrate-limited income — a mechanics design conversation
  for Mark, deliberately not attempted this round.
  **Closed 2026-08-29 (TD6): both fixes landed, and both were needed.**
  Substrate-limited income makes the enclosure's matter a fixed total (end
  biomass 3,267-27,616 mg against TD5's ~4 x 10^10), and the body plan's own
  voxel volume gives every body an adult mass. Crowding survives with its job
  restated: the soil bounds a stand's mass, crowding bounds its number, and
  removing it let a stand answer a finite budget by subdividing into 620
  ever-smaller plants. See TD6's Progress entry.
