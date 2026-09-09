# Wing bodies and character creation: one body, sovereign readings

**Status: cross-vessel contract and creator lanes; first joint Mesocosm
habitat/body slice and retained-trait creator locally verified through 2026-09-08.
Body/chronicle wire schemas remain v0.** Existing local body editors and sheets are
foundations, not a completed shared creator. Section 11 scopes C0-C5; W0-W6
remain the portable-body gates. This plan specifies what body identity means across
Mesocosm, Paredros, and Isometry. It does not give the games one capability
system, runtime, renderer, or biological simulation.

The [games wing founding record](2026-07-30_games_wing_founding.md) remains
authority for settled laws. Mesocosm's local body rules live in the
[phenotype plan](2026-07-31_phenotype_plan.md). The
[dependency ledger](2026-08-07_dependency_ledger.md) owns current ordering;
the execution waves plan is historical.

---

## 1. Why a wing plan is necessary

The wing has already ruled that a body is a part tree, loss cascades, the tree
is shared identity, and each vessel owns its capability fold. The v0 proof pair
predates that ruling:

- `mesocosm.body/v0` carries a flattened appearance grid, per-cell attribution,
  and flat part provenance;
- `mesocosm.chronicle/v0` carries species, flat part provenance, and deeds;
- neither carries parent links, stable subject identity, body revision, or a
  source address unique beyond `(species, part index)`.

The founding record also contains both the newer rule that topology travels and
the older finding that parent/child structure stays home. They cannot both
govern v1. This plan resolves the contradiction.

> **A current body's dependency topology is portable identity data. Its
> geometry and capabilities are projections and vessel-owned readings.**

That rule is narrower than sharing morphology and stronger than sharing a
sprite.

---

## 2. The terms

These are plain working terms, not product names.

- **Subject**: one stable critter identity. Naming it may make it a borg; a
  faction relationship may make it a character. Those additions do not mint a
  replacement subject.
- **Biological line**: Mesocosm's in-world descent relation among organisms.
  It is not Fili.
- **Body revision**: the anatomy of one subject at one causal point. Injury,
  grafting, regrowth, or chassis replacement produces a new revision for
  interchange even if a vessel mutates its live representation internally.
- **Part address**: a part id scoped by subject and body revision. A bare vector
  index is not a portable address.
- **Anatomy**: part identity, structural parent dependency, present or severed
  state, and provenance within a body revision.
- **Developmental rules**: Mesocosm's heritable instructions for growing a
  phenotype. Other vessels may carry them opaquely.
- **Phenotype**: one realized body in one environment, including geometry,
  material, functional links, processes, condition, and damage. Functional
  links may be cyclic; they do not replace the structural dependency tree.
- **Capability fold**: one vessel's derived answer to what that body permits.
- **Projection**: voxels, meshes, sprites, collision hints, summaries, and
  other derived views.
- **Chronicle**: append-only causal facts about a subject or body revision.

---

## 3. Ownership table

| Data | Authority | Portable treatment |
| ---- | --------- | ------------------ |
| Subject identity | shared subject profile | retained exactly |
| Biological line | Mesocosm critter facet | understood by Mesocosm, opaque elsewhere |
| Body revision identity | critter body facet | retained exactly |
| Part ids and parent links | critter body facet | retained as primitive topology |
| Present, severed, grafted state | ordered body history | retained or derived from accepted facts |
| Per-part source provenance | critter body facet | retained exactly |
| Developmental rules | Mesocosm critter facet | opaque unless a compatible consumer opts in |
| Functional links and process allocation | Mesocosm phenotype facet | optional and opaque unless a consumer opts in |
| Voxel geometry and material | current phenotype | optional appearance projection, never foreign rule authority |
| Reach, manipulation, armour, movement | consuming vessel | recomputed locally, never imported as verdict |
| Skills, affinities, trust, relationships | Paredros character facet | independent of body revision |
| Campaign role, allegiance, public history | Isometry facets | independent of anatomy semantics |
| Deeds and observations | chronicle/fact log | append-only with domain interpretation |

This is the substrate/system split applied to bodies. The portable substrate
states which part depends on which and where it came from. A game decides what
that arrangement means under its rules.

---

## 4. Individual continuity and biological descent

The same individual crossing vessels and a descendant founding a new life are
different operations.

### The same subject crosses

The subject id and body revision remain stable. A consumer may render the
provided projection, derive capabilities it understands, or preserve the body
facet opaquely. It must not silently mint another anatomy for the same revision.

If Paredros replaces a chassis, the character facet points to a new body
revision. Skills and relationships remain on the subject's person facet. The
old body may persist as an object, relic, corpse, or discarded revision.

Crossing may preserve or reinterpret phenotype, by explicit choice:

- **Carry this body** preserves current process allocation as faithfully as
  the destination permits. World-required accommodations are explicit
  adaptations and mint a causally linked body revision when they change the
  body.
- **Regrow here** preserves subject identity, genotype, developmental rules,
  and provenance while realizing a phenotype under destination conditions. It
  expects a new body revision and may look or function quite differently.

Neither route imports a capability verdict. The receiver derives its own
reading. The prior phenotype remains pointable, and an opaque consumer preserves
the Mesocosm phenotype facet if the route promises lossless continuation.
The destination declares compatibility and the cost of available
accommodations; the traveler chooses among feasible routes. An incompatible
carry is refused or redirected to regrowth rather than silently rewritten.

### A descendant is founded

Mesocosm consumes a biological-line record, ancestral anatomy and provenance,
accepted causal facts, and its developmental rules. It then mints a new subject
and a new body revision. Ancestral topology may inform inherited motifs, but it
does not make the descendant the same body.

This is where `Chronicle::found` currently collapses two concepts. Its star
body is scaffolding, and its reused species-plus-part indexing is not yet a
complete identity model.

### A memory outlives both

Isometry may retain a sprite, history, relic, or legend after the subject and
biological line are gone. That persistence is not biological descent and is not
Fili. The wing's existing tulpa proposal covers remembered survival if that
term is eventually inscribed. **(2026-09-02 note: this organ is now called
hagiograph; "tulpa" has been renamed to gemot's federated adapter-training
lane; see repo `CLAUDE.md`.)**

