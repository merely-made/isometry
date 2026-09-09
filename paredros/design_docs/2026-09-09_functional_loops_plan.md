# Paredros functional loops and wiring plan

**Status: in progress, 2026-09-09.** Lane design is recorded; bounded J0
body-sheet inspection and J1a controlled-session persistence are implemented
locally. This plan does not claim a
joined playable world, directional combat, construction, or adventure saves.

## Direction

Build systems that produce situations. A curated encounter is optional content,
not the prerequisite for developing material life, bodies, memory, or world
conditions. Existing F0-F8 milestones in the
[execution plan](2026-08-07_paredros_execution_plan.md) retain their semantic
done-conditions. This plan owns the next implementation dependencies across
them; the damaged crossing remains a reusable contact fixture.

The user requested five connected guarantees: injury affects the played body,
actions, attachments and inspection together; strange mechanics follow meaningful
world conditions; terrain changes reach creatures and rendering; saves preserve
consequences and continuation; and creatures answer from their own knowledge and
preferences. Hagiograph is a named lane alongside ordinary individual memory.

The current material and social vocabularies are intentionally small. Extend
them through actual operations rather than adding a catalog before its verbs.

## Existing owners and gaps

| Owner | Current implementation | Join still needed |
| --- | --- | --- |
| `paredros-world::GameState` | Coordinates world, movement, bodies, admitted anatomy, items, intents and events | Continuous contact effects and terrain work must enter this accepted history |
| `ContactWorld` | Fixed-step movement, board handling, attack/brace, integrity and grip/reach impairment; its own save | Bind runtime bodies to subjects and consume one durable body outcome |
| `EquipmentSession` | Authored subject, two dressings, attach/detach and restricted replay save | Safe stale/dead inspection first; host-selected subject and general session later |
| `Simulation` | Needs, navigation, population and autonomous actions over `GameState` | Controlled-subject scheduling and coordinated real work |
| `Projects` | Durable `Visit` goals | Material, repair, treatment and cooperation goals |
| `Society` / `EpistemicLog` | Agreements, deeds, observations, reports and corrections | Belief- and norm-supported answers; bounded recall |
| `Control` | Recorded begin/tag-in/tag-out/succeed pointer | Product validation of death, eligibility and outsider generation |
| `Sortie` | Older composed travel/wound/salvage/control receipt | Reuse its behavioral evidence; retire duplicate authority as the joined session adopts it |
| Mere / Mesocosm | Nisus voxel edits, Modulus traversal, Conatus spatial mechanics, body documents and generation | Product bindings and acceptance, not a new shared game evaluator |

Current Paredros dependencies pin a Mere revision. Inspect that pin's API before
adopting newer local Nisus/Modulus/Conatus work; a local implementation elsewhere
is not proof that Paredros's pinned dependency supplies it.

## Lane map

| Lane | Feature target | First useful completion | Dependencies |
| --- | --- | --- | --- |
| J: session and consequences | One played subject in the persistent simulation | Contact injury, inventory and inspection agree across reload | Existing world/body owners |
| B: body and directional action | Damage, treatment and limb-dependent attacks/defenses | Injury removes or changes a specific legal action and allows an alternative | J for playable join; pure rules can precede it |
| W: world conditions | Surgical risk plus independently composable magical laws | A conditional body operation and a stored/transmitted effect have explainable costs and counters | B for surgery; J commits; magic can precede surgery |
| T: materials and construction | Gather, carry, store, place, dig, repair | A paid-for terrain edit changes traversal and visible geometry together | J ordering; existing spatial/voxel mechanics |
| M: memory and judgment | Individual recall, preferences, beliefs and answers | An absent creature learns a report and changes its response | Accepted events; existing social owners |
| H: Hagiograph | Retelling, remembrance, significance and manifestation proposals | Remembered history changes a later interaction or admitted world proposal | M plus W for material manifestations |
| S: save and continuation | Coherent snapshots, resume, death and succession | Save/reload continues the same consequences through another life | Incremental requirement on every lane, not a last phase |

