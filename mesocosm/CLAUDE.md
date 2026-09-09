# CLAUDE.md — Mesocosm Repository Role

**Repository location, 2026-09-09:** this product now lives in Isometry's
`mesocosm/` directory, with its existing Cargo workspace and package names.
Read `../CLAUDE.md` for umbrella guidance. The root repository index is
`../design_docs/DOC_README.md`; this product's local index remains below.
Git operations from here affect the whole Isometry repository. Preserve
other products' WIP and stage only the intended paths.

This file defines how Claude Code should behave in this repository. Read it
first when starting any session.

---

## Project Identity

**Mesocosm** is an ecological roguelike of lineages: you are a critter in an
enclosed mid-scale ecosystem, presented as a terrarium section seen at a
shallow oblique (ruled 2026-08-18 as side-on; tilted to oblique by Mark
2026-09-04 after a measured three-camera slice; first person names the
agency, never the camera). You
grow only by incorporating other organisms, and you try to increase your
lineage's share of the world's biomass without triggering a trophic
collapse. Play runs in epochs: live the round in the terrarium, then
review and revise your organism on the trait board, in initiative order
by biomass. Rogue Legacy's generational loop pointed at an ecosystem
rather than a castle. See
`design_docs/2026-08-18_vessel_briefs_and_presentation.md`.

Vessel 1 of a three-game wing — Mesocosm (first person), Paredros (second
person), Isometry (third person) — that shares a world substrate, a lineage
model, and a trust plane. Sharing engine organs is encouraged where the
organ stays verb-neutral (ruled 2026-08-05); the vessels still do not share
a genre, a schedule, or their verbs.

**Early implementation.** The repo has a deterministic simulation core, body
pipeline, renderer, windowed host, epoch lab, and a proven Isometry projection.
The repeated game loop and phenotype bridge are still under design and playtest.

See `design_docs/PROJECT_DESCRIPTION.md` for the product description,
`design_docs/DOC_README.md` for the doc index, and
`design_docs/2026-07-30_games_wing_founding.md` for the wing-level
architecture that Paredros and Isometry also depend on.

## Terminology

- **critter**: the plain organism word, wing-wide, and the default. Not
  "creature".
- **borg** *(provisional word, ruled concept)*: a **named** critter. Ruled
  2026-07-31 — a critter is an organism, a borg is a named critter made
  incidentally by playing Mesocosm, a character is a faction-associated borg
  made by playing Paredros. The **concept** is settled and load-bearing; the
  **word** still carries a Gotcha Force loan and an IP shadow and has not
  passed the usual checks. Use `critter` for organisms generally and reach for
  borg only where the naming is the point. Do not treat this as clearance to
  title anything Borg. See the wing founding record §1 and open question 3.
- **animula**: the played soul — the little soul that guests in a body,
  across generations. In-product term only. **Never title anything Animula**;
  ANIMULA NOOK is a live Tencent mark in Class 9 game software.
- **metabolize**: the single verb. World into self, self into world.
- **kleptoplasty**: the incorporation mechanic, named for the real biology
  (an organism eats algae and keeps the working chloroplasts).
- **deme**: banked, unspent. Biology's local interbreeding population; the
  leading candidate if a unit word is wanted. Do not spend it without asking.
- **fili**: lineage across worlds (forks, campaign descent, cross-moot
  grafts). Not event history, not in-world biological descent.
- **hagiograph**: the memorial subsystem. It promotes unprecedented,
  legendary, and narratively significant events out of the timeline to be
  handled by the rest of the stack's procedural voxel engine. This is what
  design docs before 2026-09-02 call "tulpa" in its old, memorial sense; that
  sense is retired. Authorized by Mark, 2026-09-02.
- **tulpa**: gemot's federated adapter-training lane (personal LoRA). Not the
  memorial organ; see **hagiograph** above. Renamed 2026-09-02; do not use
  it in the old sense going forward.
- **nis**: living substance that still carries where it came from, matter
  typed by provenance, kingdom first and lineage under it. One form for
  singular and plural. From nisus, striving: a nis is a lot of matter that
  still bears its line's endeavour, and matter fully returned to soil has
  lost its nis and is untyped stock. Ruled by Mark 2026-09-04 after a
  naming round (figment failed: Rocket's config crate, a live Bedtime
  Digital title, and the wrong sense). See the
  [playable ecology plan](design_docs/2026-08-31_playable_ecology_plan.md)
  §6 ruling 4. Do not write "element" for this concept.
