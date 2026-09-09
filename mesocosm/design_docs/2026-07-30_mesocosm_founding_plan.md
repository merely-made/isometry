# Mesocosm: Founding Plan

**Status: active founding design, refreshed 2026-08-31.** Later domain plans
and live code supersede its implementation status; the
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns the current
integration proof and the dependency ledger owns dispatch order.
Two audit corrections bind readers of this document: (1) its M0/runtime
descriptions are **historical** where they conflict with the landed
`mesocosm-runtime` (fixed-step driver, traces, replay, receipts) and the
E0-E4 slices; the general model and place-graph plans carry current state.
(2) The slogan that a part has "provenance, because it used to be
somebody" is poetry, not contract: `Origin::Founding` parts and recipe
regrowth never were anybody. Provenance is **three separable kinds** —
acquired developmental vocabulary, somatic graft provenance, and filial
material/lineage provenance — and claims must name which kind they mean. The deterministic core,
body pipeline, host, epoch lab, and first Isometry projection are implemented;
the repeated phenotype loop and later M-phases remain open. Vessel 1 of the
games wing. Shared architecture, the three pipeline laws, and the wing vocabulary
live in [the games wing founding record](2026-07-30_games_wing_founding.md)
and are not repeated here.

---

## 1. The game

**First person. You are this body.** You are a critter in an enclosed
mid-scale ecosystem. You grow by incorporating other organisms. You die, and
your line continues or does not. Your goal is to increase your kind's share
of the world's biomass, which cannot be achieved by predation alone.

Structurally it is Rogue Legacy's generational run loop pointed at an
ecosystem instead of a castle — but you **build the critter as you go**,
generation by generation, trading off against a metabolic budget rather than
rolling a fresh heir. **Morph yourself, morph the world.** A successful run
establishes the lineage in the persistent world, and the world keeps what
your generations did to it.

The arena is not a room in this game. **The arena is the world**: your
lineage competing for resources, for a niche, for a survival strategy in a
strange place (see below).

### The single verb: metabolize

World into self, self into world. One input-level verb covering combat,
growth, building, terraforming, and reef-laying, read at different scales.
Movement, consumption, perception, damage, feeding, and growth are all
experienced directly, never through a menu.

Reference points for verb density: flOw (drift and eat), Vampire Survivors
(decision density over input complexity), Spore's cell stage (the good part).

### Parts come only from incorporation

**This is the keystone.** Spore's cell stage failed as evolution because
parts came from a catalog, so parts were stat sticks and evolution was
shopping. Here:

> You gain parts only by incorporating other organisms. Every part carries
> provenance, because it used to be somebody.

The real biology is endosymbiosis, which is the literal history of the
eukaryotic cell, and its sharpest form is **kleptoplasty**: an organism eats
algae and retains the functional chloroplasts.

The consequence reaches the whole wing. Items and magic in later vessels
**derive** from this rather than being bolted on: a faction's signature
weapon in Isometry is an organism its ancestors ate. An "item" may carry an
opaque body revision, biological-line reference, optional outer Fili
provenance, and an appearance projection. The v0 proof carries only part of
that record; the
[wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md) scopes
the stable identity and topology still needed at v1.

### The real axis is trait count, not cell count

**Corrected 2026-07-30.** "Multicellular, not cellular" was the wrong way to
say it. The measure is **how many traits an organism carries**:

- A single cell is a **one-trait critter**. Legitimate, playable, and the
  bottom of the ladder rather than a discarded stage.
- A cell that has absorbed mitochondria and uses them has **two traits**, and
  counts as complex for this game's purposes. Endosymbiosis is the promotion
  mechanism, exactly as in real history.

So proto-conditions are **not retired** — they are one starting condition
among many, and a good one for seeding a roster, because one-trait organisms
are the cheapest interesting things a world can be populated with. What is
retired is only the idea that the game *begins* at abiogenesis and climbs a
fixed staircase. Any world can start anywhere on the trait ladder.

Slime mold stays as the deliberate joke it deserves to be: arguably one cell,
behaves like a colony, solves mazes.

### Three kingdoms

Plant, animal, fungus — the three lineage **strategies**, which are the three
trophic roles.

The three kingdoms are the three lineage **strategies**, and they are the
three trophic roles:

- **Producers** (plants, algae, coral): slow, structural, high-biomass,
  terraforming. They fix energy and make the world habitable.
- **Consumers** (animals): fast, agile, low-biomass. They shape the producers
  by grazing and predation.
- **Decomposers** (fungi): networked, patient, spreading through the dead.
  They return locked matter to circulation.