---

## 5. Part provenance and branch transfer

`from_species + from_part` cannot distinguish two organisms of the same species
that both have `PartId(3)`. V1 provenance therefore needs a source address:

```text
source subject
source body revision
source part
biological line, when known
acquisition event
```

This is a conceptual field list, not a compile-ready schema.

When a subtree is grafted:

1. the source operation identifies a living subtree under one source revision;
2. destination-local part ids are freshly allocated;
3. internal parent relations are remapped to the new ids;
4. the graft root attaches to one destination part;
5. every destination part retains its source address;
6. internal functional links may cross with the branch, while cut boundary
   links must be re-established under the destination phenotype rules;
7. the source's loss and destination's acquisition are causally linked facts;
8. severing the graft root later removes the imported dependency subtree.

Assimilation is different. It may preserve process provenance and a cause link
while the destination developmental rules produce a different topology. A
consumer must be able to tell which operation occurred.

---

## 6. V1 artifact split

### `mesocosm.body/v1`

The body profile is the home of a current body revision. V1 should add primitive
identity and topology beside its optional appearance projection:

- subject id;
- body revision id and causal predecessor, when any;
- biological-line reference;
- root part id;
- per-part id, optional parent id, state, and source provenance;
- optional flattened cells, attribution, collision hints, or projection recipe.

Offsets, pivots, exact voxel volumes, functional links, process allocation, and
developmental rules are included only when the profile explicitly declares the
relevant optional capability. A weak consumer can preserve the fields it cannot
interpret or use only the baked projection.

The reader still mirrors primitives locally. Carrying a `parent: Option<u32>`
does not require linking `mesocosm-core`.

### `mesocosm.chronicle/v1`

The chronicle should stop duplicating a flat body snapshot. It is an event
stream scoped to a subject and, where relevant, a body revision. A part-affecting
deed names a stable part address and the revision against which the claim was
made.

The body profile and chronicle therefore travel as related engrams or related
parts of a bundle:

- body profile: what anatomy this revision claims;
- chronicle: what happened and what other vessels claim happened;
- projection: what a weak renderer may show.

This separation prevents two independent snapshots from drifting inside the
same package.

### V0 handling

There is no production save population to migrate. When v1 is built, the two
repositories update their fixtures together, retain explicit version refusal,
and remove obsolete v0-only tests rather than carrying a permanent migration
layer. A fixture converter is permitted if it helps prove the change, but it is
not a public compatibility promise.

---

## 7. Authority and concurrent facts

A live body is timing-sensitive state. One simulation authority or ordered
session accepts severing, grafting, regrowth, and body replacement. The anatomy
tree is not a CRDT.

Other vessels append facts and proposals. An Isometry fact that narrates “lost
an arm” is history. A granted operation that names a body revision and part
address may also petition the anatomy authority to recognize a loss. Mesocosm
interprets that claim under its own rules and may accept it, reject it, or
branch the subject's history. The foreign fact remains either way.

Two concurrent claims against the same revision remain visible until the body
domain's materializer sequences or branches them. Set union preserves the
claims; it does not merge two anatomies.

---

## 8. Vessel readings

### Mesocosm

- owns biological development, metabolism, process paths, injury, regrowth,
  and descent;
- treats phenotype as moment-to-moment game state;
- mints new subjects for descendants;
- exports body revisions and causal records.

### Paredros

- refers to a current body revision from the stable subject;
- derives mobility, manipulation, equipment affordances, and embodied combat
  consequences under its own rules;
- keeps skills, personality, trust, and relationships outside the body;
- may replace a chassis without replacing the person.

### Isometry

- can render the optional projection and preserve anatomy opaquely;
- mints no biological part types in its substrate;
- lets a system plugin interpret anatomy only when that campaign wants it;
- appends campaign history and granted body-affecting proposals without
  becoming body authority.

---

## 9. Proof gates

### W0. Contract reconciliation

**Done when:** the wing founding record, phenotype plan, body-profile comments,
and doc index agree that topology is portable, geometry is an optional
projection, and capability is vessel-owned.

### W1. Stable addresses in Mesocosm

**Done when:** two organisms of one species can each have `PartId(3)` and a
provenance record still identifies the exact source; body revision changes are
causally ordered; snapshot and replay preserve those addresses.

### W2. Local branch proof

**Done when:** Mesocosm transfers or assimilates a source subtree according to
the local phenotype plan, loss cascades on both sides, and provenance identifies
the source revision and part after destination ids are remapped.

### W3. Mesocosm to Isometry v1

**Done when:** Isometry reads a v1 body using local primitive mirror structs,
renders its projection, preserves unknown anatomy data, appends a deed addressed
to the body revision, and returns it without linking Mesocosm.

### W4. Isometry to Mesocosm interpretation

**Done when:** Mesocosm distinguishes narration from a granted body-affecting
claim, applies an accepted loss to the correct revision, rejects a stale or
wrong-subject address, and retains every foreign fact.

### W5. Paredros second reading

**Done when:** a character keeps skills and relationships while changing body
revision; losing a relevant subtree changes Paredros-derived capability; and a
generated body uses the same slot as a played one.

### W6. Descendant distinction

**Done when:** returning the same subject preserves body revision identity,
while founding a descendant mints a new subject and phenotype with pointable
ancestry; neither operation is implemented as a flat star rebuild.

---

## 10. Stop rules

- Do not share a wing-wide capability enum or evaluator.
- Do not treat a sprite, flattened voxel grid, collision box, or cached score as
  body authority.
- Do not use species plus local part index as unique provenance.
- Do not duplicate current anatomy in both body and chronicle v1.
- Do not let a foreign vessel mutate anatomy merely by narrating an injury.
- Do not make exact ancestral geometry mandatory for a descendant.
- Do not implement v1 before stable subject, revision, and part addresses are
  proven locally.
- Do not extract a generic body library until Paredros supplies the second
  capability reading. The wire contract may precede that extraction.

---

