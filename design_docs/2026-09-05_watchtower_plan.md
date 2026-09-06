# Ruined watchtower: first connected campaign

**Status: W1-W6 locally verified; W7 passes Windows/ThinkPad and Windows/M4 in both host roles (2026-09-06).** Mark accepted an original ruined
watchtower as the first playable procedural place, with Luna/Terra agents
and permission to improve shared stack utilities when a concrete need emerges.

## Purpose

Connect generation, inhabited maps, voxel appearance and ordinary session
events through a small original campaign. The first board uses the existing
height field. Volume geometry, environmental destruction, overmap H2,
Stickleback adoption and desktop GPU-runtime adoption remain distinct work.

## Gates

- **W1 — Inhabited campaign maps.** A draft names generic placed inhabitants
  and sheet fields. Validation rejects invalid placement before commit;
  accepted populated maps replay through the existing session events.
  Done when generation, serialization, refusal and replica tests pass.
- **W2 — Watchtower pack.** An original seeded generator supplies the ruin,
  its approach, inhabitants, a creature, and a playable party member. A lock
  preserves a chosen layout constraint. Preview lists who will be placed.
  Done when repeated seeds replay, different seeds vary the place, locked
  constraints hold and the approach reaches the encounter under movement rules.
- **W3 — Visible desktop integration.** The bundled generator is selectable,
  commits through the normal host path and displays a distinct voxel creature.
  Done when the app compiles and a rendered receipt shows the inhabited board;
  any unavailable headed/network proof is recorded separately.
- **W4 — Forest region.** Expand the same tower into a forest with a road,
  stream clearing and navigable region. Keep map identities and reciprocal
  entries explicit. Done when accepted maps can be traversed and reopened,
  and the tower frame visibly sits among trees.
- **W5 — Create character.** A host can enter a name, appearance and optional
  player owner, then create a token with the loaded system's default sheet.
  Done when real text input, refusal, placement and sheet binding pass. This
  first flow remains map-bound; a reusable character library and Knot prose
  editing are follow-ons.
- **W6 — Lookout visibility.** Finite canopy and terrain heights determine
  the desktop fog view. Done when ground-level trees obscure distant ground,
  a lookout clears nearby canopy, and distant canopy/raised terrain still
  occlude. This is height-field visibility, not voxel volume ray tracing.
- **W7 — Two-machine session receipt.** Run the authored watchtower pack
  through real Iroh QUIC on Windows and the ThinkPad, then reverse host and
  client roles. Create a complete character, acknowledge its token and sheet,
  resolve the authored doorway into Bellwood Reach and preserve the tower's
  residents. Done when both processes exit successfully with matching protocol,
  event sequence, log hash and byte-identical final snapshots in both directions.
  This is a headless session gate; two graphical clients and separate player
  map views remain distinct checks.

## Shared-world direction

Mark approved the next proof as one created character, one creature and their
forest watchtower region. The wing brief remains
`mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md`.
Ash and Bells is the initial reproducible Isometry fixture for that direction.
The other games still need admitted world/body interchange before this can
be called a shared playable world: Paredros currently consumes Mesocosm's
`Ground/Grown/Places`, while Isometry consumes `LocalMapProposal` and opaque
system sheets. Matching a seed does not make these the same terrain or actor.
Keep game-specific test worlds alongside the eventual shared fixture.

Rules, worldgen, critters, borgs and characters should meet through explicit
identity and capability mappings. This slice does not pretend that a named
token is already a Mesocosm organism or a Paredros body. Promote utilities
to Mere when a second consumer proves the shared contract.

## Findings

- **2026-09-05:** `DraftMap` currently carries only a `LocalMapProposal` and
  scale. `HostSession::commit_campaign` lowers each into `MapStored`, making
  that the narrow seam for placing inhabitants without peer rules execution.
- **2026-09-05:** The desktop's `generator_pack_roots` loads core and demo
  packs. `theme::voxel_token_css` bakes one recolored humanoid for the starter
  creatures. The new creature needs its own volume rather than another tint.
- **2026-09-05:** The shared checkout contains earlier host/dependency work.
  This pass preserves it; receipts must identify that dependency on local WIP.
- **2026-09-05:** Real postcard replay exposed an existing transport gap:
  internally tagged `GenValue`/`StoryletEffect` decoded in JSON but failed
  binary decoding with `WontImplement`. Authored JSON stays tagged; binary
  serialization needs ordinary enum representations. Protocol/ALPN v3
  explicitly refuses older session binaries.
