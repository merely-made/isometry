# The Games Wing: Founding Record

**Status: founding record, 2026-07-30.** Wing-level. Owns the shared
architecture, laws, and vocabulary for Mesocosm, Paredros, and Isometry.
Vessel-specific design lives in each repo's own founding plan. This document
is cited by `paredros/design_docs/` and should be cited rather than copied.

Everything here was ruled in conversation on 2026-07-29 and 2026-07-30.
Claims about the existing stack were verified against code and docs on those
dates; each is marked where it matters.

---

## 1. What this is

Three games that are one wing. They share a world substrate, a lineage
model, and a trust plane; they do not share a genre, a schedule, or their
verbs.

**The engine clause, narrowed 2026-08-05.** As founded this read "they do
not share an engine". The clause was guarding against coupling-as-obligation
and genre convergence, and that guard stands as written above. Sharing
engine *organs* is now encouraged where the organ stays verb-neutral:
vessels share nouns (space, bodies, fields, time, provenance); each owns
its verb and person. When a shared component starts encoding what you do,
it has crossed into a game and belongs there. Law A is unaffected: it
governs content inheritance across games, not code or format sharing. See
`2026-08-05_place_graph_engine_plan.md` §0 for the full ruling set.

| Game | Person | Relationship | Design consequence |
| ---- | ------ | ------------ | ------------------ |
| **Mesocosm** | First | I am this body | Movement, consumption, perception, damage, feeding, and growth are experienced directly. |
| **Paredros** | Second | I live with you | Companions can be addressed, persuaded, equipped, and helped. They remain peers, and they can replace you. |
| **Isometry** | Third | They occupy the world | Characters become groups, factions, visible pieces in a shared tactical and historical account. |

**Person is agency, not camera.** Ruled 2026-07-30 after the two vocabularies
collided in a render doc. "Second person" says companions are peers you
address rather than units you command; it says nothing about where the camera
sits, and Paredros may well use a close camera. When discussing renderers, say
**camera distance**; reserve *person* for agency.

### The wing's question is continuity under transformation

**Ruled 2026-08-07 (Mark).** The stronger wing identity than "three games
sharing an engine":

- **Mesocosm** asks whether a creature remains itself as its body and
  capacities change.
- **Paredros** asks whether a community remains itself as control, bodies,
  and generations change.
- **Isometry** asks whether a campaign remains itself across adjudication,
  authors, imports, and revisions.

This names what the wing's vocabulary was already circling: *animula* is a
soul continuous across bodies; *fili* is lineage continuous across worlds;
*tulpa* is what memory keeps of the dead **(2026-09-02 note: this organ is
now called hagiograph; "tulpa" has been renamed to gemot's federated
adapter-training lane; see repo `CLAUDE.md`)**; *borg* is identity acquired by
naming; the phenotype contract's same-individual-versus-descendant and
carry-body-versus-regrow distinctions are this question as a technical
contract; and the frontier's "a line you have already lived is always yours
to return to" is it as a game rule. Care granularity says **whom** each
vessel cares for; continuity under transformation says **what question is
asked about them**.

Four facts stay separate across the wing so the question stays askable:
**subject, body, role, lineage.** The phenotype contract already separates
subject, body revision, and biological line; **role** (office, standing in
a community) joins them as the fourth, first needed by Paredros
succession. A subject can inhabit a revised body, hold or lose a role,
descend from a lineage, or surrender player control without becoming a
different person, which is what unlocks succession, development, imports,
reincorporation, prosthesis, shapeshifting, inheritance, and historical
characters without one universal character schema.

A consequence recorded the same day: the fantastical layer's natural
subject matter is this same question. Curses on lineages, edible names,
metabolized memories, and places that grow organs are all perturbations of
continuity, which is why "impossible ecology" rather than spellcasting is
the fantastical direction (general model plan, F-gates).

### Historical entry and lineage-shaped lives

**Direction affirmed by Mark, 2026-09-05; entry mechanics proposed, not implemented.**
Any of the three games may be played at any supported point in a world's
history. Their characteristic historical settings differ: Mesocosm naturally
foregrounds early ecological history, when most subjects are critters;
Paredros foregrounds pre-civilization and the emergence of kin, kith, camps,
colonies, and bases; Isometry foregrounds worlds where civilizations and their
institutions have become dominant. These are historical emphases, not required
stages, population-type restrictions, or a mandatory export sequence.

Late Mesocosm can concern organisms in cultivated soil, infrastructure,
domesticated populations, or ruins. Late Paredros can concern one life among
powerful institutions. Early Isometry can concern the collective activity of
a small founding group. Each keeps its own care granularity and admits the
mechanisms it supports. An unsupported ruleset is an explicit compatibility
result, not an era lock. Naming does not create cognition; anonymous subjects
can learn and act, and faction membership is not a prerequisite for agency.

Ecology, settlement, and institutions coexist at every date in different
proportions and places. Civilization need not emerge, advance monotonically,
cover every region, or remain dominant. Collapse, dispersal, and recolonization
can make an old world newly relevant to any vessel. The intended historical
arc must not become a universal civilization-level meter.

**Proposed entry policy:** choose a world, a dated history checkpoint, a place,
and a vessel. Continue a compatible present, or explicitly branch from a past
checkpoint. Changing the past never silently rewrites an already played
future. Only causally available facts enter the branch; later discoveries are
not earlier characters' knowledge. A subject must exist at the chosen point:
before its birth, choose an ancestor or another generated subject instead.
Unplayed periods may be generated from established constraints, retaining the
provenance and uncertainty of generated versus played history. This proposes
checkpoint materialization, not a requirement to simulate every historical
tick or maintain one live simulator across all games.

History influences a borg through distinct, pointable routes:

- **Biological descent:** developmental potential, inherited mechanisms,
  tolerances, and costs. Historical conditions matter through recorded
  selection/developmental changes, not an arbitrary bonus for an era name.
- **Individual development:** nutrition, habitat, injury, grafts, symbioses,
  practice, and experiences that actually affected this subject.
- **Cultural transmission:** techniques carried by teaching, imitation,
  records, institutions, or explicitly supported memory-transfer mechanisms.
  Shared ancestry alone does not transfer learned expertise.
- **Material and social inheritance:** tools, cultivated places, obligations,
  access, and other people's beliefs about the line. Recognition and evidence
  govern reputation; ancestry does not determine personality or morality.

