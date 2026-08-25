# Runtime Profile Plan

**Status:** active (2026-08-25); host-neutral profile, resident body-position
consumer, and fixed-isometric Netrender tenant complete; desktop adoption gate
open.

## Purpose

Build Isometry's first product-owned conductor for shared runtime organs without
creating a second rules or replication authority. The current slice composes
Conatus after Isometry has accepted an ordered map event. It does not reorder
the active protocol hardening or Stickleback migration plans.

## Boundary

`isometry-core` stays pure geometry, documents, and events. The new
`isometry-runtime` crate may depend on product core types and shared runtime
organs. It is an excluded standalone workspace while the root desktop graph
still carries its open host migration, giving this profile its own lock and
focused gate without perturbing the active root dependency cone. The desktop
host may consume it after that migration and protocol H2 close.

The profile accepts an already-applied `MapDocument`. It has no raw network
message, `ActionIntent`, `BodyCommand`, or rules-plugin entrypoint. The ordered
event log validates and applies movement first; the profile mirrors the
resulting map into Conatus at zero steps. Conatus therefore supplies body
identity, colliders, queries, and spatial changes without deciding whether a
token was allowed to move.

Source bindings are product-owned and map-qualified:

```text
(MapSourceId, TokenId) -> BodyId
```

`TokenId` alone is insufficient because its documented scope is one map or
session. Switching maps removes the old bindings and materializes the new map,
so equal token numbers never alias. Tile span, elevation step, and token
collider extent are profile configuration rather than stack defaults.

The optional resident path is also product-owned. `resident-gpu` projects the
profile's accepted `IsometrySpatialFrame` into one fixed-capacity Quint F32
position plane on the host device. Isometry chooses capacity, coordinates,
source bindings, generation handling, and which tenant may consume the raw
view. Quint supplies the retained allocation, typed view, stamp, and atomic
sparse-patch mechanics. This is a disposable cache, not a shared spatial frame
or durable body table.

The optional renderer path is product-owned as well. `resident-render` binds
the exact Quint suballocation as a read-only storage buffer and draws occupied
body slots through Isometry's configurable isometric basis into a tenant-owned
same-device texture. `netrender-tenant` adapts that texture to Netrender's
existing external-texture compositor with an explicit scene boundary. The
marker lens is a forcing receipt, not a sprite ABI or a free-camera 3D mode.

## Gates

### R0 — Product profile crate

**Complete.** `isometry-runtime` owns the Conatus engine and the binding table.
It mirrors tokens as fixed bodies and publishes product-local changed and
removed records. Facing lowers to a Y-axis rotation; elevation remains an
authored map fact and lowers to Y translation. Its standalone manifest pins
the landed Conatus foundation and carries its own lock.

**Done when:** the crate compiles independently of the desktop host, rejects
invalid configuration and out-of-bounds/duplicate token documents before
mutation, and exposes no authority-bearing command path.

### R1 — Acceptance receipt

**Complete.** Tests cover initial materialization,
unchanged-frame silence, stable runtime body identity across an accepted move,
facing/elevation lowering, rejected-event silence, and map-qualified identity
across equal token numbers.

**Done when:** focused tests and warnings-denied Clippy pass, and the ordinary
core replay/protocol suites remain unchanged.

**Receipt:** four focused tests pass and warnings-denied Clippy passes under
Rust 1.96.0. The existing desktop/core sources and root lock remain unchanged;
the profile is not yet part of the desktop workspace gate.

### R1a — Profile-local resident body view

**Complete.** `IsometryResidentBodies`, behind `resident-gpu`, maps each
Conatus body slot to one `[x, y, z, occupied]` row in a retained Quint
allocation. A host-issued read epoch and Conatus revision publish together.
Removals zero their rows; a recycled slot is rebound only after the old
product source is removed, and the CPU binding retains the new body generation.
Capacity failure is explicit and leaves the resident allocation unstamped.

**Done when:** two nonadjacent accepted body changes publish under one stamp;
the allocation range stays identical; an unchanged frame performs no write;
same-slot generation replacement is distinguished; a too-small capacity is
refused before mutation; and the accepted `MapDocument` remains unchanged.

**Receipt:** the `resident_bodies` hardware test, gated by
`resident-gpu-receipt`, passes on the local adapter. The original four
host-neutral profile tests still pass without enabling or compiling the GPU
stack. Warnings-denied Clippy covers all targets with the receipt feature.

### R1b — Product-local body renderer tenant

**Complete.** `IsometryBodyTenant`, behind `resident-render`, retains the
current `RawKernelView`, binds its exact offset and size, and submits an
instanced marker pass only when the publication stamp advances. Isometry owns
the projection basis, origin, marker dimensions, color, target size, capacity,
and output texture. An allocation replacement rebuilds the bind group; a
same-stamp replacement or regressing revision/read epoch is refused before the
target changes. Capacity and target dimensions are checked against device
limits before GPU resources are created. `netrender-tenant` exposes that
texture as a premultiplied
`ExternalTextureComposite` without teaching Netrender about bodies or Quint.

**Done when:** one host device serves CubeCL, the resident position plane, the
body renderer, and Netrender; the shader reads the resident allocation without
CPU carriage; two nonadjacent moves reuse the allocation and binding; a silent
frame submits no body pass; removals clear markers; generation reuse becomes
visible only after removal; stale stamps and wrong shapes leave tenant state
unchanged; and composition preserves the product's explicit scene boundary.

