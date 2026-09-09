# R4: The Extraction Review (2026-08-10)

**Status: decided and executed 2026-08-10.** Rulings by Mark, evidence
and execution notes here. All four are carried out; nothing is owed
(see Execution state).

## The charge

Inherited from Mesocosm's archived body-pipeline plan (2026-07-30, R4),
transferred here 2026-08-07 to fire after S3:

> Decide what, if anything, becomes a shared runtime crate, with
> Paredros as the second consumer or not at all. Done when the seam is
> either extracted with two consumers named, or explicitly declined in
> writing.

S3's sim half landed 2026-08-08 and armed it.

## The organizing frame (ruled: adopt, symmetric form)

**Each vessel is a mode of the same peopled history.** Isometry is the
fortress and atlas view: care for a squad and a map, prepared,
adjudicated in turns. Paredros is the walk: the same world's people met
one at a time, embodied, negotiated. Mesocosm is the ecology beneath
both. The Dwarf Fortress caveat binds: DF's modes share one save and
one executable, and the wing's vessels do not. They stay sovereign
games joined by the pipeline laws (choices under scarcity cross,
inheritance is pointable, player history displaces procedural content),
never by a shared running world instance.

What the frame decides for extraction: the things that must be
recognizable across vessels are **people** (subjects and their deeds),
**things** (relics with provenance), and **places** (sites with
history). Facts of the world cross via the pipeline. Platform organs
are shared upward, into the stack that owns them. Verbs never cross.

Recorded in the wing founding record
(`mesocosm/design_docs/2026-07-30_games_wing_founding.md`, §1, "Each
vessel is a mode of the same peopled history") on 2026-08-10.

## Seam verdicts

### 1. The shared runtime crate (the charge's literal seam)

**Ruled: push the seam up into netrender.** Not a sideways wing crate.

Evidence: `paredros-room/src/gpu.rs` (322 lines) is today's only real
consumer of the tenancy glue: boot one `wgpu::Device` with the union of
netrender's and the tenant's required features, render the tenant into
a texture it owns, composite via `ExternalTextureComposite` at scene-op
boundary zero. Mesocosm-genet is netrender-only today; its renderling
adoption is G2, still open. One real consumer plus one anticipated is
below the two-consumer bar for a wing crate, and the propagate-up
doctrine names netrender as the owning layer anyway: the composition
half (`ExternalTextureComposite`) is already netrender API, so the seam
is finishing a wall netrender already started.

**Landed 2026-08-10** in netrender as `93a9cdf23`. What it grew:

- **Tenant requirements at boot.** `TenantNeeds` (required features,
  optional features, limits, label) plus `boot_shared` / `boot_on` and
  their async forms. `boot_async_with` delegates, so the plain path is
  the tenant path with no tenant. Features split by whether the tenant
  can do without them: required ones fail the boot naming the gap,
  optional ones are granted when present and dropped when not.
- **The tenancy contract, documented** on `TenantNeeds`: one device and
  queue, tenant renders to its own texture, composite at a stated
  scene-op boundary, tenant identity recorded in receipts, per the
  cohesion contract (landscape §8.9) and the backend-seam findings.
- **The duplicated constant, deduplicated.** Netrender's
  inter-stage-variable minimum is `REQUIRED_INTER_STAGE_VARIABLES`,
  stated once and *raised over* a tenant's limits rather than replacing
  them. The probe had been carrying its own copy of the number.
- **What stays app-side:** everything tenant-specific. The renderling
  `Context` on the shared device, stages, cameras, content. Netrender
  never learns what a tenant draws.

Consumers: `paredros-room` first — its `boot` is now a `TenantNeeds`
declaration and one call, with the adapter dance gone — and
mesocosm-genet's G2 second, which lands on the same API rather than
copying glue.

**A finding the work produced.** wgpu 29 advertises experimental
features on the adapter but refuses the device unless they were asked
for deliberately (`ExperimentalFeaturesNotEnabled`), so the obvious
"grant whatever the adapter has" rule is a boot failure waiting for the
right machine. Opportunistic grants now mask experimental features out;
a tenant that wants one names it as required. Found by a receipt, not
by a user.

### 2. The body pipeline (mesocosm-mesh)

**Already shared; no action.** Two real consumers exist now
(mesocosm-genet, paredros-room) and the seam already has a crate
boundary. Paredros consuming it as it stands is the sharing. Moving it
between repos buys nothing today.

### 3. Identity (paredros-identity)

**Ruled: promote now.** `paredros-identity` is the wing's identity
crate: `SubjectId`, `BodyRevisionId`, the facet stores, the control
pointer. The four facts stay four. Promotion executed in place:

- The manifest now says what it is and carries the library grant
  (MIT OR Apache-2.0), per the founding license convention: game code
  is MPL, a reusable library states its own license once its boundary
  is real, and this ruling is the boundary being declared real.
- Sibling vessels design against it from now on. Isometry's dead
  companion as a named campaign figure (S6) and any Mesocosm borg
  continuity name `SubjectId`, not a local id.
- The physical home stays this repository until a second vessel takes
  the dependency; where the crate then lives (and whether it moves to
  a wing-neutral home) is decided by that consumer's build, not in
  advance.

### 4. The consequence grammar

**Ruled: refused on principle, not deferred.** Deeds and premises here,
`Resolved` payloads in Isometry, sortie events in this repo: one causal
grammar (authority resolves once, everyone applies named consequences,
answers cite their evidence), sovereign implementations per vessel.
No shared crate, ever, on the same ground as the general model's
evaluator rule and the murm/moot no-neutral-oplog ruling: a grammar
shared as a library becomes a second authority with a schema.

### 5. Place identity

**No crate; the join is the pipeline.** Mesocosm's `Places` and
Isometry's `Overmap` remain two shapes with different jobs. The
adventure-mode frame names the eventual join: a place crossing vessels
as a fact with history (a sortie's trench surfacing as a campaign
site), which is S6-shaped work, not a shared partition library.

### 6. Wound's home (intra-Paredros note)

`Wound` stays in `paredros-sortie` until S4 forces the combat owner's
final shape. Recorded in the execution plan's findings 2026-08-08; this
review does not move it.

## Execution state

All four rulings are carried out.

- Framing ruling: **executed.** Wing founding record amended
  (mesocosm `ae7a6d8`), DOC_README row updated.
- Netrender tenant seam: **executed.** netrender `93a9cdf23`
  (`TenantNeeds`, `boot_shared`/`boot_on`, contract docs, four
  receipts); `paredros-room::gpu::boot` now consumes it.
- Identity promotion: **executed** (manifest, license, docs).
- Grammar refusal: **recorded**, closed.

R4's done-condition is met: every seam is either extracted with its
consumers named (identity, promoted in place; tenancy, named to
netrender with both consumers listed) or declined in writing (grammar,
place identity, mesh already shared).