In-world biological descent, cultural transmission, and Fili's cross-world
lineage remain different relations. Reproduction checkpoints, epoch lineage
review, and individual RPG learning remain different transitions. A learned
technique becomes heritable only through a named world mechanism, with its
costs and transmission limits recorded.

Traditions arise from useful mechanisms, recurrent problems, and historical
transmission. A technique may originate independently, spread, be adapted to
another anatomy, or disappear when its practitioners and records are lost.
World generation should produce working transmission paths and local variants,
not merely random school names. Player histories displace generated history;
playing the earlier games is never required to obtain a rich lineage or culture.

Paredros's proposed RPG and character-sheet design lives in
`paredros/design_docs/2026-07-30_paredros_founding_plan.md`, under
"Borg generation, techniques, and the character sheet". That product proposal
does not impose its attributes, skill points, or action rules on the other games.

### Each vessel is a mode of the same peopled history

**Ruled 2026-08-10 (Mark),** during the R4 extraction review
(`paredros/design_docs/2026-08-10_r4_extraction_review.md`), and recorded
symmetrically rather than one-directionally: no vessel is the primary one.

- **Isometry** is the fortress and atlas mode: care for a squad and a map,
  prepared ahead, adjudicated in turns.
- **Paredros** is the adventure mode: the same world's people met one at a
  time, embodied, negotiated.
- **Mesocosm** is the ecology beneath both: the world that produces the
  people in the first place.

The Dwarf Fortress comparison is the origin of the framing and also its
limit. DF's modes share one save and one executable; **the wing's vessels do
not.** They stay sovereign games — separate genres, schedules, verbs, and
authorities — joined by a shared *history*, never by a shared running world
instance. A vessel must never be able to require another vessel to be
running.

What the frame decides, and why it is here rather than only in the review:

- **Facts of the world cross; verbs never do.** The recognizable things are
  **people** (subjects and their deeds), **things** (relics with
  provenance), and **places** (sites with history). They cross by the three
  pipeline laws of §3, as choices under scarcity with pointable inheritance.
  Kinematics, tile-and-turn adjudication, renderers, and control schemes do
  not cross, ever. Paredros walking Mesocosm's voxels and Isometry baking
  voxels to sprites are the same source becoming two lenses, not one lens
  borrowed.
- **Platform organs go up, not sideways.** Where two vessels need the same
  runtime machinery, the seam belongs to the layer that owns it (genet,
  netrender, mere), not to a new wing-level crate wedged between the games.
  The R4 review's first ruling applies this to renderer tenancy.
- **What must be recognizable gets a shared name; what must be judged does
  not.** Identity is one vocabulary across the wing (the four facts above,
  `paredros-identity` promoted 2026-08-10). The consequence grammar is
  deliberately *not* shared as a library: one grammar, sovereign evaluators
  per vessel, per the general model's evaluator rule.

### The invariant is care granularity, not person purity

**Relaxed 2026-07-30, at Mark's prompting, and this correction matters.** The
first version of this record said "creep is person drift" and forbade a vessel
from changing person at all. That rule was too strict, and the proof is
internal: **Mesocosm's adaptation phase is not first person.** When every
species spends its bank in initiative order and you watch the thing that eats
your food supply decide to eat it better, you are viewing the ecology from
outside, as pieces. That is third person, and it is the load-bearing half of
the epoch loop. A rule that condemns the wing's best mechanic is mis-stated
rather than proven.

The influence set says the same thing. **Mount & Blade** is the canonical
mixed-person game — one body in the melee, a party under command, a fief to
rule, and the shifting *is* the identity. Kenshi embodies any squad member.
Helldivers is one body inside a third-person galactic war. And the sharpest
precedent is **Dwarf Fortress**, which runs fortress mode and adventure mode
over one world: walking into your own dead fortress as a lone adventurer is
among the most beloved experiences in games, and it is precisely this wing's
export arrow. The strict rule forbade the payoff the architecture was built
for.

So the invariant is the thesis, not the grammar:

> **Care granularity must not drift. Person may.**

Mesocosm is care for a **species**, Paredros for **individuals**, Isometry for
a **community**. That survives camera and control changes in a way person
does not — XCOM is third person at individual granularity, Mount & Blade is
first person at community granularity. Person was a convenient proxy for
granularity, and it should be used as a design lens rather than a fence.

### The three guardrails that replace the prohibition

The old rule did earn its keep: it resolved where Gotcha Force belongs and it
kept Paredros' settlement layer light. Those wins are preserved by three
narrower rules.

1. **Each vessel has a home person** — its default, its center of gravity, the
   one you inhabit continuously. Mesocosm's home is first, Paredros' second,
   Isometry's third. A shift is a departure you return from.
2. **Shifts are bounded and diegetic.** Earned, framed, and temporary, not a
   second mode of equal weight with its own menu.
3. **Removal test.** Can each layer be deleted with the game still standing?
   If two persons are equally load-bearing, you have two half-games — which is
   the actual Spore failure. Spore's stages were person shifts with **no home
   person** to return to.

**The cost to book**, because this is what hollowed Spore in practice rather
than in theory: every person is a camera, a control map, and a rendering need.
Two persons per vessel is a production multiplier. Mesocosm's second person is
cheap *only* because the adaptation phase is a turn-based UI over the same
world rather than a second simulation. Any shift that needs a second
simulation or a second renderer should be refused on those grounds alone.

### What the relaxation opens

Recorded 2026-07-30. Each was forbidden by the strict rule and is permitted
by the care-granularity rule; each still has to pass the three guardrails.

- **The nest** (Mesocosm). Territorial construction — beaver dams, coral
  reefs, termite mounds. Under the strict rule this read as base-tending
  drifting second person. Under the care rule it is **niche construction**,
  which is the literal win condition. The prohibition was forbidding the
  mechanic that best expresses the vessel's own thesis.
- **Configure, don't command** (Paredros). Standing behaviour negotiated with
  a peer in advance, in the FFXII gambit shape. Categorically different from
  puppeteering in the moment: you agree how someone acts rather than driving
  them. Preserves "peers, not units" while giving the player real leverage,
  and it is a better answer than the deployment queue alone.
- **Tag-in** (Paredros). Crystal Chronicles and Gotcha Force both let you
  *become* another body. That is succession in miniature, and succession is
  already ruled — becoming a companion temporarily is the same mechanic as
  becoming one permanently.
- **An embodied scout** (Isometry). Exploration mode, the pointcrawl, and
  board-to-text narration already exist there; one figure moving through the
  world is a shift that repo is nearly built for. Still gated behind
  Isometry's own render-lane ruling.
