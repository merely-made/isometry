# CLAUDE.md — Paredros Repository Role

**Repository location, 2026-09-09:** this product now lives in Isometry's
`paredros/` directory, with its existing Cargo workspace and package names.
Read `../CLAUDE.md` for umbrella guidance. The root repository index is
`../design_docs/DOC_README.md`. Shared wing design is now in the sibling
`../mesocosm/design_docs/` inside the same repository. Git operations affect
the whole umbrella; preserve other products' WIP and stage intended paths.

This file defines how Claude Code should behave in this repository. Read it
first when starting any session.

---

## Project Identity

**Paredros** is a second-person action RPG in a persistent generated world.
You name one creature and ordinarily inhabit that life until it dies. Other
named creatures live independently across settlements, dungeons, ruins,
surface and underground places. They may become allies, enemies, neighbors,
or strangers; none is a unit in a party, and none is required for the player
to build, explore, or live.

Vessel 2 of a three-game wing — Mesocosm (first person), Paredros (second
person), Isometry (third person) — that shares a world substrate, a lineage
model, and a trust plane. Sharing engine organs is encouraged where the
organ stays verb-neutral (ruled 2026-08-05, wing founding record §1); the
vessels still do not share a genre, a schedule, or their verbs. The wing's
question, ruled 2026-08-07: **continuity under transformation** — here,
whether a community remains itself as control, bodies, and generations
change.

**Early implementation.** The repo holds the name-reservation package, the
design docs, and five crates. `crates/paredros-room` is the S0 room probe
landed 2026-08-08: one room carved into a grown mesocosm hillside, one body
under near-tier kinematics, a fixed input trace with save/reload/replay, and
a headed run presenting netrender's composed master with a renderling room
in it. Its default `r1-proof` profile runs the real room and perspective
camera through the shared brick DDA, now owned by Mere's `conatus-brick`
(pinned by rev); Paredros constructs the shared `BrickMap` from its own
Ground binding. Three further gates landed as opt-in bins: `v1_residency`
(continuous-zoom residency, V1/V1a), `d1_depth` (raymarch depth composed
with renderling, D1), and `v1b_residency` (the stable capacity-fixed
resident brick cache, V1b), behind the `v1-proof`, `d1-proof`, and
`v1b-proof` features. `crates/paredros-social` is the S1
willingness owner landed the same
day: deeds, standing, confidence, refusal, standing agreements, and the
premises behind every answer, with the refusal scene as an executable
receipt. S2 (landed 2026-08-08) added the settlement in peer-agency form
to the same crate: homes offered with daily work, residence and the
daily round derived from agreement state, so moving out is the agreement
ending. `crates/paredros-sortie` is S3's joint receipt (sim half landed
2026-08-08): the one crate reading both owners, with negotiated
participation, terrain falls as body-revision wounds, the pact-governed
tag-in, the dig rule, and sortie deeds that explain later answers.
`crates/paredros-identity` holds the identity facts both owners
share and neither may own. `crates/paredros-world` owns persistent site
meanings over stable surface and underground slots, routes, containment,
inherited replacement, multi-author material edits, generated bodies and
items, needs, perception, injury, recovery, death, and regrow-plus-replay
saves. `Movement`, `Bodies`, and `Items` are separate multi-subject systems;
`GameState` coordinates them through one subject-addressed transition grammar
with no control-specific path. `Population` owns deterministic site and
migration origins; `Projects` owns durable goals and replayed completion;
`Simulation` advances every living subject through the same game intents with
no observer or selected-subject input. Navigation remains derived advice.
Traversal and named-life scenarios belong to receipts, not production
vocabulary. Its direct Mesocosm core dependency is current shared-organ
evidence, not settled permanent ownership. F0-F2 are closed; F3 memory,
belief, and standing is active, with F3a (pointable memory and belief)
landed 2026-08-26. The executable plan is
`design_docs/2026-08-07_paredros_execution_plan.md`; S0-S3 are retained as
foundation receipts, while the 2026-08-13 fundamental-layer ledger now owns
future ordering. The founding plan remains the charter with its phase section
superseded.

The `crossing` binary (2026-09-05) is the dry damaged-crossing contact
fixture: two restartable body presets, board carrying, tethering, brace,
timed strikes, impairment/recovery, and a readable shared-device HUD.
`paredros-world::ContactWorld` is a separate fixed-step input/replay probe
using Conatus character movement, not yet a join to `GameState` or F3 evidence.
Its stationary practice body has no decision system. Full encounter and
playtester acceptance remain open in the execution plan.