Detailed law design lives in [world conditions](2026-09-09_world_conditions_plan.md).
Individual memory and Hagiograph live in
[memory and remembrance](2026-09-09_memory_and_remembrance_plan.md). These are
Paredros consumer plans; any promoted shared contract needs its own owning-repo
review and a second consumer. Wing architecture remains in
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`.

## J: join the existing owners

### J0. Safe inspection

Preserve the authored equipment fixture while making historical/stale anatomy
and dead subjects inspectable. Query the existing accepted state; the UI must not
repair anatomy, synthesize capabilities, or revive a subject. A stale detailed
body must be visibly distinguished from current anatomy. Released equipment
remains at its accepted location, not silently reattached by a redraw.

Done when focused tests inspect current, injured/stale, reconciled/severed and
dead states without panic or mutation, with unavailable operations explained.
This is a prerequisite only: it does not widen the equipment save whitelist or
connect the crossing to the played subject.

### J1. A product session

**J1a, first bounded implementation:** compose existing `GameState` and
`Control` without exposing a second mutable game. Admit one living named subject,
dispatch only that subject's actions, and permit continuation to another existing
living named subject only after the controlled life dies. Save control decisions
at game-intent cuts and validate them against the state at those cuts on replay.
A once-valid successor may be dead by the final save; a currently dead subject
awaiting continuation is also a valid session state. Hash agreement cannot
replace these semantic checks. Reject post-Begin actions for another subject.

This bounded session has no autonomous scheduling, society, contact binding,
outsider generation, general host Save/Load integration, or checkpoint compaction.
It is the foundation for the wider J1/S1 contract below. Existing-life selection
here validates life state only; social eligibility and the outsider arrival policy
remain product rules to add before claiming the full F8 behavior.

Introduce a small product session coordinator, as a module before a new crate.
It composes `Simulation`/`GameState`, controlled subject, social state and an
explicit runtime binding from contact `BodyId` to `SubjectId` and body revision.
The session owns accepted command order and clock conversion; it does not copy
the state owned by its components. Autonomous policy must not issue a second
action for the controlled subject while the player drives it.

Define which contact positions/velocities must survive interruption and which
runtime objects can be rebuilt. Existing integer movement and continuous contact
coordinates need explicit units and conversion; truncating each frame into a
voxel is not a valid authority bridge. Distinguish fixed simulation steps from
accepted intent sequence numbers and world time.

Contact detects an impact; product rules admit a body consequence once, then
contact capabilities and the sheet derive from that revision. Runtime integrity
must not independently decide a different lasting injury. Reject stale subject
bindings before applying effects. Save/load uses S1's common cut.

Done when one existing named subject moves through contact, suffers a fall,
loses capability, and is inspected with the same identity, revision and item
locations; duplicate effects cannot injure twice; a reload continues identically.

## B: bodies, injury and directional action

### B1. Damage and recovery without requiring combat

Begin with a fall, crushing work accident or environmental exposure. Record
affected part addresses, cause, load/exposure, wound, and resulting revision.
Distinguish temporary impairment, tissue damage and severance. Loss follows
the admitted part tree. Treat attachments, detached matter and equipment
custody atomically with the body outcome. Recovery restores only what its law
permits; resting cannot silently grow back a missing limb.

Done when damage changes movement/work/action eligibility and equipment together;
treatment consumes an available resource and has a recorded result; descendants
of a severed branch cannot remain usable; player and autonomous subjects obey
the same rules. Impossible treatment changes nothing.

### B2. Directional combat

Adopt the requested directional attack/defense shape as a Paredros action profile.
Input chooses an intended direction relative to facing; anatomy and equipment
determine realizable trajectories. Start with a bounded set of authored arcs
(left, right, overhead, thrust), then support body-specific alternatives without
assuming every creature has two hands. Grip, reach, joint movement, occupied
parts, balance and the held implement constrain an arc. Alternative limb bindings
can preserve an action at a different cost or reach after injury.

**2026-09-09 refinement from Mark:** directions, swings and effects vary with
anatomy. Extra arms can prepare additional punches during a swing. Represent
this as a coordinated action with explicit participating part addresses and
per-limb preparation/release/recovery, rather than multiplying damage by arm
count. The player can reserve an available arm for guarding, carrying or a later
strike instead. A tail, tentacle, jaw or articulated tool can supply different
arcs and effects; the input arrangement follows reachable actions.

Additional strikes share balance, energy, footing, attention and target access.
Each has its own contact and outcome; a single successful roll cannot make every
prepared fist land. Recheck limbs at preparation, release and contact. Loss or
occupation of one arm cancels or changes only dependent subactions, with explicit
committed costs and recovery. Extra limbs expand coordination choices while
retaining interruption, congestion and resource costs. Their useful count is
bounded by admitted anatomy and concurrent-action limits, not a fixed two-arm UI.

Use explicit wind-up, committed action, contact and recovery phases. Revalidate
the binding if a part fails between preparation and contact. Input assistance,
direction selection and difficulty are settings; deterministic rules record the
accepted choice. Guarding depends on coverage and functioning bindings, not just
matching an animation label. Physics supplies contacts, product rules resolve
injury. Initial geometric envelopes need not wait for full articulated animation.

Done when two different body arrangements expose different valid arcs; loss of
one binding changes offense and defense mid-action without granting a free hit;
range, facing and timing matter; the same accepted input stream replays. Headed
acceptance checks direction readability and input comfort, without prescribing
a story, location or camera as the game's permanent form.

### B3. Strike quality and stochastic resolution: investigate before tuning

Mark proposed combining the physical swing with a roll weighted by strike
quality. The following source review supports trying this as a Paredros-native
combat policy; it does not establish exact coefficients or a RAW adaptation.

| Primary source, checked 2026-09-09 | Observed design | Paredros implication |
| --- | --- | --- |
| [TaleWorlds weapon model](https://www.taleworlds.com/en/Games/Bannerlord/Blog/32), 2017 developer account | Contrasts earlier random damage ranges with weapon length, mass, inertia and a simplified body-driven swing model | Derive contact quality from the participating body and implement, rather than a generic damage randomizer; this is historical design evidence, not a current-version benchmark |
| [OpenMW 0.49 release account](https://openmw.org/2025/openmw-0-49-0-released/), 2025 | Documents moving initial attack evaluation to release for animation/audio fidelity, with a range check still at impact | Preparation/release/contact are distinct; record exactly when chance is sampled and revalidate geometry before applying its result |
| [N'Garde author description](https://www.nexusmods.com/morrowind/mods/58658), checked 2026-09-09 | Offers glancing blows while preserving the underlying hit-chance mechanism, plus active defense | A failed roll can have a legible weak/deflected contact outcome rather than an apparently solid strike passing through a body |

These sources are design references only; no source code or game assets are
copied. The N'Garde option changes outcome behavior; preserving a probability
formula does not mean preserving the original game's complete rules.

Compare three policies against the same recorded contact cases:

1. **Contact-driven:** geometry and material response determine the outcome;
   skill changes preparation, control and recovery. Establish this baseline.
2. **Quality-weighted outcomes (recommended experiment):** valid contact,
   alignment, timing, speed, leverage, guard coverage and footing determine a
   quality record. Character technique and state shape a bounded distribution
   over glancing, effective and exceptional consequences.
3. **Attack-roll adapter:** use an existing rules procedure and its legal inputs.
   Physical input chooses legal attempts/targets or presentation. Giving a
   better swing an unauthorized roll modifier is an explicit adaptation, not
   rules-as-written fidelity.

Keep contact existence, guard interception, armor response and injury severity
separate. A roll cannot produce contact through a wall or with a severed source
part. Avoid double-counting speed/skill once in geometry and again as unrelated
bonuses. Quality weighting is a native policy over world facts; adapting the
mechanics does not require rewriting the world's material or anatomical history.
If another rules profile cannot represent those facts, retain them as annotated
facts or refuse the operation rather than replacing them silently.

Sample admitted uncertain outcomes under a saved RNG algorithm/version and
action/subaction identity. UI previews never consume/retry the authoritative
draw. Record rule revision, quality inputs, committed costs, draw identity and
outcome. Reload, a changed frame rate and sibling-limb iteration order must not
reroll the same admitted attack. Explicitly selected alternate stochasticity
settings are saved combat-policy revisions.

Done when contact cases cover empty swings, poor alignment, guarded/glancing
hits, good placement, disabled limbs, interrupted charged combinations and
different body arrangements. Fixed-input trials compare distributions,
monotonic response to improved quality (holding other facts equal), defender
agency and exact replay. Headed play checks whether players can tell why an
outcome happened. Accept a weighting policy only after that comparison; do not
make the first model's weights a permanent world law.

## T: materials, construction and work

### T1. Material transactions

Extend `Items` beyond Food/Dressing/Scrap as operations demand: quantities,
material identity, condition and location/custody. Add drop, transfer and storage
before crafting an expansive catalog. A work command names a subject, tools/body
capabilities, source material, target region, expected revisions and work cost.
Gathering, placing and repair move or consume material exactly once. Refusal
must leave inventory and terrain unchanged.

Done when one creature gathers, carries and places material with provenance,
cannot double-spend it, and can resume interrupted work. An autonomous creature
uses the same command boundary, with its own needs and agreements.

### T2. One edit, all spatial consumers

Accepted Ground edits produce revisioned dirty regions. Paredros feeds those
regions to the pinned voxel/spatial packages and its renderer adapter. Navigation,
contact, ray queries and rendering identify the source revision they represent.
If collider realization cannot complete, pause/refuse dependent simulation at
that revision; do not allow walking through a visually completed wall. A stale
render product is explicitly pending, not another editable world.

Done when adding a support permits a crossing and removing it changes collision,
navigation and rendering from the same edit; an occupied-cell edit has a defined
refusal/displacement policy; negative coordinates and chunk borders work; reload
reconstructs both consumers from accepted terrain without persisting GPU handles.

### T3. Work and consequences

Extend `ProjectGoal` from visits to gathering, placing, repairing and tending.
Agents evaluate affordability and capability, negotiate help and retain partial
progress. Structures retain builder/material references and useful effects such
as support, cover or storage before adding a full settlement economy. Changes
can affect another creature's access, safety or possessions, feeding M's observed
events and personal judgments.

Done when work can be interrupted and resumed by an eligible subject, refusal
is a complete outcome, and another creature can respond to an observed loss or
benefit without requiring a scripted encounter.

## S: saving, death and continuation

### S1. Coherent session persistence

The equipment store remains a restricted fixture. Define a versioned adventure
envelope naming world/generator/rule revisions, accepted event cut, component
records, control history, and required content identities. Save clocks, random
state, pending work and in-flight actions explicitly or restrict saves to a
documented stable boundary. Do not independently snapshot mutually dependent
owners at different ticks.

Restore into a candidate session, validate references and component hashes, and
rebuild projections before replacing the active session. Corrupt/missing content,
unsupported versions and incomplete archives must preserve the running state.
Use immutable publication already demonstrated by the equipment store as a
mechanism, not its fixed-subject admission policy.

The [long-lived save strategy](2026-09-09_memory_and_remembrance_plan.md#long-lived-save-strategy)
owns checkpoint/replay floors, recent tails, semantic retention and size receipts.
Saving an ever-growing copy of every intent remains an initial correctness
format, not the long-term world format. Mark accepts reasonable growth with
world history and created content; years of repetitive updates must not make the
world impractical to save or restore.

Done when interrupted work, injury, terrain, agreements, recall and control resume
from the same cut; the next accepted actions match uninterrupted play; injected
partial/corrupt saves never partially replace live owners.

### S2. Death and continuation

The product session checks death and successor eligibility before using
`Control::Succeed`; `Control` itself does not validate living-world facts. Preserve
the previous body's remains, item custody, commitments and others' recollections.
Use an eligible existing life or generate an outsider when no such life exists.
Record generation and selection. Optional non-death control changes remain
settings or admitted world mechanisms with their own consequences.

Done when both continuation paths work and survive reload, a dead subject cannot
act, and changing the control pointer neither transfers property implicitly nor
rewrites what other creatures remember. Hagiograph can remember the previous life
without requiring it to have become legendary.

## Orchestration and acceptance

1. Review the existing body/equipment WIP independently before committing it.
   Preserve its recorded 107-test/two-process receipt without claiming new tests
   inherit it. The current work does not absorb that patch into an unrelated commit.
2. Implement J0. In parallel, finish the W and M/H designs against existing code.
3. Settle J1/S1 command ordering, clocks, bindings and save envelope together.
   Terra owns that integration; Luna can implement bounded fixtures and rejection
   cases once the interface is written. Avoid concurrent edits to `state.rs`.
4. B1 and T1 can then use separate modules with serialized integration into
   accepted transitions. M1 can proceed independently over accepted deed evidence.
5. Add B2/B3, T2 and H retelling as their inputs exist. W's surgical operations
   consume B's revision boundary; W's first magic operation can proceed
   independently of surgery. H material manifestations also require W.
6. Validate connected sandbox loops using real production commands, varied
   subjects and changed conditions. A fixed script is a regression receipt,
   never the only supported way to play.

Every implementation reports exact files, supported operations, focused checks,
and remaining joins. Broader wing contracts are promoted only after another
consumer challenges identity/authority semantics. Reusable mechanical code may
enter its settled shared owner earlier.

## Findings

- **2026-09-09, pre-J0 inspection:** `EquipmentSession::sheet` assumed current
  living anatomy with `expect`. J0 removes that assumption; its loader still
  deliberately accepts only fixed fixture equipment history.
- **2026-09-09:** `contact.rs` owns `BodyState::integrity`, `Impairment` and
  `ContactSave` separately from `state.rs`; `crossing.rs::new_world` creates its
  own runtime subjects. Joining a widget does not close this authority gap.
- **2026-09-09:** `projects.rs::ProjectGoal` contains only `Visit`; `world.rs`
  exposes carving and site inheritance but not embodied building transactions.
- **2026-09-09:** `simulation_record.rs` already validates game/population/project
  replay; `control.rs` already replays subject changes. Neither is a whole
  adventure snapshot with contact and society.
- **2026-09-09:** local `nisus` and `modulus` provide shared mechanics, while
  `paredros-world/Cargo.toml` pins Conatus to `d82afa17`. Adoption must respect
  the actual dependency boundary.

## Progress

- **2026-09-09, follow-up:** incorporated anatomy-shaped coordinated strikes,
  independent skill/risk surgery, a proposed charge/sympathetic-coupling magic
  front, and a three-source combat-mechanics comparison. Long-lived persistence
  now plans independently restorable checkpoints and explicit replay floors.
  Terra implemented J1a with historical control validation; it does not
  close the wider contact, autonomous or social session joins.
- **2026-09-09, J1a implementation:** `paredros-world::Session` owns one
  `GameState` and a `Control` record. Its public API admits a living named
  subject, gates controlled actions, and permits existing-life succession only
  after death. `SessionSave` version 1 replays control at historical game-intent
  cuts, so later death of an earlier successor is valid; mismatched actors,
  ineligible subjects and invalid cuts remain rejected even with recomputed
  checksums. `SessionLimits` supplies caller-configurable byte and record-count
  bounds; writer and reader enforce the same archive boundaries. Byte limits
  bound encoded input/output, not all possible allocations during regeneration.
  The session has no host UI, autonomous scheduler, social eligibility, outsider
  arrival, contact bindings or compaction yet.
- **2026-09-09, J1a verification:**
  `cargo test -p paredros-world --lib --test session_boundaries --locked --offline
  -j 2 --target-dir target-contact` passed **35 library + 3 integration tests**,
  including 8 new session unit cases and 3 independent public-API boundary cases.
  It checks real item custody through death/succession/reload, dead awaiting
  continuation, repeated succession, same-state refusal, malformed historical
  cuts and exact/over-limit archives. The existing private cache is
  `C:/Users/mark_/Code/cargo-homes/paredros-save-check-20260908`.
  A concurrent Mesocosm matter receipt module lacked `StockError` in its import;
  the one-line `super::{Stock, StockError}` repair unblocked this dependency build
  without changing its behavior. No broader matter gate is claimed.
  The separate `cargo test -p paredros-room --lib body_sheet --locked --offline
  -j 2 --target-dir target-contact` regression passed **24 tests** on the updated
  dependency tree. Formatting and diff checks pass; changes remain uncommitted.
- **2026-09-09, save growth baseline:** the new
  `crates/paredros-world/examples/save_growth.rs` checks restored equality and
  unchanged physical world/body/items after repeated equipment cycles. Four
  checkpoints (0/100/1,000/10,000 pairs) produced 407/1,485/12,284/123,905 encoded
  bytes. The exact CSV is `testing/save_growth/2026-09-09.csv`; analysis and
  limitations live in the memory plan's long-lived save strategy. This is a
  single-operation history baseline, not a multi-year capacity claim.

- **2026-09-09:** Created lane/dependency plan from live code and the user's
  functional-loop direction. Terra implemented J0; Luna drafted M/H and
  Terra drafted W, with root review of causal facts, memory budgets and RAW scope.
  Further implementation is staged above, not reported done.
- **2026-09-09:** J0's `equipment_session.rs` reads the admitted historical
  anatomy and marks stale/dead state without repairing it; `equipment_view.rs`
  exposes the reasons. Four added tests cover stale attach refusal, explicit
  reconciliation/severance with released dressing, and lethal-fall inspection.
  The three state cases check that inspection leaves the authoritative hash
  unchanged; the view case checks that wrapped stale and dead reasons both remain visible.
  `cargo test -p paredros-room --lib body_sheet --locked --offline -j 2
  --target-dir target-contact` passed 24 tests using the existing private
  `cargo-homes/paredros-save-check-20260908` cache. This is automated projection
  coverage; injury input, general save admission and crossing integration remain
  open. No new native screenshot or physical-input receipt is claimed.