- **Asymmetric co-op** (all three), and this is the largest unlock. Co-op is
  deferred in every vessel and has never been designed. Asymmetric co-op is
  *inherently* multi-person — one player embodied, one strategic — so a strict
  grammar makes it unbuildable. Under the care rule it is the obvious shape,
  and it is the shared-burden friction the influence set started from (the
  Crystal Chronicles chalice, Four Swords).
- **The Dwarf Fortress payoff** (wing-level). Walking into a world your other
  vessel shaped, and meeting your own dead as legend. This makes the export
  arrow *experiential* rather than merely a data transform, and it is where
  `tulpa` stops being bookkeeping and becomes the point.

### Where the idea came from

Recorded 2026-07-31, from Mark, because it is not recoverable from the code.

The seed for Mesocosm was **the physics-overlay cellular automata experiments
on the meerkat host** — Game of Life run as a physics layer over a graph
canvas, back when meerkat was the reference application. What was compelling
there was watching a small rule produce structure nobody had authored, and
wanting to be *inside* that rather than looking at it.

That is the difference the wing is built to make good on. In meerkat the
automaton was an **overlay**: presentation drawn over a graph. Here the
relationship is inverted. `mesocosm-core` owns the simulation and every
renderer is downstream projection, which is why the core is integer-only, why
hosts only project, and why the body document carries topology rather than
pixels. The rule-producing-structure is the substrate now, not the decoration.

Meerkat itself is gone, obviated into turnstone and deleted. Its lineage
persisted anyway: `frisket` relocated, the physics-over-canvas work grew into
the conatus crates (`numen`, `quint`, `seiche`, still rapier-backed layout
over a graph), and the idea became this. The wing has vocabulary for exactly
that shape — **dead is dead, lineage persists**, and what remains when nobody
carried the line is a **tulpa**. Meerkat is one.

### The continuity: critter, borg, character

**Ruled 2026-07-31 by Mark**, and it is the sharpest statement of the wing's
thesis so far:

> A **critter** is an organism. A **borg** is a *named* critter — something you
> make incidentally, by playing Mesocosm. A **character** is a
> *faction-associated* borg — something you make by playing Paredros.

The force of it is what it does *not* require. There is no conversion step, no
promotion pipeline, no second representation. Each vessel adds exactly one
thing to an artifact that already exists:

| Vessel | Adds | The thing it makes |
| ------ | ---- | ------------------ |
| Mesocosm | a **name** | a borg, out of a critter |
| Paredros | a **faction** | a character, out of a borg |
| Isometry | a **history** | a legend, out of a character |

**Naming is the whole promotion.** A critter is a member of a species; a borg
is a particular one you can refer to. That is also exactly what the deed log
does — an anonymous organism accumulates a record, and at some point the record
is *of somebody*. This is why the founding plan's death ruling works: what
survives is a line and a memory, and a memory has to be a memory *of* someone.

**Incidentally is doing real work in that sentence.** You do not go to Mesocosm
to make a borg. You play an ecology, and some critter of yours survives long
enough, does something particular enough, and accrues enough record that it
stops being a statistic. The naming is *earned by play* rather than chosen at a
character creator, which is the partial-authorship ruling arriving at the level
of identity rather than morphology.

**And a faction is a relationship, not a property.** A borg becomes a character
by being *of* somewhere and *among* someone — which is precisely Paredros'
second person, and precisely why that vessel is the one that mints characters.
You cannot be a colleague alone.

Three consequences worth holding:

- **The interchange profile does not grow.** Name and faction are fields on the
  artifact the body pipeline already carries, not new artifact kinds. The proof
  pair does not get harder because of this ruling; it gets *sharper*, because
  what crosses is now a nameable thing rather than an anonymous body document.
- **This is the one-substrate law paying out.** Three games, three additions,
  one object. If any vessel had needed to *convert* a critter into a character,
  that would have been the hollowing Spore suffered.
- **It also explains the export arrow.** A legend in Isometry is not a fourth
  kind of thing — it is a character whose history outlived it, which is exactly
  what `tulpa` was named for.

### The influence set

**Scavengers' Reign** joined 2026-07-31, and it is the reference that named
the biggest gap. On Vesta every organism has "its own coherent place, purpose,
method and cycle of life" — nothing is an encounter, everything is a process
you walked into. Mark's diagnosis of what we had instead: *"we're just kinda
munchin'. A free meal, talk about the opposite of a game."*

What it supplies that nothing else on this list does:

- **Organisms rather than pickups.** Loose matter must run the same loop the
  player runs. Detail in the Mesocosm founding plan, "The world is not a
  larder".
- **A strategy space wider than eating**: parasitism, symbiosis, simulacra,
  signalling, counter-signalling, and making a resource *in order to be
  eaten*. Mark's framing: **miscible strategies**, combinable rather than
  exclusive classes.
- **Indifference as the tone.** Vesta "has a nature that simply exists" — no
  narrative purpose, nothing spawned for you. That is the RimWorld-vanilla
  sincerity ruling seen from the ecology side.

And one caution it supplies by counterexample: Vesta is **authored**. Its
sense of enormity comes from about a dozen organisms each carrying one strange
idea followed all the way through, not from generated variety. Author the
organisms, generate the arrangements.

**Katamari Damacy** joined 2026-07-31, from Mark watching a grown critter:
"there's something kinda neat about it, like katamari." It earns the place,
because it is the one game whose core verb is *accretion made visible*. Three
things it already validates here:

- **You can see what you rolled up.** Katamari's charm is that the ball is a
  legible record of everything it ate. That is this wing's "every part used to
  be somebody" pillar, arrived at from the other direction.
- **Size gates what you can take.** Reach and the metabolic budget do the same
  work, so the escalation curve has prior art that is known to feel good.
- **Growth changes handling.** Mass and balance shifting as you accrete is
  Katamari's whole physical comedy, and this wing already computes centre of
  mass from placed parts.

The distinction worth holding: Katamari's accretion is comic and terminal, a
run that ends when the timer does. Here it is generational and consequential,
because a part carries provenance and the world keeps what you did to it.

**Voxatron** joined 2026-08-03 as Mark's closest match for the wing's original
vision. It is the material-language lodestar rather than a core-loop template:

- body, terrain, prop, construction, and damage all read as uses or states of
  one low-resolution substance;
- adding, subtracting, breaking, and rearranging that substance form one
  interaction grammar rather than separate editor, combat, and building
  fictions;
