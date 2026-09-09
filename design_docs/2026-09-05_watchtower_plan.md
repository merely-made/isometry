# Ruined watchtower: first connected campaign

**Status: W1-W6 and W8 locally verified; W7 passes Windows/ThinkPad and Windows/M4 in both host roles; Windows-host/M4-player graphical turn verified (2026-09-06). W9-W10 pass native/protocol checks and the M4 travel-overmap visual receipt (2026-09-08), using its preserved dependency closure.** Mark accepted an original ruined
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
- **W8 — Readable forest sites.** Project existing encounter anchors and
  doorway destinations onto the active board. A player can identify the wolf
  trail, road sign, stream crossing and nest without changing movement or
  resolving an encounter merely by inspecting it. Done when the generated
  sites reach the native board, unexplored sites remain hidden, and a native
  forest-region capture shows a distinct site marker beside the doors.
- **W9 — Start on the overmap.** Campaign creation initializes the current
  session party at the world place bound to its starting map. Solo and hosted
  creation use the same authority path and ordinary `PartyMoved` event; the
  party discovers only the starting place and its adjacent routes. Map-only
  drafts remain supported. Done when default `dm` and explicit viewer parties
  open a populated overmap, actor forwarding and binary replay agree, and
  missing/ambiguous starts or blank party identities fail without mutation.
  Tactical doorway updates and initialization of other parties remain separate.
- **W10 — Doorways update the overmap.** An initialized session party follows
  the last-player crossing that activates the destination map. Its current
  place must bind the source map and the destination must have one unambiguous
  world place. Earlier split-party departures and maps without a binding leave
  regional state alone. Hosted crossings resolve and commit through one
  authority operation; solo uses the same resolver. Done when movement,
  discovery, duplicate/refused requests, replica replay and native travel agree.

## Atlas follow-on (first implementation verified, 2026-09-09)

Mark requested a generated region backdrop, items tending back toward their
arrangement after dragging, and clickable location areas instead of mandatory
circle markers. The shared primitives exist; the Isometry integration is now
present in the working tree. Focused tests, the workspace all-features check,
and a native Windows capture passed. The M4 compatibility bundle remains separate.

- **W11 — Fixed region backdrop.** Derive Bellwood Reach's overhead terrain
  from its retained region `CampaignMap`: forest, road and stream already exist,
  and its transition coordinates agree with the world-place coordinates. Use
  fixed region bounds and one aspect-preserving transform for terrain, places,
  routes, picking and discovery overlays. Replace the current discovered-node
  bounding-box fit. Cache terrain paint until its source or appearance changes.
  Done when discovering another site leaves existing geography stationary and
  pan/zoom keep all layers aligned without disclosing hidden site details.
  **Implementation:** `isometry-views` selects the source region from the
  party's current local or regional map, uses the retained source-time world
  and map pair, paints fixed bounds, and caches decorated terrain by source
  content. All 75 `isometry-views --lib` tests pass, including 12 atlas tests.
  Workspace all-features/all-targets and native Windows build/capture passed.
- **W12 — Clickable location areas.** Carry source-bound place footprints into
  the shared scene and render hover/focus/selection directly on those areas.
  Keep keyboard access and existing inspect/travel semantics. Use actual
  authored/generated boundaries; a graph-layout hull is not automatically a
  geographic boundary. Done when polygon interiors select the expected place,
  empty corners of their bounding rectangles do not, and overlapping parent
  regions and smaller sites have deterministic selection priority.
  **Implementation:** shared `GraphCanvasAtlas` fields carry source-bound
  polygon footprints, fixed-world route anchors, and explicit priority. Site
  details remain filtered by party knowledge, with no watchtower identifiers
  in the adapter. Seven shared atlas tests pass, including exact polygon picks,
  captured dragging, keyboard focus/activation, and deferred view rebuilding.
- **W13 — Return to arrangement.** Replace implicit permanent drag overrides
  with explicit home/free/pinned projection behavior. Geographic areas stay
  anchored; labels or callouts can move temporarily and return. Reuse the
  shared anchor-spring and drag pin/unpin behavior, with configurable return
  strength/damping and reduced-motion behavior. Connect the shared deferred
  drag/leaf refresh path before animating: settling must not rebuild the full
  DOM every tick. Done when release returns an unpinned item, an explicit pin
  holds it, campaign coordinates remain unchanged, and settled views go idle.
  **Implementation:** atlas field dragging feeds the existing normalized
  `OvermapMotionState` offsets and reduced-motion setting. Native harness
  coverage passes. Three deferred-motion tests and six standalone spring tests
  pass. The native watchtower suite passes 10 tests; its separate diagnostic
  benchmark also passes. On Windows, 30 atlas drag moves produced 0.152 ms median
  retained layout, with no style or text relayout. This is a CPU harness receipt,
  not presented FPS or a controlled comparison with the earlier M4 run.