## Findings

- **2026-09-07, source review, not a new runtime receipt:** reviewed Mesocosm
  `3a6fec6`, Paredros `47361ae` plus working-tree anatomy/equipment changes,
  Isometry `243c0dd` plus working-tree watchtower changes, and Mere `0a8198ba`
  plus unrelated active work. Existing test counts in their plans were not
  rerun in this planning pass. Mesocosm's checkout was clean at inspection.
- **2026-09-07:** `mesocosm-core/src/body.rs` still gives `BodyDocument`
  species/root/plan/parts, and incorporated provenance still names species
  and part. `organism.rs::OrganismId` supplies local individual identity;
  `program::RevisionId` names a lineage program, not a current body revision.
  These are ingredients for W1, not proof that W1 has landed.
- **2026-09-07:** `mesocosm-mesh/src/profile.rs::PROFILE_VERSION` and
  `mesocosm-core/src/chronicle.rs::CHRONICLE_VERSION` remain 0.
  `isometry/crates/isometry-voxel/src/body.rs` still mirrors the v0 body.
  The existing projection round trip cannot identify a portable body revision.
- **2026-09-07:** `paredros/crates/paredros-identity/src/lib.rs` has subject
  and body-revision ids. Its `paredros-world/src/technique.rs` offers one
  concrete `ArrestFall` query, with grip and adhesion readings;
  `subject_sheet.rs` projects those answers. The working-tree `anatomy.rs`
  admits bounded revision-addressed snapshots and reconciles severed parts,
  but directly carries Mesocosm `BodyDocument`. This is useful second-reader
  evidence, not a portable v1 decoder or a general technique language.
- **2026-09-07:** Mesocosm's shared body menu now includes ordinary-play
  expression as well as graft previews (HEAD `3a6fec6`; local acceptance is
  recorded in the phenotype plan). Isometry's `isometry-views/src/state/character.rs`
  stages a host-only creation request; `isometry-genet/src/sheets.rs` supplies
  the loaded system's default sheet and the `CharacterCreated` event path.
  Neither flow should be replaced by a creator-owned mutation path.

- **2026-07-31:** the v0 body profile is an appearance projection with flat
  provenance, and the v0 chronicle separately duplicates flat provenance.
- **2026-07-31:** the newer wing anatomy ruling contradicts the v0 finding that
  parent/child structure stays home. Primitive topology can cross without a
  Rust type dependency.
- **2026-07-31:** Paredros already separates chassis from skills and personhood,
  which is the real second pressure on body revision identity even though that
  repository is pre-implementation.
- **2026-07-31:** Isometry already proves opaque local-mirror decoding and
  additive history, but it does not yet consume topology.
- **2026-08-01:** cross-world phenotype handling is a choice between carrying
  the current body with explicit destination adaptations and regrowing a body
  from genotype under destination conditions. Both preserve subject continuity
  and point to the prior revision; neither imports capability verdicts. The
  destination declares feasibility and cost while the traveler chooses.

## Progress

- **2026-07-31:** founding contract written; no schema or code change made.
- **2026-08-01:** carry-this-body and regrow-here routes added after the first
  ProcessDef allocation design questions. No schema or code change made.
- **2026-08-01:** crossing authority refined: destination offers feasible,
  costed routes and the traveler chooses. No schema or code change made.
- **2026-09-02, terminology note (doc only):** Mark authorized renaming the
  memorial organ to **hagiograph** and reassigning **tulpa** to gemot's
  federated adapter-training lane. A dated note was added at this doc's
  tulpa mention rather than rewriting the historical text; see repo
  `CLAUDE.md`. No code changed.
- **2026-09-07:** researched and scoped C0-C5 and the cross-wing lane map at
  Mark's request. Added the live source findings above and reconciled the
  scheduling pointer. This is a proposed implementation scope; no new creator,
  portable schema, or runtime acceptance gate landed in this pass.
- **2026-09-07, generation correction:** Mark made procedural generation a
  first-slice requirement and welcomed discussion of its design questions.
  C0 now includes a bounded seeded generator, retained features, and structural
  variation alongside authored comparisons. Updated the dependency ledger and
  index to match; generation is scoped here, not implemented by this doc edit.
- **2026-09-07, input and system direction:** Mark chose seed plus key criteria
  before a detailed parts editor and proposed optional description inference.
  Added the taxonomy/function/trait distinction and the Isometry mapping layer
  that preserves biology while applying the selected tabletop rules faithfully.
  These are design updates; no generator or model integration was implemented.

## 11. Shared character creator scope (2026-09-07)

### Recommendation and boundary

Start here. A creator is a useful shared product surface because it makes
body generation, capability, history, rendering, and admission meet in a
small inspectable loop. It can expose weak joins before a full game hides
them. Completing a general engine or world generator first would postpone
that evidence without deciding the creator's hardest questions.

The proposed shared interaction is **choose, inspect, change, preview, enter**.
The user sees a recognizable body and the consequences of choices. Each game
supplies its available changes, prices, explanations, and commit operation.
Shared controls do not require a shared strength score, class list, or biology.

The September historical-entry ruling in the founding record governs the older
critter/borg/character shorthand: naming, agency, cognition, and faction
association are independent facts. There is no three-step unlock ladder.
An unnamed organism is a valid subject; every game must work from a generated
start without a prior game save. Use **Character creator** for a working UI
label if appropriate to the host; these are not three separate editor engines.

### What the player edits

Keep five readings available, with the host choosing which are relevant:

| Reading | Editable proposal | What the preview explains |
|---|---|---|
| Starting life | Authored/generated start; supported import; scenario and place | What exists here, source of this life, and entry constraints |
| Body | Supported developmental choices, part arrangements, expression, appearance | Present anatomy, potential, costs, lost options, and unsupported changes |
| Actions | Product-permitted equipment or technique choices | Current bindings, prerequisites, environmental limits, and why an action fails |
| History | Name and permitted background choices | Generated, played, inherited, taught, and merely reported facts |
| Enter | Destination-specific admission | Exact accepted changes, accommodations, and what remains opaque |