- the toy-like scale keeps procedural variation readable enough to inspect,
  remember, and care about.

This does not make a voxel grid shared simulation authority. Mesocosm's body
graph and developmental recipe, Paredros' close presentation, and Isometry's
sprite projection remain sovereign representations. The wing-level target is
the perceptual continuity: each vessel should make the same buildable,
damageable material history recognizable through its own lens.


Gotcha Force, RimWorld, Phantasy Star Online, Mount & Blade, Zelda: Four
Swords, FF Crystal Chronicles, XCOM 2, Caves of Qud, Vagante, Tactics Ogre:
The Knight of Lodis, FFTA, Fire Emblem (GBA), Kenshi, Helldivers 2,
Bethesda/Obsidian TES + Fallout, Destiny.

Six through-lines run through nearly all of it, and they are the wing's
design spine: the roster is the protagonist; sortie and return, with the
loot reveal ritualized at home; the body is a build surface and damage is
subtraction; the world runs without you; co-op with friction rather than
merely shared targets; and a toybox surface over merciless systems.

---

## 2. The lifecycle

Genesis (Mesocosm) produces critters. Critters graduate into lives
(Paredros). Lives accumulate into settlements. Settlements export as
campaigns (Isometry). Worlds fork, graft, and federate.

**Spore's stages are the known failure mode**: five shallow genres sharing
nothing, so no stage could deepen another. The insurance is one rule, and it
is the same rule Isometry already runs on (`isometry/CLAUDE.md`: the
substrate/system split is load-bearing).

> **A vessel must not create a private replacement for shared world identity,
> provenance, and causal history.**

Restated 2026-07-30, at the right altitude. The earlier wording ("all stages
are rule-dressings over one substrate; if any stage grows its own engine the
wing hollows out") was directionally right and technically wrong, because it
forbade too much. What hollowed Spore was five stages that shared no *world*,
not five stages that shared no renderer.

A vessel may absolutely have its own renderer, event loop, ECS, camera, or
physics dimensionality. What it may never do is mint a second, private answer
to *who this creature is, where it came from, and what happened to it*. The
sharing rules per layer are tabulated in the
[engine and render lane landscape](2026-07-30_engine_and_render_lane_landscape.md)
§5, and the shared organ that follows from them is planned in the
[body pipeline plan](archive_docs/2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md).

Consequently: **"shared world model" is too strong** — Mesocosm's live
ecology, Paredros' settlement, and Isometry's campaign state will never be one
in-memory model. They append compatible facts to one world. The shared thing
is the **world identity and fact substrate**.


### Bodies are part trees, and loss cascades

**Ruled 2026-07-31 by Mark, wing-wide.** Lose an arm, lose the hand, lose the
fingers.

The citation is RimWorld, which ships this: a `BodyDef` is a tree of
`BodyPartRecord`s, destroying a parent destroys its children, and capacities
like Manipulation and Moving are computed as folds over the parts that survive.
Mark's observation about it is the useful part of the evidence: **people mod to
enrich that system rather than to replace it.** A mechanic whose community
extends it has the right shape and not enough of it, which is a far better
signal than a mechanic nobody touches.

Three commitments follow, and they are one rule seen at three depths:

1. **A body is a tree of parts**, not a bag of them. Every part but the root
   names a parent.
2. **Loss cascades to dependents.** Severing an edge takes the whole subtree.
3. **Capability is a fold over surviving parts**, never a stored number. What a
   creature can do is a consequence of what it currently has.

#### What is shared, and what each vessel decides

This section's own rule settles the split, and it is worth being explicit
because the answer is not "everything":

> A vessel must not create a private replacement for shared world identity,
> provenance, and causal history.

**The tree is identity.** How parts relate, and where each came from, is
provenance and causal history in the most literal sense the wing has. So the
*structure* is shared, and no vessel may mint a second private answer to it.

**The fold is rules.** What capability falls out of an anatomy is exactly the
kind of thing each vessel decides for itself, like a renderer or a camera.
Mesocosm folds reach and upkeep out of geometry; Paredros folds a chassis into
manipulation; Isometry folds nothing at all.

Isometry is the interesting case and its own law already answers it: *the
substrate stays geometry and turns; rules belong in system plugins*. So Isometry
**carries the tree as data and mints no types for it.** A 5e character has hit
points, not fingers, and the day the substrate grows a `BodyPart` is the day it
has hard-coded a game system. The tree rides in the portable record; a system
plugin that cares about limbs interprets it.

#### The mind is not in the tree

Paredros already ruled the carve-out, before this rule existed: *skills are
use-based, accrued to the mind, surviving reassembly, so limb loss never costs
skill* (the Kenshi rule). That stands, and it generalises.

**Loss cascades through the body and stops at the person.** A creature that
loses an arm loses the hand and the fingers and whatever the arm let it do, and
loses none of what it knows. RimWorld agrees by construction, since skills are
pawn-level rather than part-level. Without this carve-out, injury would compound
into permanent character erosion, which is the Darkest Dungeon 2 failure the
wing already flagged under attachment-creep.

#### Booked consequence: the body and chronicle split at v1

`mesocosm.body/v0` and `mesocosm.chronicle/v0` each carry `parts` as a **flat
list**. If the tree is shared identity, neither is a sufficient anatomy record:
the body profile drops parent links, while the chronicle duplicates a snapshot
that can drift away from it.

V1 therefore gives the two artifacts distinct jobs. `mesocosm.body/v1` carries
stable subject, body-revision, and part addresses plus primitive parent links.
`mesocosm.chronicle/v1` carries causal facts addressed to those identities; it
does not carry another anatomy snapshot. The same subject crossing a vessel
keeps its body revision. Founding a biological descendant mints a new subject
and a newly grown body instead of pretending the ancestor returned.

The refusal path built for the interchange on 2026-07-31 is what makes that
cheap: magic and version sit ahead of the payload, both sides refuse an
unrecognised version before decoding, and a reader that meets a v1 record says
so instead of mis-reading it. This is the first real schema change, and it is
the case that machinery was built for.

Note what does **not** follow. Law A still says morphology does not travel as
foreign rule authority: the portable tree is topology and provenance, exact
geometry is an optional projection, and a descendant is still regrown under
Mesocosm's rules rather than restored voxel for voxel. The full contract and
proof gates live in the
[wing phenotype plan](2026-07-31_wing_phenotype_contract_plan.md).

---

## 3. The three pipeline laws

These govern what crosses between games. They are load-bearing; a violation
is a design bug, not a preference.

### Law A — What travels is choices under scarcity, not morphology

A shape cannot become a faction. What crosses vessels is **meaningful choice
under scarcity**, logged as `(scarcity context, chosen, foregone, cause-link)`.
The *foregone* field is load-bearing: values are revealed by what was given
up. Repeated tradeoffs of the same shape compile into a lineage value; values
drift into ethos; ethos is what Isometry reads as a faction.

Log "chose armor over speed while starving in a cold vent," never "grew
spikes." Otherwise Isometry inherits a generated bestiary and calls it a
history.

**Boundary:** morphology still matters enormously *within* Mesocosm, where
the simulation reads the body directly. Law A governs the across-game axis
only. The two axes meet cleanly because an incorporation event is
simultaneously a morphological act and a value statement. Shapes do not
become factions; shapes become **relics** of factions, and values become the
factions themselves.

### Law B — Inheritance must be pointable

Dwarf Fortress depth that nobody notices reads as procedural noise. Each
lineage carries a **small number of very loud signatures** over unlimited
quiet drift. If a player cannot walk into a village in Isometry and say
"those are mine," the pipeline is invisible and therefore fake. If all they
see is what they made, they have no reason to explore.

The loudness selector already exists in the wing's vocabulary: **Tulpa's
attention mechanic**. Codicil holds everything; Tulpa holds the retold
subset; Isometry surfaces the retold subset as visible signatures; the untold
remainder stays quiet texture for players who dig. Endosymbiosis supplies the
loudest signature class for free, because a faction's famous incorporated
organism is a visible motif.

### Law C — No homework

Spore's early stages were a corridor to the space stage. The fix is
structural, not motivational.

> **Same seed format whether authored by play or by RNG, at every import slot
> in every game.** Player history *displaces* procedural content; it never
> gates it.

Each vessel is standalone-complete: Mesocosm's win condition references no
later game, Paredros is fully playable on RNG worlds, Isometry already
stands alone. Inheritance is enrichment. The proof pair (§6) must
demonstrate displacement explicitly: import a played critter and an RNG
critter through the same profile and confirm the consuming game cannot tell
them apart structurally. Only the player can, by pointing.

---

## 4. Worlds, moots, and lineage

**The moot is the world. A game is one participant and lens upon it.**

This lands on the existing stack without force. "Participant" is the gate
vocabulary (denizen / petition / grant); "lens" is the ortet/ramet
vocabulary. A game walks the participant gate like any other outsider and
renders the world through its own lens. A game's presence in a moot is
**pack-shaped**: schema plus scripts plus a capability manifest, which is
what Isometry's system plugins already are.

Tier mapping, using the platform ladder as-is (`mere/design_docs/TERMINOLOGY.md`):

- solo world = **a mere** (born share-ready; sharing is a membership change, never a format migration)
- shared world = **a moot**
- lineage cluster = **a moothold** when it actively federates
- ecosystem metagame = **a gemot**

**Fili and moothold are different relationships.** Fili records ancestry,
forks, and genealogy. A moothold actively federates. Related worlds form a
fili graph while remaining independent; they become a moothold when they
establish federation.

**The gemot does not own the war simulation.** A gemot provides sovereignty,
admission, shared defaults, and coordination. Ecosystem-scale war facts (the
Helldivers reading) live in a dedicated metagame moot whose projection
aggregates campaign receipts across the gemot. Game rules stay out of
constitutional machinery.

**Identity travels; trust is locally interpreted.** Tessera accrues against a
persona chain root, so standing survives persona forks, but each moot
configures scoring and admission differently. Another game verifies the same
signed evidence and assigns it different meaning. That locality is a feature.

### Provenance rule

Every world is born with stable world identity, seed and content digests, a
rules profile, and optional parent/divergence provenance. In-world
biological descent belongs to the game's own lineage schema. **Fili engages
only at world forks, campaign descent, and cross-moot grafts.** This keeps
Fili's recorded scope exact (`mere/design_docs/TERMINOLOGY.md`: moot lineage,
not event history) while making provenance impossible to lose.

