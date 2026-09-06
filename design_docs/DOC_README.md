# design_docs Index

Canonical index for `design_docs/`. Per DOC_POLICY §5, this file wins over
any other index and is updated in the same session as any doc change.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- Wing-level architecture lives in the Mesocosm repo at
  `mesocosm/design_docs/2026-07-30_games_wing_founding.md` and is cited, never
  copied. The three pipeline laws there govern anything crossing between
  games.
- **The invariant is care granularity, not metaphysical person purity**
  (relaxed 2026-07-30; revised 2026-08-13; wing founding record §1). Paredros
  is care for **individuals**: particular others you know. Ordinary play stays
  with one named creature until death. Control may shift through an explicit
  world event or optional player rule, with its consequences recorded. Drift
  means free roster control or care widening to a squad you administer, which
  is Isometry's granularity.

## Active docs

**Current design focus, 2026-09-06:** the founding plan's
[borg generation and character-sheet proposal](2026-07-30_paredros_founding_plan.md#borg-generation-techniques-and-the-character-sheet)
specifies classless capabilities, historically transmitted traditions,
alternative anatomical technique bindings, deliberate learning, and inventory
mapped to the body. The first implementation slice is an authored three-lives
example and read-only technique query, now implemented locally and covered by
the combined 43-test gate, using existing subject/revision and
part addresses. A separate native read-only Body/Actions inspector is now
implemented locally with a combined 52-test gate; durable equipment/learning
remain later joins. The execution plan
records the independent G/release crossing
fix and unresolved presentation feedback. Charge and
inhabitants are deferred during this design pass. Historical entry and
inheritance policy live once in the Mesocosm wing founding record.

The founding plan's **2026-09-05 player-experience proposal** covers embodied
action, biological abilities, progression, communication, knowledge surfaces,
structural world differences, and bounded sky-organism/topology experiments.
The execution plan translates it into a proposed playable sequence alongside
F3b1's evidence-to-answer join. Its
[damaged crossing fixture](2026-08-07_paredros_execution_plan.md#first-encounter-the-damaged-crossing)
specifies the player flow, two bodies, local world rules, partial witnesses,
knowledge surfaces, and staged acceptance. Dry movement/contact is the first
implementation slice and now has an authored `crossing` executable consuming
Conatus character movement, with local action/replay rules and a shared-device
Renderling/Netrender host. Player acceptance remains open. The wider encounter
and social/charge proposals remain unimplemented and open to design discussion.

| Doc | What it is |
| --- | ---------- |
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Product goals and pillars (maintainer-owned, revised by instruction 2026-08-13): one named life in a persistent generated world; autonomous inhabitants; control changes only through death, an explicit world event, or an optional player rule; culture has pointable causes. |
| [2026-07-30_paredros_founding_plan.md](2026-07-30_paredros_founding_plan.md) | Vessel 2's active founding record, revised 2026-08-13: one embodied life among autonomous named creatures; persistent settlements, dungeons, ruins, and surface/underground places; cooperation without party control; construction as a world verb; causal culture; recorded and socially contested body continuity; composable subject/body/role/lineage facts; wing license split, tone, and Nemesis-patent constraint. Its P0-P5 phase section is preserved as superseded history. |
| [2026-08-07_paredros_execution_plan.md](2026-08-07_paredros_execution_plan.md) | **The executable plan; F3 active and F3a landed 2026-08-26.** S0-S3 remain landed foundation receipts rather than the required game loop. Future ordering follows fundamental layers: persistent world, one embodied life, other autonomous lives, memory/standing, coordination, material life, settlement/culture, danger, and death/continuation. Ordinary control stays with one named creature until death; tag-in is an optional player rule or explicit world process. R4 was decided 2026-08-10. **R1 shared traversal landed 2026-08-20 and moved to its platform owner 2026-08-26:** Mere's `modulus` (renamed from `conatus-brick` 2026-08-28, pinned at `33f9b6b6`, published on crates.io) owns the sparse brick ABI and camera-neutral WGSL DDA; Paredros owns its Ground binding and carries that organ in the default compile path while retaining its own camera and presentation policy. **V1 continuous-zoom residency landed 2026-08-21; V1a cache coherence closed 2026-08-26:** the headed planning scene keeps its 127-voxel visible radius within a 1 MiB exact-page budget and recovers after abrupt zooms. **D1 raymarch depth composed with Renderling closed 2026-08-26:** the tracer writes fragment depth against renderling's stored depth surface, judged by a headed witness-pillar receipt. **RG3e caller-owned Renderling encoding landed 2026-09-05:** the 20-pass room tenant records into one encoder, Paredros submits it once, and the graph receipt retains its 466-colour byte-match while reporting the tenant and graph submissions separately. **V1b stable resident brick cache closed 2026-08-26:** one capacity-fixed 1,791-slot cache under the same 1 MiB budget retargets in place with retained slots, per-brick transition uploads, zero texture or bind-group creation, tracer-validated lease epochs, and byte-identical wgpu allocator reports; the shared-engine consolidation chain in the mesocosm engine review is closed. Larger travel footprints and clipmaps remain consumer-gated on a real footprint exceeding that exact cache. **F0 persistent world closed 2026-08-21:** `paredros-world` owns stable surface/underground slots, site meanings, routes, edits, and regrow-plus-replay persistence. Generic multi-subject movement persists accepted inputs while navigation remains derived. **F1 embodied life closed 2026-08-21:** separate body, item, and movement systems compose through one subject-addressed transition grammar covering naming, needs, perception, inventory, capability, injury, recovery, and death. **F2 other lives closed 2026-08-22:** deterministic site and migration origins, durable projects, and a control-neutral scheduler advance every living subject through that same intent grammar. Unattended rounds leave factual reports and pointable decision causes; population, projects, and simulation restore exactly. **F3a pointable memory and belief landed 2026-08-26:** accepted deeds now feed actor-scoped observations, claims, exact reports, claimant-owned correction, deterministic belief folds, and validated exact replay. F3 remains active for forgetting, adjudication, deception/intent, norms, observer-relative standing, and a consequential answer with its evidence chain. `mesocosm-lens` remains a product presentation adapter, not the owner of shared traversal. |
| [2026-08-10_r4_extraction_review.md](2026-08-10_r4_extraction_review.md) | **R4 decided 2026-08-10.** The wing frame adopted symmetrically: each vessel is a mode of the same peopled history (facts cross via the pipeline, platform organs go up the stack, verbs never cross). Tenancy seam pushed **up into netrender** and landed there the same day (`TenantNeeds` + `boot_shared`/`boot_on`, contract documented, four receipts; paredros-room consumes it, mesocosm G2 is the second consumer). `paredros-identity` **promoted to the wing identity crate** in place (MIT OR Apache-2.0). Consequence grammar extraction **refused on principle**. mesocosm-mesh already shared; place identity joins via the pipeline, not a crate. Founding record amended. **All four rulings executed.** |

## Archive

None yet. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.