- **2026-09-05:** `commit_campaign` retained secrets privately but also copied
  them through `GameEvent::Generation`. Public records now strip the private
  payload after staging it, with public-stream/history/snapshot checks.
- **2026-09-05:** The first native frame exposed rectangular tile and cliff
  boxes: owned Livery still classifies `clip-path` as unimplemented. Netrender
  already accepts path clips. W3 now includes connecting polygon styles to
  paint and hit clipping in Genet, then proving the actual Isometry grid.
  A literal backslash in the cliff-face style also invalidated its `z-index`;
  the authored declaration is corrected.

## Progress

- **2026-09-06, M4 network receipt:** Windows-host/M4-client and
  M4-host/Windows-client both pass the same headless forest scenario. Each
  reaches protocol v4, sequence 26, log hash `257a7345d27a34a2` and the same
  14,399-byte snapshot as the ThinkPad runs. Both processes exit 0 in each run.
  Receipts under `Code/testing/isometry/network-2026-09-06/` are
  `windows-host-imac-m4-023206-2191a156.json` and
  `imac-m4-host-windows-023210-55831cca.json`. The staged source manifest
  matches the earlier headless closure; the M4 native graphical dependency
  closure also builds successfully with Rust 1.97.1.
- **2026-09-06, graphical pair remains open:** The M4 joins over Iroh, but
  early runs found a locked desktop. During a later unlocked interval,
  `headed-session/headed-031626-35943a2c/before.json` records identical public
  UI snapshots from both clients on Ash-Bell Watchtower. The Mac display was
  asleep and its surface remained occluded, so no Mac frame or post-turn
  receipt was produced. The desktop locked again before the next retry.
  A postcard roundtrip also passes on M4. Completing the graphical gate
  requires an unlocked, awake M4 desktop. Intel SSH remains unavailable
  through the tested routes.
  Opt-in native capture now exports each applied network snapshot, allows the
  selftest turn to wait for a controller trigger, and finishes PNG encoding
  before replacing the capture file. `headed.py` checks the desktop lock,
  compares both UI snapshots before and after an acknowledged turn, and
  retains frame captures. After validating an unlocked session, it applies
  a three-minute display activity assertion without changing sleep settings.
  The forest checkpoint uses the app's underscore
  campaign filename. Failed runs remain under `headed-session/`; none is a
  passing graphical receipt.
- **2026-09-06, capture verification:** Windows native build and all 31 host
  tests pass. The full workspace all-feature/all-target check passes on both
  Windows and M4. M4 host tests pass 29 of 31: two panel disclosure tests
  expect fixed Windows heights but measure 722/743 on Mac. A staged control
  removing the capture call, trigger gate and atomic PNG path reproduces
  those exact failures. Their cross-platform layout expectations remain open.
  M4 sources were restored, hash-verified and rebuilt after that control.

- **2026-09-06, W7 verified:** Real Windows-host/ThinkPad-client and
  ThinkPad-host/Windows-client sessions both pass. Each pair applies 26 events
  under protocol v4 and produces log hash `257a7345d27a34a2`. All four final
  snapshots are byte-identical (14,399 bytes), SHA-256
  `1d43929e840b1100d393dd10de3399ebab0836c707311092e5e9d81358cdcbc6`.
  All four processes exit 0 after acknowledged receipt comparison. Results:
  `Code/testing/isometry/network-2026-09-06/windows-host-005822-d982fe3c.json`
  and `thinkpad-host-005829-4c86af42.json` in that same directory.
  The 136 staged files match on both hosts; manifest SHA-256
  `efec3a1d48a4d05e76a6f9441d0c3504e38466f54813846a4c9e5517a44e26a8`.
  Both use Rust 1.97.1 and Iroh 1.0.3. The final product
  `cargo check --workspace --all-features --all-targets --offline` also passes
  on the existing local sibling graph (`workspace-check-final.log`). The
  ThinkPad checkout remains clean. The configured iMac routes were unavailable
  by SSH; they are not part of this receipt. Connection-path selection was not
  instrumented, so this does not distinguish direct LAN from relay transport.
  Two graphical clients, reconnect/restart, independent player map views,
  published dependency pins and cross-game interchange remain unverified here.
