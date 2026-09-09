# Paredros

Paredros now lives in the [Isometry umbrella repository](../README.md), under
`paredros/`. It retains its own Cargo workspace and package names. See the
[wing index](../design_docs/DOC_README.md) for all three products.

A second-person action RPG in a persistent generated world.

You name one creature and ordinarily inhabit that life until it dies. Other
named creatures live their own lives across settlements, dungeons, ruins,
surface and underground places. You may help, persuade, equip, house, work
beside, coordinate with, avoid, rob, or fight them. Allies are useful but not
required, and never units in a party.

Control changes only through death, an explicit world event, or an optional
player rule. Creative settings may allow tag-in; difficult or obscure world
mechanics may permit possession, domination, transplantation, cloning, or
stranger ways of continuing. The body and consequences left behind remain in
the world.

Vessel 2 of a three-game wing (Mesocosm, Paredros, Isometry); the wing-level
architecture lives in the sibling mesocosm repo and is cited, not copied.

## Status (2026-09-05)

Early implementation. Four proof scenes landed 2026-08-08; nothing here is a
shipped game yet.

- S0 room probe (`paredros-room`): one body walking one room carved into
  grown mesocosm terrain, with save/reload replay to a matching state hash,
  rendered through the shared renderer stack.
- S1 refusal scene (`paredros-social`): three companions with different
  histories answer the same offer three different ways, with their premises
  attached; one standing agreement is formed, exercised, renegotiated, and
  ended.
- S2 negotiated home (same crate): homes offered with daily work; residence
  and the daily round derive from agreement state, so moving out is the
  agreement ending.
- S3 sim half (`paredros-sortie`): negotiated participation, terrain falls
  as wounds, pact-governed tag-in, and sortie deeds that change later
  answers. The headed real-time action half is open, as are the playtester
  judgments for S1-S3.
- The 2026-08-10 extraction review promoted `paredros-identity` to the
  wing-wide identity crate (relicensed MIT OR Apache-2.0) and pushed the GPU
  tenancy seam up into netrender.
- The headed room now proves both frame-validation policies against its real
  swapchain calls. Awaited validation suppressed attempt 1 before acquisition
  and presented attempt 2. Optimistic validation presented attempt 1,
  suppressed attempt 2 after the result resolved, then presented attempt 3.
- Shared faults now rebuild the complete GPU client set through the same path
  as initial boot while preserving the host window and surface. A headed
  synthetic-fault receipt suppresses generation 1 before acquisition and
  presents successfully from generation 2.
- The Renderling room tenant now records its 20-pass frame into a caller-owned
  encoder and Paredros submits it once. RG3c reports that physical tenant
  submission separately from Netrender's one graph submission and retains the
  466-colour byte-match against the legacy composition path.

Current plans live in [design_docs/](design_docs/DOC_README.md); the
executable plan retains S0-S3 as foundation receipts and orders future work
through F0-F8: persistent world, one embodied life, other autonomous lives,
memory and standing, coordination, material life, settlement and culture,
danger, then death and control continuity. Camera choice remains open until
those spatial laws can judge it.

## Use

The dry damaged-crossing sandbox is interactive:

```sh
cargo run -p paredros-room --bin crossing
```

WASD moves, arrows or right-drag orbit, wheel zooms, Shift braces, E handles
the board, Q attaches/releases a tether, Space strikes, R repairs capability,
and F requests a practice counterstrike. Keys 1/2 restart as crawler/climber.
This is authored contact testing, not the generated-world or NPC encounter yet.
Set `PAREDROS_CROSSING_SMOKE=1` for a deterministic headed capture/replay
check; each run writes a separate directory beneath `PAREDROS_CROSSING_OUTPUT`
(default `Code/testing/paredros/crossing`). Set `PAREDROS_FONT` to a local
TTF/OTF if the host cannot find one of its platform font fallbacks.
`PAREDROS_CROSSING_AMBIENT` sets fixture ambient light from 0 to 1 (default
0.72). The older room probes keep their original torch lighting.

Scene receipts are runnable. Building requires the sibling repos (mesocosm,
netrender, and a local renderling fork) checked out at their expected
relative paths, since cross-repo deps are path deps.

```sh
cargo run -p paredros-social --bin refusal   # three companions answer one offer
cargo run -p paredros-social --bin home      # the negotiated home
cargo run -p paredros-sortie --bin sortie    # one sortie and return
ROOM_TRACE=1 cargo run -p paredros-room --bin room   # headed room probe
PAREDROS_RG3_HEADED_PROBE=awaited cargo run -p paredros-room --bin room
PAREDROS_RG3_HEADED_PROBE=optimistic cargo run -p paredros-room --bin room
PAREDROS_RG3_REBUILD_PROBE=1 cargo run -p paredros-room --bin room
```

## License

Two-part boundary (see [LICENSES.md](LICENSES.md)):

- game code, repository documentation, and `paredros-identity`: MPL-2.0
- original game assets: CC BY-SA 4.0

The published `0.0.1` name reservation remains MIT OR Apache-2.0; the split
begins with `0.0.2`. A promoted reusable library no longer takes a separate
permissive grant, per Mark's 2026-09-03 ruling retiring the 2026-07-31
clause.

---

*This README was generated by AI and will be edited by the author upon
release.*
