# Wing functions

Shared, deterministic functional networks for the Isometry games. MPL-2.0.
This crate has its own small Cargo workspace and no rendering or host dependency.

```powershell
cargo test --manifest-path shared/wing-functions/Cargo.toml --offline
cargo run --manifest-path shared/wing-functions/Cargo.toml --example inspect_functions --offline
```

A caller supplies stable `PartRef` identities, a current set of live parts, and
`WorldRules`. Networks connect finite charge sources and stores to gates and
effect sites. `preview` leaves state unchanged; `evaluate` and `evaluate_chain`
commit charge changes only if the entire request or sequential batch succeeds.
Strengthen and Project return typed receipts for the product to adjudicate.
Store transfers charge; it does not instantiate a spell or material.

Routing uses deterministic breadth-first selection and remaining edge capacity.
It is a first-fit routing policy, not a maximum-flow guarantee. Some networks
can require a different routing policy to use all feasible supply. Capacities
apply per operation; a batch executes sequentially. This does not yet model
simultaneous limb timing, sustained charging, recovery or interrupts over time.

`generation` samples functional blueprints for caller-supplied creature or staff
sites. Settings control actuator count, capacities and gate state. Form seeds
remain stable if another form is removed or reordered. It samples connections
and functions, not physical anatomy or item geometry. Material/tissue admission
and site assignment belong to the construction rules used by the caller.

Use `NetworkSnapshot::validate` after loading; execution also validates its
network. The consumer chooses storage, identity lifetimes and migration rules.
Never substitute a saved live-part set for current body authority. Mesocosm's
`functions` module supplies that reading. Isometry's `ConstructionProposal`
carries the accepted network and operation requests inside its existing
`GenValue` envelope and binary generation record, without rerunning generation.

The abstract charge units here do not modify Mesocosm's existing matter ledger.
An ecology adapter must explicitly account for any conversion or replenishment.
