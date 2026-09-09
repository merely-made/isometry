# Vessel Briefs and Presentation (2026-08-18)

**Status: ratified 2026-08-18 (Mark).** The fullest statement yet of what
each vessel is, plus the camera and presentation decisions the founding
record deliberately left open ("camera distance is not person"; §8's
dimensionality carry). Wing-level: sibling repos cite this, never copy it.

The founding record's laws are untouched: care granularity is the
invariant, person is agency rather than camera, home persons stay
first/second/third for Mesocosm/Paredros/Isometry.

## 1. The briefs

### Isometry

An isometric, procedural voxel VTT. Local, region, and world maps;
turn-based gameplay; exploration, combat, and interaction loops;
pluggable rulesets. Someone can design as much of a world as they want:
NPCs, reputation, preferences, dialog trees, enemies, items, abilities,
location maps, and possibly divinatory, ideological, and religious
systems. They can bring their own inference to generate more from the
game's existing context.

Worldbuilding is critical: the system must support comparable rulesets,
an SRD-flavored ruleset with a different world binding because that
world's magic system, physical properties, or creatures and ecology
differ.

**First release goal:** run a PF2e- or SRD-compatible campaign end to
end through Isometry. The PC/GM split is **Role and Creator modes**,
which participants can be allowed to switch between depending on the
session's rules, who convened it, and what rules everyone accepted when
joining.

### Paredros

An adventure-mode take on the same world and character data, focused on
developing the individual's story. Real-time action RPG: control your
character, explore the world, gather a coterie, build a base. **Kenshi
made of voxels** is the best reference. Factions, cities, bases, NPCs,
and legends develop naturally and accrete on a homebrewed world through
solo, parallel, and co-op play, in a world that Isometry can share.

### Mesocosm

An ecological roguelike focused on developing the lineage of organisms
in the same world. Goal: increase your lineage's share of the world's
biomass without triggering a trophic collapse. Organisms are realistic
body-plan projections of your inventory of traits, where which traits
are proximate to which triggers unexpected synergies or contradictions.
You play through each epoch; at the end you review the round, revise
your organism, and potentially apply modifiers to the next epoch if you
have the requisite traits, in an initiative order determined by largest
biomass first.

Because worldstate is shared, Mesocosm writes the foundations of the
world the other games are played in: organisms can change the planet
the way algae did on Earth. Fantastical elements are welcome, per the
existing general-model research.

Multiplayer in all three rides the stack (mere, gemot, murm), nothing
handrolled.

## 2. Presentation rulings

### Mesocosm: the terrarium section

Side-on orthographic cross-section of the voxel world, a few voxels of
depth with parallax. Mark's grounds: rendering simplicity, mechanical
complexity, and speed of the game loop. Design grounds recorded with
it: ecology is only legible from outside the eye (the dread of an
ecology is watching the predator exist before it matters to you);
vertical structure (canopy, surface, soil, water table, burrow) is
ecology's native axis and a section presents it directly; and a section
keeps world-sharing honest, since a tunnel dug in Paredros is simply
there.

Presentation splits with the loop: the live epoch is the terrarium
section with direct control of your organism; the end-of-epoch review
is the trait graph itself as a diagram board, adjacency spatially
legible because adjacency is the mechanic, with the body-plan
projection as the compile step. This extends the 2026-08-06 Rain World
pull-back ruling and closes it.

Home person stays first (agency: you are the critter); the camera is
not the person.

**Amended 2026-09-04 (Mark): the section is tilted, not level.** The
ruling above stands in every part except the word "side-on". A measured
three-camera slice (DC4, Q9 — one golden replay held at one tick,
photographed down `-z`, down `-x`, and at a twenty-degree oblique) found
that a level section draws every body end-on, because segments chain
along `+z`: the whole part budget of a thirty-part animal lands in one
pixel column. The oblique arm is the only one that shows a body's parts
*and* keeps the vertical structure this ruling exists to present, so it
is now the shipped default. Everything the ruling gives as grounds is
unaffected — it is still an orthographic cross-section a few voxels
deep, still read from outside the eye, still vertical-first. Only the
axis moved, by twenty degrees on each of the two free rotations. The
measurement and its costs are in the default creatures plan's Q9
Findings entry.