- **2026-09-06, network harness:** `isonetry/examples/forest_session.rs`
  generates the actual watchtower pack with seed 91 and exercises campaign
  commit, atomic character creation and the resolved doorway. Client action
  requests acknowledge each phase; the controller releases the client only
  after comparing both receipts and complete postcard snapshots. Mira remains
  on the active tower board and acknowledges the final receipt, because action
  ownership currently resolves only on that board. Mara's token and sheet move
  into the stored region map, while the creature remains in the stored tower.
  This checks split-party storage, not independent player map views.
  The isolated receipt workspace under
  `Code/testing/isometry/network-2026-09-06/` preserves both machines' checkouts.
  `stage.py` copies production sources byte-for-byte and hashes them; its
  generated manifests select only the headless Iroh dependency graph. Optional
  campaign transports and graphical hosts are outside that staged graph.
  `verify_source.py`, `run.py`, the resolved lock and build logs accompany the
  receipt. Run the example's host command first, pass its ticket to the client
  with a fresh `--release-file`, compare receipts/snapshot bytes, then create
  that release file. The controller performs this sequence in both directions.
- **2026-09-05, forest extension:** W4-W6 implementation connects Bellwood
  Reach, the Forest Road, Alder Stream Clearing and the tower through named
  reciprocal doors. The tower has a forest floor, voxel canopy, stone faces
  and a height-6 lookout reached by stepped platforms. Height-aware fog uses a
  finite canopy rather than treating every tree as an infinitely tall wall.
  Character authoring stages a name, appearance and owner before the host
  attaches the loaded system's default sheet. `CharacterCreated` makes the
  network operation atomic; protocol and ALPN are now v4. Doorway activation
  distinguishes uncontrolled faction residents from player parties.
- **2026-09-05, forest verification:** 266 tests pass across core, system,
  views, voxel and isonetry with all features (one pre-existing scaled-world
  test remains ignored). The workspace all-feature/all-target check passes.
  Native captures verify the forest tower, traveled region and character form
  under `Code/testing/isometry/images/2026-09-05_forest/`. The foreground
  lookout was removed after visual review hid the creature; the rear lookout
  remains climbable from either seeded encounter layout. Final host tests
  cover real name/owner input, complete sheet creation, missing-system refusal,
  map travel, ownership, visibility, combat and checkpoint reopen. Receipts
  are the `forest-*` logs under `Code/targets/isometry-watchtower/receipts/`.
  These remain local WIP receipts; live multi-machine play and cross-game
  world/body interchange are not claimed.
- **2026-09-05:** Started W1-W3. Terra owns the inhabited-map/session seam;
  Luna owns original pack content and generation checks; the parent owns
  desktop discovery, appearance and integration verification.
- **2026-09-05:** Added the Ash and Bells pack, placed inhabitants and sheet
  lowering, preview rows, a distinct four-facing creature and voxel props.
  W1 and W2 pass: 175 tests across campaign, system, voxel and isonetry cover
  seeded layouts, locked breaches, traversable approaches, placed sheets,
  actual binary replay and private-payload exclusion. The three pack tests
  also pass after aligning hero ownership and wall props with desktop defaults.
- **2026-09-05:** `cargo check --workspace --all-features --all-targets
  --offline -j 1` passes over local WIP. W3 desktop dispatch/persistence and
  rendered receipts remain in verification. Protocol v3 is required at both
  endpoints; a real two-machine session has not been exercised by this pass.
- **2026-09-05:** The desktop host test passes generation, commit, ownership,
  fog, a resolved attack using generated sheets, and checkpoint reopen. The
  native selftest commits five inhabitants and renders a frame. That initial
  frame is a failing grid receipt, not completion of W3; shared polygon
  clipping and its downstream visual check are in progress.
- **2026-09-05:** Shared Genet/Livery polygon support passes 42 value tests,
  72 paint tests and 25 interaction tests. The property catalog explicitly
  records the admitted `none`/`polygon()` subset. Isometry's local source
  override now selects that implementation through Mere's rootstock host;
  published dependency pins are unchanged. The workspace check also passes
  on this local graph. Native recapture and consumer click verification are
  still required before W3 closes.
- **2026-09-05:** W3 passes. The native frame shows the diamond field,
  clipped cliff faces, markers, inhabitants and the upper-ledge creature.
  Both desktop tests pass: diamond corners reject hits at zoom 1 and 0.93;
  generation/commit/ownership/fog/attack/checkpoint reopen also passes. All
  59 view tests and three pack tests pass after the final authoring changes.
  Removed 14 invisible placeholder props that added hover targets without
  appearance. The snapshot is saved at
  `Code/testing/isometry/images/2026-09-05_watchtower/watchtower-grid.png`;
  compiler/test logs are under
  `Code/targets/isometry-watchtower/receipts/`. These are local WIP receipts,
  not a published dependency update or a live two-machine proof.