Do not require all five as a wizard. A generated start should already be
inspectable and usable. Name, vary, compare, and enter should be short paths;
deeper anatomy and causal history are disclosures. A rooted body must not be
forced through a walking test, nor an Isometry character through Paredros's
learning model. Anatomical validity, destination support, and suitability for
a particular scenario are separate answers.

The spatial body view uses actual part arrangement with stable part selection,
an optional separated-part view, and a complete keyboard/list alternative.
Injury, attachments, interior parts, and overlapping branches need readable
selection; a humanoid paper doll is not the underlying model. Changing the
camera or separating parts for inspection changes presentation only.

Authoring scope is supplied by the game/session: a scenario may allow bounded
founder customization, a live game may offer only a paid graft or expression,
and an explicitly selected sandbox may offer freer design. Expose the active
constraints and configurable budgets. Sandbox authorship retains its origin
and still needs admission into a constrained game. Draft undo does not rewind
accepted gameplay history.

### Three product adapters

| Host | First useful integration | Authority and boundaries |
|---|---|---|
| Mesocosm | Extend the existing graft/expression menu and founder preview into a bounded starting-body flow | Core admission and developmental programs own changes. Somatic expression, reproduction, and epoch lineage revision remain different operations. Founder material and environment determine viability; the UI adds no universal point currency. |
| Paredros | Extend the three-lives Body/Actions fixture into inspectable starting lives, then join the selected life to ordinary `GameState` | Admit anatomy and equipment through world transitions. Knowledge survives an injury; current action availability may not. The separate `ContactWorld` crossing probe is not already the durable game. |
| Isometry | Expand the existing host character panel with body appearance and an optional system reading, preserving atomic token/sheet creation | The loaded system and host decide sheet legality. A campaign may retain foreign anatomy opaquely and use baked appearance. Token ownership is a session grant, not portable subject ownership. |

Mesocosm is the first donor for checked body changes; Paredros is the first
donor for body-shaped inspection and source-bound action explanations.
Neither current UI is automatically the shared implementation. Build adapters
against their real seams, then extract the repeated mechanics into the existing
Cambium/projection stack when two hosts demonstrate them. The Paredros
anatomy/equipment working lane retains ownership of its files during that join.

### Draft, body, and projection are separate artifacts

A creator draft records chosen inputs, pinned content/generator revisions,
seed and locks, destination context, and optional source subject/revision.
Preview is disposable and does not mint a living subject, advance the world,
consume resources, or append history. Randomizing an unlocked choice must not
silently change locked choices. Saving a reusable design saves a template;
instantiating it twice produces two subjects. Continuing an existing subject
preserves its identity and records any accepted new body revision.

Commit rechecks the current source revision, destination rules, permission,
inventory, and place. If the world changed after preview, explain the stale
inputs and require a refreshed proposal. Save/reopen must retain accepted
facts; rendering and collision caches rebuild from them. A thumbnail or
portable sprite is never the only copy of the design.

For W1, qualify local subject ids with their issuing world/history domain;
raw `OrganismId(1)` and `SubjectId(1)` from different worlds cannot alias.
Specify branch identity, revision ancestry, and import mapping before choosing
the wire representation. Do not mistake a lineage-program revision, a
geometry digest, a render-cache generation, or a token id for a body revision.
This adds a W1 acceptance case, not a new universal identity service.

For v1, use an explicitly extensible envelope with required and optional
features, bounded decoding, and lossless retention of uninterpreted payloads.
The current positional postcard mirror does not automatically preserve fields
it does not decode. Test preservation through an edit and re-export, rather
than only loading a file. Unknown required biology refuses active simulation;
an optional baked projection may still permit display. Carry and regrow are
explicit destination choices, and regrow is offered only where implemented.

### Generation inputs and system interpretation (2026-09-07)

**Mark's direction:** start with a seed and a few key criteria; a detailed
parts editor follows later. Natural-language descriptions interpreted by local
or other inference are an interesting later input route. Isometry may mute
biological mechanics to embed the chosen tabletop system faithfully, with
world-generation and metagame composition around that system.

The first UI should expose seed, criteria, generate, and retain/regenerate.
Represent criteria as required, preferred, or unconstrained choices, with
unsupported requests visible. Candidate criteria for discussion are biological
family/line, environment, scale, body organization, means of movement and
feeding, and desired traits. Begin with a small supported vocabulary and
explain which criteria a candidate satisfies. Do not require the player to
fill in a complete taxonomy before seeing a body. Required criteria cannot
be silently relaxed to produce an answer.

Separate three kinds of classification:

- **Descent and taxonomy:** which biological line this belongs to and how
  the world's classification describes it. A request may constrain an existing
  family; an unfamiliar generated line can acquire classification afterward.
- **Ecological function:** how it obtains resources, survives, and reproduces.
- **Realized traits:** anatomy, processes, tolerances and capabilities that
  the developed body actually supports, distinct from unexpressed potential.

The current `mesocosm-core/src/organism/kingdom.rs::Kingdom` is explicitly a
trophic-role reading (producer, consumer, decomposer), not biological kingdom
taxonomy. That naming must not accidentally define the creator's classification
system. How much taxonomy is authored, generated, or inferred remains open.
The detailed editor later exposes available/unlocked parts and their effects;
the host defines availability, and the relationship between play progression,
editor access and sandbox freedom needs discussion rather than an assumed
universal unlock ladder.

**Optional description input:** first make the structured criteria work through
a deterministic generator and ordinary admission. A later model translates a
description into that same supported criteria representation, showing the
interpretation and uncertainties before generation. It does not mint arbitrary
working organs, facts, or rules. Record the accepted structured request and
generator inputs so recreating a body does not depend on repeating inference.
Evaluate description parsing on authored examples, ambiguities, contradictions,
and unsupported requests before selecting a local model or provider. Model
size, host compatibility, latency and quality remain unmeasured here.