### Layered beings, places, and engrams

**Ruled 2026-07-31.** `character(borg(critter))` is useful conceptual
shorthand, but it must not become literal nested wire data. A cross-vessel
subject has one stable identity and independently versioned profile facets:

| Profile | Owns |
| ------- | ---- |
| Critter | metabolism, body topology, traits, biological descent, incorporated-part provenance |
| Paredros character | continuity of person, skills, affinities, personality, trust, history, relationships |
| Place | buildings, inhabitants, dependencies, deeds, accumulated customs and institutions |
| Isometry participant/faction | campaign role, authority, allegiance, ethos, public history |

A character may refer to a critter body profile; a changed chassis mints or
selects another body revision without copying the whole person into it. A
place likewise refers to inhabitants and facilities rather than containing
authoritative copies of them. Profiles compose through stable references and
capabilities, so a consumer can understand the layers it knows and preserve
the rest opaquely.

These are still **engrams** when made portable. An engram is the canonical
portable contribution payload; the profile names the schema of what it
carries. `mere.pack/v1` is the signed installable-bundle envelope that can
carry one or more related engrams plus scripts and capabilities. Engram,
profile, and pack are therefore transport, meaning, and bundle respectively,
not competing names for one object.

### Graduated interoperability

1. **Observe** — render or inspect a world
2. **Import** — understand creatures, places, maps, items
3. **Contribute** — add facts and artifacts under the moot's rules
4. **Participate** — control an actor or operate part of the simulation
5. **Co-simulate** — share a live runtime when rules and versions genuinely match

These are trust rungs and the test plan at once. Rung 5 is the netcode
dragon; rungs 1–4 carry the entire ecology and need no new machinery.

**Presentation negotiates the same way** (ruled 2026-07-31): a peer holding
the body document and a geometry renderer projects a token *live*, with
procedural articulation; a peer holding only an image decoder shows the baked
sheet. Neither is privileged, the document is canonical, and sheets are
optional derived artifacts riding beside it. Detail in the
[body pipeline plan](archive_docs/2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md) §3.

**Protocol keystone: additive facts, opaque preservation, deferred
interpretation.** Games append history in their own vocabulary and never
mutate foreign facts without a grant. Each game retains facts it cannot
interpret. Re-entry is *interpretation*, not merging: Mesocosm reads "lost an
arm at the ford" from Isometry's appended history and derives the
morphological consequence itself, under its own rules.