**Receipt:** the `resident_render` real-adapter test, gated by
`resident-render-receipt`, renders and reads back expected projected markers,
composites the tenant texture through Netrender, and exercises silence,
sparse moves, removal, replacement, stale-stamp refusal, shape refusal, device
limits, and scene ordering across the composite boundary. The default
four-test profile remains GPU-free. Warnings-denied Clippy covers all targets
with the renderer receipt feature.

### R2 — Desktop adoption

**Open behind existing prerequisites.** The current Isometry desktop host still
targets Genet's deleted compatibility layout cone. That migration is broader
than this profile and is not smuggled into R0/R1. Protocol hardening H2 also
remains first in the repository's audit order.

**Done when:** those prerequisites close; `isometry-genet` constructs the
profile; every accepted active-map change is synchronized once; a still redraw
is silent; and a local render/spatial consumer consumes the published frame.

The host-neutral resident and renderer consumers now prove the final clause
independently: accepted frames reach a local renderer without CPU position
readback. Desktop construction and event wiring remain open, so R2 is not
complete.

### R3 — Second consumer

**Open.** Paredros or Mesocosm must challenge the actual needs of the profile:
source identity, cadence, authorization, subsystem selection, and frame
consumption. Only the common minimum may then move to Mere.

**Done when:** the second product carries an executable receipt and the shared
contract is smaller than both product-local profiles.

## Findings

- **2026-08-23:** Genet deleted the remaining Stylo compatibility cone on
  2026-08-21, while Isometry's desktop graph still names it. Resolving the
  whole root graph for an unrelated host-neutral profile consumed the package
  solver without reaching compilation. The root graph is left unchanged and
  the profile is explicitly excluded until the broader `genet-layout`
  migration closes as R2 work.
- **2026-08-23:** A profile belongs above `isometry-core`. Putting Conatus in
  core would make a durable document crate carry Rapier-backed disposable
  state and violate its explicit purity boundary.
- **2026-08-23:** An event-driven turn-based product does not need to tick
  Conatus merely to prove composition. `advance(0)` publishes admitted source
  changes and preserves the product clock.
- **2026-08-24:** `FrameUpdate` needed no new shared body-frame contract. The
  existing product-local `IsometrySpatialFrame` contains enough final body
  state and source identity for a resident projection, while Quint needed one
  reusable mechanism: validate and publish several nonoverlapping ranges under
  one stamp.
- **2026-08-24:** Unconditional GPU dev-dependencies made the four ordinary
  profile tests compile Burn/CubeCL. The hardware harness now has a required
  `resident-gpu-receipt` feature, leaving the default test graph host-neutral.
- **2026-08-25:** Netrender and the resident path are both on wgpu 30. Its
  existing tenancy contract already names the right boundary: one device and
  queue, a tenant-owned texture, and explicit external-texture composition.
  Netrender needs no body-buffer API because the product tenant binds Quint's
  exact storage range and hands over only its output texture.
- **2026-08-25:** The available wgpu-30 Renderling fork can adopt host handles,
  but its `Stage` has no public primitive that attaches an external resident
  position plane. The first proof would still require this product sidecar
  pipeline, so pulling Renderling into Isometry's shipped 2D lens would add a
  dependency without testing another boundary. It remains a later 3D-lens
  candidate.
- **2026-08-25:** A retained `RawKernelView` carries allocation lifetime and a
  copied publication stamp. The tenant refreshes that view when the stamp
  advances, reuses its bind group while the allocation is stable, skips equal
  stamps, and refuses allocation changes that lack a new stamp.

## Stop rules

- Do not let Conatus adjudicate a move, rules action, map transition, or peer
  request.
- Do not serialize `BodyId` or Conatus state into a campaign checkpoint.
- Do not invent a shared `ProductSourceId`, spatial frame, or conductor before
  the second consumer.
- Do not silently grow or compact the resident body plane. A capacity rebuild
  changes the allocation and requires product-selected tenants to rebind.
- Do not treat a body slot as identity; the product binding keeps the Conatus
  generation and processes removal before replacement.
- Do not promote the body-marker target, projection settings, Netrender
  placement, or tenant stamp checks into a shared renderer contract from this
  one product receipt.
- Do not make the locked 2D tenant depend on Renderling merely to reserve a
  future 3D lens. Adopt its scene facilities only when that lens needs them.
- Do not defer unrelated Isometry product work behind R2 or R3.
- Do not use this slice to bypass protocol H2 or the no-second-runtime gate.

## Progress

- **2026-08-23:** R0 and R1 completed in a new host-neutral product crate: four
  focused tests and warnings-denied Clippy pass. Desktop wiring was attempted,
  exposed the stale Genet baseline, and was deliberately moved back behind R2
  rather than expanding scope.
- **2026-08-24:** R1a completed as an optional product-local resident consumer.
  The real-adapter receipt proves nonadjacent sparse updates, retained
  allocation identity, silent frames, generation-aware slot reuse, explicit
  capacity refusal, and unchanged product facts. R2 host wiring and R3's
  second-consumer challenge remain open.
- **2026-08-25:** R1b completed as an optional product-local renderer tenant.
  A real adapter carried accepted body positions from Quint's exact pooled
  allocation into a configurable isometric marker texture and through
  Netrender's same-device compositor. Default profile tests remain GPU-free;
  desktop host adoption and the second-consumer challenge remain open.