**Isometry's composition:** world generation supplies bodies, histories and
places; a versioned campaign/system adapter maps admitted facts into the chosen
system's legal representations. The chosen system resolves play. A biological
trait can become a supported mechanical feature, remain appearance/history,
or require an explicit custom rule. Extra appendages, for example, do not by
themselves award extra attacks or equipment uses. Show the mapping and any
mechanically muted traits while preserving the source biology for later use.
For each overlapping concern such as injury, movement or resource consumption,
name which layer resolves it and how accepted results update other readings;
do not independently apply both ecological and tabletop consequences to the
same action. Background world advancement has an explicit campaign policy.
System id/version, content and mapping version belong in the admission context.
This is a compositional world/metagame layer, not a new universal tabletop
ruleset. A mapping receipt must show both fidelity to the selected system and
preservation of foreign facts, including when the system ignores them.

### C0-C5: implementation lanes and done-conditions

**C0. First generator, fixtures, and admission matrix.** Generation is part
of the first creator deliverable (Mark's correction, 2026-09-07). Begin by
working through what the player asks for and what the generator chooses;
unanswered design questions invite discussion and experiments rather than
automatically deferring generation. Mesocosm's existing `mesocosm-core/src/axis.rs`
recipe machinery and `world/genesis/` founding route are the first donors to
inspect. They do not yet constitute this creator flow, and axial animal
recipes must not silently become the universal grammar for every life form.

The first loop is: supply a seed and key criteria, generate several bodies,
inspect their form and capabilities, retain selected features, and vary the
rest. A detailed parts editor follows this initial generator. Start with a
small declared set of structural
choices such as proportions, segment count, or appendage arrangement, using
the existing developmental and admission rules. Meaningful variation must
affect anatomy or a relevant capability; recolouring a preset alone is not
the generation receipt. Which choices should be direct controls, requests
to the generator, or discoveries during play remains a design conversation.

Use authored bodies as comparison cases and stable regression fixtures.
Build a small corpus and expected answers: a rooted/radial body, an articulated
mobile body, an asymmetrically injured or grafted body, and one unknown optional
mechanism. Reuse Paredros's three wetland lives for the action comparison;
they alone do not challenge the breadth of morphology. Record each specimen's
source and whether each game supports simulation, appearance-only retention,
or refusal. Include an unnamed start and a factionless named life.

**Done when:** the first bounded generator produces inspectable candidate
bodies with recorded seed, recipe/content revision, conditions, and locks.
Identical inputs reproduce the body; varied seeds produce structural variety;
retained features survive regeneration; impossible requests report their
conflict or bounded search failure. Preview generation does not create a live
subject or spend world resources. Compare generated and authored bodies in
the same admission matrix, recording accepted/rejected candidates, reasons,
generation cost, and visible differences. At least one supported body family
must be genuinely generated; the other fixture families may initially be
authored. Wider biological generation remains iterative. Novel multi-body
colonies still need PE5's subject/body decision.

**C1. Body inspection and checked preview.** Reuse Mesocosm's menu and
Paredros's sheet to select parts, compare before/after, show costs and
blockers, and cancel. Adapt a small amount of repeated presentation into
Cambium only after both hosts use it. Keep game-specific queries local.

**Done when:** two native hosts show the C0 shapes with keyboard and pointer
access, preserved selection through rotation, small-window fit, and readable
invalid choices. Browsing, randomizing, and cancel leave authoritative state,
resources, and event logs unchanged. Present at 960x540 and 1920x1080 as the
initial acceptance sizes; dimensions remain configurable. Report frame time,
resident bytes, and preview rebuilds with device and fixture, using the local
baseline rather than inventing a shared performance threshold.

**C2. Local creation and entry.** Add a bounded founder/start proposal using
the first game's real transition path, then the second game's adapter.
Isometry's existing create flow is the third consumer, not a replacement
biological authority. Persist drafts separately from instances.

**Done when:** a generated and a customized start can enter a supported local
scene, perform one characteristic action, save, reopen, and replay. A stale
preview, invalid placement, exhausted cost, and failed admission produce no
partial subject, sheet, or item grant. Creating two lives from one template
does not reuse identity. Each host receives its own receipt; one pass does
not close all three. C1/C2 do not wait for portable v1 or world-scale work.

**C3. Portable body and destination preview.** Execute the existing W gates,
starting with stable addresses and local branch provenance. Add a destination
reading to the same creator only after those prerequisites hold. Generated
and imported starts use the same destination validator.

**Done when:** W1-W6 have their own evidence, including wrong-world and
wrong-subject rejection, stale revision refusal, required-feature refusal,
opaque preservation through re-export, and same-subject versus descendant
behavior. One portable body has distinct, explained game readings, an Isometry
projection, and a causally addressed return fact. File-based transfer suffices
for this gate; live federation is a later transport receipt.

**C4. Body in motion and use.** Feed a created body into a small local task:
Mesocosm's applicable movement/intake or growth, Paredros's supported
attachment/rescue action, and Isometry's system-resolved turn. This is the
voxel/motion lane's forcing scene, not a shared action enum.

**Done when:** a consequential body change remains recognizable in preview
and play; appropriate motion, picking, collision, and action sources agree
with the current revision. A rooted form is not made to walk. Failed motion
or missing rendering has a truthful fallback. C4 can use local bodies while
C3 proceeds; broad procedural animation remains separate from C1 acceptance.

**C5. Historical starts.** Use a tiny authored world/place/checkpoint slice,
then add bounded generation. Supply a life with a biological source, one
transmitted technique, equipment supply, and one known relationship; vary a
checkpoint to make a teacher or item unavailable.

**Done when:** a player can select a causally possible start and understand
the refusal of an impossible one. An earlier branch admits no later knowledge;
generated history cannot overwrite accepted events; importing a played history
displaces a compatible generated slot. This requires the world/history lane's
small context contract, not a civilization simulator or a universal world map.

### First joint habitat/body slice (2026-09-07)

World generation and character generation start together in Mesocosm. A small
habitat supplies the conditions for a life; changing the life does not silently
regenerate its habitat. This is a bounded local C0/C2 experiment, not completion
of the two-host C1 creator or the W1 portable-identity contract.

The implementation lives in `mesocosm-core::world::generation`, with a runtime
constructor and native `generate-start` / `--start` entry. Versioned request
JSON holds a habitat seed, independent body variation, selected place, bounded
search, initial population, nutrient range, and body criteria. It reuses seeded
developmental recipes, rather than selecting authored bodies. Initial criteria
are trophic role, movement organs, mass, segment bounds, and maximum part count.
Movement organs means actual contractile anatomy; producers can creep without
it, and player steering has its own rule. It is not an immobility toggle.
Symmetry remains the existing generator's geometry policy; accepting a label
override would not actually rearrange its axial development. Taxonomy, novel
body grammars, fine parts editing, and description inference remain open.

The habitat is the existing nine-place terrain with independently seeded soil
supply per place. The default has 24 background founders plus the played life;
background roles retain the existing producer-heavy founding distribution.
Admission checks developed organs, actual body-sized footing,
and nearby matter sufficient for body plus reserve. Entry pays that material
from the local patch and uses ordinary simulation transitions thereafter.
Candidate variation preserves habitat and background inhabitants. Bounded
refusals are recorded, including empty candidate sets; criteria are never
silently relaxed. Initial recipe variety is not evidence of resilient ecology
or open-ended evolution. Temperature, moisture, and historical causes require
their own modeled semantics before becoming controls.

Generate and inspect a request, then enter a selected life from the repo root:

```powershell
cargo run -p mesocosm-genet --bin generate-start -- --seed 7 --output generated-start
cargo run -p mesocosm-genet -- --start generated-start/start-1.json
```

The output directory must be fresh. `report.json` retains recipes, realized
bodies, habitat conditions, and counted refusals; each `start-N.json` selects
one candidate. `--variation 1` varies bodies, `--role consumer --movement-organs yes`
constrains them, and `--request request.json` exposes the full bounded request.
Native traces retain the selection and resolved content. This JSON is a local
generator input, not a portable body identity or substitute for a world save.
Keep the matching generator/content version when reproducing an old start.

Core verification: 435 library tests passed (one pre-existing ignored test).
The six generator tests also pass separately after the final material guard,
including a seven-seed corpus, actual movement, unchanged habitat/background
when varying bodies, unavailable candidate refusal, matter conservation, and
snapshot/replay. All eight native content tests pass on the final source,
including saved-content replay and refusal of mismatched recording metadata;
both native binaries build with `--offline --locked --target-dir target`.
The broad native all-target check found an existing non-exhaustive `CameraMode`
match in `examples/camera_compare.rs:125`; that wider check remains unpassed.

Local executable receipts are under `Code/testing/mesocosm/generated_start/`:

- Seed 7 produced four candidates in four attempts, 0.941 seconds for the
  preview: producer 46 parts, consumer 25, decomposer 11, producer 90. Selected
  place 4 has 67 mg per soil column and 3,283 mg available for founding nearby.
- Variation 1, consumer plus movement organs, produced four candidates in eight
  attempts. Four anatomies were refused for lacking the requested organs.
- Soil range 1..1 mg produced zero candidates, 128 material refusals, an
  inspectable report, and exit code 1.
- The consumer entered the native world and played 24 ordinary demo actions.
  `played.json` and `replay-receipt.json` agree at `37752396bba1dfb2`;
  `state_hash_matches` is true and `dev_intents` is zero.
- `consumer-inspection.png` and `producer.png` show the generated bodies in
  the existing graft-review framing. Both captures stayed at tick zero.
  The broad ecology camera leaves substantial empty headroom; the review
  framing is currently the clearer way to inspect a selected body (press H,
  Escape to cancel). This reuses an existing view, not a new candidate picker.

Wider body grammars, longer ecological experiments, and a second game's
consumer remain subsequent lanes. C0's broader fixture/admission matrix and
C1's two-host editing gate remain open.

### Local graphical starting-life picker (2026-09-07)

**Status: implemented and locally verified in the isolated
`work/character-picker` lane.** `mesocosm-genet --create --seed 7` opens the existing body-menu
chrome over a disposable generated body. Arrow keys select candidates, R varies
bodies, N advances the habitat seed, P cycles the nine places, C cycles feeding
role, M cycles the movement-organ constraint, and +/- changes starting mass.
Z/V rotates the preview. The full request remains available through `--start`.
This is keyboard navigation through admitted results; unlocked parts editing
and a second host remain separate gates.

Generation runs on a worker with bounded requests. New control changes retire
old candidates immediately; queued requests coalesce, and obsolete results are
discarded. Empty or refused searches show their reasons and offer no body to
enter. The parked startup world does not advance during inspection. Escape
discards the creator and restores the camera; Enter regenerates the selected
request, checks its world hash against the reviewed preview, and installs it
once. The renderer rebinds to the new ground on its existing device. The saved
trace records the exact selection and resolved content through the existing
generated-start replay path. An unfinished creator writes no played trace.

The core's opaque `Prepared` context holds admitted candidates and their common
founding world. It returns independent preview worlds so selecting another row
does not repeat the search. It is local transient state, not a new serialized
body identity. Mesocosm owns this host flow; shared UI extraction still requires
the second actual consumer in C1.

Verification: 436 core library tests and 94 native library tests passed, each
suite retaining one existing ignored test. The final focused generator and
creator runs pass all seven and three tests respectively. The native executable
builds with the exact dependency pins. Checks cover independent preview worlds,
stale worker results, empty searches, candidate paging, camera restoration,
inert cancellation, single entry, and replay through saved content.

Headed receipts live under `Code/testing/mesocosm/character_picker/`:

- `first.png` and `second.png` compare generated producer and consumer bodies
  at tick zero. The large capture and the 960x600 `recovered.png` show the body
  and complete keyboard controls without clipping. Long axial bodies become
  small in the narrow preview space; wider morphology and presentation remain
  subsequent work.
- `refused.png` shows zero bodies after a producer-plus-movement-organs request:
  all 128 attempts are refused. Enter stays in the creator. Changing the role
  yields candidates again; rotating then cancelling restores the original
  camera and records zero steps.
- Entry selects candidate 1 from seed 8, variation 1, place 4. `played.json`
  records that request and the content pack. After 24 ordinary actions,
  `played-receipt.json` and `replay-receipt.json` agree at
  `845c16d1bf876a2d`; `state_hash_matches` is true and `dev_intents` is zero.

This closes the local picker slice, not C1's two-host editing gate. The earlier
all-target `camera_compare` example limitation remains outside this receipt.

### Retained traits and criteria drafts (2026-09-08)

**Status: implemented and locally verified.** Complete the C0
generate/inspect/retain/vary loop using existing request constraints. K copies
the selected candidate's realized feeding role, presence of movement organs,
and exact segment count into the filters and generates a new batch. R varies
within those filters. U clears those three filters, preserving mass, part
budget, habitat, and variation. C and M remain independent overrides.

`--draft PATH` opens a saved criteria request, or starts a new one when that
file does not exist; it implies `--create`. S explicitly saves the current
request through atomic replacement. It stores criteria rather than the selected
candidate or a played world. Reopening regenerates candidates. A structurally
valid request with zero viable results is still a useful draft; malformed or
unsupported requests fail before opening, and failed saves preserve the old
file. `--draft` and `--start` are distinct input routes and cannot be combined.

Done when retained traits survive meaningful structural variation without
changing the habitat or parked world, clearing filters preserves unrelated
criteria, saved requests regenerate identical candidates, impossible requests
can reopen for revision, and a selected result still enters and replays. Verify
the longest menu page at a small window size. This adds no new biological
grammar, taxonomy, portable identity, or detailed part-editing authority.

Run `cargo run -p mesocosm-genet -- --draft criteria.json --seed 7` to start
or reopen the criteria editor. Saved inputs take precedence over the startup
seed. The file is also accepted by `generate-start --request`. Enter still
records the selected candidate and resolved content in the played trace;
draft saving does not write that trace or create an individual.

Verification: 100 native library tests passed with one existing ignored test;
all six focused creator/draft tests pass after the final status-message change.
Tests prove structural recipe variation under retained traits, unchanged
habitat and parked world, clearing only the named filters, deterministic draft
reopening, refusal preservation, and non-destructive failed saves. The native
executable builds with the existing pins and the already-resolved tempfile
dependency used for atomic replacement. Generator request version remains 1.

Headed receipts are under `Code/testing/mesocosm/creator_traits/`. The six-row
menu fits at 960x600. Seed 7, variation 1, consumer, movement organs present,
and exactly 16 segments produced five candidates within 128 attempts, with
35, 47, 33, 52, and 39 parts. The sixth requested result was not supplied by
relaxing constraints. Saving leaves candidate counts and refusals visible.
The saved criteria reopen, candidate 0 enters and takes 24 ordinary actions,
and replay matches `6f6d63362f3ce9fd` with zero assisted actions. A one-part
budget draft reopens with zero candidates and can be saved; Enter remains
inert. Captures and logs record the retained, reopened, full-page, and empty
states. Broader body-plan generation remains the next substantive C0 question.

### Generated branching bodies (2026-09-08)

C0 now admits two explicit developmental arrangements: axial and branched.
The branched generator keeps the seeded feeding organs and generates a trunk
with at least two daughter stretches attached to its realized segments.
Producers raise that trunk; consumers and decomposers extend it horizontally.
Branch lengths and attachment anchors vary with the body seed. These are
ordinary heritable recipe layouts consumed by development, collision and
rendering. Feeding role remains a separate criterion. This is a bounded
branching grammar, not taxonomy, radial symmetry, a colony, or evidence of
ecological resilience. The existing ecological symmetry field does not assert
geometric symmetry of these layouts.

B switches the creator's body plan while preserving habitat and other filters.
K, R and U preserve the selected plan; U still clears only role, organs and
segment bounds. Saving a draft retains its plan. The report tool accepts
`--body-plan axial|branched`. New requests use generator version 2; explicit
version-1 requests keep the exact axial stream and refuse branched criteria.
Choosing a plan in the creator explicitly upgrades an old draft to version 2.
Axial version-2 requests use the same generation behavior as version 1.

Acceptance checks cover geometry differences, all three feeding roles,
bounded admission, habitat-preserving variation, entry, matter conservation,
save/restore and replay. Broader articulated and radial grammars, free parts
editing and population-level persistence remain separate follow-ups.

Verification: all nine focused generator tests pass, including a three-seed,
three-role branching corpus, retained lineage recipes, geometry changes,
snapshot restoration, replay and unchanged axial version-1 behavior. All 101
native library tests pass with one existing ignored test. Creator tests also
cover opening an old JSON draft, switching its plan while Enter is inert,
keeping the plan through K/R/U, serialized criteria round trip, unchanged
habitat and parked world, and confirmation at the exact preview hash.

Headed receipts live under `Code/testing/mesocosm/creator_body_plans/`.
At 960x600 the full six-choice menu fits. Seed 7 produces six branched
candidates in six attempts; the saved request reopens and the report tool
reproduces the batch. Candidate 1 completes 24 ordinary actions and replays
at `4d16b4cb1d20ffec`, with zero dev intents. The older axial recording still
replays at `6f6d63362f3ce9fd`. A branched one-part budget returns zero candidates,
128 part refusals and exit 1. Both native binaries build; changed Rust files
pass formatting and the 600-line ceiling.

These first captures used cardinal previews that hid branches behind a trunk.
Z/V exposed an alternate view, recorded for both producer and consumer; the
rotated producer showed a crosswise crown and stem. The follow-up below
addresses clipping and the initial view. Broader body-motion gates stay open.

### Whole-body preview follow-up (2026-09-08)

The branching capture exposed a local inspection defect: isolated previews
kept the world's 16-voxel cutaway depth. Rotating a long body could therefore
remove geometry from the image as well as occlude it. The shared creator/graft
framing path now fits the complete body's extent along the upright cutaway
normal, including margin. The renderer uses that depth for projection,
culling and clipping, and temporarily clears habitat bounds during isolated
inspection. Closing the menu restores the scene's depth and bounds.

The creator starts with the existing oblique camera. Z/V cycles through that
overview and four cardinal views in either direction. The original play
camera is restored on cancellation or entry; camera choices remain host
presentation and do not change the generator request or recorded intents.
This is a local C1 inspection repair. Free orbit/zoom, two-host editing and
body-motion acceptance remain open.

Verification: 103 native library tests pass with one existing ignored test.
The geometry regression covers axial and branched candidates, every camera
mode, pitched and unpitched framing, and two window sizes. A renderer-backed
test checks temporary depth/bounds and exact scene-matrix restoration for
both ordinary and terrarium views. Creator checks cover cycling all five
views, reversal, inert world state and restoration on cancel.

Headed verification under `Code/testing/mesocosm/creator_preview/` shows the
complete branched producer and consumer in the initial oblique overview and
cardinal views at 960x600, with the six-choice menu fitting beside them.
The consumer's formerly sliced side view now retains its branches and organs.
Cycling back to the overview and entering candidate 1 still produces the
previous 24-action hash, `4d16b4cb1d20ffec`, with zero dev intents; both the new
trace and the previous branching trace replay exactly. The executable builds,
formatting and the 600-line ceiling pass, and the strengthened cancellation
check passes after explicitly leaving the overview before closing the menu.

### Held body and habitat comparison (2026-09-08)

The local creator now holds one generated body while the habitat changes.
`H` retains its development seed and role plus the request's existing body
criteria. `N` changes terrain/population seed, `P` changes starting place,
`F` cycles patch, uniform and contrasting nutrient distributions, `[`/`]`
halve/double the nutrient bounds, `,`/`.` change population by three, and `T`
cycles a minimum of zero to four body-sized directional steps at the start.
Body edits remain unavailable until `H` releases the body. Failure makes one
attempt and explains the refusal instead of substituting another critter.

Generator request v3 records these inputs. Older v1/v2 requests retain their
streams and reject v3-only criteria. The descriptor is local generation input;
changing its body criteria changes development. It does not establish portable
identity. Soil distribution is per place, uniform within each place. Terrain
variation uses the existing seeded generator. Starting access tests geometry;
it does not establish a route to food or account for movement energy.

The graphical comparison shows the candidate's actual terrain and inhabitants.
Each accepted held-body habitat runs 32 idle ticks on a disposable copy in the
worker. Readings show living producer/consumer/decomposer counts before and
after, whether the subject survived, and initially diet-compatible living
neighbours within eight horizontal voxels. Food compatibility does not establish
reachability. Entry still starts at tick zero. `S` saves the held request through
the existing criteria-draft path. `generate-start` exposes the same habitat
criteria and optional `--observe 0..128`, writing per-candidate trial reports.

Verification: core library and native library suites passed (446 core tests;
106 native tests and one existing ignored test before the final refusal test).
The focused creator suite then passed all eight tests, including held-body
controls, refused entry and saved-request round trip. All 12 focused generation
tests pass, covering deterministic body preservation across seeds, refusal without
substitution, soil distribution, legacy compatibility and conservative,
repeatable trials that leave entry untouched.

Headed artifacts are under `Code/testing/mesocosm/habitat_comparison/`.
Seed 7 and seed 8 show distinct actual terrain with the same 25-part branched
consumer. The second habitat's 32-tick trial changes living P/C/D counts from
18/7/3 to 18/6/3, with the subject alive and total matter unchanged at
2,014,448 mg. Entry, 24 ordinary actions and replay match `25f08b64e7c50c9f`.
The previous branched recording also replays to `4d16b4cb1d20ffec`.
Changed Rust files remain below 600 lines; formatting and diff checks pass.
These observations do not close ecological persistence.

The biological rules and palette remain unchanged. This advances the local
world/body experiment; PE4 world rules, TG6 persistence, historical entry and
cross-vessel interpretation retain their own acceptance gates.

### Research and its consequences

Primary sources checked 2026-09-07. These inform the proposed design; they are
not dependency selections or proof of this stack's implementation.

- [Spore's morphology-independent animation paper, Hecker et al., 2008](https://chrishecker.com/images/c/cb/Sporeanim-siggraph08.pdf)
  describes semantic part selection, generalized motion, and runtime pose
  goals solved for varied bodies. This supports separating body authorship
  from motion realization. Our inference: admit a bounded morphology set and
  prove one useful motion before promising arbitrary animated anatomy.
- [Cataclysm: DDA's body graphs and limb scores](https://docs.cataclysmdda.org/JSON/JSON_INFO.html#body-graphs)
  separate body navigation from readings affected by wounds and encumbrance.
  Borrow the visible link from part to capability and the nested/list access
  pattern. Its particular limb taxonomy and numerical rules are not a wing
  schema; Paredros already supplies a narrower local reading.
- [Thrive's Microbe Editor development design](https://wiki.revolutionarygamesstudio.com/wiki/Microbe_Editor_Development)
  places editing, metrics, constraints, and a test environment together, with
  a separately accessible free editor. It is a design document, not a verified
  inventory of today's shipped features. The useful proposal here is a cheap
  experiment loop and explicit editing context. Retain Mesocosm's own material
  economy rather than adopting Thrive's mutation-point budget.
- [Jason Grinblat's GDC 2018 history-generation talk](https://gdcvault.com/play/1024990/Procedurally-Generating-History-in-Caves)
  describes generating events and rationalizing them without a full historical
  simulation. That supports a bounded C5 fixture. Our stricter requirement is
  to constrain generated causes against already accepted history; plausible
  prose alone cannot establish a teacher, lineage, or carried item.

### Deliberately open design

The first editable morphology vocabulary, amount of founder authorship,
Paredros learning economy, and public shared component API remain proposals
to test. Free voxel sculpting cannot automatically award a functional organ;
if wanted, it needs an explicit geometry-to-function admission rule. A rich
creator can begin with meaningful developmental choices and later add sculpting.
Multi-body organisms, arbitrary world gravity, a shared combat ruleset, asset
marketplaces, procedural civilizations, and online simultaneous editing each
need their own consumer evidence. They do not block C0-C2.