This prevents **destructive merge and fact loss**; it does not make semantic
conflict impossible. Independent signed facts converge by set union.
Incompatible claims remain visible until the relevant domain materializer and
group policy interpret, reject, sequence, or branch them. A true CRDT belongs
only where a domain needs concurrent edits to one mergeable value, such as
text, a counter, or possibly map cells. Live tactics and timing-sensitive
simulation retain an ordered sequencer or authority. Isometry already proves
both sides: signed multi-writer campaign space and a separately ordered
tactical session.

**Authority:** lenses hold revocable per-domain grants. Isometry's shape
generalizes unchanged — a client sends an *ask*, never a verdict, and the
grant holder resolves (`isometry/design_docs/2026-07-14_adjudication_and_representation_plan.md`).

**Time** is causal order; wall time is presentation. Cross-game clocks (a
colony decade against a tactics session) are unresolvable any other way, and
Isometry's per-map tick clocks already taught this at small scale.

**Space** is lens-local. Shared space interoperates at place-graph
granularity only; whether a lens draws diamonds, voxels, or hexes is its own
business. Isometry's tile-geometry seam plan already proved rules can be
geometry-independent.

---

## 5. Verified state of the stack

Checked 2026-07-29/30. Recorded because the first draft of this analysis
overstated it, and the correction matters.

**Ready, with receipts:**

- `mere.pack/v1` is **done**, not embryonic. B4 completed 2026-07-22 (typed
  `PackManifest`, personae signing in `TrustEnvelope`, `verify_pack` with
  Trusted/Unsigned/Broken, tamper cases proven Broken); B5 proved
  distribution over a real retinue link with moot curation and a refused
  mid-flight scope-widening tamper.
  (`mere/design_docs/mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md`)
- Isometry has landed: typed worldgen (W0–W5: factions, places, characters,
  routes, laws, history, storylets), parties and recruitment, app-side
  adjudication, split-party per-map clocks, a pointcrawl overmap on the graph
  canvas, `isometry-voxel` recipe-not-image appearance, and signed
  multi-writer P2P sessions. C7 factions **landed 2026-07-17/18** once the
  moot/murm rebase cleared.
- Participant gate: denizen / petition / grant / pack / mod, with servitor as
  the installed-helper case.

**Not ready, stated honestly:**

- **The world-noun profile does not exist.** Chartulary is generic
  (containers, facets, nesting, attributed edits) and contains zero faction
  vocabulary; factions, places, characters, and history are Isometry types in
  `isometry-campaign`. A portable profile is **extracted after two real
  consumers**, never declared in advance. Do not describe world nouns as
  "chartulary-typed."
- **The missing arrow is adoption, not wire.** `isometry-campaign` does not
  yet lower to or recover from `mere.pack/v1`.
- **Genuinely new organs** the wing needs and the stack does not have:
  real-time netcode (Isometry's current tactical lane is deliberately ordered
  and turn-based), voxels as live attachable bodies and possibly playable
  volume (Isometry keeps them as an asset substrate), and settlement/
  production simulation.
- **Parked deliberately:** R³ fields (numen is R² today, and numen/quint are
  a field algebra — an influence-map substrate — while seiche is rapier2d
  graph layout, so Mesocosm takes its chosen Rapier2D or Rapier3D dependency
  directly, with seiche's reconcile-a-physics-world-to-a-host-graph pattern
  as the template).

**Reconciled 2026-08-07 (audit).** This annex was checked 2026-07-29/30
and the week moved under it:

- "Voxels as live attachable bodies and possibly playable volume" is no
  longer a missing organ: volumetric world truth landed additively
  (place-graph plan G0/G1: grown graphs, brick `Ground`, roofed burrows,
  carve lifecycle), world adoption pending; and bodies develop from
  recipes into voxel parts (`development.rs`, PD1a).
- "Mesocosm takes its chosen Rapier2D or Rapier3D dependency directly" is
  **superseded** by the three-tier physics ruling (place-graph plan
  §0.10): integer authority, parry-plus-owned-kinematics advisor, GPU
  ambience, rapier in reserve.
- Renderer posture is now governed by the landscape's §8.9 cohesion
  contract, with renderling the lead mesh-tenant candidate on receipts
  (device unity proven, wgpu-29 fork green 95/95).
- The ecology gained E0-E4 implementation slices (general model plan;
  acceptance gates open).