Adding the third kingdom is what makes the biomass economy a **cycle rather
than a ratchet**. Producers fix, consumers eat, decomposers release. Without
decomposition, dead biomass locks up and the world stalls — so a world with
no fungal line is a world running down, and that is a legible, playable
pressure rather than a rule in a manual. The fungal strategy also plays
unlike the other two: it is infrastructure, not action. A mycelial line
spreads through what has died, which means your own dead generations are its
substrate.

Biomass share is therefore unwinnable by any single kingdom, structurally
rather than as a tuning knob. The goal is a balancing act, and it is niche
construction rather than growth: you engineer the world to favor your line.

**Kleptoplasty is the boundary-crossing move.** A consumer line that steals
photosynthesis becomes partly a producer. The incorporation mechanic is also
the bridge between kingdoms, not merely a parts pipe.

*Trophic cascade* is the candidate name for world-morphing-by-eating (remove
a predator and the vegetation changes). Unruled.

**Naming note:** using plant / animal / fungus directly sidesteps the
platform-side reservation on the bare word *flora* (a moot's accumulated
engrams). Do not reintroduce it.

### Worlds

A world is a **set of conditions**, and the roster of worlds is a design axis
in its own right rather than a backdrop. Named 2026-07-30: proto-conditions
(the old soup, now one option), ocean, ice, hothouse, and gas giants where
life rides the upper stratosphere. And explicitly **fantastical, extreme,
bizarre, impossible, or magical ones** — this is not an earth-history
simulator, and a world whose physics or chemistry is frankly invented is a
first-class citizen.

Conditions are what make a trait good or useless, so the world is the thing
that gives the metabolic budget its meaning. The same body is a triumph in
one world and a corpse in another.

**Worlds seek balance through flows and costs.** Gameplay gating is necessary
but insufficient: to have a trait, something has to acquire it diegetically
(eat it, be given it, be infected by it, strike a bargain for it), but an
acquired advantage would still ratchet upward if it were free to keep. Every
trait therefore participates in the metabolic budget and in the world's
material economy: acquisition, upkeep, development, reproduction, dependency,
and opportunity cost. Energy and matter move among producers, consumers,
decomposers, corpses, and the environment. Scarcity, niche overlap,
predator/prey and host/parasite feedback, migration, dormancy, and spatial
refugia supply the balancing pressure. The game may settle into a moving
equilibrium, oscillate, or collapse; none is guaranteed to be kind.

#### World-condition grammar