See `design_docs/PROJECT_DESCRIPTION.md` for the product description,
`design_docs/DOC_README.md` for the doc index, and the wing-level
architecture in the sibling repo at
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`.

## Terminology

- **critter**: the plain organism word, wing-wide.
- **character**: **this game's unit word, ruled 2026-07-31.** A
  faction-association added to a borg, which is itself a name added to a
  critter — `character(borg(critter))`, which this repo's founding plan
  already described as one stable subject with independently versioned
  profile references. Not a coinage: Isometry uses `character` for the same
  artifact, so the two vessels agree rather than each inventing a word. A
  faction is a *relationship*, not a property, which is why the second-person
  vessel is the one that mints characters.
- **borg** *(provisional word, ruled concept)*: a **named** critter, made
  incidentally by playing Mesocosm. The concept is settled; the word still
  carries a Gotcha Force loan and an IP shadow and has not passed the usual
  checks. Prefer `character` here — borg is Mesocosm's output, not this
  game's. See the wing founding record §1 and open question 3.
- **The battle-frame noun** — the machine a character pilots, if this game
  keeps the Gotcha Force silhouette — **remains unnamed.** It is a separate
  question from the unit word, and not a gap to fill casually.
- **companion / peer**: a relationship to another named creature, not a
  required entourage or roster slot. Never "unit" or "party member" — both
  imply command.
- **succession**: play continuing through another subject after death. An
  existing connected creature and a newly generated outsider are both valid.
- **fili**: lineage across worlds. Not event history.
- **tulpa**: the legend and memorial organ — persistence through memory when
  no one carries the line. Proposed, **not yet inscribed in mere's lexicon**;
  treat as provisional.

Do not coin new names for these concepts mid-session. Naming rounds are
deliberate here: candidates get crates.io, game, studio, and trademark checks
before adoption, and the receipts are recorded.

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

Wing-level material lives once, in Mesocosm, and is cited by path. Never copy
it here.

## General Guidelines

- Rust: standard idioms. No `unsafe` without documented justification.
- 600-LOC ceiling per source file. Split before adding when approaching it,
  and trim comment volume while splitting.
- Plans go in `design_docs/` per the date-keyword-plan convention with
  done-conditions, not time estimates. Never `.claude/plans/`.
- Follow `DOC_POLICY.md` for documentation changes.
- Check the Merely ecosystem before writing a new module: mere, genet,
  netrender, isometry, mesocosm, and the wgpu-* repos may already have the
  piece or the pattern. Name the owning layer before building anything
  app-local.
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

- **Do not add real-time puppeteering of a party.** This is the scope canary,
  narrowed 2026-07-30 when the wing replaced person purity with care
  granularity (wing founding record §1). Paredros is care for **individuals**;
  drift means care widening to a squad you administer, which is Isometry's
  granularity. Permitted: *configure, don't command* (standing behaviour
  agreed in advance, which a peer may refuse). Tag-in may exist as an optional
  player rule or explicit world process, but ordinary play stays with one
  named creature until death. Forbidden: free roster selection and
  moment-to-moment orders to several characters at once.
- **Do not make body switching a menu operation in ordinary play.** Control
  changes through death or a recorded world event. Possession, domination,
  transplantation, cloning, resurrection, and similar exceptions must leave
  material, causal, and social consequences. Record what occurred; do not
  collapse disputed continuity into a universal `same_person` flag.
- **Do not violate the three pipeline laws** (wing founding record §3). What
  crosses between games is choices under scarcity, not morphology;
  inheritance must be pointable; player history displaces procedural content
  and never gates it.
- **Do not build a Nemesis system.** Procedurally generated rivals with
  promotion hierarchies are patented to August 2036. Generic grudges and
  remembered encounters are fine (Dwarf Fortress prior art); the
  rival-hierarchy-promotion machinery is what to design around.
- **Do not ship the social simulation without its legibility surface.** Depth
  nobody notices reads as procedural noise; the Legends-mode equivalent is
  day-one work, not polish.
- **Do not let relationship drift run without player levers** (the Darkest
  Dungeon 2 lesson): drift the player cannot influence reads as random
  punishment.
- Do not add rollback netcode or a universal CRDT world state speculatively.
  Single-player action comes first, but signed multi-writer world and
  settlement authoring remains allowed. Additive operations preserve
  concurrent claims; each collaborative domain must name its materializer,
  conflict UI, and any true CRDT it actually needs.
- Do not add features beyond the active plan's current target without
  surfacing the scope change first.