- Still genuinely missing, unchanged: real-time netcode and
  settlement/production simulation (now Paredros S5's charge).

**Discipline:** the platform is extracted from shipped games, never built
platform-first. Mesocosm is a *candidate* for that proof, not yet a real
second consumer.

---

## 6. The next architectural threshold — **CROSSED 2026-07-31**

Not a federation platform. **A portable world profile proven by two actual
games.**

**The proof pair is built and passing**, both directions, with committed bytes
rather than agreeing types. Two schemas ride one framing: `mesocosm.body/v0`
carries appearance to `isometry-voxel`, and `mesocosm.chronicle/v0` carries the
record to `isometry-campaign` and back. Neither repo depends on the other.
Receipts and findings in the
[execution waves plan](2026-07-31_execution_waves_plan.md) §1.4 and §2.4.

Three things the build settled that this section had not:

- **The v0 profile is an appearance projection, not the body document.** Putting
  `BodyDocument` itself on the wire would force the reader to link
  `mesocosm-core`, a type dependency wearing a data dependency's clothes. The
  proof instead carries primitives: a flattened grid, a parallel attribution
  grid naming the part behind each voxel, and flat per-part provenance.
  **Per-part provenance was the right keystone.** V0's omission of parent links
  was a proof limitation, not the permanent contract; v1 can carry primitive
  topology while preserving local mirror types.
- **The record and the appearance are separate artifacts.** A chronicle carries
  no geometry, and v1 also stops using it as a second flat anatomy snapshot.
  Re-entry of a descendant regrows a body; return of the same individual keeps
  the addressed body revision.
- **Law C's proof belongs in the consumer, and size is a tell.** A generator
  that only makes small creatures breaks the law with no marker at all, because
  the part count sorts them.

The pair as originally specified:

1. Mesocosm mints an organism.
2. Isometry imports it as a token, preserving the v0 appearance and flat
   provenance through local mirror types. V1 adds opaque topology and stable
   subject/body addresses without making Isometry interpret Mesocosm rules.
3. Isometry appends a played history.
4. Mesocosm reads the descendant back without either side losing facts.
5. The same slot accepts an RNG-authored organism indistinguishably (Law C).

This forced **interchange profile v0** into existence as the wing's first
portable artifact. Its implemented fields are a flattened voxel projection,
per-cell part attribution, flat **per-part provenance**, species, mass, and
collision hints. The fuller topology once proposed for v0 is now the scoped v1
contract in the
[wing phenotype plan](2026-07-31_wing_phenotype_contract_plan.md): stable
subject and body-revision identity, primitive parent links, source addresses,
and optional projection data.

Projection recipes are optional on purpose. `isometry-voxel` recipes are an
excellent first projection codec and probably Isometry's, but making them
canonical would let today's renderer leak into the substrate. Each vessel
derives its own presentation from topology.

It rides `mere.pack/v1`, so no new wire format is invented. Keep schema
negotiation microscopic in v0: profile strings, versions, required and
optional capability sets. Resist building a negotiation framework before two
consumers exist.

**Held to, and narrowed further 2026-07-31.** Negotiation is one magic and one
`u16`, checked before any payload is decoded — a version cannot live *inside* a
postcard payload, because a decoder cannot reach a field whose layout just
changed. The pack envelope itself is deliberately not wired yet: a pack carries
content-addressed blobs, and these are what goes inside one. Wiring the envelope
means depending on eidetic, and the platform is extracted from shipped games
rather than built before them.

One rule the round trip added: **a verb two games both act on is a contract, not
vocabulary.** A game's own verbs are opaque and may carry any payload; a shared
verb needs an agreed one, or the reading game is inventing consequences for
another game's fiction. The shared vocabulary is one verb long, and each
addition is a coupling two vessels must keep in step forever.

---

## 7. Vocabulary

Names verified on crates.io and against games, studios, and trademarks on
2026-07-30 unless noted. Stub publication and any trademark filing are the
maintainer's manual step.

| Word | Role |
| ---- | ---- |
| **Mesocosm** | Vessel 1. Ecology's mid-scale enclosed experimental ecosystem. The simulated enclosure is mesoscopic even when a playable one-trait critter is cellular; each generational run is one experiment in it. |
| **Paredros** | Vessel 2. "The one who sits beside": the Greek Magical Papyri's acquired companion, and in classical civic use an assessor seated beside a magistrate. A colleague, not a servant. |
| **critter** | The plain organism word, wing-wide. |
| **animula** | The played soul in Mesocosm: Hadrian's *animula vagula blandula, hospes comesque corporis*, the little soul that guests in a body. In-product term only — **ANIMULA NOOK** is a live Tencent mark in Class 9 game software, so the word must never title a game. |
| **kleptoplasty** | The incorporation mechanic. Real biology: an organism eats algae and retains the functional chloroplasts. |
| **metabolize** | Mesocosm's single verb: world into self, self into world. |
| **fili** | Lineage across worlds. Reserved in `mere/design_docs/TERMINOLOGY.md` for moot ancestry, forks, and genealogy. Not event history, not content descent. |
| **tulpa** | The legend and memorial organ: what memory makes of history, sustained by continued attention. Proposed 2026-07-30, crates.io free, **lexicon inscription still pending the maintainer's ruling.** |
| **borg** *(provisional)* | A **named** critter — the concept ruled 2026-07-31, the word not yet cleared. Made incidentally by playing Mesocosm: a critter accrues enough record to stop being a statistic. Carries a Gotcha Force loan and an IP shadow; see open question 3. |
| **character** | A **faction-associated** borg, made by playing Paredros. Not a new coinage — Isometry already uses `character` for the same artifact, so the word is agreement between two vessels rather than a fourth term. |
| **deme** | Banked, unspent. Biology's local interbreeding population; the leading candidate for Mesocosm's unit word if one is wanted. |

Also banked clean and unspent: **coppice**, **diaspore**, **holobiont**.
Killed with receipts: *zoophyte* (rejected on taste), *imago*,
*palingenesis*, *ecotone*, *loam* (all occupied), *kleptocosm*, *idiocosm*,
*ipsocosm* (argued down on tone, meaning, and register).

**Collision to respect:** the bare word *flora* is already spoken for
platform-side (a moot's accumulated engrams and geist). Game vocabulary must
not reuse it.

---

## 8. Open questions

Carried forward deliberately. Each needs a ruling before the work it gates.

1. **Licensing — adopted 2026-07-31.** Game code and repository
   documentation use **MPL-2.0**. Separately identified reusable library
   crates use **MIT OR Apache-2.0** after their boundary is proven. Original
   game assets use **CC BY-SA 4.0**, with per-asset attribution and explicit
   notices for imported material.

   One correction to carry into the decision: **CDDA is CC BY-SA 3.0 for
   code and content both**, which is an unusual choice and not a good model
   to copy literally. Creative Commons themselves advise against CC licenses
   for software — they carry no patent grant and do not address
   source-versus-object distribution, both of which matter for a Rust
   project with dependencies. The adopted split uses a software license for
   code and a culture/content license for assets. Each game repository keeps
   a `LICENSES.md` scope record so the permissive library texts cannot be
   mistaken for a dual license on MPL game code.
2. **Render and engine stack.** Given its own research doc:
   [`2026-07-30_engine_and_render_lane_landscape.md`](2026-07-30_engine_and_render_lane_landscape.md).
   The earlier framing here (Bevy versus custom wgpu) was a false binary and
   is superseded. Short version: Fyrox is the forkable engine and the only
   Rust one with an editor; "custom" is an **assembly over an existing host
   skeleton** — winit, netrender/vello, isometry-voxel, Firewheel, armillary,
   codicil/muniment, numen/quint, cambium, parley, and rapier via seiche are
   all owned — but the shelf is **not an engine**, and the missing middle is a
   coherent game runtime (fixed timestep, authoritative world,
   snapshot/replay, input actions, asset graph, scene representation, camera
   and animation, spatial queries, game audio, inspection). The renderer gap
   is a 3D renderer and a mesher. Renderers need not be shared across vessels
   — the restated law binds world identity, provenance, and causal history,
   not the pixels — so splitting the bet is live.

   Carries Mark's proposed dimensionality (Mesocosm 2 or 2.5D, Paredros a
   close camera, Isometry a distant one) with two flags: **camera distance is
   not person** (say close camera, never "first-person Paredros"), and **a 3D
   Isometry contradicts a standing ruling in that repo**, which needs its own
   plan there rather than arriving as a side effect.

   **Closed 2026-08-18.** The camera decisions are ruled in
   [the vessel briefs and presentation record](2026-08-18_vessel_briefs_and_presentation.md):
   Mesocosm is a side-on terrarium section with a trait-graph board for
   the epoch review; Paredros is third-person 3D on one continuous
   Kenshi-style zoom, with first person composable as a Paredros-only
   setting under the guardrails; Isometry stays isometric. Person and
   care rulings here are untouched.

   **Decided 2026-07-31 for Mesocosm**: a small custom wgpu body renderer,
   with netrender owning the device and compositing, built headless-first so
   visibility is testable. The engine lane is dropped, because this game's
   geometry is flat-shaded palette quads with no textures, skinning, or
   authored materials, and an engine's rendering value sits in exactly the
   parts it does not need. Renderling was rejected on its pinned-nightly
   rust-gpu requirement rather than on staleness; vello stays a live option if
   the look settles 2.5D, since it has no depth buffer and would change the
   rendering approach rather than the host. Reasons and costs in the
   [execution waves plan](2026-07-31_execution_waves_plan.md) §1.3.

   **Paredros is not bound by this.** Renderers are per-vessel, and a close
   camera with real lighting is the case where an engine, or Renderling, would
   earn its keep.
3. **Paredros' unit word — resolved 2026-07-31, though not as this question
   assumed.** It had "borg" pencilled in as Paredros' word. Mark's continuity
   ruling (§1) puts borg in *Mesocosm's* output and gives Paredros
   **character**: a critter is an organism, a borg is a named critter, a
   character is a faction-associated borg. So Paredros' unit word is
   `character`, which is also already Isometry's word for the same artifact —
   agreement across two vessels rather than a coinage.

   **What is still open is the word "borg", not the concept.** The concept is
   now ruled and load-bearing: the wing needs a noun for *a critter that has a
   name*. The word carries a Gotcha Force loan and an IP shadow, and that
   caution has not been withdrawn — this repo's `CLAUDE.md` still lists borg
   under terms not to use. Treat "borg" as **provisional shorthand for a ruled
   concept** until it survives the usual crates.io, game, studio, and
   trademark checks or a replacement is chosen. **deme** is banked and is not
   a candidate here: it names a population, and this concept names an
   individual.

   The battle-frame noun — the machine a character pilots, if Paredros keeps
   the Gotcha Force silhouette — remains genuinely unnamed and is a separate
   question from the unit word.
4. **Hagiograph's inscription and shape** (named tulpa in this section's old
   sense; renamed 2026-09-02, see repo `CLAUDE.md`), including the attention
   mechanic that Law B depends on.
