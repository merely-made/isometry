# World Conditions and Authored Laws Plan

**Status: plan (2026-09-09).** This is Paredros's proposal for conditions-led
fantastical mechanics. It refines the founding record's call for structurally
different worlds; it does not make a shared games-wing rules engine. Wing-wide
identity and pipeline vocabulary remain at
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`.

Wing-wide procedural composition scoping lives in
`mesocosm/design_docs/2026-08-06_general_model_plan.md` §7.1 (2026-09-09):
multiple causal families, explicit couplings, and variation in laws,
embodiment, practice, and presentation. The charge example below is one
candidate within that scope, not a universal magic model. This file retains
Paredros's action-evaluation ownership.

## Purpose

Worlds should differ by causes that change what people can do, grow, carry,
repair, fear, learn, and build. A green fire that consumes oxygen, a tide that
remembers spoken promises, and tissue that can host a foreign organ only during
a lightning season are different worlds because they alter prerequisites,
costs, possible effects, and durable consequences. Recolouring damage types,
renaming mana, or changing only numbers is not sufficient.

The player sees a condition before committing when ordinary observation makes
that possible, learns its source and exceptions through play, and can inspect
why an attempted action was accepted, altered, or refused. A graft works only
where its donor, recipient, attachment, local conditions, and applicable laws
make it possible. It must never become a universal body-edit button.

This plan covers Paredros-owned action evaluation and durable world facts. It
offers optional separate adapter profiles for rules-as-written play. It neither
imports a tabletop rules engine into Paredros nor relabels directional-action
combat as RAW.

## Existing seams and findings

- `paredros-world` already separates requests from accepted results:
  `WorldIntent`/`WorldEvent` in `crates/paredros-world/src/world.rs` and
  `GameIntent`/`GameEvent` in `src/transitions.rs`, applied by `src/state.rs`.
  This is the durable event seam for a rules decision and its receipt.
- `src/technique.rs` has `TechniqueId`, known-technique records, and query
  inputs. The founding plan already says a technique has intended effect,
  alternative body/equipment/symbiont implementations, commitments, timing,
  interruption, and environmental conditions. Conditions extend that contract;
  they do not replace it with an ability list.
- The locally edited anatomy lane already makes body revisions and
  revision-scoped part addresses durable. Conditions may refer to those facts,
  but they must not own anatomy or silently retarget an address after a body
  revision.
- Mesocosm supplies a useful *neutral fact* precedent, not an evaluator to
  reuse: `mesocosm-core/src/graft.rs` owns a serialized, digestible directed
  affinity table. Its graft transaction previews a candidate, validates it
  before moving donor matter, records provenance, and refuses an unsupported
  carry instead of approximating it. Paredros may consume an admitted fact
  such as a tissue-domain verdict; it decides what that means in a Paredros
  action and social world.
- Isometry already keeps campaign laws and their storylet requirements in
  `isometry-campaign/src/world`, while `isometry-genet/src/adjudicate.rs` owns
  action verdicts. This confirms the boundary: portable facts can be named and
  compared; each product owns consequences, UI, authority, and adjudication.

## The model

### Typed facts, not palette families

An authored `WorldRulesRevision` is an immutable, content-addressed collection
of declared laws. It has an opaque revision id, a canonical digest, author or
pack provenance, dependency digests, and an explicit replacement relation. A
saved world pins the exact revision that adjudicated each accepted event. A
later revision is a new fact, never a silent reinterpretation of an old save.

A law declares only the vocabulary it owns:

| Kind | Declares | Does not decide |
| --- | --- | --- |
| condition | typed predicates and observable state, such as `air:ionized`, `surface:wet`, `tissue:accepting`, or `oath:spoken` | whether a given product action succeeds |
| operation | a limited causal transform with typed inputs, outputs, costs, timing, and failure modes | a generic capability or combat move |
| relation | directed compatibility, exclusion, supply, containment, transmission, or counter relationships | nearest-match substitution |
| invariant | a property every accepted transition must preserve or account for | a repair mechanism |

Names are presentation labels only. Matching uses namespaced ids and typed
arguments. `storm:charge` and `craft:charge` may share a word without sharing a
ledger. This prevents a world generator from making a fresh-looking world by
permuting colours, labels, and damage numbers over one hidden system.

An authoritative causal fact is separate from a person's knowledge of it. The
world evaluates physical preconditions against authoritative state; a credible
report, observation, or inference is an epistemic record and cannot satisfy a
physical requirement. Belief or attention can be causal only where an authored
operation explicitly reads a typed belief or attention fact.

The minimum authoritative condition record is:

```text
ConditionFact {
  id, schema_version, subject_or_place, parameters, source,
  observed_at, valid_from, valid_until?, rules_revision, provenance
}
```

`source` names the causal producer or transition, not a witness's confidence.
A separate observation/claim record may identify a reporter, evidence,
observer, and confidence. Paredros's belief and observation systems remain
responsible for who knows it and why.

An operation is declarative and bounded:

```text
OperationDef {
  id, inputs: [typed requirement], preconditions: [condition query],
  commitments: [attention, body binding, item, matter, energy, time],
  transforms: [typed effect], byproducts: [typed effect],
  interruption: [declared outcome], invariants: [invariant reference],
  explanation: [reason template]
}
```

This is illustrative data shape, not compile-ready Rust. A transform must name
the particular facts it may change. A typed operation cannot execute arbitrary
script, create an untyped status effect, mint an unaccounted resource, or
mutate another subject's body without the governing product transition.

### Conditions, costs, effects, and invariants

Execution produces `Accepted`, `Refused`, `Blocked`, or `RiskOutcome` against
the authoritative state. `Blocked` says the world establishes a missing
condition or binding. `Refused` says the requested operation is not admitted
by this rules revision. A preview separately says `Known`, `Unknown`, or
`Inferred` for each displayed reason. Unknown is not an execution result: the
world may accept or refuse on hidden true state, while the player receives the
least revealing explanation consistent with the product's disclosure policy.

Every result carries a causal receipt with the rule revision, facts read,
body/equipment addresses read, committed costs, effects, and any invariant
witness. Validation rejection is inert: it happens before admitted mutation.
An admitted attempt can instead consume the exact declared costs and fail by
its authored `RiskOutcome`, such as damaged graft tissue, an injured recipient,
or a spent mediator. A partial ritual likewise leaves only its declared
byproduct and records why it stopped.

Core invariants for the first slice are:

1. A durable fact has a declared source and exact rules revision.
2. Matter, energy, charge, attention, time, and social obligation are distinct
   accounts. An operation cannot exchange them without an authored transform.
3. A body requirement resolves to a currently live, revision-scoped part
   address, equipment binding, or symbiont binding. It never becomes true from
   a capability label alone.
4. A rules revision cannot reinterpret a past accepted event. Conversion is a
   new, explicit event with both old and new revisions recorded.
5. A consequence crossing products is a neutral fact with provenance. Paredros
   decides its personal and social consequences; Isometry decides campaign and
   group consequences; Mesocosm decides ecological consequences.

### Grafts as a forcing case

`GraftOperation` requires a donor branch and provenance, recipient anatomy and
revision, an attachment site, a valid directed tissue relation, viable local
conditions, a declared payment, and a recovery/failure policy. A carry can be
native, need a named adapter, or be refused. Regrowth is a separate operation:
it may preserve provenance and topology while realizing tissue under the
recipient world, but it does not silently claim that donor tissue survived.

World authors can make a graft possible only in a fungus-rich cavern, while a
seasonal condition holds, with a living mediator, after consent or a specific
bargain, or only through a particular cultivated organ. A global allow/refuse
setting is a valid world constraint, but does not replace the tissue, cost and
recovery laws where grafting is available. A world where foreign
tissue is impossible is valid, and gives a clear refusal with a path only when
the author supplied one.

### Surgery is a baseline body operation

Surgery, implantation, removal, and grafting can be practical body work in the
same family as Rimworld bionics without being magic. The baseline evaluates a
surgeon's learned technique and practice, usable instruments and facility,
patient anatomy, cleanliness/anaesthesia or their local equivalents, available
materials, consent where the Paredros evaluator requires it, and a bounded
stochastic risk table. It records whether the operation was validation-refused
before mutation, or admitted and then succeeded, failed, injured, contaminated,
or consumed the implant under its declared risk outcome.

Randomness is reproducible authority, never a UI reroll. Each admitted risky
operation records an event-keyed seed or a named saved RNG stream, its draw
order, the risk-table revision, and the sampled outcome. Preview can show
known risk bands and missing prerequisites without exposing a hidden draw.
Magic may modify this baseline only through an explicit law, for example a
charged instrument reducing contamination on conductive tissue at a stated
cost. It does not turn ordinary surgical skill into spellcasting or let a
generic magic flag waive anatomy and risk.

### Proposed first magic front: stored charge over material connections

**Design proposal, not settled magic taste.** Start with transferable stored
charge, because it reaches travel, building, work, sensing, and conflict before
it reaches a damage palette. The bounded first law set is `accumulate`,
`store`, `conduct`, and `release`.

- `accumulate` draws charge from a declared emitter or environmental condition,
  such as storm exposure, a heat gradient, motion, or an organism's organ. It
  records source loss, transfer loss, rate, and any emission.
- `store` puts charge into a named organ, implanted reservoir, carried vessel,
  or built accumulator with capacity, leakage, overload outcome, and inherited
  material/law parameters. A descendant or imported item retains only the
  parameters and charge state its provenance says it carries.
- `conduct` transfers charge along an inspected material connection: a tether,
  wire, wet mineral seam, grafted conductor, tool, or constructed route. It
  requires continuity, resistance/loss, endpoints, and local grounds. It can
  energize a door, pump, lamp, signal line, work tool, shelter, or vehicle;
  conflict is one possible downstream use.
- `release` spends stored charge through a body or equipment binding for a
  declared local effect: light, heat, motion, signal, latch, repulsion, or a
  forceful contact. Each release names its emission, residue, depletion, and
  counter rather than becoming typed elemental damage by another name.

The first conditions are local and inspectable: source present, conduit
continuous, reservoir capacity, endpoint grounded or insulated as required,
material wet/dry or damaged state, body/equipment address live, and a sensing
path. Sensing is its own operation: a probe, glowing indicator, audible
resonance, or trained bodily perception reveals charge or continuity to the
extent its authored law permits. It does not grant omniscient electrical
vision. Grounds, insulation, breaks, absorption, leakage, competing loads, and
counter-emissions create routes around, through, and away from a charged place.

This proposed law set branches visibly: a party can run a safe route to a
storm accumulator, power a workshop, bridge a dangerous crossing with a tether,
leave a detectable signal trail, drain a hostile installation, or choose a
body-bound reservoir with injury and maintenance risks. Its accounts remain
separate: transferred charge, material wear, bodily energy, time/attention,
and emitted heat/light/noise are not one mana pool.

The charge facts and connection topology can be neutral, provenance-bearing
world facts. Paredros alone decides whether a charged punch, tool, rescue line,
or built relay is offered, how it targets, what it risks, and its social
consequence. Sampling, naming, and visual motifs are artistic proposals that
produce candidate worlds; they become authority only after the selected
revision, parameters, seed/stream, and resulting facts are admitted and saved.

**What makes this magic, rather than just an electrical model?** The store and
transfer vocabulary alone does not. Use one explicitly impossible causal
relation as the first proposed magical law: paired marks can conduct across a
gap while both receive an admitted matching vibration. Pairing is a recorded
operation over particular marked materials, with a source, payment, capacity,
range and loss rule; matching decorative appearance alone grants no link.
Erasing a mark, damping its vibration, exhausting its source or breaking a
required ownership/binding condition interrupts that link. The initial profile
should choose the smallest such condition set rather than require all of them.

This gives a concrete difference from ordinary conduction: a builder can power
a disconnected mechanism, a creature can carry the counterpart, and an opponent
can silence or damage a mark. Recorded simulation actions supply the vibration
condition; audio output or microphone recognition is not authoritative. This
sympathetic-coupling profile is a candidate for the first magic experiment, not
a universal law or a settled presentation theme. Oaths, true names, sacrifice,
memory and other symbolic causes can be separate later profiles when their
facts and counteractions have defined owners.

## Authored composition and limits

Typed authored operations are the recommended default: they are inspectable,
statically bounded, testable, and give the clearest player explanation. Two
later alternatives have narrower jobs. A budgeted sandbox script may propose
the same typed effects and then lower through the identical validation and
receipt path; it cannot mutate the world directly. A general constraint solver
may search a bounded candidate space during world/pack admission, but is not a
whole-runtime evaluator. These are extensions after the typed first slice,
not a permanent ban on scripting.

The first schema set is deliberately small: condition predicates, directed
relations, account transforms, body/equipment bindings, spatial/material
predicates, and event provenance. A package can combine existing operations
when their input/output types and invariants agree. It cannot introduce a new
product evaluator by composing them.

Admission runs before world generation and on pack import:

- resolve all law and condition ids and exact dependency digests;
- reject duplicate ids, import/dependency cycles, dangling counter or
  requirement references, and incompatible schema versions;
- check declared operation read/write sets against invariants and account
  types;
- bound composition by a package-selected maximum operation depth, condition
  query fanout, generated relation count, and candidate search budget;
- run authored contradiction cases, including at least one refused operation,
  one alternative valid implementation, and one conservation/invariant case.

Causal feedback is allowed across bounded time steps: a ritual can charge the
storm that later changes the ritual, and ecology can feed itself. Such a loop
declares ordering, fuel or depletion, a maximum per-step work budget, and an
explicit pending/reschedule outcome. Within one transition, effects are applied
once from the admitted candidate; an unbounded cascade is refused rather than
recursing until a fixed point appears.

Compatibility is exact and directional. `Compatible` means same pinned
revision, or a declared migration that maps every required id and invariant.
`SupportedSubset` means the consumer names each supported operation and refuses
the rest. `Incompatible` means no entry. A consumer must never substitute a
similar condition, collapse a directed relation to an undirected one, or map a
missing rule to a generic spell/action. Seeds are insufficient evidence of
compatibility.

Rules revision changes are explicit world events. A revision may be scheduled
as a historical change, localized to a place, or selected for a new world. A
live rewrite must state its scope, migration operations, subjects affected,
facts left historical, and rollback or refusal result. Existing saves retain
their old rules snapshot until an accepted migration records both snapshots.

## Paredros evaluator ownership

Paredros owns the evaluator that lowers an expressed technique, material
action, or world operation into its own `GameIntent` and `GameEvent`. It owns
targeting, bodily risk, consent and social consequence, interruption,
perception, UI explanations, persistence, replay, and the decision to offer
an action at all. The evaluator may query neutral conditions and relations;
it does not ask a wing service whether a person can act.

The first UI is read-only and inspectable: an action preview lists satisfied
and missing conditions, committed bindings and costs, the rule and source for
each claim, alternatives the authored rule declares, and whether the answer is
known or inferred. Confirmation emits one intent. Replay must reconstruct the
same accepted/refused result from the pinned revision and factual receipt.

## Optional pinned RAW profiles

An optional `SrdRawProfile` or `Pf2eRawProfile` is an adapter selected at
world/campaign creation, with its own exact source edition/release identifier,
coverage table, adapter digest, and adjudication policy. It is a separate
evaluator and event vocabulary. Paredros conditions may provide inputs to it
only through an explicit mapping owned by that adapter.

Each profile must declare, before play:

- exact covered material and pinned source/revision;
- supported operations and state, plus a refusal for every unsupported entry;
- what it maps from Paredros facts, what it preserves only as annotation, and
  what it cannot represent;
- who adjudicates ambiguity, how a ruling is recorded, and whether a later
  ruling changes only future events or starts a new campaign revision.

Name a partial profile precisely, for example **“SRD 5.2.1 subset, faithful
for listed operations.”** A full RAW claim requires the full promised scope.
An adapter must implement the relevant procedure, timing, state, and
adjudication rules for its declared coverage, or refuse the action. Its pinned
procedure mode may reuse Paredros world facts through explicit mappings; it
must not silently import missing rules or relabel directional-action combat.

Source-bound note: the current D&D Beyond SRD page lists SRD 5.2.1 and SRD 5.1
under CC-BY-4.0. Paizo's compatibility FAQ describes game mechanics as
generally ORC or historical OGL material and treats the compatibility logo as
a separate agreement; the [ORC License](https://paizo.com/orclicense) is its
official text. Any adapter must pin the actual source it implements and keep
setting, art, brand, and logo decisions out of a generic mechanics claim.
Relevant sources: [D&D Beyond SRD](https://www.dndbeyond.com/srd) and
[Paizo compatibility FAQ](https://paizo.com/licenses/compatibility/faq).

## Implementation phases

### W0 — Revision and condition receipts

Add Paredros-local serialized `WorldRulesRevision`, `ConditionFact`, typed
condition query, provenance, digest, and save/replay carriage in
`paredros-world`. Preserve `WorldIntent`/`WorldEvent` ownership. Do not alter
the in-progress anatomy/equipment files.

**Done when:** two revisions with equal display names but different relation
bytes have different digests; a replay pins and verifies the exact revision;
unknown conditions and mismatched revisions refuse with a reason; and save/load
round-trips rule provenance without changing existing world replay behavior.

### W1 — One Paredros-owned graft operation

Build a preview and confirmation path around one authored graft law, using the
existing anatomy/revision admission seam. Require a location condition, a
directed tissue relation, a concrete attachment, a distinct cost account, and
a declared aftermath. Keep the existing Mesocosm affinity table in Mesocosm;
Paredros consumes only an explicitly admitted neutral relation or defines its
own world fact.

**Done when:** native, adapter-required, and refused carries have distinct
receipts; an absent local condition refuses before donor/cost mutation; a
validation-candidate failure is inert while an admitted attempt follows its
declared risk outcome; body revision invalidates stale bindings; the
accepted receipt identifies donor, recipient, parts, law, conditions, costs,
and aftermath; and the inspector explains each result.

### W1a — Surgery baseline

Add one Paredros-owned implantation or graft surgery operation with a concrete
skill/practice input, facility and instrument bindings, patient anatomy,
material accounting, and a revisioned bounded risk table. This is independent
of magic and may proceed before, beside, or without W1's fantastical graft-law
variant.

**Done when:** validation refusal changes nothing; each admitted attempt saves
its RNG authority and draw order; reload/replay reproduces the outcome; no UI
action can reroll it; and success, injury, contamination, lost material, and
recovery all explain their concrete causes.

### W1b — Charge law proof

Implement only the proposed `accumulate`/`store`/`conduct`/`release` law set
as one Paredros-owned evaluator slice. Start with one environmental source, one
stored reservoir, ordinary conduction and one explicitly magical sympathetic
connection, one counter/ground, and one sensing route. Do not wait for W1 or
W1a: its rules prerequisite is W0's revision/provenance receipt; actual material
effects also need their owning J/T/B transition path.

**Done when:** a saved scenario proves a charged source powering travel or
construction/work, one loss or counter route, one live anatomical or equipment
binding, one sensed and one hidden condition, and one reproducible saved
stochastic outcome where an authored release calls for it. The same facts must
support a conflict use without making damage the law's defining effect.
Paired marks transfer across a gap only under their admitted condition; erasure
or damping interrupts the transfer without leaving copied charge at both ends.

### W2 — Compositional authored laws

Implement the bounded schema registry, dependency resolution, contradiction
tests, read/write-set checks, and generation admission. Start with two worlds
whose operations share a presentation word yet have different causal graphs,
plus one impossible combination caught before a world starts.

**Done when:** a world changes a viable body, available operation, material
counter, and durable consequence rather than only palette/numbers; all stated
bounds are enforced; import order cannot change semantics; and an unsupported
product returns `SupportedSubset` or `Incompatible` visibly.

### W3 — Optional adapter spike

Choose exactly one small, pinned source subset for one adapter. Implement its
coverage/refusal table, its independent adjudicator, recorded rulings, and a
fixture that proves a Paredros-native action is refused rather than called RAW.
Do not begin this phase until W0's provenance/replay rules are stable.

**Done when:** every exposed adapter action has a covered procedure or an
explicit refusal; the event stream names its adapter revision and adjudicator;
ambiguous cases produce a saved ruling; and native and adapter events cannot
be replayed under each other's evaluator.

## Stop rules

- Do not promote a universal capability, spell, condition, or consequence
  evaluator into a shared wing crate.
- Do not allow a script direct mutation, unbounded execution, or an effect that
  cannot lower into declared typed facts, costs, invariants, and receipts.
- Do not silently map revisions, relations, body addresses, or RAW concepts to
  their nearest available counterpart.
- Do not make the player discover an ordinary refusal only after irreversible
  mutation when the governing condition is observable.
- Do not disturb the active body-sheet/equipment WIP or claim it validates this
  plan.

## Progress

- **2026-09-09:** Plan drafted from the Paredros founding/execution records,
  local `paredros-world` seams, Mesocosm's digestible graft-affinity precedent,
  and Isometry's split between campaign-law facts and product adjudication.
  Linked from the canonical index and execution plan. The surgery baseline and
  stored-charge law set are design proposals; no world-conditions code or
  shared contract is implemented.
