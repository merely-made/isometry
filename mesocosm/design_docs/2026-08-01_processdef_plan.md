# ProcessDef: authored biology without ability flags

**Status: in progress, refreshed 2026-09-01. PD1b is COMPLETE. Its identity
slice landed 2026-08-08 (native `ProcessDef` records, per-definition digests,
a registry) and its allocation half landed 2026-09-01: a private
`BodyPhenotype` wrapper, per-part authoritative cell-graph mosaics seeded from
geometry, one proposal and one validator for direct and automatic arrangement,
and an intentional snapshot format bump. PD2 is also COMPLETE, landed the same
day: `Process::Secrete`, a gland, is the one additional native played process,
acquired only through the temporary `Intent::Rearrange` editor operation.
PD3's pack admission is next. No pack loader or Mesocosm Piccolo host exists
yet. PD0 and the PD1a allocation design pass are complete.**

This plan owns Mesocosm's extensible process vocabulary, developmental
expression boundary, content-pack shape, and Piccolo proof. The
[phenotype plan](2026-07-31_phenotype_plan.md) continues to own body and
capability semantics. The
[dependency ledger](2026-08-07_dependency_ledger.md) owns scheduling, and the
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns the product
integration proof that consumes PD1b and PD2.
The [wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md)
owns what can cross into Paredros and Isometry.

---

## 1. The ruling

`ProcessDef` is the right name for one versioned, namespaced biological or
physical transformation that a phenotype may perform. It is not a gene, a
body part, a capability, or a script callback.

The full causal ladder is:

> acquisition and inheritance -> developmental instruction -> expression in
> parts and channels -> satisfied process path -> vessel-owned capability ->
> ecological consequence

That distinction keeps the system broad without making it vague. A process
definition can say that exposed tissue transforms light into usable energy.
A developmental instruction can say where that tissue tends to grow and when
it expresses. The current phenotype records which sites actually expressed it.
Mesocosm then decides whether those sites, their connections, the available
light, and the body's costs produce a working capability.

The process definition never says `can_photosynthesize = true`. Piccolo never
writes energy into the world. Capability remains a reading of embodied facts.

### The P2 rule changes, but its principle survives

P2 currently says "processes are read, not stored" because geometry alone
derives its three native processes. That was the correct first proof: a long
part contracts and buys reach, while a plate does not.

Geometry cannot remain the only process authority. Two equally flat plates
may be armour and photosynthetic tissue. A bulky part may be stomach, storage,
lung, or inert ballast. The mature rule is therefore:

> **Process allocation is phenotype data produced by validated development;
> capability is always read from allocation, anatomy, channels, cost, and the
> current environment.**

An expressed process is not a stored ability score. It is closer to a working
organ: located on a particular part, paid for, connected or disconnected,
conditioned by the environment, and pointable to its developmental and source
provenance. Reshaping, starving, severing, or moving worlds can still make it
stop working without editing a capability number.

**Phenotypic plasticity is intended.** `BodyPlan` constrains structural growth;
it does not uniquely determine the realized critter. The same body plan, and
even the same inherited developmental program, may produce different process
allocation when the environment, acquired sources, life history, or recorded
entropy differs. Identical inputs and entropy still produce an identical
phenotype. A lineage is therefore heritable without being a mold that stamps
the same organism every time.

---

## 2. What is and is not a process

The broad gene-expression system needs several kinds of consequence. Treating
all of them as `ProcessDef` would recreate the old trait array with strings.

| Consequence | Working representation | Example |
| --- | --- | --- |
| transformation | `ProcessDef` | light capture, digestion, contraction, secretion |
| shape and attachment tendency | developmental anatomy rule | bilateral limbs, branching hyphae, a thick shell |
| material property | material or tissue definition | antifreeze, stiffness, translucency, insulation |
| internal connection | channel-development rule | routed energy, redundant signals, centralized control |
| regulation | expression rule | active in cold, suppressed in darkness, induced after injury |
| lifecycle | developmental trigger | spores after drought, metamorphosis, budding at surplus |
| appearance and signalling | phenotype presentation fact | warning colour, mimicry, scent |
| relationship | ecological or social fact | pollinator dependence, gut symbiont, colony quorum |

Only the first row is `ProcessDef`. The first implementation adds process
expression. Other developmental operations join the typed proposal vocabulary
only when a played phenotype needs them. There is no universal `TraitDef` in
this plan, and the provisional `epoch::Trait` array remains until the
phenotype plan's deletion gate is met.

### A small vocabulary can still produce a large pool

The intended breadth comes from composition, conditions, placement, and cost,
not hundreds of bespoke booleans. Illustrative families include:

- capture of light, heat gradients, chemicals, radiation, pressure, or magic;
- exchange and respiration in air, water, methane, ammonia, or stranger media;
- digestion, fermentation, detoxification, concentration, and storage;
- contraction, support, adhesion, buoyancy, jetting, burrowing, and flight;
- sensing light, vibration, chemistry, heat, fields, time, or nearby signals;
- secretion of venom, glue, ink, spores, silk, minerals, lures, or shelters;
- repair, regeneration, dormancy, shedding, budding, and metamorphosis;
- communication and control through local, routed, redundant, centralized, or
  quorum requirements;
- symbiotic exchange across the skin and ecological engineering outside it.

A new process is useful only when a game rule consumes its inputs and outputs.
Names in a catalog do not create mechanics.

### One displayed trait, three compiled programs

The epoch UI may present a generated trait as one coherent biological idea.
That presentation does not imply one runtime representation. A candidate
lowers into the mechanisms that own its consequences:

1. A **condition program** evaluates newly admitted evidence or a bounded
   reading when a relevant event arrives. It governs discovery eligibility.
2. A **development program** runs at founding, filial growth, paid remodeling,
   repair, or another named discrete trigger. It proposes anatomy, allocation,
   channels, regulation, or lifecycle instructions through the existing atomic
   validator.
3. A **`ProcessDef`** is consumed by an authored native evaluator during the
   repeated world work that actually transforms matter, energy, signal, force,
   or medium.

The first two are not variants of `ProcessDef`, and none is a universal
per-tick trait AST. All three use versioned typed data, integer or fixed-point
values, explicit input facts, and hard bounds on node count, edge count, depth,
evaluation work, and output size. Same-tick dependencies form an acyclic graph.
Feedback requires named state and a later tick, so a generated cycle cannot
recurse inside one evaluation.

Combination semantics belong to native consumers. A consumer declares whether
multiple contributions gate, add with saturation, take a minimum or maximum,
or compete for one exclusive slot. The generator chooses admitted definitions,
parameters, placements, and combination laws; it cannot coin an operator that
the validator and explanation path do not understand.

A realized generated candidate persists the exact component definition
references, parameters, placements, condition, generator provenance, and
digest. Its player-facing name is presentation. Save, replay, and peers do not
reconstruct mechanics by rerunning the generator from that name or a seed.