**Direction accepted 2026-09-05 (Mark), prototype pending:** Mesocosm should
play primarily up/down/left/right with shallow depth, while allowing deliberate
quarter-turn views of a cutaway terrarium. Mark favours this combination of
2D legibility and 3D space. The earlier oblique result established a useful
view of individual anatomy; it does not settle scene composition or interior
visibility. The current default stays in place while a clearing-and-burrow
prototype tests the new direction. Exact pitch, framing, cutaway rules and
depth traversal remain prototype questions. The proposed sequence is body-part
inspection (VB3), then [CP1](2026-08-30_default_creatures_plan.md#cp1-clearing-and-burrow-camera-prototype).

### Paredros: third person, one continuous zoom

Full 3D, third-person follow camera pulling out through orbit to an
elevated tactical view, in Kenshi's register continuum: near acts, mid
leads, far plans. Companions legible at every zoom, because the
second-person vessel's story is witnessed. Two exclusions: not first
person as the default (melee and coterie awareness die there), and not
fixed isometric (that is Isometry's identity; cameras are most of what
keep vessels distinct over one world).

**First person as a setting, Paredros only.** Mark toyed with a
first-person Barony-like perspective, possibly too Minecraft-adjacent
as a default but composable with the zoom continuum as an option. Ruled:
the two compose, and only at the Paredros level. This is a camera
setting under the founding record's guardrails, not a person shift, and
it must not require a second simulation or renderer.

### Isometry

Isometric, by name and by its own repo's standing rulings.

### The rhyme, protected

One loop at three timescales: Isometry is the turn, Paredros the
moment, Mesocosm the epoch. Care widens with timescale (community,
individual, species) and camera intimacy runs inverse to it.
Initiative-by-biomass makes Mesocosm's epoch review a table scene,
Isometry's register one level up. When a feature blurs this symmetry,
that is the signal to stop and check.

### The shared chrome stack (ruled 2026-08-29, Mark)

The three vessels share one GUI stack: cambium view-fns in the genet
host for chrome that says things (menus, status bars, panels, sheets),
and painted overlays (sprigging leaves and vello scenes through
paint_list and netrender) for in-scene marks. The games are designed
differently, but they draw on the same stack — let them strengthen
each other and the stack. Isometry is the working proof of the text
half; Mesocosm's minimap and Paredros's chrome bar are the painted
half. Chrome surfaces are projections in mere's sense — a status bar
is a reading plus an encoding plus a realization — so the projection
grammar catalog (mere, 2026-08-15) supplies the vocabulary and no new
theory is needed.

Two clarifications this ruling settles:

- **Perspective governs the scene, not the chrome.** First, second, or
  third person is about the world view and what is diegetic inside it.
  It does not prohibit a title menu, a status bar, or a number.
- **No vessel is textless.** Mesocosm's "textless HUD" (views founding
  plan, 2026-08-02) was lane discipline — no ad-hoc lettering in the
  painted lane — that drifted into a de-facto text ban while the
  cambium lane waited for a consumer. Superseded; the amendment is in
  the views founding plan §6.

## 3. The shared-world fidelity contract

Mesocosm happens **before the world is formed** enough for all but a
few primordial beings to have names. Cross-vessel consistency is
therefore not pixel consistency: the requirement is **broad strokes
correct on the metadata, recognizable motifs and patterns**. Mesocosm's
section-view world projects into 3D for the other two on those terms.

**The primordial-name hook:** play your cards right in Mesocosm and
your organism can become a named primordial being referenced in
Paredros and Isometry. This is the deepest-timescale case of the
mechanism in §4.

## 4. Irrevocability: one mechanism at three scales

Each vessel needs a way for play to change the world permanently, and
they are the same mechanism at three timescales, all record-first:

- **The arc** (Isometry): a narrative arc defined in terms of the
  constituent elements of the world, authored or generated, that when
  developed changes the world irrevocably. Primitive: a named tension
  with stakes, fulfilled or betrayed, minting irreversible facts.
- **The unprecedented world event** (Paredros): an event that defines a
  person and enshrines their name in history, as a fact with provenance
  carrying a personae identity.
- **The epoch modifier** (Mesocosm): lineage outcomes that alter the
  planet's foundations for every later vessel.

Kenshi wants these outcomes and its world does not react (players
modded reactions in), because its state is scattered flags rather than
a record. The wing's third pipeline law (player history displaces
procedural content) plus the stack's attributed-fact record is the
architecture past that: generation consults the record, reputation is a
read over facts. Ratified 2026-08-18: "we have architected our way past
Kenshi's reactivity problem," with the stated hope of being more
extensible than the mods that patched Kenshi.

## 5. Ethics and alignment: bindings, not the nine-grid

Ruled 2026-08-18. The nine-grid (good/evil, lawful/chaotic) is retained
as a **ruleset binding**, because SRD compatibility requires it in the
data model. Note for PF2e compatibility: the PF2e Remaster removed
alignment in favor of edicts and anathema plus sanctification.

The arc primitive is the **commitment**, not the label: edicts and
anathema are testable in play, which is what an arc's fulfillment or
betrayal needs. The general shape is **axes as data**: ideological,
ethical, divinatory, and religious systems are per-world bindings
exactly as magic systems and physical properties are. Each world binds
its own; the SRD world binds the nine-grid.

## 6. Record hygiene

The founding record's §8 dimensionality carry is closed by §2 above; a
dated note there points here. Mesocosm's CLAUDE.md identity paragraph
is corrected in the same commit ("first-person game" implied a camera;
the person-is-agency ruling and this doc supersede that reading).

The engine consequences and their critical review are recorded separately in
[2026-08-18_engine_ecology_rulings_and_review.md](2026-08-18_engine_ecology_rulings_and_review.md).
That document is provisional and does not amend the vessel identities or
camera rulings here.
