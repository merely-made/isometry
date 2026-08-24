# Runtime Profile Plan

**Status:** active (2026-08-23); host-neutral profile and focused acceptance
receipt complete, desktop adoption gate open.

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

### R2 — Desktop adoption

**Open behind existing prerequisites.** The current Isometry desktop host still
targets Genet's deleted compatibility layout cone. That migration is broader
than this profile and is not smuggled into R0/R1. Protocol hardening H2 also
remains first in the repository's audit order.

**Done when:** those prerequisites close; `isometry-genet` constructs the
profile; every accepted active-map change is synchronized once; a still redraw
is silent; and a local render/spatial consumer consumes the published frame.

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

## Stop rules

- Do not let Conatus adjudicate a move, rules action, map transition, or peer
  request.
- Do not serialize `BodyId` or Conatus state into a campaign checkpoint.
- Do not invent a shared `ProductSourceId`, spatial frame, or conductor before
  the second consumer.
- Do not defer unrelated Isometry product work behind R2 or R3.
- Do not use this slice to bypass protocol H2 or the no-second-runtime gate.

## Progress

- **2026-08-23:** R0 and R1 completed in a new host-neutral product crate: four
  focused tests and warnings-denied Clippy pass. Desktop wiring was attempted,
  exposed the stale Genet baseline, and was deliberately moved back behind R2
  rather than expanding scope.