- **scruple**: a part's stable heterogeneous mix, a measured lot of nis; the
  part layer of composition. The apothecaries' weight of twenty grains, free
  on crates.io, only a plural board-game mark nearby. Ruled by Mark
  2026-09-04 with nis.

Do not coin new names for these concepts mid-session. Naming rounds are
deliberate here: candidates get crates.io, game, studio, and trademark checks
before adoption, and the receipts are recorded.

**Collision to respect**: the bare word *flora* is spoken for platform-side
(a moot's accumulated codicils). Game vocabulary must not reuse it.

## Document Structure

All authoritative design material lives in `design_docs/`. Read
`design_docs/DOC_README.md` first.

| Path | What's there |
| ---- | ----------- |
| `design_docs/DOC_README.md` | Index and AI working principles |
| `design_docs/DOC_POLICY.md` | Documentation governance |
| `design_docs/PROJECT_DESCRIPTION.md` | Product goals, pillars (maintainer-owned) |
| `design_docs/<date>_<keyword>_plan.md` | Active plans |
| `design_docs/archive_docs/<date>/` | Retired plans |

Wing-level material lives once, here, and is cited by the sibling repos.
Do not copy it into Paredros or Isometry.

## General Guidelines

- Rust: standard idioms. No `unsafe` without documented justification.
- 600-LOC ceiling per source file. Split before adding when approaching it,
  and trim comment volume while splitting.
- Plans go in `design_docs/` per the date-keyword-plan convention with
  done-conditions, not time estimates. Never `.claude/plans/`.
- Follow `DOC_POLICY.md` for documentation changes.
- Check the Merely ecosystem before writing a new module: mere, genet,
  netrender, isometry, and the wgpu-* repos may already have the piece or the
  pattern. Name the owning layer before building anything app-local.
- Prefer runtime verification over extended static code tracing. If runtime
  diagnostics are blocked, surface that blocker early.

## Licensing Boundary

- Game code and repository documentation are MPL-2.0.
- Promoted reusable library crates are MPL-2.0 as well (ruled 2026-09-03;
  the earlier MIT OR Apache-2.0 path for a proven boundary is retired). The
  only exception route is the fork/vendor criterion in mere's license posture
  brief, recorded in `LICENSES.md` when it applies.
- Original game assets are CC BY-SA 4.0 and require an attribution entry.
  Imported assets retain their own licenses and must be recorded explicitly.
- See `LICENSES.md`. Do not blur code, library, and asset grants.

## Important Don'ts

- **Do not violate the three pipeline laws** (games wing founding record §3).
  What crosses between games is choices under scarcity, not morphology;
  inheritance must be pointable; player history displaces procedural content
  and never gates it.
- **Do not let a stage grow its own engine.** All stages are rule-dressings
  over one substrate. This is the anti-Spore insurance and the wing's single
  most load-bearing rule.
- **Do not describe world nouns as "chartulary-typed."** Chartulary is
  generic (containers, facets, nesting, attributed edits). Factions, places,
  characters, and history are Isometry types today. A portable profile is
  extracted after two real consumers, never declared in advance.
- **Do not build the federation platform first.** It is extracted from
  shipped games. Mesocosm is a candidate for that proof, not yet a consumer.
- Do not add rollback netcode or a universal CRDT world state speculatively.
  Signed multi-writer authoring is already real in Isometry and is permitted:
  additive operations converge without erasing concurrent claims. Each domain
  still names its materializer and conflict rule; introduce a true CRDT only
  when that domain proves it needs mergeable concurrent values. Live action
  remains separately ordered or authoritative.
- Do not add features beyond the active plan's current target without
  surfacing the scope change first. **The invariant is care granularity, not
  person purity** (relaxed 2026-07-30; wing founding record §1). Mesocosm is
  care for a **species**. Person may shift — the adaptation phase is
  deliberately third person — provided first person stays home, the shift is
  bounded and diegetic, and each layer could be removed with the game still
  standing. Refuse any shift that needs a second simulation **authority**, or
  that duplicates functionality the stack already owns (narrowed 2026-08-05):
  parallel authorities are the multiplier that actually hollowed Spore.
  Projections stay plural and cheap; see the place-graph engine plan §0.