**Adopted 2026-07-31 from Mark's
[Exocosm resources](https://exocosm.org/resources/) reference.** Exocosm's
useful method is to begin with a question — life under high gravity and a
dense atmosphere, on a tidally locked world, through a violent eccentric
year, or permanently aloft — and derive niches from the resulting pressures.
Mesocosm should generate worlds the same way rather than choose a decorative
biome.

| Family | Example parameters |
| ------ | ------------------ |
| Energy schedule | spectrum, intensity, day/year shape, tidal locking, flares, geothermal and chemical sources |
| Medium and mechanics | gravity, pressure, density, viscosity, buoyancy, solid/liquid/gas layers, vertical stratification |
| Chemistry and materials | solvent, atmospheric mix, redox pairs, nutrients, salinity, acidity, scarce structural materials |
| Topology | connected oceans, islands, caves, canopy, cloud bands, rings, moving habitable zones |
| Cycles and disturbances | tides, freeze/thaw, storms, eruptions, impacts, resource pulses, magical seasons |
| Initial ecology | founder lineages, trophic roles, body-plan priors, dispersal, dependency graph, generation tempo |
| Altered law | one explicit impossible or magical rule and the consequences it permits |

These values are not all scalar knobs. Some are constants, some are spatial
Numen/Quint fields, some are cycles, and some are event distributions for the
storyteller. A world profile records the question, the parameters, their
generator/content versions, and the derived pressures. Exocosm is a
worldbuilding reference library, not a runtime dependency or a demand for
astrophysical realism.

### Acquisition is not only eating

Incorporation is the keystone, but it is not the only route, and the others
are what make the world social rather than merely edible:

- **Provide something with what it needs, and it may follow you.** The
  beginning of symbiosis, and the beginning of company.
- **Dependency forms whether you plan it or not.** You come to rely on a
  particular tree for your reproductive cycle; that tree is now part of your
  strategy and part of your exposure.
- **Infection runs both ways.** A viral critter starts infecting the tree you
  depend on. As an animal you may simply have no way to fight it — which is
  not a failure state, it is the *shape of the problem* that the next
  adaptation phase exists to answer.

That last example is the design in miniature: a threat you cannot solve in
your current form, surfaced during play, answered between epochs.

### The nest

**Permitted 2026-07-30** by the care-granularity relaxation (wing founding
record §1). Territorial construction — a dam, a reef, a mound, a burrow — was
previously refused as base-tending drifting toward Paredros' person. That
reading was wrong: **building is how a lineage constructs its niche**, and
niche construction is this vessel's stated win condition rather than a
borrowed mechanic.

It also gives the three kingdoms distinct construction verbs, which is free
characterisation: producers build by *growing* structure (reef, canopy),
consumers by *excavating and hoarding* (burrow, cache), decomposers by
*spreading* (mycelial network through the dead). And it gives the world
something to remember you by between epochs, which Law B needs anyway.

Home person is unchanged. You build as the critter, in the world, in first
person — not from a construction menu.

### The world is not a larder

**Ruled 2026-07-31, from Scavengers' Reign.** The anti-pattern, in Mark's
words: *"we're just kinda munchin'. A free meal, talk about the opposite of a
game."*

A morsel today is `{species, volume, mass, position}` sitting still, waiting
to be collected. It costs nothing, risks nothing, and decides nothing. **Right
now nothing in the world does anything except the player** — which is exactly
the difference between a field of pickups and an ecology, and it is a larger
gap than anything else outstanding.

On Vesta every organism has "its own coherent place, purpose, method and cycle
of life": clone-breeding plants that reproduce by killing herds, things that
run a whole birth-to-death cycle in three minutes to renew a forest. Nothing
is an encounter. Everything is **a process you walked into**.

> The rule: loose matter becomes **organisms with a lifecycle stage**, running
> the same loop the player runs. Everything is trying to do what you are trying
> to do.

**Miscible strategies, not classes.** The point is not a rock-paper-scissors of
predator and prey but a set of survival strategies an organism may *combine*.
A producer can also parasitise. The three kingdoms are trophic roles, not
character classes, and a lineage may run several.

| Strategy | What it means here |
| -------- | ------------------ |
| **Predation** | Take the parts by force. What we have, and currently the only one. |
| **Symbiosis** | A living neighbour is worth more than an eaten one, because it keeps giving. Already flagged; still unbuilt. |
| **Parasitism** | The reversal: **something incorporates you.** A part you carry that was once somebody may still *be* somebody, with upkeep, preference, and a reason it is there. |
| **Simulacra** | Mimicry. A thing that is not what its shape says it is. |
| **Signalling** | Advertising what you are: toxic, armed, mating, kin. |
| **Counter-signalling** | Faking the advertisement, or seeing through one. |
| **Distribution** | Making a resource *in order to be eaten*, because being eaten spreads you. |

Two of these are unusually strong here because they collide with machinery
that already exists.

**Mimicry breaks the shape contract, on purpose.** Roles are read from
geometry, so the game teaches "shape tells you function" from the first meal.
A mimic violates exactly that lesson: limb-shaped, but a trap. That is a
mechanic which cannot exist without the role system and which makes the role
system feel earned rather than administrative. Signalling then supplies the
second-order cue — material and colour are already carried per part, so
"bright means toxic" is nearly free, and a mimic wearing brightness without
toxicity is the counter-signal.

**Distribution inverts the free meal completely.** An organism that grows a
rich, tempting part *so that* something eats it and carries its offspring
elsewhere. Because incorporation means what you eat becomes part of you, the
seed rides in your body. You were not the predator; you were the vector. This
is endozoochory, it needs no new substrate, and it is the sharpest available
answer to "a free meal is the opposite of a game": the meal *wanted* this.

**Illegibility is a feature, and Law B does not forbid it.** Corrected
2026-07-31, after an over-application of the law. Vesta's stated danger is
that "a creature's threat is often impossible to estimate until it's too
late", and the first draft here called that a contradiction of Law B and
proposed *legible mechanism, unpredictable consequence* to soften it. That was
wrong twice over.

First, **Law B is about the player's own marks**, not the world's dangers. It
says a lineage's inheritance must be pointable — that you can walk into a
village and say "those are mine". It says nothing about whether a threat can
be read, and applying it there was a category error.

Second, **this is a roguelike, and unfair is the tradition**. Mark: *"I
actually don't mind unfair."* The genre's canon, and this wing's own influence
set, is built on exactly what the first draft flinched from: Caves of Qud will
kill you with something you had no way to evaluate, Vagante hides its traps,
NetHack's whole item game is *quaff and find out*, and Dwarf Fortress named
the phenomenon **!!FUN!!**. The world does not scale to the player and never
should.

The tension dissolves on timescale rather than on caution:

> **Illegible within a run. Legible across the lineage.**

Knowledge is the thing that survives a death. A generation that died to a
mimic leaves that fact behind, and the machinery is already ruled: the deed
log records what happened, **tulpa** is precisely "what memory keeps"
**(2026-09-02 note: this organ is now called hagiograph; "tulpa" has been
renamed to gemot's federated adapter-training lane; see repo `CLAUDE.md`)**,
and **fili** carries it across the fork. Learning the bestiary *is* an
inheritance, which fits Law B better than the reading it was given.

**The one line worth holding is unfair versus unknowable.** A death you could
not have predicted is good; a death there was *nothing to predict* is a coin
flip wearing a costume. NetHack's potions are unidentified but never
arbitrary — price tiers, consistent appearance within a run, engrave-ID, the
floor test. There is always a tell, even one you only recognise in hindsight.
So: **no cap on cruelty, and a hard floor on arbitrariness.** Every threat
carries a tell that a second encounter can find.

**The authoring caution.** Vesta's ecology is *authored*. "Clone-breeding
plants that reproduce by killing herds" is a specific weird idea a person had,
and a generator produces variance rather than ideas — the Spore trap in new
clothes. What makes Vesta feel enormous is roughly a dozen organisms each
carrying one strange mechanism followed all the way through: **complexity from
specificity, not quantity**, which is Law B's few-loud-signatures at ecology
scale. So: **author the organisms, generate the arrangements.** That is the
wave 2.2 ruling (three authored worlds, not procedural generation) holding one
level further down, at the bestiary.

**Cost, stated plainly.** This changes `Morsel` in the core from inert matter
into an organism with state and a step, and it means the world simulates
during an epoch rather than only during adaptation. It is the substrate every
other strategy above needs, so it comes first or none of them can exist.

### The metabolic budget

Every part costs upkeep. Armor is mass to haul, speed burns fuel,
photosynthesis wants light and stillness, a fungal network wants substrate
and time. **The budget is the scarcity that Law A's deed log records**, and
it is where Gotcha Force's point budget actually lands in this vessel:
reincarnated as metabolism rather than imported as a roster cap.

Structurally this is Rogue Legacy with a body instead of a manor: you build
the critter as you go, and each generation is a set of tradeoffs and
improvements against a budget that will not let you have everything. "Chose
armor over speed while starving in a cold vent" is a budget decision before
it is a story, which is exactly why the story is trustworthy.

### The epoch loop

**Ruled 2026-07-30. This is the game's structure, and it is what makes it a
roguelike rather than a sandbox.** Two alternating phases:

**Clarified 2026-08-31:** reproduction is an individual-scale
micro-checkpoint inside this structure, not a synonym for the epoch boundary.
A birth makes descent and a possible continuation through the offspring
pointable. The boundary below remains the lineage-scale review and revision;
its exact world trigger is still open.

**1. The epoch — played, embodied, first person.** Acquire resources,
explore, fight, test what your body can do. Form relationships and
dependencies. Discover, by living in it, what your form cannot handle. The
epoch ends on a timer or on a condition; whether it is timed is undecided
and worth probing both ways.

**2. Adaptation and world examination — turn-based, everyone.** What you
metabolized during the epoch is your **bank of possible filial changes**.

Note that **this phase is third person**, and deliberately so: you view the
ecology from outside, as pieces, yourself among them. It is the vessel's
sanctioned person shift, and it passes all three guardrails — first person
remains home and you return to it, the shift is bounded and diegetic (an epoch
ends), and it is cheap because it is a turn-based UI over the same world
rather than a second simulation. This phase is why the strict
person-purity rule had to be relaxed (wing founding record §1).

Then:

> **Every species takes a turn**, spending its own accumulated resources to
> adapt, in initiative order — and the same initiative order carries over
> from the epoch. Mutations can be swapped, not merely added.

**Initiative is descending metabolic complexity, ruled 2026-07-31.** The most
complex lineages commit first. Simpler lineages act later and can respond to
what those expensive, slower-generating forms just became. This compresses
generation time into one legible adaptation round: the fruit fly can pass
through many generations within one cicada lifecycle, so its lineage receives
the informational advantage rather than asking the player to watch hundreds
of repeated turns. Whether generation tempo also changes candidate count,
mutation variance, or bank growth remains a tuning question; it must not add
more visible turns merely to imitate elapsed generations.

The player is one species among many at this table. That single decision is
what makes the world feel like it is playing too, and it is where trophic
cascades and extinctions become legible: you watch the thing that ate your
food supply spend its bank on a better way to eat your food supply.

The run is some number of these rounds, **limited or unlimited** — also
undecided, and the choice changes the genre (a fixed count is a scored
roguelike; unlimited is a world you live in until you lose).

### Prior art, checked 2026-07-30

Mark asked whether this direction has been explored. Findings:

- **Thrive** (Revolutionary Games, open source) is the serious Spore
  successor: microbe stage reached 1.0 in December 2025, **including an
  endosymbiosis feature**, with 9 planned stages and multicellular next. It
  is the closest existing thing to the incorporation keystone, and it is also
  a live cautionary datapoint about stage-based scope — one stage took the
  project **more than ten years**, and even at 1.0 its own GDD still marks
  Evolution, NPCs, and the Microbe Editor as work in progress. Those are
  precisely the systems this vessel calls novel. Reinforces the
  one-substrate rule and the standalone-completeness law.

  **Studied in detail 2026-07-30** (see §1c for what was taken).
- **Sipho** (2018, 1.0 in 2023) is an action-roguelike where you build a
  creature from zooids and eat to grow, each zooid specialising in a
  function. The nearest existing feel for the *epoch* half.
- **Bite the Bullet** does eat-enemies-to-power-up as a run-and-gun.
- **Dominant Species** (GMT, board game) is the nearest prior art for the
  *adaptation* half, and it is closer than expected: players are animal
  classes, spend action pawns, **adapt by spending adaptation cubes to take
  trait cards**, migrate and speciate, suffer a Healthy → Endangered →
  Extinct track, all against a slowly-encroaching ice age — and **turn order
  for the next round is determined by standing**. The initiative-order idea
  has been proven at a table.
- **Evolution: The Origin of Species** does trait cards and predator/prey
  economies.

**The gap is the seam.** No verified prior art alternates a *real-time,
embodied, first-person epoch* with a *turn-based all-species adaptation
phase*. Evolution games are either simulations you steer from outside
(Thrive's editor, Species, Niche) or board games with no embodied layer. The
combination is the thing worth building, and the risk it carries is the usual
one for hybrids: two good halves that do not want to meet. The M-phases
should therefore prove the seam early rather than polish either half.

### Multiple lineages

**Ruled 2026-07-31.** At an epoch boundary the player may step out of the
current descent loop and enter another unlocked lineage, or branch a new
critter from the world's stock. The gate is the world's established
**complexity frontier**: an unlocked lineage in that world must be more
metabolically complex than the target. This lets the player step downward and
explore newly viable niches without minting an unearned peer at the frontier.

Leaving a lineage does not freeze it. Earlier played lines return to the
world's adaptation policy and continue to grow, decline, split, migrate, or
become extinct while the player inhabits another. Returning means entering
their current descendants, not restoring the exact body that was left.
Creative mode may offer an explicit freeze setting, but stasis is not the
default fiction.

When multiple players currently inhabit the same line, adaptation follows the
[epoch-boundary plan's shared-lineage rule](2026-08-01_epoch_boundary_plan.md):
agreement adopts one descendant revision, while disagreement branches without
editing the shared parent in place.

Lines can advantage or disadvantage one another, so a mature world may hold
a player-shaped producer line and consumer line that support each other.
Favoritism is real: some lines are outcompeted entirely and leave only traces,
or mutate into other critters. Lineage favoritism is the first identified
attachment-creep lever.

### Death

Dead is dead; lineage persists. Descent requires more than a rebuild — a
line must actually be carried, by offspring or by a tended continuation.
Impact without anyone carrying the line is not descent at all; it is
persistence as **tulpa**. The two organs partition the afterlife cleanly:
**fili records continuity of line, tulpa records what memory keeps.**

### Tone

RimWorld vanilla. Sincere, affectionate, mortal. Dark events may happen;
the endless organ-theft and incest-joke register is out. This matters more
than usual here, because incorporation carries real ethical weight once
critters are not necessarily unintelligent — play it with ritual
seriousness (Qud's water ritual), never as a loot economy.

### Expression

Nonverbal, or Tomodachi Life-style quirk vignettes. Gibberish voice is fine.
Personality renders as movement, habits, and emotes — Rain World is the
proof that per-individual movement personality produces attachment with zero
dialogue.

### The arena is the world

**Ruled 2026-07-30.** The label was "cell+arena" and both halves were wrong.
Not *cell*, because critters are multicellular. And the arena was never a
combat mode: **the arena is ecological competition itself.** Your lineage
competes for resources, for a niche, for a survival strategy in a strange
world. That is the arena, and it is the whole game rather than a room
inside it.

So the fights were never missing. Predation and territorial encounters in
the world are the fights, experienced first person, and the competitive
pressure that gives them stakes is the niche itself.

Two consequences worth stating precisely:

- **Collection runs the other way.** It is not that the player collects
  lineages; **the world collects lineages, and yours displace generated
  ones.** That is Law C seen from inside the fiction: same seed format,
  player history displacing procedural content. The persistent world's
  roster of lineages is the collection, and your successful runs are how
  entries in it become yours.
- **Gotcha Force's point budget stays here**, as the metabolic budget
  above — not as a roster cap. What genuinely belongs to Isometry is
  *taming*: collecting critters, characters, and companions into a faction
  roster, which is befriending a goblin or taming a dog, and which Isometry
  already implements as `convince` (an attack-shaped social action whose
  consequence is allegiance). That is recruitment, not force-building, and
  it needs nothing new here.

The label retires in favour of **critter and arena**, read correctly.

### Where new lineages come from

**Corrected 2026-07-31: soup stays; the fixed staircase is retired.**
Proto-conditions are a first-class world and an excellent starting roster of
one-trait critters. What the game rejects is only the assumption that every
world begins at abiogenesis and must climb through a prescribed cellular
stage before the real game starts. Any world may begin anywhere on the trait
ladder.

A player-shaped lineage arrives by **branching, not a blank creator**. You
split from something already living — a one-trait soup organism, the world's
generated stock, or one of your established lines — and the split is where
authorship enters. You influence what the branch commits to without designing
it outright. A critter that grew, which you steered but did not wholly author,
retains the otherness that attachment needs. The complexity-frontier rule
above also gives a new branch a real ceiling rather than drawing arbitrary
power from nothing.

**And it is never a chore.** Branching is a place you go, not a menu you
clear. The world's biota speciates on its own whether or not anyone is
playing, so unclaimed lineages exist to be met, adopted, or competed with.
You go there when you want the next one to be *yours*. This is Law C applied
locally: player-authored branches displace generated ones through the same
slot.

---

## 1a. Open design questions from the epoch ruling

Raised 2026-07-30. Questions 1 and 4 were substantially answered the same day
by studying Thrive (§1c); 2 and 3 remain real forks.

1. **How large is the species roster?** ~~Open.~~ **Largely answered 2026-07-30.**
   It sets the cost of the adaptation phase (every species takes a turn, so the
   phase is O(species)) and the texture of the world. Thrive's auto-evo shows
   the per-species turn can be *cheap* — generate five candidate mutations,
   score, keep the best or none — so a large roster is affordable. The
   remaining shape is still the Law B one: a modest number of **tracked**
   species with real banks, visible turns, and names, over a broader substrate
   of background biota evolving by the same cheap rule without individual
   attention. Dominant Species runs six animal classes at a table; the ceiling
   here is set by how many turns a player will *watch*, not by compute.
2. **How do branches, forks, and randomisation produce new creatures?** Three
   distinct mechanisms are tangled here: **speciation** (a line splits),
   **hybridisation** (two lines combine — which may be the endosymbiosis
   mechanic seen at organism scale rather than a separate system), and
   **drift** (random change). They need separate rules, and hybridisation in
   particular needs a ruling on whether it is player-directed or emergent.
3. **How are trophic cascades and extinction events handled?** Two candidate
   sources, not exclusive: **emergent** (the adaptation phase produces them
   naturally when a keystone species is out-competed — the best outcome,
   since it needs no authoring) and **evented** (a world throws a glaciation,
   an impact, a plague). The design should prefer emergent and use evented
   pressure only to keep worlds from settling.
4. ~~**Timed or untimed epochs?**~~ **Answered 2026-07-30: neither.** An epoch
   ends **when you have earned the right to reproduce** (§1c, from Thrive).
   Diegetic and player-paced, with no arbitrary clock. **Limited or unlimited
   rounds is still open**, and still genre-defining: a fixed count is a scored
   roguelike, unlimited is a world you live in until you lose.
5. **Determinism as a constraint.** Adopted 2026-07-30 as a design constraint
   rather than an open question, but recorded here because it binds early
   decisions: `mesocosm-core` should be a **pure function of (seed, ordered
   inputs)** behind a boundary that can be snapshotted wholesale. See the
   [body pipeline plan](archive_docs/2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md) §R0.
   Cheap to design in now, brutal to retrofit.

## 1c. What we take from Thrive

Studied 2026-07-30 as the nearest shipped relative. Four things are worth
taking, and one is worth deliberately rejecting.

**The editor is triggered by reproduction, not a timer.** In Thrive you gather
phosphate and ammonia, your organelles duplicate one at a time, and when they
have all doubled the reproduce button appears — which opens the editor. The
resource loop *is* the upgrade loop. **Adopted**, and it closes open question
4 below: an epoch ends **when you have earned the right to reproduce**. Fully
diegetic, player-paced, no arbitrary clock; a cautious player runs long epochs
and a reckless one cycles fast.

**Auto-evo's algorithm is cheap and sufficient.** Thrive evolves NPC species by
generating five random mutations per species, scoring them, and keeping the
best — or keeping none if none beats the status quo — then separately
evaluating migration between patches. That is hill-climbing with N=5. **Adopted
as the starting algorithm** for the adaptation phase's non-player species, and
it substantially answers open question 1: per-species turns need not be
expensive to be believable, so the roster can be large.

**The prediction window.** Thrive's editor runs auto-evo forward and shows how
a proposed change will affect the species *before* mutation points are spent.
**Adopted**, because it is the honest fix for Law B's concern — depth nobody
can see reads as noise — applied exactly where the player is making a
tradeoff. Our version predicts against the metabolic budget.

**Patches.** Thrive's world is compartmentalised biomes with distinct compound
availability and physical conditions, species migrate between them, and
**resource availability shifts from species activity** (photosynthesisers raise
O₂, predators lower it). That is the three-kingdom cycle validated as a shipped
mechanic. **Adopted as an option to weigh**: the world-conditions roster (§
Worlds) could be *patches within one world* rather than separate worlds, which
is cheaper and makes migration a strategic move rather than a menu.

**Rejected: invisible auto-evo.** Thrive's NPC evolution happens in the
background between generations, and the player infers it from population
numbers. Our adaptation phase makes every species take a **visible turn in
initiative order**. Same underlying algorithm, opposite legibility — you watch
the thing that eats your food supply choose to eat it better, rather than
discovering afterwards that it did. This is the deliberate divergence, and it
is the reason the seam is worth building.

Also confirmed rather than adopted: **extinction, not death, is the failure
state** (a cell dying is not game over; losing the species is), which is
independent support for the death ruling. And their per-generation Mutation
Point budget (100 by default) is the simpler cousin of the metabolic budget —
which puts the burden on ours to produce *better decisions*, not merely better
fiction.

## 1b. A storyteller, and where it belongs

Mark's suggestion, 2026-07-30: a **RimWorld-style storyteller** — a director
that paces pressure rather than a fixed difficulty curve — and the
observation that it would apply to more than one game in the wing.

That is right, and it is a candidate for the first genuinely shared *game*
component, distinct from the platform substrate. All three vessels want the
same organ at different scales: Mesocosm pacing extinction pressure and world
events across epochs, Paredros pacing what an expedition meets and what the
settlement suffers, Isometry pacing a campaign (where it also has an obvious
relationship to the storylet engine that already exists there).

**Do not build it as a shared crate yet.** The wing's own extraction
discipline says a shared component is pulled out once two real consumers
exist. Build a storyteller inside Mesocosm first, let Paredros want one, and
extract at the second consumer. Recorded here so the eventual seam is not a
surprise.

## 2. What is genuinely new here

Stated plainly so the cost is visible:

- **A live voxel body pipeline.** Isometry keeps voxels as an asset substrate
  and bakes sprites. Mesocosm must attach, remove, animate, collide, and
  remesh parts during play. Whether the world projection is 2.5D or 3D is
  deliberately left to the host probe; the live body requirement is new in
  either lane.
- **Physics-legible bodies.** Spore shipped 228 parts that resolved to stat
  icons, so the simulation never read form. The fix is that form *is* the
  dynamics input: voxel mass is inertia, part placement is balance and reach
  and hitbox, armor distribution decides where subtraction hurts. The chosen
  Rapier dimensionality reads the body directly, so form cannot be ignored.
- **Procedural animation over grown morphologies.** Hecker's real-time motion
  retargeting is the prior art for animating bodies nobody has seen.

Available rather than new: `numen`/`quint` is a portable field algebra
(scalar and vector fields, couplings, closed-form evaluation with
finite-difference gradients, Rhai authoring, a Burn lowering with a wgpu
backend) — an influence-map substrate suited to threat fields, flee
gradients, hazard falloff, and drift. `seiche` is rapier2d graph layout, and
its real contribution is the pattern: a physics world continuously reconciled
to a host graph the physics never owns.

---

## 3. Phases

Done-conditions, not estimates. Each phase must be worth playing on its own.

### M0 — The verb
One critter, one enclosure, metabolize. Eat, incorporate, deposit, and attach
one provenance-bearing part during play. This is built once in
`mesocosm-core` and projected through the two hosts named in the body-pipeline
probe. The probe chooses 2.5D versus 3D presentation and the corresponding
Rapier lane rather than this founding phase hardcoding either.

**Done when** a playtester will re-enter the world voluntarily, without a
goal being offered.

### M1 — Incorporation with provenance
Parts arrive only by incorporation. Each part records what it used to be.
Bodies are physics-legible: mass, balance, and reach follow from placement.

**Done when** two runs that ate differently *play* differently, without a
stat screen being consulted.

### M2 — The seam
The epoch ends and the adaptation phase opens: what you metabolized becomes a
bank, you spend it on filial changes, and **at least one rival species takes
its turn too**. This is the riskiest joint in the design (§1, prior art) and
it is proven here rather than after the halves are polished.

**Done when** a player finishes an epoch wanting the adaptation phase, and
finishes the adaptation phase wanting the next epoch.

### M2a — The deed log
Runs write `(scarcity context, chosen, foregone, cause-link)` entries.
Append-only, codicil-shaped. The adaptation phase is a natural writer: every
spend is a recorded tradeoff.

**Done when** a run's log reads back as a legible story of what that critter
valued, to someone who did not watch the run.

### M3 — Lineage and the world that remembers
Death, descent, and traces. Successful runs establish a lineage in the
persistent world; the world keeps the marks.

**Done when** a player recognizes their own earlier line's handiwork in a
world they did not expect to.

### M4 — The three kingdoms, and a world that pushes back
Producer, consumer, and decomposer strategies; lineage switching; biomass
share as the win condition; the metabolic budget as the constraint that makes
each generation a real tradeoff. The full species roster takes turns, and
cascades and extinctions emerge from it. A storyteller paces what the world
throws on top (§1b).

**Done when** a player deliberately grows one kingdom's line to support
another's, unprompted — notices a world running down for want of
decomposers — and loses a lineage to something they watched another species
decide to become.

### M5: export, v0 proof complete; identity contract pending
The direct v0 body and chronicle proof pair with Isometry is complete without a
`mere.pack/v1` envelope (wing founding record §6). Body v1 adds stable subject,
revision, part, and topology identity after those addresses are proven locally;
the pack envelope still waits for a real federation consumer.

**Done when** a played critter and an RNG critter enter Isometry through the
same slot indistinguishably; Isometry returns history without losing opaque
body identity; and Mesocosm distinguishes the same returning subject from a
new biological descendant.

---

## 4. Findings

*Verified facts discovered during the work, dated, with references. Empty at
founding.*

- **2026-07-30, corrected after stack review**: the Merely stack supplies a
  credible host skeleton — winit/device/presentation ownership, NetRender and
  Vello 2D composition, Cambium UI, Armillary actors, Firewheel, persistence,
  fields, and an Isometry voxel baker — but not a game runtime or live body
  renderer. The missing middle and the Bevy/Genet/Bones/Renderling probe are
  specified in the engine landscape and body-pipeline plan.
- **2026-07-30**: `numen`/`quint`/`seiche` live at
  `mere/crates/conatus/{numen,quint,seiche}`; there is no `repos/conatus`
  (absorbed by the 2026-07-23 consolidation). `seiche` wraps `rapier2d`
  0.33 and is graph layout, not a game physics engine. Confirms that the
  chosen Rapier2D or Rapier3D lane is a direct dependency rather than a
  conatus consumer.

---

## 5. Progress

- **2026-09-02, terminology note (doc only).** Mark authorized renaming the
  memorial organ to **hagiograph** and reassigning **tulpa** to gemot's
  federated adapter-training lane. A dated note was added at this doc's first
  mention of tulpa rather than rewriting the historical text; see repo
  `CLAUDE.md`. No code changed.

- **2026-08-31:** linked the current playable ecology architecture and
  clarified reproduction as an individual checkpoint distinct from the
  lineage-scale epoch boundary. No founding ruling was otherwise replaced.

- **2026-07-30**: repo founded, name reserved, design recorded. No code.
- **2026-07-31**: recorded the world-condition grammar, metabolic-complexity
  initiative, complexity-frontier lineage switching, autonomous inactive
  lineages, and the restoration of soup as a valid proto-condition.