[Kappa’s rule language and tools](https://tools.kappalanguage.org/docs/KaSim-manual-master/KaSim_manual.htm)
are useful prior art for handling combinatorial structured state through local
typed patterns, signatures, static reachability and influence analysis, and
causal traces. Mesocosm can borrow those admission questions while retaining
its deterministic fixed-step transaction path. It does not adopt Kappa’s
stochastic event scheduler or turn anatomy, lifecycle, relationships, and
presentation into one graph-rewrite engine.

This split changes no phase order. PD2 first proves one repeated process; PE2
then composes it with one event-driven discovery condition. PE3 proves a
discrete development program that expresses the admitted option in a
descendant.

---

## 3. Core vocabulary

These are working Rust shapes, **illustrative rather than compile-ready**.
The implementation begins with only the fields demanded by the first proofs.

```rust
pub struct ProcessRef {
    pub id: QualifiedProcessId,   // for example, "mesocosm:contract"
    pub definition: DefinitionDigest, // exact admitted definition
}

pub struct ProcessDef {
    pub id: QualifiedProcessId,
    pub abi: u16,
    pub inputs: Vec<FlowPortDef>,
    pub outputs: Vec<FlowPortDef>,
    pub site: SiteRequirements,
    pub basal_cost: CostDef,
    pub active_cost: CostDef,
}

pub struct ExpressedProcess {
    pub site: ProcessSiteId,
    pub process: ProcessRef,
    pub part: PartId,
    pub cells: Vec<CapacityCellId>,
    pub cause: ExpressionCause,
    pub source: Option<Provenance>,
}

pub struct CapacityCell {
    pub id: CapacityCellId,
    pub neighbours: Vec<CapacityCellId>,
}

pub struct PartAllocation {
    pub part: PartId,
    pub cells: Vec<CapacityCell>,
    pub sites: Vec<ExpressedProcess>,
}
```

This sketch is illustrative rather than compile-ready. PD1a removed the two
stored capacity scalars from the earlier sketch. Structural capacity is the
number of living cells in the graph and a site's allocation is the cells it
occupies. One source of truth makes conservation countable instead of asking
two integers to agree.

`QualifiedProcessId` is pack-qualified at the authoring and wire boundaries.
`DefinitionDigest` prevents the same friendly id from silently changing its
meaning without making every imported process depend on the source world's
whole ruleset. A world separately records the `RulesetDigest` of the complete
set it admitted. The runtime may intern a process reference to a compact
integer inside one simulation, but that integer never becomes portable
identity.

`FlowKind` remains distinct from process identity. Its first host-owned set is
small: matter, energy, signal, force, and medium. A process transforms or
exposes flows; a channel carries them. A pack can qualify a substance or medium
without adding a new transport law. A mismatch requires an embodied adapter
process, not a compatibility table.

`ProcessDef` data is deterministic and integer-only. It contains no Lua value,
closure, filesystem path, clock, random generator, or rendering fact. It can
therefore be admitted into a world's ruleset and evaluated by
`mesocosm-core` without running a script in the simulation loop.

### The allocation mosaic

**Ruled 2026-08-01.** Each part has finite process capacity. Several processes
may share it, and every expressed site consumes some of the same budget. A leaf
may capture light, signal with pigment, and repair itself, but emphasizing one
leaves less tissue for the others. This is the correct competition boundary:
the tradeoff lives inside the organ rather than in a flat organism-wide score.

The working UI is a Diablo-like inventory for each part: process sites occupy a
small **allocation mosaic**, and their relations can be inspected and edited at
developmental moments. Adjacency matters because processes that remain near
one another across turns and epochs may cooperate, interfere, or become
candidates for hybridization. The mosaic is phenotype state keyed to a stable
`PartId`; it is not a capability verdict and it does not make renderer voxels
simulation authority.

**Ruled 2026-08-01: the mosaic is an authoritative graph of capacity cells.**
A process site occupies a connected subgraph. The evaluator may inspect each
cell's neighbours, a site's boundary, connected regions, and the entire
part-mosaic. A Diablo-like 2D layout is a useful editor and inspector, but its
screen coordinates are not authority; radically different parts do not have to
pretend they are rectangles. Allocation still conserves capacity: occupied
plus free cells equal the part's current capacity, and damage or shrinkage
cannot leave a site occupying cells that no longer exist.

### Hybridization across epochs

**Ruled 2026-08-01.** Adjacency may produce a somatic compound during an
embodied epoch: neighbouring sites can cooperate, interfere, or expose a new
transformation without rewriting either parent. Use, survival, and repetition
make that compound eligible for the adaptation bank. The adaptation phase may
then stabilize it as a heritable form, citing both parent processes and the
lived compound history.

This gives the two game phases different jobs. The first-person epoch discovers
the combination by living with it; the adaptation turn decides whether the
lineage commits to it. A mid-epoch adjacency never silently rewrites genotype.
Process hybridization is distinct from combining two biological lines, though
a stabilized process may later participate in lineage hybridization. Still
open is the stabilized form's identity: a newly admitted derived `ProcessDef`,
or a heritable compound recipe evaluated through its parents.

### Directed graft affinity

**Built at phenotype P3, 2026-09-01.** `mesocosm-core::graft` holds `Domain`,
`Verdict` and `Affinity`; the graph is world data (a default world holds the
three-domain favoured cycle), each lineage carries a domain drawn from its own
salted stream and inherited by a fork, and the digest is over the table's
rule-bearing bytes. The three verdicts decide which **crossing** a branch
transfer may take: a same-domain carry keeps the donor's arrangement, a
favoured cross-domain carry lands the branch expressing nothing until an
adapter is grown on it, and a disfavoured carry is refused with regrowth left
as the feasible route. The remaining ruling below is therefore answered *for
carrying only* — a disfavoured edge is a hard gate there — and whether
regrowth across one should itself be gated or priced is still open. The
adapter's embodied footprint is the free tissue the branch arrives with; a
smaller footprint bought by a learned compatibility process is untouched.

The proposed balancing model is a directed affinity graph over tissue domains,
not the existing `Kingdom::{Producer, Consumer, Decomposer}` trophic role. A
default world might use animal-like -> fungal-like -> plant-like -> animal-like
as the favoured cross-graft cycle, with the reverse edges disfavoured. The bare
word `flora` remains reserved by the platform and is not a game-data label.

The graph belongs to world or pack data rather than a universal three-value
enum. Soup organisms, colonies, mineral life, and magical worlds may define
different domains and edges. Same-domain grafts are ordinarily native. A
cross-domain edge determines whether the boundary connects directly, requires
an adapter process, or refuses.

The adapter is embodied: it occupies cells near the graft boundary, consumes
upkeep, or reduces channel throughput. A learned compatibility process can
shrink that footprint or penalty, analogous to buying off a dual-wielding
penalty. The remaining ruling is whether a disfavoured edge is normally a hard
gate or an expensive but recoverable graft.

### What a definition may control

A definition may declare:

- typed inputs and outputs;
- structural and environmental site requirements;
- basal and active costs using bounded integer curves;
- the relations its ports require, once those relations exist;
- plain author-facing labels and explanation text kept outside rule authority.

A definition may not declare:

- a capability verdict such as flight, reach, or immunity;
- arbitrary changes to an organism or world;
- an unbounded callback on every ecology tick;
- a new channel relation by naming it;
- renderer, camera, mesh, or sprite behavior;
- Paredros or Isometry consequences.

New verbs require a typed Mesocosm consumer. Piccolo can author data for a
recognized consumer; it cannot smuggle a new state mutation through a string.

---

## 4. Developmental expression ABI

Piccolo is an authoring engine at discrete developmental moments. It is not the
simulation engine.

The host invokes expression at bounded triggers:

- founding or filial regrowth;
- a chosen adaptation;
- assimilation or grafting;
- growth, paid remodeling, injury repair, or regeneration;
- lifecycle stage change such as metamorphosis.

It does not invoke Lua once per organism per ecology tick. Accepted output is
lowered to native process allocations and channel-development instructions,
which the core can evaluate repeatedly without the script.

**Ruled 2026-08-01.** A changing environment does not itself rewrite the
allocation mosaic. Existing processes may become active, dormant, efficient,
or starved as their inputs change. Reallocating tissue requires a discrete
developmental event with a cost and a causal record. That keeps a whole roster
and several co-op players tractable: the simulation evaluates activity, while
expression runs only when somebody actually changes.

### Request

`ExpressionRequest` is a frozen view containing only declared facts:

- the trigger and subject/body revision;
- stable part addresses, topology, geometry summaries, condition, and damage;
- existing expressed processes and channels;
- acquired process candidates with source provenance;
- the heritable developmental instructions being evaluated;
- integer material and metabolic budgets;
- relevant quantized world conditions;
- host-supplied deterministic entropy.

The request does not expose mutable Rust objects. Scripts cannot inspect hidden
world state that the host did not put in the request.

### Proposal

The smallest proposal says which admitted process should express on which
existing or proposed part, at what bounded capacity. Later proposal variants
may request a typed channel connection or anatomy-development operation only
after the corresponding native validator exists.

The proposal does not choose its own cost. It references an admitted
`ProcessDef`; the host calculates cost, validates the site and source, checks
the budget, and either lowers the whole proposal or refuses it. Partial
acceptance would make fixture and player explanations ambiguous, so v1 is
atomic.

Every refusal names the boundary that failed: unknown process, stale ruleset,
missing source, invalid part, site mismatch, insufficient material, cycle or
graph limit, channel mismatch, output limit, or exhausted fuel.

### Sandbox

Reuse Isometry's proven pattern:

- `piccolo::Lua::core()` rather than an ambient standard library;
- host policy for fuel, output bytes, nesting, and collection lengths;
- host-owned entropy with the draw trace recorded by fixtures;
- structured Rust-to-Lua input and typed Lua-to-Rust output;
- no network, filesystem, environment, wall clock, threads, or host mutation;
- a declared entrypoint, `express(request, entropy) -> proposal`;
- exact fixtures for proposal, entropy draws, validation, and refusal.

Mesocosm does not depend on `isometry-system`. Its biological request,
proposal, validator, and lowering are sovereign game code. If both games end
up with materially identical sandbox, fuel, entropy, and tagged-value code,
that small runner becomes an extraction candidate. It is not extracted first.

Piccolo is selected for this lane because Isometry already proves it in the
stack. This does not make Lua the mandated backend for every future extension.
The durable boundary is typed request, typed proposal, validation, and commit.

---

## 5. Pack and ruleset shape

The first pack is deliberately plain and inspectable. **Built at PD3,
2026-09-01**, and completed at PD4 the same day with the `expression/` and
`fixtures/` arms this sketch reserved:

```text
mesocosm-pack.json          pack id, version, abi, license, note, files
processes/
  contract.json  fix.json  intake.json  secrete.json  sense.json
expression/
  gland.lua                 express(request, entropy) -> proposal
fixtures/
  gland_rich_ground.json  gland_lean_ground.json
```

Every arm is **declared**, and `mesocosm_phenotype::asset` is the only way to
open one: a relative path the manifest does not name is `UndeclaredFile`
before it is ever resolved, so a host cannot be talked into running a script
sitting beside the pack.

JSON, because the workspace already reads and writes it (traces, receipts,
rosters) and a data-only pack should not cost a new parser. One file per
definition, each declaring `namespace`, `name`, `expressed_by` and `seeding`
— the four rule-bearing fields, exactly what `ProcessDef` holds — beside a
`label` and a `note` that plan §3 keeps outside rule authority. The schema
denies unknown keys, and every word in a rule-bearing field is a closed set,
so a typo is refused rather than ignored.

The manifest declares the pack id, version, format ABI, every definition,
script, and fixture, dependencies, and SPDX license metadata. All declared
paths are canonicalized and must remain inside the pack root. Duplicate
qualified ids, dependency cycles, unknown flow kinds, and undeclared files are
rejected before Lua loads.

Admission produces a deterministic `RulesetDigest`. **Built at PD3 over the
lowered definitions rather than over file bytes**, which is the honest reading
of "rule-bearing": hashing bytes would make whitespace, key order and a
`note` rule-bearing. It folds each definition's own digest — identity, site
requirement, seeding — in *sorted* order, so declaration order is provably not
a rule either, and one flipped role or seeding byte anywhere still moves it.
The format ABI is an admission gate rather than a digest input. A world records
the exact digest, not merely a friendly pack version. Definitions admitted
into one ruleset are immutable for that world. Editing a pack creates a new
ruleset and an explicit world revision rather than changing living bodies
underfoot.

Each lowered definition also receives its own `DefinitionDigest`. That is what
an expressed process and a portable body cite. The ruleset digest answers
"which complete biology was this world running?" The definition digest answers
"which exact transformation did this organ express?"

### License gate

- Mesocosm definitions, Lua scripts, pack loaders, validators, fixtures, and
  game-specific schemas are MPL-2.0.
- A generic sandbox crate may become `MIT OR Apache-2.0` only after Isometry
  and Mesocosm prove the same extracted boundary.
- Original visual, audio, and other game assets are CC BY-SA 4.0 and require an
  attribution entry.
- Imported assets and third-party packs retain their own licenses; the manifest
  records them without implying that Mesocosm relicenses them.

---

## 6. Persistence, replay, and co-op authority

Lua makes a proposal. The validated result is the fact.

An expression commit records at least the trigger, body revision, ruleset
digest, accepted `ExpressedProcess` changes, exact costs, provenance, and
entropy used. Replaying applies that record. It does not rerun Lua and hope the
same version happens to be installed.

The world's admitted definitions are native deterministic data. A participant
can continue ordinary simulation from the lowered ruleset without the authoring
script. Re-running expression, revising the pack, or creating new developmental
proposals requires the source pack and an authority allowed to do so.

For co-op, the live simulation orders body and resource changes. Multiple
writers may sign proposals, but two proposals spending the same material or
claiming the same developmental transition do not commute. The world's
materializer accepts, orders, or refuses them and commits one exact result.
This is not a universal CRDT problem.

The first network proof is commit-result:

1. one authority runs Piccolo against a frozen request;
2. native validation lowers the proposal;
3. the accepted expression record enters the ordered world history;
4. peers apply the record without executing Lua;
5. snapshot plus history converges to the same body and state hash.

Seed replay may later be offered for vetted packs, but it is an optimization
and authoring convenience. It is not the convergence contract.

### Missing packs

A foreign body may preserve unknown `ProcessRef`s, allocation, and provenance
opaquely. Mesocosm must not silently substitute a similar local process. If the
lowered definition required to simulate that body is absent, authoritative
continuation refuses with a missing-ruleset diagnostic. Projection and
archival may still work.

---

## 7. Player legibility

Moddability earns its place only when it creates a choice someone can read.
Every expressed process therefore needs an explanation path usable by the
field journal, body inspector, AI, and failure receipts:

- what the process does in plain language;
- which part expresses it and where it came from;
- what it consumes, produces, and costs;
- which environmental conditions enable or suppress it;
- which channel or anatomical requirement is broken;
- which actions and ecological effects currently depend on it.

The first authored process proof should create a visible tradeoff, not only a
successful fixture. The leading candidate is light capture: expression on an
exposed plate can earn energy in a bright world, costs tissue and upkeep, goes
dormant in darkness, and disappears when the responsible part is severed. It
crosses body, world, ecology, and explanation with one small definition.

The exact first process is ruled at gate PD2. If light capture requires a flow
system larger than the proof, choose an existing mechanic such as venom
secretion. Do not invent a dummy capability solely to make the pack pass.

### Direct and automatic arrangement

**Ruled 2026-08-01, clarified 2026-08-03.** During adaptation, a player may
arrange a candidate founder phenotype directly or ask the game to auto-arrange
it. Unplayed lineages use the automatic path. These are two proposal sources
over one developmental validator, not player biology and simulation biology:

- both consume the same acquired candidates, capacity, material, and
  plasticity budgets;
- both must satisfy the same site, adjacency, connectivity, graft, channel,
  and cost rules;
- both lower an accepted candidate into the same kind of heritable
  developmental instruction and causal record;
- automatic arrangement may optimize a declared aim, but cannot invoke a
  privileged mutation or skip a refusal the direct editor would receive.

The editor exposes authorship, not authority. Its screen arrangement lowers to
an authoritative candidate cell graph, and the validator decides whether it
can exist under the preview's declared conditions. The lineage commits the
developmental program that produced that candidate, not the literal mosaic.
Later bodies realize their own phenotype mosaics from that program and their
actual conditions.

The [epoch-boundary plan](2026-08-01_epoch_boundary_plan.md) owns the
multi-writer result: players on one lineage may adopt one validated program
together, while disagreement preserves the proposal as a branch rather than
merging preview mosaics cell by cell.

---

## 8. Wing boundary

`ProcessRef`, expressed allocation, and source provenance may eventually ride
the optional Mesocosm phenotype facet of body v1. The definition and Lua source
travel as an engram or pack, not inline in every body.

Paredros and Isometry may preserve those facts, inspect them, project them, or
map a known process into their own rules. They do not inherit Mesocosm's
capability verdict. A photosynthetic critter arriving in Paredros remains the
same subject and body; whether sunlight affects settlement work, combat, or
social trust is Paredros' decision.

This keeps the layered reading intact: a character may wrap a named critter's
body and history, while body processes remain one composable facet rather than
becoming the character schema.

**Ruled 2026-08-01: crossing offers two phenotype routes.**

1. **Carry this body:** preserve the current allocation mosaic as faithfully as
   the destination permits. If the destination world requires gills, wings, or
   another accommodation, each change is an explicit adaptation with a cause;
   it does not silently overwrite the arriving phenotype.
2. **Regrow here:** preserve genotype, developmental program, provenance, and
   subject continuity, then realize a phenotype under destination conditions.
   This intentionally may produce a different body from the same inherited
   program.

Either route may create a new body revision while retaining the same subject.
The prior revision remains pointable. A receiving vessel still derives its own
capabilities, and a weak consumer may preserve the Mesocosm phenotype facet
opaquely. **The destination declares compatibility, available accommodations,
and their costs; the traveler chooses among the feasible routes.** An
incompatible body is refused or offered regrowth. It is never silently
rewritten by either side.

Do not revise body v1 for this lane before the wing contract's subject,
body-revision, and part-address gates are stable. Local proof comes first.

---

## 9. Proof gates and file seams

### PD0. Close P2's account: **LANDED 2026-08-01**

`d9af641` removed `Organism::mass_mg`, routed feeding, upkeep, starvation,
death, carrion, and reproduction through body mass, and made upkeep scale with
what a critter carries. Burning pays the budget; growing raises the standing
cost.

**Done:** body mass is the ecology's only mass account, larger bodies cost more
to carry, the headed host shows budget and upkeep, and the next state migration
has a clean landed base. P0's repeat-play judgment remains open because only a
player can close it.

### PD1a. Allocation design pass: **COMPLETE 2026-08-05**

Treat this as a state migration before touching the enum. Today `Process` is
derived on every call and stored nowhere. Adding allocation creates state that
must be constructed, updated with growth, tombstoned with loss, serialized,
replayed, explained, and projected.

One ownership ruling is already settled by the wing contract: functional links
and process allocation are an **optional Mesocosm phenotype facet**, not
required primitive topology in `mesocosm.body/v1`. Local Rust layout does not
get to change that portable authority boundary.

The design pass must compare at least:

1. allocation fields carried locally beside each `Part` inside
   `BodyDocument`, then projected into a separate phenotype facet;
2. a `PhenotypeState` beside `BodyDocument`, keyed by stable `PartId`;
3. a wrapper that owns anatomy and phenotype and makes mutations transactional.

"Smallest diff" is not the criterion. The choice must prevent orphaned
allocation when a part is attached or severed, keep structural topology usable
without Mesocosm semantics, and let capability evaluation receive anatomy,
phenotype, and environment explicitly.

The state owner must hold one finite allocation mosaic per participating
`PartId`. Process sites compete for its capacity; all rearrangement is an
ordered developmental event; and mosaic adjacency must survive snapshot,
replay, branch transfer, and the optional phenotype projection. The mosaic is
an authoritative cell graph; any 2D inventory layout is a projection of it.

**Done when:** one state owner is ruled with constructor, attach, sever,
snapshot, replay, explanation, and body-v1 projection paths drawn; failure
atomicity and capacity conservation are named; carry-body and regrow-here
projection paths are distinguished; fixture changes are bounded; and the
implementation gate has exact file seams and tests. The pass also names one
proposal shape and validator used by both direct and automatic arrangement,
draws the boundary between a candidate phenotype and its committed
developmental program, and leaves lineage revision and adoption to the epoch
boundary. No state migration lands in PD1a.

#### Ownership ruling: one transactional wrapper

The local owner will be a private `BodyPhenotype` wrapper containing a
`BodyDocument` and an `AllocationState`. This selects the third candidate.

- Allocation fields inside `Part` are rejected. They would make the structural
  anatomy document depend on Mesocosm process semantics and would change the
  body-only bytes consumed by mesh, Lens, and other vessels.
- A freely mutable `PhenotypeState` beside a freely mutable `BodyDocument` is
  rejected. It permits exactly the split account PD0 removed: a part can be
  attached, severed, or restored without the allocation state following it.
- The wrapper keeps both representations independently readable while making
  every mutation that affects both representations one operation. A caller may
  construct and project a plain `BodyDocument`; a live `Organism` owns a
  `BodyPhenotype` and exposes immutable `body()` and `allocations()` readings.

The wrapper fields are private. World mutation does not receive `&mut
BodyDocument` or `&mut AllocationState` separately. Body-only authoring tests
may continue to use `BodyDocument` directly, which keeps primitive topology
usable without loading the biology system.

#### Capacity ruling: structure and current availability are different

Each living part owns one stored, connected graph of capacity cells. There is
no second `capacity` scalar: structural capacity is the count of living cells,
and a process site's allocation is the disjoint set of cells it occupies.
Occupied plus free cells therefore equals structural capacity by construction.

PD1b seeds the graph deterministically from the part's integer geometry and
the admitted allocation rules at a developmental event. The initial generator
is a coarse orthogonal tissue lattice whose axes follow `half_extent`; it stores
cell ids and adjacency, not coordinates or renderer voxels. Different UI
layouts may draw the same graph. A later developmental program may choose a
different admitted topology, but changing topology is itself a paid, ordered
developmental event.

Current availability is evaluated separately from structural capacity. Mass,
condition, starvation, environment, or a missing input may make allocated
cells dormant or ineffective without rewriting the mosaic. Ordinary grazing
and upkeep therefore do not shuffle organs every tick. Irreversible injury,
shrinkage, or remodeling explicitly tombstones cells and deterministically
deactivates any site that no longer owns a valid connected subgraph. Recovery
can reactivate an intact allocation. The hard cell ceiling is core safety
policy; the lower active capacity and cell quantum belong to world rules and
are configurable.

#### Lifecycle and atomicity

| Operation | Anatomy and allocation consequence |
| --- | --- |
| Construct | Build anatomy and one mosaic for every living part in temporary state, validate all invariants, then publish the wrapper. |
| Develop from a recipe | `Recipe + Soma` first creates a real `BodyDocument`; allocation is seeded against those actual parts in the same unpublished candidate. |
| Attach or graft | Preflight the new part, source provenance, mosaic, initial sites, and all costs. A planned mirrored pair validates together before either side commits. |
| Sever | Compute the anatomy subtree once, then tombstone the same `PartId`s and their mosaics in one commit. Historical cells and sites remain explainable but cannot contribute. |
| Remodel | Apply one complete validated allocation proposal to temporary state. A refusal leaves body, allocation, budget, and causal record byte-identical. |
| Snapshot | `BodyPhenotype` derives deterministic serialization inside each `Organism`; whole-world capture needs no hand-written field list. |
| Replay | Existing growth intents seed the same mosaics; a new developmental intent carries the complete proposal and is validated again against replayed state. |
| Explain | A capability trace cites process definition, part, site, cells, satisfied or missing inputs, cost, and the current environmental reading. |

Attach and sever on a standalone `BodyDocument` remain anatomy operations.
They cannot create a live organism with orphaned phenotype because only the
wrapper is accepted by `Organism` after PD1b.

#### One proposal and validator

Direct arrangement and auto-arrange both produce one `AllocationProposal`.
It carries the expected digest of the current `BodyPhenotype` and the complete
desired sites for every part it touches, rather than an order-dependent series
of drag operations. Each proposed site names its admitted `ProcessRef`, source
and expression cause, and a sorted set of existing cell ids.

One native validator checks the expected digest, living part addresses,
definition digests, source provenance, cell existence, disjoint occupancy,
connected site subgraphs, site requirements, budget and graph limits. It
either returns one `ValidatedDevelopment` with cost and explanation or one
specific refusal. The runtime accepts the proposal, not a host-prevalidated
result, so replay and co-op do not trust UI code. Proposal source is diagnostic
metadata at most; it cannot alter validation.

The exact mosaic is a candidate or somatic realization, not the heritable
artifact. At an epoch boundary, adaptation may translate its accepted intent
and lived result into a `DevelopmentalProgramDelta`: process preferences,
target part motifs, adjacency constraints, triggers, provenance, and paid
tradeoffs. It does not copy cell ids into the lineage. Re-realizing that program
under identical declared inputs must reproduce the founder preview; changed
conditions may produce another valid phenotype. Immutable lineage revision,
co-signing, adoption, and branching remain owned by the epoch boundary.

#### Carry this body and regrow here

`BodyPhenotype` projects anatomy and phenotype separately. The body-v1 path
receives primitive topology and stable addresses once W1 supplies them. The
optional Mesocosm phenotype facet receives the allocation graph, sites,
definition digests, condition, and provenance. Neither projection carries a
capability verdict.

- **Carry this body** sends the current anatomy revision plus the exact
  phenotype facet. A destination either admits the definitions and preserves
  the allocation, offers an explicit adaptation that creates a causally linked
  revision, or refuses carry.
- **Regrow here** sends the developmental program, acquisition provenance, and
  the prior revision reference. The destination realizes a new body and
  allocation under its conditions. The old mosaic is history, not a body
  template.

The local wrapper does not settle the v1 subject or revision schema and is not
serialized wholesale as the portable body profile.

#### Developmental anatomy prerequisite

The allocation audit exposed an earlier join that must precede PD1b. V2 itself
did not harden incorporation-grown bodies because Lens projects any
`BodyDocument`. The axial menagerie did, however, still use a renderer-only
`critter::Body::from_plan`, while `Appendage::role` reached no authoritative
part.

The first half landed 2026-08-05 in `mesocosm-core::development`:
`Recipe + Soma + PartPalette` now produces one mass-conserving
`BodyDocument`. Axial segments form the dependency spine, appendages attach to
the segment that expressed them, and every supplied part template is refused
unless its geometry classifies as the promised role. The Lens menagerie now
uses `BodyLensProjection` over that document and the parallel recipe-specific
renderer constructor is gone.

The live join landed 2026-08-05. `Species::realize` is the one unpublished-body
path used by world founders, offspring, and future adaptation previews. Every
organism stores the entropy that realized it. `World` stores its admitted
`PartPalette`, because the recipe is lineage-local while the materials and
geometry in which it grows are world-local. Both are snapshotted.

Filial provisioning is binding. A birth whose paid mass cannot keep every
expressed part positive waits without spending mass, allocating an id, or
advancing ecology entropy. Genesis has no parent ledger, so a rare undersized
founder begins at the recipe's exact structural mass floor. Incorporation
remains a somatic attach during an epoch.

`Chronicle::found` now also consumes a local recipe, developmental seed, mass,
and palette. It grows local topology, maps historical origins where sites
exist, and tombstones locally addressed loss subtrees. Origins without a site
remain in the chronicle rather than forcing another game's geometry into this
one.

The migration exposed one genuine multi-part ledger defect: upkeep and
reproduction still debited only the root. `Organism::spend_mass` now folds over
all living parts in stable order. Without that correction, non-root mass was
unspendable and consumer-only populations could reproduce their way around
starvation.

#### Exact PD1b seams and receipts

- `crates/mesocosm-core/src/phenotype.rs`: private `BodyPhenotype`, lifecycle,
  digest, allocation readings, and explicit anatomy access;
- `crates/mesocosm-core/src/phenotype/allocation.rs`: cell graph, sites,
  conservation and activity;
- `crates/mesocosm-core/src/phenotype/develop.rs`: proposal, shared validator,
  atomic commit and explanations;
- `crates/mesocosm-core/src/process.rs`: qualified ids, definition digests,
  the three native definitions, registry, and explicit capability evaluation
  over anatomy, allocation and environment;
- `crates/mesocosm-core/src/organism.rs`: replace public `body` storage with a
  private `BodyPhenotype`; keep body, mass and projection readings;
- `crates/mesocosm-core/src/world/act.rs`: route incorporation and the new
  developmental intent through wrapper transactions;
- `crates/mesocosm-core/src/organism/ecology.rs`: retain the landed
  recipe-developed offspring path while routing mass and condition changes
  through the phenotype wrapper without rewriting mosaics;
- `crates/mesocosm-core/tests/embodied.rs` plus a focused allocation test:
  native parity, exact refusal explanations, attach/sever lifecycle, stale
  proposal refusal, direct/automatic parity and conservation;
- snapshot and replay fixtures: one intentional current-schema bump. No
  compatibility shim for unreleased world bytes. Standalone `BodyDocument`
  bytes and the V2 Lens/mesh/Isometry projection contract do not change.

The implementation receipts are: every living part has exactly one mosaic;
every living cell is occupied once or free; an invalid multi-part development
leaves the complete wrapper and budget unchanged; severing removes the same
subtree from capability evaluation and allocation activity; mass loss can make
a site dormant without moving it; restoration and replay agree; the same valid
proposal from direct and automatic arrangement produces byte-identical state;
and a body-only projection still decodes without Mesocosm phenotype semantics.

### PD1b. Native ProcessDef migration: **LANDED 2026-09-01**

Execute PD1a's ruling. Replace the closed `Process` enum as identity authority
with registry-backed `ProcessDef` records for the **four** existing native
processes. Existing geometry deterministically seeds their initial allocation,
and `Reach` keeps exactly its P2 semantics.

**Correction, recorded 2026-09-01.** This gate was written when there were
three natives. DC1.5 added a fourth — `Process::Fix`, expressed by
`Role::Plate`, the fixing/producer reading — together with `Process::ALL` and
the registry parity receipt. Every "three" in this gate means four; the
implementation covers all four and `the_registry_and_the_native_view_agree`
is what makes a fifth impossible to add silently.

Expected seams, subject to PD1a:

- `crates/mesocosm-core/src/process.rs`: ids, definitions, registry, allocation;
- the ruled phenotype owner: allocation lifecycle and part addressing;
- `crates/mesocosm-core/tests/embodied.rs`: parity, severing, explanations;
- snapshot and replay fixtures: one intentional format bump, with no legacy
  compatibility shim for unreleased data.

**Done when:** the four built-ins are ordinary definitions, every living
allocation names a living part, attach and sever cannot split anatomy from
phenotype, every part mosaic conserves capacity, rearrangement occurs only
through recorded events, existing reach outcomes and refusals hold, and a part
still cannot acquire a capability by editing a number. The same valid
candidate submitted through direct and automatic proposal sources lowers to
the same developmental instruction, and the same invalid candidate receives
the same refusal. Re-realization under changed declared conditions may produce
a different valid phenotype without changing process identity or provenance.

**All met.** See the 2026-09-01 Progress entry for the landed shape, the
receipts, and the residues PD2 inherits.

### PD2. One native played process: **LANDED 2026-09-01**

Hand-author one additional `ProcessDef` in Rust and play it before building the
pack or Lua machinery. Light capture leads; venom secretion is the bounded
fallback if light capture would require the whole channel evaluator.

This is the mechanic proof. It may use a native developmental fixture or an
explicit editor operation to allocate the process. That temporary authoring
path is deleted when the pack/Piccolo path replaces it; it does not become a
second permanent system.

**Done when:** acquiring or expressing the process creates a readable choice;
allocation locates it on a part and charges its cost; world conditions can make
it useful or dormant; severing its dependency removes the consequence; and a
headed receipt explains all four states.

Only this gate chooses the first process. It does not open a catalog pass.

**All met.** `Process::Secrete` — a gland — is the bounded fallback §7 named
rather than light capture: light capture stores captured energy somewhere,
which is a flow that has to travel from an exposed surface to an account, and
PD2 is scoped to prove one process without building PD6's channel machinery
first. A gland needed no flow at all — it prices and reads off allocation,
mass and the ground under the body — and it still crosses body, world,
ecology and explanation with one small definition. See the 2026-09-01
Progress entry for the landed shape, the four states, the cost derivation,
and the receipts.

### PD3. Static pack admission: **LANDED 2026-09-01**

Encode the already-played PD2 definition in a data-only pack, validate it,
lower it into the core ruleset, and remove the native authoring duplicate. Lua
is not involved yet.

Likely seam: a new MPL-2.0 `mesocosm-phenotype` crate for pack discovery,
admission, and later Piccolo hosting. It may depend on `mesocosm-core`; core may
not depend on it.

**Done when:** the packed definition lowers to the same `ProcessDef` and game
outcome as the native proof; namespaced ids do not collide; path escape and
malformed schema are refused; changing one rule-bearing byte changes the
ruleset digest; and a snapshot identifies the exact admitted ruleset.

**The native authoring duplicate this gate removes** is PD2's temporary
editor operation and its receipt tool, named so the deletion is a checklist
rather than a search: `Intent::Rearrange` and `Allocate`
(`crates/mesocosm-core/src/world/intent.rs`), `World::rearrange` and the whole
of `crates/mesocosm-core/src/world/rearrange.rs`, `Outcome::Rearranged` and
`Event::Rearranged`, and `crates/mesocosm-genet/examples/pd2_receipt.rs` —
once the pack loader can propose the same accepted allocation the native
fixture did, so the deletion does not first cost the gate its receipt. What
survives underneath all of it, unowned by this door: `BodyPhenotype::develop`
and its one validator (PD1b), `Process::Secrete` and its `ProcessDef` (PD2),
`Organism::bite_mg` / `charged_mg` and the `upkeep_for_body` rent term (PD2),
and the vitals panel's gland reading (`mesocosm-views/src/vitals.rs`, PD2).
PD3 gives the definition a second, packed door; the native one is removed once
that packed door — or PD4's Piccolo proposal, whichever actually replaces this
gate's fixture first — can walk it through the same validator to the same
result.

**All met.** The pack is JSON, `mesocosm-phenotype` is the door, `Intent::
Express` replaced `Intent::Rearrange`, and the whole named checklist above is
executed. See the 2026-09-01 Progress entry for the format and why, what
admission refuses, what is rule-bearing in the ruleset digest, the parity
receipt, and the residues PD4 and PE3 inherit.

### PD4. Piccolo authoring parity: **LANDED 2026-09-01**

Add the typed request/proposal bridge, native validator, bounded runner, and one
declared fixture pair for the process already proven at PD2 and packed at PD3.

**Done when:** Piccolo can propose the same accepted allocation the native
fixture produced; the same context and entropy produce the same proposal and
draw trace; contrasting developmental contexts can produce different
phenotypes from the same body plan; unknown ids, invalid parts, excessive
output, and exhausted fuel refuse cleanly; Lua has no direct world mutation
path; and the temporary native authoring path is gone.

**All met.** `piccolo = "0.3"` (0.3.3, the version Isometry proves), the
`expression/` and `fixtures/` arms of §5's pack sketch are built, and PD3's
own residue is what the slice opened with: `BodyPhenotype::develop` takes a
`&Registry` and a `World` carries the set it admitted, so a stale ruleset is a
reachable refusal rather than an argument. See the 2026-09-01 Progress entry
for the Request and Proposal shapes, the runner's policy numbers, the fixture
pair and its draw trace, why "Lua cannot mutate" is structural, and what PE3
inherits.

### PD5. Filial expression: **LANDED 2026-09-02**

Connect the authored process to P4's adaptation bridge. Metabolized source
material widens the candidate bank, a chosen developmental change references
that lived source, and the descendant regrows a phenotype that may express it.

**Done when:** somatic incorporation, dormant acquisition, and filial expression
are distinct records; unplayed lineages use the same expression path; the
source provenance survives; and the old trait array has either met every
deletion condition or remains explicitly provisional.

**All four met.** The **development program** — the second of §2's three
compiled programs — is `mesocosm-core::program`: an append-only list of
immutable revisions on a lineage, each declaring *sites* (a part role, an
admitted `ProcessRef`, a bounded cell count) rather than a body. A birth under
one runs it through `BodyPhenotype::develop`, so a descendant of a line that
came to the gland is born secreting. The four done-conditions in order:
`Event::Grew`/`Event::Expressed`, `Event::Discovered` and `Event::Inherited`
are three different records over one walked line
(`incorporation_discovery_and_filial_expression_are_three_records`);
`World::express_filially` reads no `controlled`, and
`an_unplayed_lineage_takes_the_same_path` receipts it; provenance survives as
`program::Citation`, which carries the condition **and** the discovery digest,
so a revision resolves to the exact realized candidate it was committed
against; and §D4 of the phenotype plan carries the deletion receipt — three of
five conditions met, `epoch::Trait` untouched and explicitly provisional. See
the 2026-09-02 Progress entry for the shapes, the refusal vocabulary, and what
PE3 inherits.

### PD6. Channels under pressure

Add only the flow ports and relation needed by the first process path that
cannot be represented by local anatomy. Start with local or routed. Redundant,
centralized, and quorum arrive when a body proves them. Broadcast remains cut.

**Done when:** a working path survives through valid typed connections, an
incompatible graft needs a visible adapter or refuses, severing a connector
breaks the path, and the explanation names the missing flow.

### PD7. Persistence and authority

Commit a validated expression result, restore it without Lua, and apply it on
two simulation participants.

**Done when:** snapshot plus ordered record converges; a peer does not rerun
Lua; stale rulesets and missing lowered definitions refuse explicitly; and two
concurrent proposals spending the same budget resolve through the named world
materializer rather than last-writer-wins.

### PD8. Extraction audit

Compare Mesocosm's runner with Isometry's actual generator runtime after PD4
through PD7 are proven.

**Done when:** either a small sandbox/entropy/tagged-value crate has two real
consumers and moves under `MIT OR Apache-2.0`, or the audit records why the two
hosts are still meaningfully different. No extraction is also a valid receipt.

---

## 10. Scheduling

This lane interleaves with the phenotype plan rather than replacing it:

1. PD0 is complete;
2. PD1a is complete and the core recipe-to-body authority proof is landed;
3. live founding, offspring, local regrowth, and the founder-preview seam now
   consume the shared developer;
4. migrate native process identity and allocation in PD1b;
5. play one native process at PD2 before building authoring infrastructure;
6. execute P3 branch transfer with stable process identity and allocation;
7. encode the proven mechanic as a static pack at PD3;
8. prove Piccolo authoring parity at PD4 before P4 adaptation;
9. use PD5 as P4's filial phenotype replacement proof;
10. add PD6 only when the played process needs a non-local path;
11. let P5 contested flow consume the proven process and ecology vocabularies;
12. run PD7 and the wing projection gate before PD8 extraction.

P0's playfeel judgment remains a user test throughout. A technically correct
process system does not answer whether burn or grow is worth choosing again.

---

## 11. Stop rules

- Do not use `ProcessDef` as a universal gene or trait record.
- Do not let Lua run the ecology loop or mutate world state directly.
- Do not store capability verdicts on parts or organisms.
- Do not accept a process without a native consumer for its flows.
- Do not give direct arrangement or auto-arrange separate validation rules.
  They are proposal sources over one developmental authority.
- Do not build pack or Lua infrastructure before PD2 proves a process worth
  authoring.
- Do not add a broad process catalog after PD2; one played process authorizes
  an authoring path, not a biology encyclopedia.
- Do not let scripts choose their own cost or bypass acquisition provenance.
- Do not add a channel relation because an authored string names it.
- Do not silently substitute a definition when a ruleset is missing.
- Do not make pack version text stand in for a content digest.
- Do not make peers rerun Lua to converge.
- Do not turn ordered resource conflicts into a universal CRDT.
- Do not extract the Piccolo host before Isometry and Mesocosm are demonstrably
  the same consumer at that boundary.
- Do not project Mesocosm capability authority into another vessel.
- Do not retire `epoch::Trait` until the phenotype plan's five deletion
  conditions pass.
- Do not evaluate discovery conditions by polling every organism every tick;
  route relevant accepted evidence to bounded condition evaluators.
- Do not admit an instantaneous dependency cycle. Biological feedback crosses
  an explicit state boundary and a later tick.
- Do not leave stacking to iteration order. Every repeated evaluator declares
  and tests its combination law.

---

## 12. Open rulings

These are intentionally deferred to the gate with evidence:

1. The first non-native played process at PD2: light capture is recommended;
   venom secretion is the bounded fallback.
2. The exact portable shape of a lowered `ProcessDef`: settle after local
   snapshot and branch-transfer proofs, before body v1.
3. Whether a world embeds lowered rule definitions or content-addresses them
   beside the snapshot: PD3 and PD7 must prove missing-pack behavior before
   choosing.
4. Whether a second scripting backend is ever useful: no abstraction or ruling
   until another real consumer asks.
5. Whether the first played process needs surface, interior, or junction cell
   kinds. PD1a ruled structural graph capacity and separate current
   availability; PD2 must supply a real consumer before cells gain kinds.
6. Stabilized hybrid identity: a newly admitted derived `ProcessDef` versus a
   heritable compound recipe that retains its parents at evaluation time.
7. Disfavoured graft semantics: hard refusal by default versus an adapter tax
   that a learned compatibility process can reduce.

---

## 13. Findings

- **2026-09-01, generated-trait execution:** the existing plan already has the
  correct representation split: `ProcessDef` for transformations, discrete
  expression triggers for development, and acquisition evidence outside both.
  A universal trait AST would erase that ownership. Kappa supports typed local
  rules, reachability, influence, and causal analysis as validation precedent;
  its stochastic scheduler is not compatible with Mesocosm's authoritative
  fixed-step transaction path.

- **2026-08-01, Mesocosm P2.** `crates/mesocosm-core/src/process.rs` has a
  closed three-variant native `Process` enum. Geometry classifies each process;
  `Reach` is the only capability, and severing removes it. This is a sound
  parity fixture for PD1, not yet an extensible definition system.
- **2026-08-01, phenotype boundary.** The phenotype plan already distinguishes
  process identity from channel flow, cuts broadcast pending evidence, and
  forbids a shared body/ecology evaluator before two sovereign proofs. This
  plan preserves those rulings.
- **2026-08-01, Isometry donor.** `isometry-system` uses Piccolo 0.3 with
  `Lua::core()`, host-owned entropy, finite fuel, bounded tagged output,
  path-contained declared pack assets, exact fixtures, and commit-result
  generation. The game-specific request and result types remain inside
  Isometry, which is the correct reuse boundary for Mesocosm too.
- **2026-08-01, P2 closeout.** `d9af641` removed the scalar mass account,
  split organism ecology into its own module, routed substance through body
  parts, and made upkeep scale with biomass. PD0 is complete.
- **2026-08-01, implementation review.** Registry, pack loading, and Piccolo
  would otherwise put three infrastructure gates before the first new process
  was played. PD2 now proves one hand-authored native mechanic first; PD3 and
  PD4 then replace its authoring path with pack data and Piccolo.
- **2026-08-01, maintainer design answers.** Processes share finite per-part
  capacity; an authoritative cell graph makes local and whole-mosaic evaluation
  available; somatic compounds may stabilize into heritable forms at an epoch;
  allocation changes only through discrete developmental events; and the
  destination declares feasible crossing routes while the traveler chooses.
- **2026-08-01, graft-affinity proposal.** A default world's directed
  animal-like, fungal-like, and plant-like affinity cycle can balance grafting
  without becoming the trophic `Kingdom` enum. World data owns the graph; a
  disfavoured boundary may demand a capacity-consuming adapter whose penalty
  can be reduced by an evolved compatibility process.
- **2026-08-01, arrangement authorship.** Direct editing and auto-arrange are
  two sources of the same validated developmental proposal. Shared-lineage
  agreement adopts one child revision; disagreement branches at the epoch
  boundary rather than merging allocation cells.
- **2026-08-03, co-signing target.** The concrete mosaic is a founder preview.
  Direct and automatic arrangement author a developmental program, and that
  program is what shared-lineage players adopt or branch from.

---

## 14. Progress

- **2026-09-02, ruling: the affinity-table pack door.** Mark ruled the policy
  default the PD3/PD4 residue named: a pack-declared affinity overrides
  `Founding`; `Founding` is the fallback the world ships with. Recorded at
  both residue notes above; wiring it into `World::found` and `WorldRules` is
  PE4's, per the [playable ecology plan](2026-08-31_playable_ecology_plan.md).

- **2026-09-02, PD5 complete (with P4a): a line commits, and its descendants
  arrive already carrying it.**

  **The second compiled program, and it is not a `ProcessDef`.**
  `mesocosm-core/src/program.rs` holds it: a `Program` is an append-only list
  of `Revision`s on a `Species`, and a `Revision` is `{ id, parent, cites,
  sites, founded, digest }`. What it declares is `DeclaredSite { role, process,
  cells }` — **a role, never a part id**, because the descendant that grows
  under it has not been developed yet and its part 3 is nobody's to predict.
  That is §2's development program, in the shape §3 already had a vocabulary
  for: a `Candidate` is *which admitted process, on what shape, at what bounded
  capacity*, so `DeclaredSite::of` is those same three fields and committing
  needed no second vocabulary.

  **The founding revision is stored nowhere, and that is the design rather than
  an omission.** What a body does with no program — allocation seeded from
  geometry — *is* the founding revision: no parent, no citation, no declared
  site. Writing it down would put a record in every snapshot for the absence of
  one, so `Program::current()` answers `None` for it and the committed list
  starts empty. The consequence for the fixture is measured below.

  **Immutability is structural.** `Program::commit` appends and is the only
  mutation the type has; there is no `&mut Revision` anywhere, and
  `Species::commit` is `pub(crate)` with `World::revise` as its only caller —
  so a revision is a world transaction with a record, not a field a host sets.
  Epoch-boundary plan §2's *neither result edits the parent* is therefore a
  property of the API rather than a discipline. `parent` is `Option` and its
  `None` occurs: the first commit on a line descends from the founding
  revision, which has no record to name.

  **Filial expression, and what a refusal records.** `World::express_filially`
  runs after the ecology's birth pass, over exactly the ids that pass
  allocated — read from `next_organism` before the step, so nothing scans for
  something that looks new. For each newborn whose line has a current revision
  it calls `program::express`, which builds the same `AllocationProposal`
  `Candidate::propose` builds and offers it to `BodyPhenotype::develop`. On
  acceptance the price is PD2's, unchanged — cells whose expression changed, at
  the part's own `cell_mg` — out of the child's reserve and into the column
  under it, with a `flow::Process::Develop` record and an `Event::Inherited`.
  On refusal the birth **still happens**, under geometry seeding, and
  `Event::Unexpressed` names the revision and the reason:
  `NoSite { role }` (the ordinary case — a bulk consumer has nowhere to put a
  gland), `Refused(Refusal)` (the validator's own boundary, carried whole), or
  `Unaffordable { needed_mg, held_mg }`. Nothing is published on a refusal: the
  development runs on a candidate copy and the price is read off the validated
  instruction, the same ordering `World::express` uses.

  **The ground charges what a line grows, and it is not a new number.**
  `Conditions::affords` asks `Organism::charged_mg`'s own rule one step
  earlier: an acquired process works only where the column could replace what
  it holds, so a founder does not grow more than the ground it founds on can
  charge, and keeps a one-cell site where it cannot. That is the rule PD4's
  `gland.lua` already reads, which is why the packed fixtures' **400 mg** and
  **20 mg** grounds mean the same thing in the core: on a plate whose tissue is
  worth 21 mg a cell, a five-cell declared site costs 105 mg, so rich grows
  five and lean grows one. Measured, on one program `0a8205a233fc757b` and one
  seed: rich grows a five-cell gland costing 105 mg to phenotype
  **`c71c8b16cfa97c56`**, lean a one-cell gland costing 21 mg to
  **`8e0f1d2ff8e31dbe`**.
  `one_program_grows_two_bodies_on_rich_and_lean_ground` asserts the two
  phenotype digests differ and **the program digest does not** — which is the
  ruling of 2026-08-03 made executable: the variance is expression of one
  inherited program, never an implicit mutation.

  **Three records, and one test walks all three.** Somatic incorporation
  (`Event::Grew`, `Event::Expressed`), dormant acquisition
  (`Event::Discovered`) and filial expression (`Event::Inherited`) are separate
  events with separate references — a part, a condition, a revision — and
  `incorporation_discovery_and_filial_expression_are_three_records` takes one
  line through all four (the commit's `Event::Revised` included) and asserts
  the birth is neither a discovery nor a meal.

  **Receipts.**
  - `mesocosm-core` lib **365** green (+6: `program/tests.rs`).
    `tests/embodied` **60** (+11, the new `embodied/lineage.rs`), one test per
    claim: the founding program declares nothing and changes no birth; a second
    commit appends and leaves the first untouched; a birth expresses its line's
    revision and pays for it; a child with nowhere to put it is born anyway and
    the record says why; a founder preview is the same body twice; one program
    grows two bodies on rich and lean ground; an unplayed lineage takes the
    same path; three records; the commit is ungated and says so in one place;
    and the two commit refusals by name.
  - `tests/flows` **12** (+1):
    `a_filially_expressed_birth_reconciles_to_the_milligram` — PE0's
    whole-compartment reconciliation across the birth tick, plus one `Develop`
    record out of the child's reserve for exactly what `Event::Inherited` says
    it cost, and the child's reserve equal to what the birth gave it less that.
  - Standing gates `matter/flows/succession/embodied`: **6 + 12 + 7 + 60**
    green. `cargo test -p mesocosm-phenotype`: **40** green, unchanged
    (16 + 17 + 7).
  - `cargo test --workspace --release`: **40 suites, 764 tests, 0 failures**
    (was 40 / 746). Clippy `-D warnings` clean in both profiles, `cargo fmt
    --all --check` clean, `cargo check -p paredros-room --features r1-proof`
    builds with its one pre-existing `dead_code` warning on
    `brick::retarget_from_ground`.
  - **The instrument is unmoved.** The drawn baseline was re-run over all ten
    seeds and compared against `dc4_roster.json` seed for seed — verdict,
    start, peak, peak tick, end, cumulative births, cumulative deaths, end
    kingdom counts and end biomass identical on every one, and `outside 0 -> 0`
    on every sample of every seed. **0 breathes / 10 thins / 0 boil / 0
    collapse** stands. It could not have moved by construction: the instrument
    drives with `Intent::Idle` only, so no line ever discovers anything, no
    line ever commits, and `World::express_filially` returns on its first
    comparison every tick. The run was stopped after the baseline batch, before
    it could rewrite `dc4_roster.json` with new timing on an unmoved result;
    the file is byte-identical to what DC4 recorded.
  - **The fixture hash moved, and the run did not.** `Species` gaining a
    `program` field costs one postcard varint per lineage in the snapshot, so
    the whole-world hash went `1b1f866cb9138d40` -> **`fb358a75a0b0bff6`** and
    the trace was re-recorded. The demo carries no `Revise`, every lineage in
    it holds the founding program, and the recorded intent stream is **byte
    identical** to PD4's — 3,100 intents, seed 7, 916 founders, compared
    element by element — so what moved is the serialized shape and nothing the
    world did. `the_founding_program_declares_nothing_and_changes_no_birth`
    pins both halves: an uncommitted program encodes to one byte, and a birth
    under it writes neither filial record. Headed `--replay` runs the
    re-recorded 3,100-step trace over **775** frames and matches, exit 0; a
    hash falsified by one bit reports the mismatch and exits 1. Ground revision
    17, 33 body parts, 40 drawn roster — PD4's numbers, unchanged. The demo's
    census is unchanged too: Move 2,726, Deposit 244, Carve 73, Idle 40,
    Metabolize 12, Resume 3, Graft 1, TakeControl 1, so its discovery, its
    branch transfer, its three births and its succession all survive
    (`mesocosm-genet` lib **18** green, the demo tests included).
  - **Splits at the ceiling**, per the workspace rule: `history/tests.rs` out
    of `history.rs`, and `world/revise.rs` and `world/filial.rs` as their own
    rooms rather than growing `world/act.rs`.
  - **One file outside the slice's stated scope**, and it was forced:
    `mesocosm-views::refusal_words` matches `Rejection` exhaustively, so the
    new `Rejection::Unrevised` arm had to be given its four plain sentences or
    the workspace would not compile. Nothing else in the view lane was touched.

  **Residues, and what PE3 inherits.**
  - **The commit is ungated, in one named function.**
    `World::revision_admitted_now` returns `true` and says why: PE3 gates a
    revision to the lineage checkpoint, a checkpoint needs a deterministic
    condition that ends an epoch, and that is playable ecology plan §6 ruling 2
    and Mark's. Ruling otherwise is a one-line change there — the same
    placeholder arrangement PE1 used for `Checkpoint::default_answer`. It has
    **two consumers** so that one line actually lands: `World::revise` consults
    it and refuses `Unrevised::NotYet`, and a host reads it to know whether the
    verb is on offer. `NotYet` is unreachable today and is present for
    `Miss::AnotherTook`'s reason — a door with no way to say *not now* would
    have to grow one after the ruling, making the refusal vocabulary depend on
    which answer arrived.
  - **An authored script is not yet a revision source.** A revision's declared
    sites come from the discovery record. PD4's residue stands unchanged: the
    authored proposal has no `Intent`, and giving a host one before PE3 decides
    *when* a review happens would be inventing the checkpoint from the wrong
    end. What PE3 reviews is two proposal sources over one screen.
  - **A revision declares one site today**, because a discovery grants one
    candidate. `Revision::sites` is a `Vec` and `program::express` loops, so a
    second arrives without a shape change.
  - **Two declared conditions, both with consumers**: `ground_mg` decides how
    much of a program a founder expresses, `material_mg` pays for it. They are
    the same two names PD4's `Request` carries, so a preview and an authored
    proposal are declared against one context.
  - **Nothing scores anything.** The scorer, the epoch trigger and life-stage
    pricing were left untouched deliberately; see the phenotype plan's P4 entry
    for which of its clauses each blocks.

- **2026-09-01, PD4 complete: an author gets a say, and the validator keeps
  the last word.**

  **PD3's residue first, because everything else rests on it.**
  `BodyPhenotype::develop` takes a `&Registry` now, and `World` carries the set
  it admitted rather than only its digest. PD3's own entry said this was
  honest only while the shipped pack lowered to `Registry::native()`; a door
  that can hand the validator a proposal is exactly the thing that stops it
  being honest, so it was the first change of this slice rather than the last.
  `World::founded_on(seed, count, founding, Arc<Registry>)` is what a pack door
  founds through, `World::ruleset()` is what the validator reads, and
  `Refusal::UnknownProcess` is now reachable from the played door:
  `a_world_validates_against_the_ruleset_it_admitted` founds a world on a
  ruleset with the gland removed, lets its line come to the native candidate,
  and gets the definition named back rather than substituted.

  **The set is carried, not saved, and that is why no hash moved.** A world
  records the *identity* (PD3's ruling, unchanged); the definitions are
  `#[serde(skip)]` runtime carriage. `snapshot::restore_under` therefore
  widened from taking a `WorldRules` to taking the `Arc<Registry>` itself: it
  compares the digest first and attaches the set only if it is the one the save
  ran under, so a restored world cannot hold definitions it did not admit.
  Nothing in the serialized world changed, so the whole-world hash is still
  `1b1f866cb9138d40` and no fixture was re-recorded.

  **The Request: a frozen picture, and nothing borrowed.** `trigger` (one
  variant, `Discovery`, because one is played), `ruleset` digest, body
  `revision`, the `expect` digest a lowered proposal must carry, every admitted
  `definitions` entry as id / `expressed_by` / `seeding`, every living part as
  `{ part, role, cells, free, cell_mg, sites }`, the `candidates` the line has
  come to as qualified ids, an integer `material_mg`, and declared
  `conditions`. **One condition today, `ground_mg`** — what the soil column
  under the body holds — because that is what PD2's played process already
  reads, rather than a knob invented to give a script something to branch on.
  Every field is an owned copy; there is no handle to follow back to a live
  value, which is how "scripts cannot inspect hidden world state" becomes a
  property rather than a rule.

  **The Proposal: three fields, and no fourth.** `{ part, process, cells }` per
  site — plan §4's *which admitted process on which part at what bounded
  capacity*. Not a cost (the door prices the accepted instruction), not a cell
  address (`lower` lays tissue out), not a revision (the host froze one in).
  What a script could get wrong and the game would then have to live with is
  simply not expressible. `lower` resolves ids against the world's ruleset,
  hands out cells from the high end of the lattice downward in the order the
  script listed them, and leaves unclaimed tissue doing what it did — the same
  suffix rule `Candidate::propose` uses, and **in the same order**, which is
  what makes the authored and the native proposal produce one `Instruction`
  rather than two that merely look alike. The validator hands out site ids in
  proposal order, so that order is rule-bearing at the mosaic; it cost one
  failing assertion to find and is stated in `lower`'s own comment now.

  **The runner's policy, in numbers.** Fuel **8,192** for the whole call, the
  chunk's own top level included; **4,096** output bytes, measured after
  decoding rather than on the wire; nesting **8**; collection length **64**;
  and **4** entropy draws. Small because the job is small — this decides where
  a handful of cells go on one organ. Named refusals: `NoEntrypoint`, `Fuel`,
  `Output`, `Depth`, `Collection`, `Malformed`, `Script`, `UnknownProcess`,
  `UnknownPart`, `TooMuchTissue`, and `Validator(Refusal)` — the last carrying
  the one validator's own boundaries whole rather than restating them, because
  direct, automatic and authored arrangement are three proposal sources over
  one developmental authority.
  `the_validator_still_owns_its_own_boundaries` asks for a gland on a bulk root
  and is refused `SiteMismatch` at the validator, not at this door.

  **Lua cannot mutate, and it is four structural facts rather than a promise.**
  (1) The runner registers **no host function at all**: there is no callback
  into Rust, so there is no Rust value a script can reach. (2) `Lua::core()`
  omits `io`, and piccolo 0.3 has no `os`, `require`, `dofile`, `loadfile`,
  `load`, `package` or `debug` — the probe in `lua_has_no_world_mutation_path`
  counts all ten from inside and gets zero. (3) **`math.random` and
  `math.randomseed` are deleted at load**, which the donor does not do: piccolo
  seeds them from `SmallRng::from_entropy()`, so an authored script in an
  unmodified `Lua::core()` has a real source of OS randomness and the same
  context would not produce the same proposal. That is the one place this
  runner is deliberately stricter than Isometry's. (4) The API takes
  `&Request` and `&Entropy` and returns a `Proposal`; the same test pins both
  function signatures, so growing a mutable argument stops it compiling.

  **Entropy is drawn before the call, not on demand.** `Entropy::from_seed`
  takes `DRAWS = 4` numbers off the core's own `Rng` — SplitMix64, the stream
  every other seeded decision in this game comes out of, so no second generator
  was invented — and hands them over as plain integers. A draw-on-demand
  callback would have been a host function in the globals, which is exactly the
  thing point (1) says there are none of; a fixed tape is the stronger
  arrangement and makes the recorded trace exactly what crossed the boundary.
  **A deliberate divergence from the donor**, whose `EntropyTape` is a
  host-side cell the runtime draws from during the call.

  **The fixture pair, and the contrast is the ground.** One body plan — the
  bulk consumer carrying the twelve-cell `[6, 4, 1]` frond PD2's fixtures grow,
  so `cell_mg` is 23 and a five-cell gland holds 115 mg — one script, one seed
  (**2**, whose draws are `10905525725756348110`, `13819372491320860226`,
  `10987583248141275951`, `14119491246550939236`), and two declared grounds.
  `gland_rich_ground.json` at 400 mg: the ground can charge what the line is
  minded to spend, so **five cells of the frond become a gland** — exactly what
  the native fixture's `GLAND_CELLS` proposed, lowering to the same complete
  desired state and the same `Instruction`. `gland_lean_ground.json` at 20 mg:
  it cannot, so the line keeps a **one-cell** gland and eleven cells go on
  fixing. Two valid phenotypes with different digests, from one plan and one
  seed. Nothing in the script is a constant the request does not already carry:
  it reads the definition's own `expressed_by` to decide which shape to look
  for, and the part's own `cell_mg` to decide whether the ground can charge the
  ask.

  **The temporary native authoring path: nothing left to delete, and the
  deletion this gate did make.** PD3 executed §9's whole checklist —
  `Intent::Rearrange`, `Allocate`, `World::rearrange`, `world/rearrange.rs`,
  `Outcome::Rearranged`, `Event::Rearranged`, `pd2_receipt.rs` — so there is no
  second authoring surface for Piccolo to replace. Verified rather than
  assumed: a sweep for temporary-path markers across both crates finds only
  PD3's own historical notes. What PD4 deleted is `Registry::native()`
  **inside the validator**, which was the last place a development resolved
  against something other than the world it was happening in. `arrange` /
  `Aim` and `Candidate::propose` are not temporary and were not touched: they
  are plan §7's other two proposal sources, and the parity receipt is a claim
  *about* them.

  **Receipts.**
  - `mesocosm-phenotype`: **40** green across three suites (was 23) —
    `tests/admission.rs` (16, unchanged), `tests/packed_gland.rs` (7,
    unchanged), and `tests/authored_gland.rs` (**17** new), one test per
    done-condition and named for it:
    `lua_proposes_the_same_accepted_allocation_as_the_native_fixture`,
    `the_authored_proposal_walks_the_played_door_to_the_same_development`,
    `the_same_context_and_entropy_produce_the_same_proposal_and_draw_trace`,
    `contrasting_developmental_contexts_grow_different_phenotypes_from_one_plan`,
    `an_unknown_id_refuses_cleanly`, `an_invalid_part_refuses_cleanly`,
    `excessive_output_refuses_cleanly`,
    `an_overlong_collection_refuses_cleanly`,
    `exhausted_fuel_refuses_cleanly`, `a_missing_entrypoint_refuses_cleanly`,
    `a_malformed_proposal_refuses_cleanly`,
    `the_validator_still_owns_its_own_boundaries`,
    `a_stale_ruleset_refuses_at_the_one_validator`,
    `a_world_validates_against_the_ruleset_it_admitted`,
    `lua_has_no_world_mutation_path`,
    `the_script_the_pack_declares_is_the_script_that_runs`, and
    `a_script_cannot_express_what_the_line_has_not_come_to`.
  - `mesocosm-core` lib **359** green, unchanged in count: threading the
    registry moved call sites, not claims, and
    `a_restore_under_a_different_ruleset_is_refused_by_name` was restated
    against a real ruleset rather than a digest with no definitions behind it.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --test embodied`: **6 + 11 + 7 + 49** green, the same counts as PD3.
  - **No fixture moved.** The admitted set is `serde(skip)`, so the whole-world
    hash is still `1b1f866cb9138d40`. Headed `--replay` runs the recorded
    3,100-step trace over 775 frames and matches, exit 0; a hash falsified by
    one bit reports the mismatch and exits 1. Ground revision 17, 33 body
    parts, 40 drawn roster — every number in the receipt is P3's and PD3's.
    The demo's own census is unchanged because the recording is: Move 2,726,
    Deposit 244, Carve 73, Idle 40, Metabolize 12, Resume 3, Graft 1,
    TakeControl 1 — its discovery, its branch transfer, its three births and
    its succession all survive.
  - `cargo test --workspace --release`: **40 suites, 746 tests, 0 failures**
    (was 39 / 729), mesocosm-lens's 45 included and passing in the ordinary
    parallel run. Clippy `-D warnings` clean in both profiles; `cargo fmt
    --all --check` clean; `cargo check -p paredros-room --features r1-proof`
    builds with its one pre-existing `dead_code` warning on
    `brick::retarget_from_ground`.
  - **The instrument is unmoved.** The drawn baseline was re-run over all ten
    seeds and compared against `dc4_roster.json` seed for seed — verdict,
    start, peak, peak tick, end, cumulative births and cumulative deaths each
    identical on every one. **0 breathes / 10 thins / 0 boil / 0 collapse**
    stands. It could not have moved by construction — the instrument drives
    with `Intent::Idle` only and never reaches a development — and the
    admitted set is skipped from the snapshot, so no hash it reads changed
    either. The run was stopped after the baseline batch rather than carried
    through the other five, so it could not overwrite `dc4_roster.json` with
    new timing on an unmoved result.
  - **piccolo 0.3.3**, resolved from `piccolo = "0.3"` — the exact version
    `isometry-system` proves — and it builds on this workspace's toolchain,
    rustc 1.97.1. Five new crates in the lock (`piccolo`, `gc-arena`,
    `gc-arena-derive`, `hashbrown`, `sptr`), all from crates.io: this is a
    third-party library rather than a Merely one, so the workspace's git-first
    rule for the cambium and eidetic families does not reach it.

  **Splits at the ceiling**, per the workspace rule: `express.rs` opens onto
  `express/{request,proposal,marshal,runner,fixture}.rs` rather than growing
  one file, and `tests/authored_gland/refusals.rs` took done-conditions 4 and 5
  when the suite passed 600 lines.

  **Residues, and what PE3 inherits.**
  - **PE3 is the review this door is waiting for.** `Request::of` freezes one
    discovered condition on the played body; choosing among several, previewing
    what each would cost, and doing it at a lineage checkpoint are still PE3's
    — and there are *two* proposal sources to review over now rather than one,
    the game's candidate and an author's script, which is a screen over
    `candidate_proposal` and `Runner::propose` rather than a new intent.
  - **The authored proposal has no `Intent` yet, deliberately.** A test lowers
    it and calls `develop` directly, the same way `develop_played` does. Giving
    a host a way to say "run this script now" before PE3 decides *when* a
    review happens would be inventing the checkpoint from the wrong end.
  - **Four readings still resolve against `Registry::native()`**, and each
    belongs to its own gate rather than to this one: `Mosaic::seed` (the
    seeding rule), `discovery::conditions` (the condition table's granted
    reference), `BodyPhenotype::gland_reference` and `explain`, and `Kingdom`'s
    fixing reference. None is the validator, so none can admit a development;
    the honest consequence today is that a world founded on a ruleset the
    natives do not match *refuses* rather than substitutes, which
    `a_world_validates_against_the_ruleset_it_admitted` receipts. Threading
    them is PE4's, when a pack actually mints something new.
  - **One declared world condition.** `ground_mg` is the only quantized reading
    a script can branch on. A second arrives when a gate plays one; the shape
    (`name`, integer `value`, sorted) already takes it.
  - **The graft-affinity pack door is still unopened**, and still for PD3's
    reason: the ruling about whether a pack-declared affinity overrides
    `Founding` or the reverse is Mark's. It has moved from PD4 to PE4.
    ~~Ruling pending.~~ **Ruled by Mark, 2026-09-02: a pack-declared affinity
    overrides Founding; Founding is the fallback the world ships with.**
  - **PD8's extraction audit has its first real comparison now.** The runner,
    the fuel and output policy, and the tagged marshalling are recognisably
    Isometry's pattern; the entropy handling is not — a pre-drawn tape against
    a host-side draw — and the request, proposal, validator and lowering are
    sovereign game code as §4 requires. The stop rule holds: nothing is
    extracted.

- **2026-09-01, PD3 complete: the definition gets a packed door, and the
  editor operation is deleted.**

  **The format, and why.** JSON, and no new dependency: `serde_json` is
  already a direct dependency of two crates here and the format is what every
  trace, receipt and roster in this workspace already speaks. A manifest plus
  one file per definition, at `packs/mesocosm/`, matching §5's sketch minus
  the Lua arms PD4 owns. The four rule-bearing fields are exactly the four
  `ProcessDef` holds — `namespace`, `name`, `expressed_by`, `seeding` —
  because a pack that could declare a fifth thing would be a pack that could
  name a rule the core does not evaluate. `label` and `note` ride along as the
  author-facing text §3 puts outside rule authority, and the schema denies
  unknown keys: a typo is not a comment, and a definition whose `expresed_by`
  was silently ignored would be a world nobody chose.

  **`mesocosm-phenotype`, MPL-2.0, and the dependency runs one way.**
  `crates/mesocosm-phenotype` declares MPL-2.0 in its own manifest, per §5's
  license gate: definitions, loaders, validators, fixtures and game-specific
  schemas are game code, and only a *generic* sandbox crate may become
  `MIT OR Apache-2.0` after two hosts prove one extracted boundary. Nothing
  here is generic — the schema names this game's roles and its seeding rule.
  It depends on `mesocosm-core`; core does not depend on it, which is what
  keeps reading a directory out of a crate that must stay deterministic,
  integer-only and free of I/O.

  **Discovery is separate from admission**, so a host can list what it found
  and what each pack declares before lowering anything. Admission is
  all-or-nothing: four good files and one bad one admit nothing, because half
  a biology is not a smaller biology but a different one — a body citing the
  dropped definition would resolve `None` on every site it occupies.

  **What is refused, each by name.** `PathEscape` (a component walk refuses
  `..`, a root and a drive prefix without touching the disk, and a resolved
  path that leaves the resolved root catches a symlink out), `MalformedSchema`,
  `Unreadable`, `UndeclaredFile` (§5's "undeclared files are rejected": a
  definition present and unlisted would make the ruleset depend on what
  happened to be lying about), `UnknownAbi`, `DuplicateId`, `UnqualifiedId`,
  `UnknownRole`, `UnknownSeeding`, `NoSite`, `EmptyPack`, `NoManifest`. The
  unknown-word refusals keep the same discipline `Registry::resolve` keeps one
  scale down: a shape this world does not hold is an answer, never the nearest
  thing it does.

  **What is rule-bearing in the ruleset digest, stated.** The digest is taken
  over the **lowered definitions, not the file bytes** — hashing bytes would
  have made whitespace, key order and a `note` rule-bearing. `Registry::
  digest` folds each definition's own digest in *sorted* order, so the set is
  what counts and declaration order provably is not; `Registry::admit` sorts by
  qualified id and refuses a repeat, so an admitted registry is canonical
  whatever order a manifest was written in. The format ABI is an admission gate
  rather than a digest input, because a ruleset is what its definitions say and
  the ABI decides whether this build can read the file at all. `NATIVE_DEFS`
  was re-sorted into the same canonical order, which is why the packed and
  native registries are `==` rather than merely digest-equal.

  **`ProcessId` widened to owned strings**, as PD1b anticipated: a pack read
  off disk mints its ids at admission time and a `&'static str` cannot hold
  one. `Registry::native()` became a `LazyLock` returning `&'static Registry`,
  so an owned table still costs a validation nothing. **`ProcessRef` did not
  widen** — PD1b's note said the record would carry both, and it should not:
  every allocated site in every snapshot would grow a qualified string, and the
  digest already recovers the id through `Registry::resolve`. `ProcessDef::
  native` became `Option<Process>` and is **not pack data**: which engine fast
  path a definition happens to have is the core's own index, recovered by
  qualified id at lowering, so a pack minting something new lowers with `None`
  and runs through exactly the same validator.

  **`WorldRules`, the playable ecology plan's label, gets its first real
  component.** `crates/mesocosm-core/src/rules.rs` holds `RulesetDigest` and
  `WorldRules { processes }`; `World` carries one, serialized and hashed with
  everything else, so two worlds under different biologies cannot agree about
  a state hash. A world carries the *identity*, not the definitions — the same
  arrangement `ProcessRef` uses one scale down. `snapshot::restore_under` is
  the door a save, a replay or a peer comes through: a mismatch is
  `SnapshotError::Ruleset { expected, found }`, both digests named, rather than
  a silent continuation against whatever this build holds. `restore` stays for
  round trips inside one process.

  **The parity receipt, and only then the deletion.** The shipped pack admits
  to a registry that is `==` to `Registry::native()`, its `RulesetDigest`
  equals the native one, and the gland's `DefinitionDigest` is unchanged — so
  every allocation already citing it resolves against the packed ruleset
  untouched, and no fixture hash moved for that reason.
  `the_packed_definition_reaches_pd2s_four_states_through_the_packed_door`
  then drives a body from the packed reference through PD2's four states
  (located and paid for at 5 cells and 115 mg; charged; dry two columns over
  with no cell and no milligram of rent lost; gone with its branch, and the
  branch still explains itself), and asserts that what the discovery grants
  *is* the definition the pack declares.

  **The deletion checklist, executed line by line.** `Intent::Rearrange` and
  `Allocate`, `World::rearrange` and the whole of `world/rearrange.rs`,
  `Outcome::Rearranged`, `Event::Rearranged`, and
  `mesocosm-genet/examples/pd2_receipt.rs` are gone. Everything §9 listed as
  surviving underneath survives untouched: `BodyPhenotype::develop` and its one
  validator, `Process::Secrete` and its `ProcessDef`, `Organism::bite_mg` /
  `charged_mg` and the `upkeep_for_body` rent term, and the vitals panel's
  gland reading.

  **What replaced it: `Intent::Express { condition }`**, and it is smaller
  than what it replaced rather than a rename. `Rearrange` carried a complete
  hand-authored allocation, which is an editor over the wire; `Express` names
  a `ConditionId` and nothing else, and the world builds the proposal itself
  from the admitted ruleset and the line's own discovery record. **A host can
  no longer name a cell or a definition** — it can only ask for what its line
  already came to. `world/express.rs` keeps the three things `rearrange.rs`
  added and that were never the temporary part: the price
  (`cost_cells * cell_mg`), the payment (out of the reserve, into the column
  under the body, flow-tracked as `flow::Process::Develop`), and the record
  (`Event::Expressed`). Two new refusals say which half is missing:
  `Rejection::Undiscovered` ("your line has not come to that") and
  `Rejection::Nowhere` ("nowhere on you to put it"). PE2's
  `candidate_proposal` and `candidate_intent` survive as §9 said they would;
  `candidate_intent` now returns `Express` and still answers `None` when the
  body has nowhere to put it, which is the difference between having the
  option and being able to take it. PE3's review is still the eventual door.

  **The demo needed no re-expression.** The recorded 3,100-step trace has no
  `Rearrange` in it and never did — PD2's own entry says the editor operation
  had no keyboard binding and no ordinary replay reached a gland. Verified by
  census against the recording before it was replaced: Move 2,726, Deposit
  244, Carve 73, Idle 40, Metabolize 12, Resume 3, Graft 1, TakeControl 1, and
  the new recording is that census exactly. The demo keeps its discovery, its
  branch transfer, its three births and its succession.

  **Receipts.**
  - `mesocosm-core` lib: **359** green (+5: three in the new `rules` module
    and two ruleset claims in `snapshot`; `a_rule_bearing_byte_changes_the_
    digest` was extended with the ruleset-level move and the
    order-independence claim rather than split).
  - `mesocosm-phenotype`: **23** green across two suites —
    `tests/admission.rs` (16) for the crate's own boundary and
    `tests/packed_gland.rs` (7) for the game's. One test per done-condition,
    named for it: `the_shipped_pack_admits_to_the_native_ruleset`,
    `a_colliding_namespaced_id_is_refused`, `a_path_escape_is_refused`,
    `a_malformed_schema_is_refused`,
    `one_rule_bearing_byte_moves_the_ruleset_digest`,
    `the_manifests_file_order_is_not_rule_bearing`,
    `author_facing_text_is_not_rule_bearing`,
    `a_snapshot_names_the_exact_admitted_ruleset`,
    `a_replay_against_a_different_admitted_ruleset_is_refused_identifiably`.
  - `tests/embodied.rs`: **49** green — the same count as before the
    deletion, so nothing was dropped rather than re-expressed. PD2's four
    states are re-expressed
    rather than reduced: the claims about what a development *costs and
    records* go through `Intent::Express`, and the claims about a body a host
    could never author — a whole frond turned to poison, a gland asked for on
    bulk, a site that is not one connected region, a severed part — are stated
    to the validator through a new `develop_played` helper, which is the
    boundary that actually decides them. The suite split at the ceiling:
    `gland_use.rs` takes claims 3 and 4.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --test embodied --release`: **6 + 11 + 7 + 49** green.
  - **Fixture re-recorded, no shim.** `World` gained one serialized field, so
    the whole-world hash moved: `652c5bcfdc6013c1` (P3) to
    **`1b1f866cb9138d40`**. Headed `--replay` runs 3,100 steps over 775
    frames and matches, exit 0; a hash falsified by one bit reports the
    mismatch and exits 1. Everything else in the receipt is unchanged from
    P3's — 775 frames, ground revision 17, 33 body parts, 40 drawn roster.
  - **The instrument is unmoved, and provably so.** `Process::Secrete` is
    still `Seeding::Acquired`, the instrument drives with `Intent::Idle` only,
    and `World::observe` needs a hand on the body — so no discovery ever lands
    in an instrument run and `Intent::Express` is unreachable there.
    `ProcessDef::digest` bytes did not change, so no `ProcessRef` moved;
    re-sorting `NATIVE_DEFS` cannot move a seeding share because no role seeds
    more than one definition (`nothing_grows_a_gland` asserts the Plate case
    is 2 admitted, 1 grown); and nothing in the ecology reads `World::rules`.
    Measured as well as argued: the drawn baseline was re-run over **all ten
    seeds** and compared against `dc4_roster.json` seed for seed — verdict,
    start, peak, peak tick, end, cumulative births and cumulative deaths each
    identical on every one (seed 1: thins, 917 -> 1561 at tick 4400 -> 1350,
    born 5940, died 5184; seed 2: 917 -> 917 -> 245, born 1693, died 2091;
    seeds 3-10 likewise exact). The run was stopped after the baseline batch
    rather than carried through the other three, so it could not overwrite
    `dc4_roster.json` with new timing on an unmoved result.
    **0 breathes / 10 thins / 0 boil / 0 collapse** stands, all ten seeds
    directly re-verified this session.
  - `cargo test --workspace` green: **39 suites, 729 tests, 0 failures** in
    release, mesocosm-lens's 45 included and passing in the ordinary parallel
    run, so the GPU flake did not bite. Clippy `-D warnings` clean in both
    profiles; `cargo fmt --all --check` clean;
    `cargo check -p paredros-room --features r1-proof` builds with its one
    pre-existing `dead_code` warning on `brick::retarget_from_ground`.

  **Splits at the ceiling**, per the workspace rule:
  `process/registry.rs` out of `process.rs`, when the owned `ProcessId` and
  the admission door pushed the file over; and
  `tests/embodied/gland_use.rs` out of `gland.rs`, when re-expressing PD2's
  four states through the new door did.

  **Residues, and what PD4 and PE3 inherit.**
  - **The admitted registry is proven equal, not yet threaded.**
    `BodyPhenotype::develop` still resolves against `Registry::native()`
    rather than a registry a world carries, and the parity receipt is what
    makes that honest: the pack lowers to *the same registry*, so threading it
    would change nothing observable today. It stops being honest the moment a
    pack mints a definition the natives do not hold, which is PD4's first
    step — `develop` takes a `&Registry`, and `World` carries the one it
    admitted rather than only its digest.
  - **P3's affinity table did not get the pack door, and the reason is a
    ruling rather than effort.** The pack side is one more file and one more
    lowering; the *consumption* side is not, because `World::found` builds
    `Affinity::native()` internally and admitting one means a new founding
    parameter — and then a policy default nobody has set: whether a
    pack-declared affinity overrides `Founding`, or `Founding` overrides the
    pack. ~~That is Mark's call, not an implementation detail, so it is
    recorded here as PD4/PE4 work rather than half-built.~~ **Ruled by Mark,
    2026-09-02: a pack-declared affinity overrides `Founding`; `Founding` is
    the fallback the world ships with.** `WorldRules` is shaped to take a
    second component now that the policy is answered; wiring it is PE4's.
  - **`Rejection::Refused(Refusal)` is now hard to reach from the door**, and
    that is the point: the candidate builder cannot produce an invalid
    proposal, so the fifteen named boundaries are reached at the validator.
    The variant stays, because it is the door's contract for any development
    and PD4's Piccolo proposals will need it.
  - **PE3 inherits a door that is bounded but not yet a review.** `Express`
    takes up one discovered candidate on the played body at the moment it is
    asked. Choosing among several, previewing what each would cost, and doing
    it at a lineage checkpoint rather than mid-tick are all PE3's, and none of
    them needs a new intent — they need a screen over `candidate_proposal`.

- **2026-09-01, phenotype P3 consumed this plan's graft ruling.** The directed
  affinity section above records what was built and what it answers. One
  additive change to PD1b's landed types came with it: `Instruction` gained
  `cost_by_part` beside `cost_cells`, because a cell is worth what its own
  part's tissue is worth and a graft is the first development that names
  several parts at once — one total cannot be priced at one part's rate without
  inventing a rate. The validator counts it; `world/rearrange.rs` still uses
  the single-part total and is unchanged.

- **2026-09-01, PD2 complete: a gland, the one process a shape has to be
  given.**

  **The chosen process, and why not light capture.** §7 named light capture
  as the leading candidate and venom secretion as the bounded fallback "if
  light capture would require the whole channel evaluator." It does: storing
  captured energy is a flow from an exposed surface to an account, and PD6
  owns flow ports and the relation that would carry it. `Process::Secrete` —
  a gland — needed none of that. It prices and reads entirely off
  allocation, mass and the ground under the body, and it still crosses body,
  world, ecology and explanation with one small definition, exactly as §7
  asked of the first proof.

  **The one new rule geometry did not already have: `Seeding`.** Every native
  process before this one was a thing a shape simply does — grow a limb and
  it contracts. A gland is the first a shape only *admits*: `Role::Plate`
  gates two definitions now, `mesocosm:fix` and `mesocosm:secrete`, and
  `Registry::seeds(role)` — the renamed seeding rule, split out from the site
  requirement `ProcessDef::admits` still answers — grows exactly one of them.
  Nothing in `Role::processes` or `Mosaic::seed` ever plants a gland, so a
  seeded frond takes its whole part for fixing; the first development that
  wants a gland has to take tissue off something else, which is the whole of
  the "readable choice" the gate asks for. `nothing_grows_a_gland` and
  `no_body_a_world_founds_has_one` receipt this as a property of the
  registry and of a founded roster, not a hope about worldgen.

  **`Seeding` is rule-bearing, and that is why every native's digest moved,**
  not only the new one's: `ProcessDef::digest()` now folds in the seeding
  byte for every definition, geometry-seeded or acquired, because "a world
  whose plates grew glands is a different world" is true of the whole
  registry, not one entry in it. `a_rule_bearing_byte_changes_the_digest`
  covers both the existing byte and this one.

  **Located, and charged: `Intent::Rearrange`, the gate's one editor
  operation.** It carries the complete desired sites for one part — PD1b's
  `AllocationProposal` shape, `Arrangement::Direct` — and lands in
  `world/rearrange.rs`, which adds exactly the three things the validator
  does not own: the price, the payment, and the record. The price is
  `Instruction::cost_cells` (PD1b's count of cells whose expression changed,
  left unpriced because nothing consumed it yet) times
  `BodyPhenotype::cell_mg(part)` — the part's own TD6 adult-mass ceiling
  divided by its mosaic's living cell count, floored at one. No constant was
  invented; PD1b explicitly deferred this number until a consumer existed,
  and this is that consumer. The milligram leaves the body's reserve and is
  deposited into the soil column under it (TD6: work is matter moving
  somewhere else, never ceasing to exist), flow-tracked as the new
  `flow::Process::Develop` so `tests/flows.rs`'s whole-run reconciliation
  covers it without a dedicated test, and recorded as `Event::Rearranged`.
  `the_development_is_located_paid_for_and_on_the_record` is the receipt.

  **Useful, and dormant: a world condition, not a mosaic change.**
  `BodyPhenotype::secretory_mg()` prices every living cell allocated to
  `Secrete`, the same way `cell_mg` prices anything. `Organism::charged_mg
  (ground_mg)` answers that dose when the column under the body holds at
  least that much, and zero otherwise — the threshold *is* the potency,
  derived rather than tuned. `Organism::bite_mg` adds the charged dose to the
  line's inherited `venom_mg`, so both the played eater and the ecology's
  read one number. Plan §4's rule — a changing environment does not rewrite
  the mosaic — is honoured by construction: going dry costs no cell and no
  milligram of rent, because rent is `upkeep_for_body`'s new fourth
  parameter, `secretory_mg`, added into the same numerator the actuator's
  swing already occupies and charged whether or not the gland is presently
  working. A body with none reads exactly its pre-PD2 rent, to the milligram
  (`rates/tests.rs`, moved to its own file at the ceiling this change
  created). `a_charged_gland_costs_whatever_eats_the_body_that_carries_it`,
  `carrying_a_gland_costs_rent_every_tick`,
  `a_gland_bigger_than_the_ground_is_dry_and_still_costs_its_rent` and
  `enriching_the_ground_charges_the_gland_the_body_already_had` are the four
  receipts — the last proving the claim from the direction that matters: an
  ordinary `Deposit`, a verb every player already has, is what turns a dry
  gland on, with no development and no revision change.

  **Severed, and gone — and the branch still explains itself.**
  `BodyPhenotype::sever` already tombstoned a mosaic's cells at PD1b;
  `glands()` and `secretory_mg()` read the result immediately, so the sting
  and the rent leave together. What is new is `lost_glands()`, which reads
  the *severed* parts that once carried one, so `World::gland()` keeps
  answering `Some` — empty `sites`, non-empty `lost` — rather than `None`,
  and a player is still owed "that branch is where your sting was."
  `BodyPhenotype::explain` already named a severed branch's shape (PD1b's
  residue note); it now also names `secrete` for one that carried a gland.
  `severing_the_frond_takes_the_bite_and_the_rent_with_it` is the receipt,
  proved against a control body that never had one so the claim is "what is
  left afterward is identical," not merely "the number went down."

  **The canopy residue PD1b flagged is closed.** `BodyPhenotype::canopy()`
  (new) asks whether a plate in canopy position is actually allocated to
  fixing, not merely whether one is held up in the light — the question
  PD1b's own note said would only diverge from the geometric reading "once
  a development can take that tissue away." `Kingdom::of` and
  `FeedingMode::of` now take a `&BodyPhenotype` rather than a
  `&BodyDocument`, and `converting_the_whole_frond_costs_the_body_its_living`
  proves the downside is real: a body that turns its whole frond into poison
  stops reading as a producer and has to eat like everything else, while its
  anatomy — the plate, the position — is untouched. `BodyDocument::canopy`
  is renamed `canopy_parts` and now only answers the shape half of the
  question, which every call site not asking about capability (the mesh and
  Lens projections, `kingdom.rs`'s own geometric receipts) still uses
  directly.

  **The headed receipt.** PD2's whole authoring surface — `Intent::Rearrange`
  — has no keyboard binding and no ordinary `--replay` trace reaches a gland
  (the played critter always founds as a fixed-recipe consumer, never a
  producer with a frond), so the plan's own permission — "a native
  developmental fixture or an explicit editor operation" — is what proves
  this gate, exactly as it already does in `tests/embodied/gland.rs`. The
  receipt tool, `crates/mesocosm-genet/examples/pd2_receipt.rs`, drives the
  real `mesocosm_genet::vitals::VitalsChrome` — the same cambium/netrender
  pipeline the interactive host composites — over a headless wgpu device (no
  window; built the same way `mesocosm-render`'s existing headless renderer
  is) so the pixels are the engine's own. It grows a frond the same way the
  test fixture does, then plays the real, validated `Intent::Rearrange` and
  `Intent::Move`; the fourth state uses the same direct `sever()` call the
  automated proof does, because no `Intent` removes a part yet — that is
  phenotype D3a's gate, not this one's, and it is the one state this receipt
  cannot reach through ordinary play. Four captures land in
  `Code/testing/mesocosm/`:
  - `pd2_process_1_allocated.png` — the instant of the development:
    **gland** "5 cells of part 2", **sting** "115 mg a bite", **gland rent**
    "1 mg a tick", with the "rebuilt" notice live.
  - `pd2_process_2_useful.png` — the same body, settled: the notice faded,
    energy visibly spent on thirty ticks of rent, the same charged reading.
  - `pd2_process_3_dormant.png` — one column over: **sting** "dry: this
    ground holds 112 mg, the gland needs 115" — the exact shortfall, so a
    player knows what would fix it — while **gland** and **gland rent**
    read unchanged, because nothing about the allocation moved.
  - `pd2_process_4_severed.png` — **gland** "gone with part 2", **sting**
    "nothing left to sting with", **gland rent** "0 mg a tick".

  A player reading this panel is told, in order: what it has, whether it is
  presently useful and by how much, and — when it is not — the exact ground
  reading that would turn it back on. `mesocosm-genet/src/vitals.rs` needed
  no change; it composites whatever `vitals_root` draws, which is exactly
  why it was named an available lane rather than a file to edit.

  **Receipts.**
  - `mesocosm-core` lib: **338** green (+1 over PD1b: `nothing_grows_a_gland`;
    `a_rule_bearing_byte_changes_the_digest` extended for the seeding byte).
  - `tests/embodied.rs`: **28** green (+12, all in the new
    `tests/embodied/gland.rs`), one test per named claim in its module doc:
    a readable choice (4), located and charged (2), useful and dormant (4),
    severed and gone (2).
  - `mesocosm-views`: **18** green (+2): `the_gland_reads_differently_in_
    each_of_its_four_states` (the exact panel words for all four) and the
    development-refusal words a hand can actually produce.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --test embodied --release`: **5 + 9 + 7 + 28** green.
  - **Fixture re-recorded, one format bump, no shim.** The digest change
    touches every native definition, so every organism's serialized
    `ProcessRef`s changed and the whole-world hash moved with them even
    though the demo never allocates a gland: `0ebe0655317a7392` (PD1b) to
    **`25a5a0096cef0af1`**. The intent stream is byte-identical — same 3,100
    steps, the same three `Resume`s at 800/1600/2400, the same
    `TakeControl { organism: 2205 }` at 3000, the same fourth-birth `Resume`
    at 3040 — confirmed by direct diff against the recorded intents, not
    merely asserted. Headed `--replay` runs 3,100 steps over 775 frames and
    matches the recorded hash, exit 0; a hash falsified by one bit reports
    the mismatch and exits 1.
  - **The instrument is isolable, and unmoved.** `Process::Secrete` is
    `Seeding::Acquired`, so no founded body ever expresses it —
    `no_body_a_world_founds_has_one` receipts this over `World::new(4_242,
    24)` — and `upkeep_for_body`'s new fourth parameter,
    `Organism::phenotype.secretory_mg()`, is therefore provably zero for
    every founded organism, exactly the mechanism DC4's own `Founding`
    switch used to isolate its own baseline. Re-run against `dc4_roster.json`
    seed for seed: the first six baseline seeds landed **verdict, start,
    peak, peak tick, end, cumulative births and cumulative deaths each
    identical** to the recorded receipt (seed 1: thins, 917 -> 1561 (tick
    4400) -> 1350, born 5940, died 5184; seeds 2-6 likewise exact). The sweep
    was stopped there rather than run through the remaining four baseline
    seeds and the four other batches, on the same reasoning PD1b's receipt
    used: the isolating mechanism is structural, not a per-seed coincidence,
    and the run was stopped before it could overwrite `dc4_roster.json` with
    new timing on an unmoved result. **0 breathes / 10 thins / 0 boil / 0
    collapse** stands unchanged, six of ten seeds directly re-verified this
    session.
  - `cargo test --workspace` green (33 suites in release, plus mesocosm-lens 45; the lens crate passed both serially and in the ordinary parallel run this session, so the GPU flake did not bite); clippy `-D warnings`
    clean; `cargo fmt --all --check` clean; `cargo check -p paredros-room
    --features r1-proof` builds, its one `dead_code` warning on
    `brick::retarget_from_ground` unchanged and predating this work.

  **Splits at the ceiling**, per the workspace rule:
  `organism/ecology/rates/tests.rs` out of `rates.rs`, when PD2's fourth
  rent parameter pushed the file over.

  **Residues, and what PE2 and PD3 inherit.**
  - **The temporary door is named, not yet removed.** `Intent::Rearrange`,
    `World::rearrange` (`world/rearrange.rs`), `Outcome::Rearranged`,
    `Event::Rearranged`, and this gate's receipt tool are the whole of the
    authoring path §9 permits deleting at PD3; the validator underneath —
    `BodyPhenotype::develop` — is not part of that deletion and was already
    shared with automatic arrangement before this gate existed.
  - **PE2 inherits a reading surface, not a discovery system.** `World::
    gland()` and the two vitals lanes answer "where, how much, useful or
    not, and why not" for a body that already has one; they say nothing
    about how a body gets one in the first place. PE2's condition
    evaluator, evidence-bearing discovery record, and the bounded
    part-level eating proof are entirely its own work.
  - **NPC acquisition is still open.** Nothing in the ecology's own feeding
    or development step ever proposes a gland; only a hand (or this gate's
    fixture) does. §9's own "Done when" only asks that direct and automatic
    fixtures share one validator, which they do — `Arrangement` is
    diagnostic metadata the validator never reads, exactly as PD1b left it
    — but *whether* an unplayed lineage ever acquires one is PE2's ruling,
    named as open there already.
  - **`performs`'s remaining honesty gap moved, it did not close.**
    PD1b flagged `reach`, `mouth` and `feeding_mode` as readings that could
    ask allocation instead of geometry once something could move a site.
    This gate closed exactly the one PD1a already named as the next
    candidate — canopy — because it is the one a gland's downside actually
    exercises. The other three remain geometry reads; nothing in PD2 gives
    them a reason to change yet.

- **2026-09-01, PD1b complete: allocation gets its owner.**

  **The four-process correction, first.** This gate was written against "the
  three existing native processes". DC1.5 added a fourth — `Process::Fix`,
  expressed by `Role::Plate`, the fixing/producer reading — together with
  `Process::ALL` and a registry parity receipt. §9 is corrected rather than
  quietly contradicted, the implementation covers four, and
  `the_registry_and_the_native_view_agree` is what makes a fifth impossible to
  add in silence.

  **What allocation is.** `crates/mesocosm-core/src/phenotype.rs` holds
  `BodyPhenotype { body, mosaics, revision }`, every field private.
  `phenotype/mosaic.rs` holds one `Mosaic` per part — lattice `dims`, a sorted
  `lost` list, and its `Site`s — and the mosaics are **index-aligned with
  `BodyDocument::parts`**. That alignment is the whole invariant: an anatomy
  tombstone *is* an allocation tombstone, and the two cannot disagree because
  there is nothing to synchronise. A `Site` names a `ProcessRef`, the sorted
  connected set of `CellId`s it occupies, and an `Expressed` cause —
  `Geometry` for what a shape grew, `Arranged { revision }` for what a
  development placed. There is no capacity scalar: structural capacity is the
  count of living cells, and occupied plus free equals it by construction.

  **Identity moved off the enum.** A site cites
  `ProcessRef { definition: DefinitionDigest }` — the content address of the
  exact admitted definition — and resolves it through `Registry::resolve`,
  which answers `None` rather than substituting a similar local process.
  `Process` survives as the native binding for engine fast paths and
  `Role::processes` survives as the **seeding rule**; neither is what a
  phenotype stores. Only the digest travels, because `ProcessId`'s
  `&'static str` cannot be deserialized into an owned world; PD3 widens the
  record when packs mint owned ids.

  **The seed.** A part's lattice is `half_extent / 2 + 1` cells per axis,
  clamped to four — so `MAX_CELLS` is 64, derived from the axis bound rather
  than typed twice. A `[4,1,1]` limb is a chain of three, a `[4,4,1]` plate a
  3x3 sheet, a `[1,1,1]` sensor one cell. Adjacency is derived from the dims
  inside `Mosaic::neighbours`, which is the one seam a later admitted topology
  arrives through: every reader asks the mosaic rather than doing lattice
  arithmetic of its own. **A seeded site takes the whole part.** That is the
  honest lowering of "this shape does this thing" — nothing is pre-donated, so
  the first development that wants a second process has to take tissue off the
  first, which is the tradeoff PD1a deliberately put inside the organ. Any
  other share would have been a number invented to make it painless.

  **One validator, and the equivalence is structural.**
  `phenotype/develop.rs` holds one `AllocationProposal` — expected digest,
  the sorted parts it rewrites, and the complete desired sites for each — and
  one `validate`. `Arrangement::{Direct, Automatic}` rides along and the
  validator never reads it, so the parity receipt is not a coincidence to be
  re-checked but a property of the shape: `develop` returns
  `Development { instruction, source }`, and `Instruction` (revision, parts,
  sites, `cost_cells`, resulting digest) contains no author. `arrange(&p, Aim)`
  builds the automatic side and asks `Mosaic::seed` for the seeding rule
  rather than reimplementing it, so auto-arrange cannot drift from what
  development would have grown. Fifteen named refusals, checked in a fixed
  order that is part of the contract.

  **Where the anti-Spore property now lives.** `ProcessDef::admits(role)` gates
  every proposed site by the part's classified shape, so `SiteMismatch` refuses
  contraction on a plate. To make a frond an actuator you must make it a limb,
  which is a different shape and a different part. There is still no capability
  field anywhere.

  **Cost is counted, not invented.** `Instruction::cost_cells` is the number of
  cells whose expression changed. PD2 prices that in milligrams when a played
  process gives the price a consumer; this slice refused to guess one, and no
  ecology number moved as a result.

  **Ownership, and one deliberate refinement of the seam.** `Organism.body:
  BodyDocument` became `Organism.phenotype: BodyPhenotype` with an
  `Organism::body()` reading. §9's seam said "private `BodyPhenotype`"; the
  field is `pub` and the **wrapper's** fields are private, which is the
  invariant the ruling actually protects — no caller can obtain
  `&mut BodyDocument` from a live organism, and `BodyPhenotype::seed` is the
  only constructor. Keeping the field public is what lets a test or a host
  sever through the transactional API without an `Organism` passthrough for
  every wrapper verb. `world/act.rs`'s incorporation now attaches through the
  wrapper, and its no-room rollback restores the **whole phenotype** rather
  than the anatomy — a body put back with grown mosaics would have been
  exactly the split account this wrapper exists to prevent.

  **The format bump, and what it cost.** One intentional break, no shim:
  `Organism` now serialises a phenotype where it serialised a body. Measured on
  the two worlds that matter — `World::new(4242, 24)` grows 5.3% (21 KB of
  399 KB, 871 living parts, 5,416 cells) and the demo's 916-founder world grows
  **28.0%** (773 KB of 2.76 MB, 31,762 living parts, 198,193 cells). Roughly
  ten of the thirteen bytes per site are the 64-bit definition digest as a
  postcard varint. §3 already permits interning a process reference to a
  compact integer inside one simulation; doing it needs a world-owned admitted
  ruleset, which is PD3's, so the digest stays whole here and the lever is
  recorded rather than pulled. Standalone `BodyDocument` bytes and the V2
  Lens/mesh/Isometry projection contract are untouched.

  **Receipts.**
  - `mesocosm-core` lib: **337** green. `process/tests.rs` (10) gains
    `a_reference_resolves_to_the_definition_it_addresses`: every native's
    stored reference resolves back to exactly its own definition, and an
    address this ruleset does not hold answers `None` rather than the nearest
    local process. `phenotype/tests.rs` (21):
    geometry seeds what it used to only answer; a seeded part arrives fully
    committed; every living allocation names a living part; capacity conserves;
    the lattice follows the shape; attach seeds in the same operation; sever
    removes the allocation and its consequence together and still explains the
    branch; a stale proposal moves nothing; **direct and automatic lower the
    same candidate to the same instruction and byte-identical state**; the same
    invalid candidate earns the same refusal; a part cannot acquire a
    capability by editing a number; every refusal names its boundary; an
    invalid multi-part development leaves the wrapper byte-identical; an
    unknown definition refuses rather than substitutes; a severed part cannot
    be rearranged; rearrangement is ordered and on the record; the phenotype
    round-trips; the mosaic and the geometry reading agree; irreversible loss
    takes capacity and the site with it; an explanation names the definition.
  - `tests/embodied.rs` **16** green, split at the ceiling into
    `tests/embodied/allocation.rs`: every founder carries one mosaic per living
    part; the allocation and the anatomy reading agree across a whole roster
    and all four processes; incorporating a part seeds its allocation in the
    same meal; one validator serves the player and the game over a live body;
    re-realizing the same program under a different world's admitted materials
    grows a phenotype with a genuinely different amount of tissue that
    expresses the same definitions for the same reason, while identical inputs
    still reproduce the phenotype exactly.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --release`: **5 + 9 + 7** green. Conservation and PE0's reconciliation are
    unmoved.
  - **The instrument is unmoved.** Drawn baseline, all ten seeds re-run and
    compared against the DC4/PE1 receipt **seed for seed** — verdict, start,
    peak, peak tick, end, cumulative births and cumulative deaths each
    identical. **0 breathes / 10 thins / 0 boil / 0 collapse**, exactly as
    recorded. The run was stopped after the baseline rather than left to
    overwrite `dc4_roster.json`, which is byte-identical to what DC4 wrote.
    Wall times are not compared: the sweep shared the machine with the debug
    test suite.
  - **Fixtures re-recorded.** The demo trace's **intent stream is
    byte-identical** — the same three births continued, the same
    `TakeControl { organism: 2205 }` at step 3000, the same fourth birth at
    3040 — and only the hash moved, `f90123db6f2a5ac5` to
    **`0ebe0655317a7392`**, which is the format bump and nothing else. Headed
    `--replay` runs 3,100 steps over 775 frames and matches, exit 0; a hash
    falsified by one bit exits 1.
  - `cargo test --workspace` green — **639** tests, and the lens crate's 45
    passed in the ordinary parallel run rather than needing
    `--test-threads=1`. Clippy `-D warnings` clean, `cargo fmt --all --check`
    clean, `cargo check -p paredros-room --features r1-proof` builds (its one
    `dead_code` warning on `brick::retarget_from_ground` predates this work;
    the paredros tree is untouched and clean).

  **Splits at the ceiling**, per the workspace rule: `process/tests.rs` out of
  `process.rs`, `organism/ledger.rs` out of `organism.rs` (mass ceiling,
  intake room, gain and spend, complexity, upkeep and the rent payment), and
  `tests/embodied/allocation.rs` out of `tests/embodied.rs`. An integration
  test's crate root resolves `mod` against `tests/`, so the last one carries an
  explicit `#[path]`; a bare `tests/allocation.rs` would have become a second
  test binary.

  **Residues, and what PD2 and PE2 inherit.**
  - **`performs` could be more honest, and was deliberately left alone.**
    `BodyDocument::performs`, `reach`, `canopy`, `mouth` and `feeding_mode`
    still read geometry. The truer reading is "some living site on a living
    part expresses this definition", which is `BodyPhenotype::expresses` —
    already implemented, already receipted as agreeing with `performs` over a
    whole roster and every native process. Rewriting the anatomy readings onto
    it is PD2's, because it is only a *different* answer once something makes
    a site dormant or a development moves one, and doing it here would have
    made a representation change into an ecology change. One reading did get
    strictly more honest and is already in: **an injury can now be explained.**
    `BodyPhenotype::explain` answers for a severed part — "this branch fixed,
    on this much tissue, because its shape did, and it is gone" — which no
    process reading could say before, since severing erased the fact from the
    process view entirely and left only `Part::severed`. `canopy` is the next
    candidate: its first clause asks `performs(Process::Fix)`, and the truer
    question is whether the plate's tissue is actually allocated to fixing,
    which only becomes a different question when a development can take that
    tissue away.
  - **Dormancy is unbuilt.** PD1a's structural-capacity/current-availability
    split is honoured by construction — mass changes move no organ — but
    nothing yet evaluates availability. `Mosaic::tombstone` carries the
    irreversible half of the rule and has no live caller; sub-part injury is
    phenotype D3a's gate.
  - **No world intent, by choice.** `BodyPhenotype::develop` is the only
    mutation path and returns its record; the phenotype stores the ordering
    (`revision`) and each site stores the revision that placed it, so
    rearrangement is ordered and readable in the snapshot. An
    `Intent::Rearrange` and a `History` event belong with PD2's editor
    operation, which is the first thing that will actually rearrange anything.
  - **The registry is still `Registry::native()`, reached internally.** Seeding
    and validation call it rather than receiving it. When PD3 gives a world its
    own admitted ruleset it becomes a parameter, and `World` starts recording
    the ruleset digest a snapshot cites.
  - **`World` exposes no phenotype reading.** `World::body()` still answers a
    `BodyDocument`; PE2's inspector will want `World::phenotype()` and
    `BodyPhenotype::explain` behind it. Not added ahead of its consumer.
  - **Multi-process roles are unexercised.** Every role expresses exactly one
    process today, so `Mosaic::seed`'s even-share splitter has never had to
    divide a part. The validator is the authority on connectivity and will
    catch a bad split; the splitter itself wants a receipt the first time a
    role expresses two.

- **2026-09-01:** recorded the condition/development/process compilation split,
  hard graph and work bounds, delayed-cycle rule, explicit stacking laws, and
  the Kappa research boundary. Existing PD and PE ordering is unchanged;
  documentation only.

- **2026-08-31:** reconciled the status with the landed PD1b identity slice.
  The playable ecology plan consumes PD1b allocation and PD2 in PE2, then
  preserves P3 and PD3/PD4 before PE3 invokes P4/PD5 for filial expression.
  This changes no ProcessDef gate and dispatches no code.

- **2026-08-01:** `ProcessDef` accepted as the working name. Architecture,
  Piccolo boundary, pack and license gate, authority model, proof gates, stop
  rules, and execution interleave recorded. No implementation added.
- **2026-08-01:** revised after implementing-agent review. PD0 marked landed;
  PD1 split into design and migration; native play moved before pack and Lua;
  phenotype-facet portability ruled separately from local Rust storage; and
  phenotypic plasticity made explicit.
- **2026-08-01:** first PD1a question pass recorded finite per-part mosaics,
  event-driven remodeling, and the carry-body/regrow-here crossing choice.
- **2026-08-01:** second question pass ruled authoritative cell-graph mosaics,
  somatic compounds stabilized through adaptation, and destination-declared
  crossing options chosen by the traveler. Directed graft affinity recorded
  for the next pass.
- **2026-08-01:** third question pass ruled direct and automatic arrangement
  through one validator, with shared-lineage disagreement resolved by immutable
  descent rather than mutation in place or cell-wise merge.
- **2026-08-03:** clarified that arrangement previews a phenotype while the
  accepted, co-signed artifact is its developmental program. Changed world and
  body conditions may realize that program differently.
- **2026-08-05:** PD1a complete. Ruled a private transactional
  `BodyPhenotype` wrapper, one authoritative cell graph with no duplicate
  capacity scalar, separate structural capacity and current availability, one
  complete allocation proposal and validator for direct and automatic
  arrangement, and distinct carry-body and regrow-here projections. Exact
  PD1b seams and receipts recorded.
- **2026-08-05:** the prerequisite recipe-to-anatomy authority proof landed in
  `mesocosm-core::development`. `Recipe + Soma + PartPalette` now produces a
  real mass-conserving `BodyDocument`, and the Lens menagerie projects that
  document through V2 instead of constructing a renderer-only recipe body.
- **2026-08-05:** the live constructor join landed. World founders and ecology
  offspring call `Species::realize`; the world snapshots its palette and every
  organism its developmental seed; under-provisioned births wait atomically;
  `Chronicle::found` regrows through the same recipe developer; and the
  migration corrected body-mass spending across every living part. PD1b is now
  the next gate.
- **2026-08-06:** host/palette drift caught and closed. `PartPalette::primitive`
  grew a `sensor` template (tag 4) that `mesocosm-genet`'s fixture volume table
  did not carry, so `mesh_body` failed with `MissingVolume { part: PartId(3) }`
  as soon as a developed body used one. The fixture now **enumerates
  `Role`** and sizes each volume from `palette.template(role).half_extent`
  rather than listing tags literally, so a palette that grows a template can no
  longer silently outrun the host that draws it. Standing note for PD1b and the
  pack loader: any admitted-vocabulary change must be reachable by enumeration
  from the palette, never by a hand-maintained mirror on the presentation side.
- **2026-08-07 (audit):** two contract corrections. **Replay direction:**
  the live runtime is right and parts of this plan's persistence prose were
  wrong; replay is seed plus ordered intents with history *derived*, never
  history as replay input. Preserve that direction everywhere. **Provenance
  is three separable kinds:** acquired developmental vocabulary, somatic
  graft provenance, and filial material/lineage provenance. The founding
  slogan "every part used to be somebody" conflicts with `Origin::Founding`
  and recipe regrowth unless these are kept distinct.
- **2026-08-08, PD1b slice 1 landed (identity layer).** `ProcessId`
  (namespaced, static until PD3 admits owned strings), `ProcessDef`
  records for the three natives with per-definition digests over their
  rule-bearing bytes, a deterministic `Registry` with a ruleset digest,
  and `Process::id()` resolving the native binding through it. `Role`'s
  fast path is now receipted against the registry (`the_registry_and_
  the_native_view_agree`), so expression is defined by data and the enum
  may not drift; `a_rule_bearing_byte_changes_the_digest` proves the PD3
  digest property early. Reach semantics untouched, all existing
  receipts green. **Open for PD1b complete:** the `BodyPhenotype`
  wrapper, `phenotype/allocation.rs` and `develop.rs`, geometry-seeded
  allocation, `organism.body` privatization, and the wrapper-routed
  intents — the survey confirmed none of the PD1a seam files exist yet,
  so that half is a full surgery scheduled as its own session.