### Atlas implementation findings (2026-09-09)

- Source selection is explicit: a party at a local map resolves one unique
  retained region through its transition; ambiguity keeps the graph fallback.
- Terrain is complete and cached independently of knowledge-filtered fields,
  so discovering a site does not move existing homes or disclose hidden site
  details. Fixed bounds govern terrain, routes, fields, and normalized homes.
- The shared atlas is a projection surface. It does not edit campaign maps or
  world coordinates, and Signalman's geographic/coverage view remains a
  future consumer rather than a verified integration.
- The current text path still uses simple left-to-right platform-font glyph
  outlines where text is needed; full text-engine shaping remains open.

Reuse boundaries from the live source audit:

- `mere/crates/cambium/scenes/sceno/src/scene.rs`: `Backdrop`, `Space`,
  `ProjectedItem.hit` and region contours; polygon geometry/containment lives
  in the adjacent `footprint.rs`. These are contracts, not a terrain generator.
- `mere/crates/conatus/seiche/src/anchor_force.rs` and
  `mere/crates/canvas/canvas/src/physics_board.rs`: existing arrangement anchors
  and temporary drag pinning. Adapt the behavior through a measured shared
  integration rather than creating an Isometry-only solver.
- `mere/crates/cambium/cambium/src/atlas.rs` and `graph_canvas.rs`: the new
  fixed-world atlas, polygon fields, retained compound callout paths, and
  separate `GraphAtlasEvent` preserve the existing graph-canvas consumer API.
  Terrain production and campaign knowledge remain in Isometry. The shared
  runner supports deferred paint updates; Scenotime owns lightweight return
  motion. These current-source APIs are not automatically available in the
  older M4 compatibility closure used for the performance receipt.

This should also serve a Signalman geographic/coverage view. That is a target
for reuse, not an already-verified Signalman atlas integration. The local receipt
is `Code/testing/isometry/atlas-2026-09-09.md`; its final native PNG verifies the
forest backdrop, known location fields, readable labels, and return controls.

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

- **2026-09-08, interaction lag investigation:** The M4 debug-build native
  drag harness measures 30 real graph-pointer steps: pointer median 1.622 ms,
  explicit post-dispatch 0.950 ms, and relayout 235.421 ms (p95 239.164 ms).
  Building the four-node swatch averages 0.007 ms. This is a windowless CPU
  interaction workload, not a presented-frame rate or release-build result.
  Initial occluded frame comparisons do not measure GPU capture cost. Source
  inspection also found synchronous readback/PNG writing on every captured
  frame, source-adapter cloning before its unchanged-history guard, and repeated
  historical replay on unchanged slider cursors. The capture/history fixes pass
  two capture tests, the source-time regression, seven watchtower tests and the
  all-feature/all-target M4 check. Capture now saves once after pending
  self-tests; `ISOMETRY_CAPTURE_EVERY_FRAME=1` explicitly enables continuous
  diagnostic capture, including network receipts that await an asynchronous
  turn echo. Capture does not request idle redraws. The fixed debug benchmark
  still measures 240.427 ms median relayout; optimizing only `cambium-rootstock`
  and `genet-livery` to level 2 gives 232.648 ms. These fixes do not yet resolve
  the dominant lag. Shared layout phases and consumer optimization remain
  under measurement. Logs: `Code/testing/isometry/*fixed-2026-09-08.log` and
  `profile-targeted-opt2-2026-09-08.log`.
  A profiling-only Mere patch adds `RelayoutProfile` for windowless harnesses
  and style/text/extent spans to presented-frame profiles. Whole-file staging
  initially mixed an unrelated range-scrubber API into the M4 closure;
  recovery uses the six Cambium host files from `e02a08f1` plus the narrow
  profiling patch, not a claim of byte-identical restoration. Its fresh
  baseline is 236.387 ms median relayout: 104.235 ms style resolution,
  131.549 ms text/layout and 0.252 ms extent traversal. Subsequent comparisons
  must use that recovered baseline. The recovered closure passes all seven
  watchtower tests and the all-feature/all-target check. Livery's
  empty-declaration-family selector shortcut reduces median style resolution
  to 63.047 ms and total relayout to 194.210 ms (p95 198.568 ms), about 18%
  faster overall. Text/layout remains 130.735 ms. The isolated selector suite
  passes six tests; its source is staged separately from the incomplete Genet
  workspace, with only the inherited cssparser dependency made explicit.
  Earlier guard and four-package optimization timings used a misapplied
  staging hunk and are superseded. The regression caught it; the corrected
  function-specific guard placement is the basis for further comparisons.
  Optimizing `isometry-genet`, `cambium-rootstock`, `genet-livery`, `livery`,
  `buckram` and `genet-taffy` at level 2 reduces relayout to 39.196 ms median
  (p95 39.563), split into 17.287 ms style resolution and 21.652 ms text/layout.
  Those six overrides now live in Isometry's dev profile, preserving debug
  symbols/assertions and other packages' existing settings. The final run with
  those manifest defaults and no CLI overrides measures 36.753 ms median
  relayout (p95 40.580), with 16.337 ms style resolution and 20.235 ms text/layout.
  Exact-current isolated Livery selectors pass six tests; watchtower passes
  seven, and the full optional-feature/all-target check passes in 14.54 s.
  The optimized native build passes and replaces the idle M4 capture-app
  executable with a hash-matched copy. The GUI was not relaunched: these CPU
  timings are not a presented FPS claim. Adding Parley optimization improved
  the earlier six-package result by only 2.4%, so it is excluded from defaults.
  Final receipts are `Code/testing/isometry/*-final-manifest-opt2-2026-09-08.log`
  and `livery-current-selectors-verified-2026-09-08.log`.