5. **Fili v0**: home (beside chartulary and codicil in mere's eidetic
   family), data model, and relationship to `chartulary::stemma` — build
   beside it, not on it.
6. **Attachment-creep levers.** Relationship drift the player cannot
   influence reads as random punishment (the Darkest Dungeon 2 lesson).
7. **Co-op**, deferred to last in every vessel, but no longer shapeless.
   Two concrete shapes now exist: **asymmetric co-op** (one player embodied,
   one strategic), which the care-granularity relaxation made buildable and
   which is the natural home for the shared-burden friction the influence set
   started from (the Crystal Chronicles chalice, Four Swords); and **visiting**
   (your character arrives in another player's settlement — Hammerwatch's
   "bring your own hero", and at world scale the graft the lineage model
   already describes).

   **The two shapes want different netcode, and conflating them would be a
   mistake.** Shared-session co-op (both players in one world at once) is
   deterministic-lockstep-shaped. Visiting is authority-plus-state-transfer
   shaped, which is closer to what the pack, grant, and signed-space machinery
   already does.

   **The lockstep half got much cheaper than feared** (studied 2026-07-30 via
   [Tangle](https://github.com/kettle11/tangle)). Rollback multiplayer need not
   be hand-written: if the simulation is a pure function of seed and ordered
   inputs behind a wholesale-snapshottable boundary, peers exchange only inputs
   and the runtime does the rest. Tangle gets this from WebAssembly's linear
   memory, where capturing the world is a memcpy — and this stack already runs
   wasm in-browser, in the participant gate, and in packs. The constraint is
   recorded in the body pipeline plan §R0, and it is the piece with a deadline
   because it is nearly free to design in and brutal to retrofit.

   Two structural gifts: the epoch loop's **adaptation phase is turn-based and
   therefore trivially co-op-able**, so only the epoch half needs the hard
   machinery; and asymmetric co-op is free in this model, since the strategic
   player is simply another input stream. One API rule regardless of
   implementation: **co-op must not appear in a game core's API at all.**

   Mesocosm now supplies the first concrete same-lineage conflict rule. Two
   writers may propose adaptation from one parent revision. Explicit agreement
   adopts one shared child; disagreement lets the proposer follow a child
   branch while preserving the other continuation. This is a domain
   materializer applying **adopt or branch**, not evidence for a universal
   CRDT. The detailed rule belongs to the
   [epoch-boundary plan](2026-08-01_epoch_boundary_plan.md).

   Honest caveats: whole-heap snapshotting scales with heap size and an ecology
   is a large-mutable-state profile; cross-platform float determinism is the
   classic killer; and Tangle itself is web-only, TypeScript-hosted, and last
   pushed July 2024, so the *technique* is the transferable part rather than
   the library. Still deferred; no longer unexplored.
8. **The constellation boundary.** Two vessels are named. A colony game or
   strategy game as further vessels is not ruled in; vessels earn existence
   by shipping.
9. ~~Where Gotcha Force lands.~~ **Resolved 2026-07-30**, and my proposed
   answer was wrong in an instructive way. The arena is not a mode and was
   never missing: **the arena is ecological competition itself** — a lineage
   competing for resources and a niche in a strange world. Collection also
   runs the other way round from how I first put it: the *world* collects
   lineages, and the player's displace generated ones, which is Law C seen
   from inside the fiction. Gotcha Force's point budget stays in Mesocosm as
   the **metabolic budget**, not as a roster cap. What genuinely belongs to
   Isometry is **taming** — collecting critters and characters into a faction
   roster, which is befriending a goblin or taming a dog, and which Isometry
   already implements as `convince`. Detail in the Mesocosm founding plan.
10. **Run rhythm across the wing** (2026-07-30). Mesocosm runs generations,
    Paredros runs expeditions against a settlement that keeps (the Heroes of
    Hammerwatch shape), Isometry runs campaigns. One rhythm, three scales,
    and it delivers the sortie-and-return through-line under half the
    influence set. It also produced the wing's first concrete co-op design:
    your character visits another player's settlement, which is
    "bring your own hero" and, at world scale, the graft the lineage model
    already describes.
