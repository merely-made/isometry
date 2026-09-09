# Functional networks, operators and generation

**Status: first slice implemented, reviewed and committed, 2026-09-09.** Authorized first implementation slice
following the general model discussion sections 7.2 and 7.3.

## Scope and ownership

One shared pure Rust evaluator lives in `shared/wing-functions/`. Its standalone
Cargo workspace preserves the three products' dependency resolution. It carries
part references, bounded networks, finite charge, typed operation requests and
receipts. It owns neither a second body nor product world state. Mesocosm supplies
current living part identities from `BodyDocument`; Isometry carries proposals
through its existing `GenValue` and generation-record machinery.

Sources and stores supply finite charge. Directed routes have capacities. Gates
and effect sites refer to parts. Strengthen and Project consume charge and return
typed effects for product adjudication; Store transfers it into a bounded store.
Range is caller-supplied, not a collision result. Units are authored abstract
charge units; this does not replace Mesocosm's conserved matter ledger.

## Lanes and done-conditions

1. **Functional evaluator (Terra):** deterministic allocation, bounded traversal,
   validation after load, atomic refusal and sequential operation batches.
   Done when missing parts, closed gates, bottlenecks, exhaustion and overflow
   have tested outcomes and errors leave state unchanged.
2. **Candidate generation and inspection (Luna):** parameterized deterministic
   networks bound to caller-supplied construction sites, for creatures and
   objects. Done when a seeded batch exposes outcomes and rejection reasons,
   and accepted values survive serialization without regeneration.
3. **Consumer wiring (root):** body membership adapter and typed proposal payload
   using the current generator carrier. Done when body loss changes evaluation
   and a proposal round-trips through a generation record's binary carrier.

## Findings

- 2026-09-09: existing `mesocosm-core::flow` records trophic matter movements,
  not internal functional connectivity. It remains the authority for that matter.
- Existing Isometry generator values already support objects and text. A
  versioned, validated construction payload can use that carrier without changing
  the public enum's binary variant ordering or introducing another Lua runtime.
- Concurrent edits in Mesocosm phenotype/graft, rules and world consumption are
  outside this slice and must remain intact.

## Open work after this slice

Directional gesture timing, physical strike adjudication, live Paredros session
integration, construction UI, authored material/process admission, dynamic graft
network reconciliation and gameplay save ownership remain consumer work.
Second-order operators, vows, Raise and generated world laws follow supported
cost and interruption semantics. An atomic batch is not a timed action: sustained
charging requires a product-owned action lifecycle and reevaluation on mutation.

## Progress

- 2026-09-09: implementation lanes started; no acceptance receipt yet.

### First-slice receipt

- `shared/wing-functions`: 16 unit tests passed using its isolated
  `target-wing-functions` directory; zero doctests. Covers finite supply, stored
  charge reuse after source disconnection, atomic batches, graph admission,
  schema/identity validation, bounds, cycles, generation and mutation.
- `isometry-campaign --lib`: 42 tests passed on the combined tree, including
  construction proposal carriage through postcard generation records and
  present-body admission. Uses `.targets/wing-functions-campaign` to avoid
  unrelated host builds.
- `mesocosm-core --lib functions::`: two tests passed, covering subtree loss,
  replacement identities and body/network binary save/restore.
- `inspect_functions`: 50 seeds, 100 candidates (both forms), all 100 preserve
  serialization equality and lose techniques when an actuator is removed.
  Receipt: `../../testing/functional-generation/2026-09-09-batch.json`.
- `inspect_body_functions`: uses the existing seeded body generator and an
  explicit authored inspection profile. Removing part 3 also severs descendant
  part 4; both corresponding Strengthen requests become unavailable. Receipt:
  `../../testing/functional-generation/2026-09-09-body.json`.
- The body example initially caught a transient syntax error during another
  lane's Chronicle extraction. Its owner completed that edit; the example and
  campaign suite then passed on the combined tree. No foreign work was reverted.
- Combined publication review includes the shared-format extraction and live
  interchange tests. The earlier working-tree receipts remain scoped as stated.

### Design findings from implementation

The v1 evaluator uses deterministic first-fit breadth-first routing. This is an
explicit routing policy, not maximum-flow feasibility: some graphs can have
unused reachable supply requiring rerouting that v1 does not perform. Tests
preserve that boundary and the refusal explains selected-route capacity.

Batches are sequential and atomic. Edge capacities reset per operation, while
finite source/store charge is shared across the batch. Simultaneous multi-arm
swings need a timed action allocation policy before these values can represent
concurrent throughput. Store currently transfers charge; storing an executable
operator chain as an enchantment remains later work.

The two generated forms are functional blueprints over caller-admitted sites.
They do not yet construct staff geometry or allocate magical tissue. Keeping
that distinction explicit lets construction rules grow without making this
sampler another body authority or silently assigning magic from shape.
### Publication review, 2026-09-09

- Rechecked current live interchange: four tests passed after the independent
  typed-stock commit. No unrelated source changes were absorbed.
- Campaign construction now carries and validates an explicit network format
  version on decoding and inspection. Unknown versions are refused before use.
- The full ecology suite and native timed charging remain open as documented.
- Final campaign regression suite after the network-version fix: 43 passed.