- **2026-09-08, travel-overmap visually verified:** After the M4 was unlocked,
  the native artifact app launched through LaunchServices with
  `ISOMETRY_WATCHTOWER_SELFTEST=1`, `ISOMETRY_WATCHTOWER_VIEW=travel-overmap`,
  and `ISOMETRY_CAPTURE_DIR`. Brief display-awake assertions and activation
  allowed its own GPU readback to save the final frame. The inspected image
  shows Bellwood Reach highlighted as the current party location, with the
  Ash-Bell Watchtower, Forest Road and Alder Stream Clearing visible and three
  connecting routes. The board state also names Bellwood Reach after travel.
  Receipt: `Code/testing/isometry/images/m4-travel-overmap-2026-09-08/overmap.png`
  with sibling `launch.log`. This closes the pending W9-W10 visual check for
  the preserved M4 dependency closure; the current Windows dependency build
  remains unverified. Intel is now in use and excluded from further probes.
- **2026-09-08, native capture retry:** M4 native rebuild passed. Direct SSH
  launch remained occluded; LaunchServices succeeded with the native executable
  inside the artifact app bundle and reached the watchtower selftest. OS screen
  capture over SSH could not access a display. The subsequent native GPU
  self-capture attempt used `ISOMETRY_CAPTURE_DIR`, but the desktop relocked
  before rendering and produced no PNG. Both owned app processes were stopped.
  The working LaunchServices command is ready for an unlocked-session retry.
  Intel SSH still times out during banner exchange. Windows full check still
  waits on the shared package-cache lock. Launch logs are under
  `Code/testing/isometry/mac-launchservices-native-2026-09-08.log` and
  `mac-direct-travel-overmap-2026-09-08.log`.
- **2026-09-07, W10 started:** Doorway resolution now has explicit session-party
  context. The desktop sends hosted travel to the authority actor so a stale
  view cannot commit regional movement independently of the crossing. Solo
  copies the resulting world state back alongside map, sheet and clock state.
  Independent per-party map views and party-membership editing remain separate.
  The M4 frozen dependency closure passes all seven native watchtower tests
  and three network-bridge tests, including W9 creation and W10 queued travel.
  These exercise default `dm` and explicit `player` identities, map-only
  compatibility, return travel, and four known sites after forest arrival.
  Logs: `Code/testing/isometry/mac-watchtower-2026-09-07.log` and
  `mac-net-2026-09-07.log`. The current Windows workspace gate still waits on
  another build's package-cache lock; it has not compiled this revision.
  Six source-matched isolated protocol tests also pass: final departure, split
  departure, missing/ambiguous target binding, unrelated party preservation,
  postcard client replay/discovery, and refused-request invariance. Repeating a
  crossing request after a later regional move cannot move the marker back.
  The M4 frozen closure passes `cargo check --workspace --all-features
  --all-targets --offline --target-dir ../target -j1`; see
  `Code/testing/isometry/mac-workspace-all-features-check-2026-09-07.log`.
  All 15 transferred Rust files match local SHA-256 hashes; receipt:
  `Code/testing/isometry/mac-source-hashes-2026-09-07.txt`. A fresh overmap
  image remains unverified because the M4 console session reported locked.
  No GUI process was launched. `ISOMETRY_WATCHTOWER_VIEW=travel-overmap`
  now supplies the arrival/discovery assertions and overmap view for that check.
- **2026-09-07, W9 started:** The desktop passes its current viewer identity
  (or existing `dm` default) to an explicit party-aware campaign commit. The
  authority resolves the starting world place and stages `PartyMoved` after
  map activation. No new wire event or inference from inhabitants is needed.
  The legacy commit remains available for callers without party context and
  map-only drafts. Party identity stays exact, matching existing viewer lookup.
  Regression cases cover exact identity, existing-party preservation, rejected
  starts without mutation, JSON/postcard replay, the host actor, and the desktop
  overmap projection. Formatting checks pass. The focused protocol run did not
  reach compilation in the main workspace; a bounded diagnostic and the full workspace gate both
  waited on the shared Cargo package-cache lock. Desktop tests and a fresh
  overmap capture remain unverified. This is implementation progress, not W9
  acceptance. Tactical doorway travel still needs its own explicit party policy.
  A source-matched isolated default-protocol harness subsequently passed all
  three `campaign_start` tests (46 filtered). Its copied manifest excludes
  optional network dependencies, so this proves placement, replay and atomic
  refusal only. The copied harness and its limitations are retained in the
  local consolidation archive at `Code/archive/wing-consolidation-20260909/party-start-core/`.
- **2026-09-06, W8 started:** Existing `EncounterAnchor` records are carried
  through campaign generation and replication but never projected onto the
  board. This slice exposes those authored cues and destination labels.
  A separate audit found the larger overmap connection still missing:
  campaign commit and tactical doorway crossing do not initialize/update
  `party_node` or `party_known`, leaving a fresh party's filtered overmap
  empty. Follow-on work should emit existing authority-owned `PartyMoved`
  events with an explicitly selected party, including split-party behavior,
  rather than infer positions in each client's view.
- **2026-09-06, W8 implementation verified:** Authored sites now have gold
  tiles, readable hover labels and stable accessible identifiers. Doorways
  identify their destination maps and retain purple tiles. Hidden terrain
  emits no site cue; dim terrain remembers public static landmarks. The
  metadata lookup runs once per emitted tile, and inspection emits no game
  event. All five native watchtower tests pass on Windows, including projection
  of anchors on every generated map. The final focused label/fog/inspection
  test and full workspace all-feature/all-target check pass on the isolated
  M4 closure. The Windows full workspace check also passes. Native visual
  review passes at `Code/testing/isometry/images/forest-sites-b158a2bf/forest-sites.png`:
  Bellwood Reach displays the gold wolf-trail diamond, purple doors and readable
  site status. The first frame exposed terrain styles overriding the marker;
  the final selector outranks those terrain classes. Logs and source hashes
  are `Code/testing/isometry/site-*`. These are local WIP receipts.
- **2026-09-06, graphical pair verified:** Run
  `Code/testing/isometry/network-2026-09-06/headed-session/headed-214736-fbb42c72/`
  passes with Windows hosting and M4 joining as `player`. Both UI snapshots
  agree before and after the normal end-turn path, advancing active index
  0 to 1 while preserving tokens. Both native before/after frame pairs are
  complete and distinct; visual review confirms the isometric forest tower,
  full host view, player fog view and next-turn highlight. `receipt.json`
  records final state SHA-256
  `97501f39619e922da2dc665c53a3f7d60ee02c8cc310d80993f84d8f852ce393`.
  The bounded display activity assertion resolved the earlier occlusion.
  This proves one graphical host/player turn; reconnect, reverse graphical
  host roles and independent player map views remain unverified. Intel SSH
  still times out at its configured and previously known direct addresses.
- **2026-09-06, M4 network receipt:** Windows-host/M4-client and
  M4-host/Windows-client both pass the same headless forest scenario. Each
  reaches protocol v4, sequence 26, log hash `257a7345d27a34a2` and the same
  14,399-byte snapshot as the ThinkPad runs. Both processes exit 0 in each run.
  Receipts under `Code/testing/isometry/network-2026-09-06/` are
  `windows-host-imac-m4-023206-2191a156.json` and
  `imac-m4-host-windows-023210-55831cca.json`. The staged source manifest
  matches the earlier headless closure; the M4 native graphical dependency
  closure also builds successfully with Rust 1.97.1.
- **2026-09-06, earlier graphical attempts:** The M4 joins over Iroh, but
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
