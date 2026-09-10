# Games wing repository consolidation

**Status: published; standalone repositories archived, 2026-09-09.** Isometry is the repository
home for the Isometry tabletop, Mesocosm, and Paredros. Package names, product
behaviour, licenses, source histories and existing uncommitted work survive.

## Scope and layout

Keep the tabletop in the existing root workspace. Import Mesocosm under
`mesocosm/` and Paredros under `paredros/`. Initially retain their Cargo
workspace roots: repository consolidation can land without changing dependency
resolution, root patches, profiles or product semantics. A single Cargo
workspace is a subsequent integration choice, not a prerequisite for atomic
cross-product commits. No application is required to run another application.

The root `design_docs/DOC_README.md` becomes the wing entry point and links the
existing product indexes. Existing documents remain at their imported paths
so historical references remain intelligible. Root platform dependencies still
belong to Mere and Genet. Shared game mechanisms can be developed alongside
their consumers without duplicate implementations imposed by old repo borders.

## Phases and done conditions

1. Inventory and preserve: record source heads, dirty/index state, linked
   worktrees and checksums of tracked plus untracked source files. Archive
   original checkouts including ignored local state; preserve external
   worktrees and source refs. Done when every source file has a verified copy
   and original repositories remain recoverable.
2. Import history: merge both committed source histories at their prefixes,
   then carry uncommitted overlays without claiming them reviewed. Done when
   both source heads are ancestors of Isometry and dirty source work survives.
3. Wire paths and navigation: rebase relative external Cargo paths, retain
   intra-wing paths, update repository guidance/indexes, and provide old-path
   junctions for existing local consumers. Done when old entry points resolve
   to the new home and manifests resolve from the canonical new paths.
4. Verify: validate file preservation, Git ancestry and linked worktrees,
   Cargo metadata for each workspace, and focused domain tests using existing
   caches. Report pre-existing or external failures separately. No visual or
   physical-input acceptance is inferred from repository migration.

## Findings

- Isometry starts at `243c0dd78dc096212a5fff960b8c098c791b6124`, with local
  changes and external worktrees. Mesocosm starts at
  `284eb6584a688706dd9125b5f0e3b26ab61e75a7` with four linked worktrees.
  Paredros starts at `47361ae94680261233f5ad7de9e7c3e7ae828777` with local
  equipment, session and documentation changes. All indexes were initially
  empty. No matching running game, Cargo or rustc process was found.
- Paredros already directly consumes Mesocosm crates. Isometry owns generator
  hosting, system plugins and campaign proposal types. Consolidation makes
  cross-product changes atomic without requiring immediate library extraction.

## Progress

- 2026-09-09: inventory and preservation preparation started. No product code
  semantics are in scope for the migration.
- Source histories imported in three-parent merge `20d044f6cd11e0cdd55b359400f35c808bf73d3b`.
  All 414 Mesocosm and 144 Paredros tracked/untracked source files were copied
  and byte-verified before path/guidance edits. Original checkouts, ignored
  state, bundles, dirty diffs and file inventory are retained under local
  `Code/archive/wing-consolidation-20260909/`.
- Old `Code/repos/mesocosm` and `Code/repos/paredros` are junctions to the new
  directories. Target caches remain in the archive, with local junctions from
  the new workspaces. Four existing Mesocosm worktrees were repaired using
  `git worktree repair`; they retain their old repository branch histories,
  so subsequent work there needs explicit import into Isometry. The temporary
  import worktree was restored clean and removed normally.
- Fourteen external manifest paths were rebased, preserving their targets.
  Product-to-product paths remain unchanged. All three Cargo metadata roots
  resolve, with 8, 9 and 6 workspace members respectively (including product
  reservation packages where applicable).
- Isometry's local `.cargo/config.toml` moved to the ignored
  `.cargo/tabletop-local.toml`: it would otherwise leak patches and compiler
  flags into both child workspaces. `scripts/wing.ps1` passes it explicitly
  only for tabletop builds and preserves product working directories.
- Isometry's previously dirty lockfile needed one local `scenograph` package
  entry to resolve its existing working tree; that change remains part of the
  uncommitted tabletop work. Paredros tests pass with its preserved lockfile.
  A Mesocosm test attempt using the Paredros cache stalled in dependency
  resolution and was stopped; its normal cache is being used instead.
- Windows Cargo preserves a junction's lexical path. Old-path source access
  works, but bare Cargo from the old Paredros path misresolves rebased external
  paths. Product `build.ps1` launchers resolve the canonical product directory
  before invoking the root dispatcher; new canonical paths need no launcher.
- Verification complete: 479 Mesocosm core tests passed (one existing ignored),
  58 Isometry core tests passed, 35 Paredros world plus 3 session boundary tests
  passed, and 24 body-sheet tests passed. Total: 599 passed, one ignored.
  These exercised the preserved working trees, including pre-existing WIP;
  they do not certify that WIP as committed or physically playtested.
- Independent read-only audit verified source-head ancestry, all external
  path rewrites, worktree repair, ignored local state, bundle integrity and
  successful Git object checking. All baseline source bytes outside declared
  migration edits remain identical. Both old-path `build.ps1` launchers resolve
  to the canonical new Cargo workspaces.
- Follow-up: one Cargo workspace would require reconciling Mere pins, source
  identities, root patches, profiles and lockfiles. It is deliberately not
  part of this completed repository move. Existing archived-worktree lanes
  need deliberate transfer when resumed. No remote repositories were deleted,
  no product licenses changed, and no pre-existing gameplay WIP was committed
  as part of the migration.

## Publication and retirement receipt (2026-09-09)

The local migration record above describes the initial handoff. Subsequently:

- Committed the preserved tabletop work as `e5d1b97`, Paredros anatomy,
  equipment and persistence work as `e82a17f`, and procedural-system design
  notes plus repository metadata as `c8168b8`. Published the full imported
  history to Isometry main and verified both product manifests on GitHub.
- Published eight historical branch refs under `archive/mesocosm/` and
  `archive/paredros/`. Four clean Mesocosm linked worktrees now use Isometry's
  Git storage and these historical branches. Their HEADs and source trees
  were verified unchanged. They retain the old standalone tree layout.
- Archived both standalone GitHub repositories, with descriptions and homepages
  pointing to their Isometry directories. GitHub repositories remain available
  as read-only historical records.
- Removed the old checkout paths from `Code/repos/` by moving their junctions
  into the recovery archive as `mesocosm-retired-alias` and
  `paredros-retired-alias`. Use canonical Isometry product paths going forward.
- Automatic approval review blocked removal of cache junctions with the generic
  reason `blocked by policy`. The safer retirement retains the original recovery
  checkouts and target caches under `Code/archive/wing-consolidation-20260909/`.
  Physical deletion of those recovery copies was not performed. Current product
  target junctions still reference those caches; preserve that archive.
- Recovery also contains verified complete Git bundles, source tar archives,
  original diffs and file inventory. The copied party-start test harness and
  its transient resolver log were moved there instead of publishing duplicate
  source trees.
- Expanded review: Isometry views 75 passed; Paredros world 83, room library 37,
  body-sheet host 3 passed. Combined with the earlier distinct Isometry core
  and Mesocosm core selections, 735 tests passed and one was ignored.
- Native watchtower, current full replication, all-feature/all-target workspace,
  capture and interaction benchmark checks remain unverified. Pending review
  test processes were stopped when unrelated jobs held the shared Cargo cache.
  Physical input and headed acceptance remain open. Paredros formatting checks
  also encountered unrelated external Renderling formatting differences.
## Platform alignment follow-up (2026-09-09)

Status: implemented and consumer-verified. The repository move above remains
complete; this follow-up retains the three Cargo workspace roots. Separate
upstream workspace blockers and headed acceptance are recorded below.

Align the Mere and Genet consumer edges with the published Netrender
`c77b0be84fb6fc28a3c1602a2b1637f7d913acc0` API. Mesocosm replaces its
`netrender_graph` bridge and two renderer instances with one renderer. Product
paint order, sRGB conversion, alpha, and tenant submission receipts stay intact.
Paredros adopts the same immutable renderer source instead of an implicit sibling
checkout. Direct Genet DOM edges and root patches must match Mere's chosen pin.

The Mere and Genet main checkouts contain unrelated active changes. Narrow owner
integration worktrees isolate dependency updates from those changes; their exact
commits become consumer pins only after verification. The final Mere selection
also includes the existing atlas and deferred-input APIs used by tabletop, so
those consumer calls do not depend on an unpublished local override.

Done conditions:

- One reachable identity per renderer, device, paint-list and DOM package in each
  product, and the same source across products wherever those packages occur.
- Mesocosm compiles with one Netrender instance and retains its opaque section
  composition comparison and submission-count receipt.
- Focused consumer checks plus all-feature/all-target workspace attempts record
  concrete pass results or blockers; local overrides are labelled separately.
- The source audit can consume saved Cargo metadata and reject duplicate package
  identities without requiring another dependency resolution.

### Single-renderer composition receipt

The actual Mesocosm `chrome.rs` and `mesocosm-render::composite` source were
compiled by path in an isolated probe against Git Netrender `c77b0be`. The
`opaque_section_graph_byte_matches_direct_composite` test passed on an NVIDIA
GeForce RTX 4060 Laptop GPU using Vulkan at 32 by 24 pixels. It exercises a
visible UI raster between two tenant-master frames on the same renderer and
asserts stable allocation count and identical tenant-master bytes. Presentation
retains the existing maximum channel difference of 3, with one reported physical
tenant submission, one logical producer, one graph encoder batch and one graph
submission boundary. See [the machine receipt](../testing/platform-alignment/rg3b-single-renderer.json).
The [source receipt](../testing/platform-alignment/rg3b-source.json) identifies
the actual source hashes and isolated probe lockfile. This is a GPU composition
check, separate from full-host or headed acceptance.

### Upstream source selection

Genet's isolated integration commit
`3a7b50230d447f6fa7ed6921cba019f78347d932` is published on
`wing-platform-alignment-20260909`, from the previously consumed `9e8f9dc` base.
It aligns renderer, device, paint-list API/lowering, the registry paint patch,
and the standalone renderer smoke package to Netrender `c77b0be`. The focused
`genet-livery` and `genet-render-host` check passed. The full Genet workspace
all-features/all-targets check stopped in vendored Parley tests. Restoring the
tracked Lato font omitted by the sparse checkout isolated the remaining error:
`parley_dev::font_dirs()` is unresolved in `test_builders.rs:59`. The focused
production-host check remains green. Verification used the isolated sparse
source checkout, preserving the active main checkout.

Mere's matching owner update is published as
`fb7e136b13298b1e9c56ece8a282fb1b1fac4d8c` on the same named integration branch.
Its pin-only parent is `fc382ac4`; the final commit promotes the existing atlas,
retained paint, return-motion and host profiling APIs used by tabletop. The
focused all-target check of `cambium-genet-winit-host`, `cambium`, `sprigging`
and `scenotime` passed, as did all 36 Scenotime library tests. The broader
Mere check, repeated on the final source, stopped in Knot's `EditableTextV1`
initializers at `endpoint.rs:1067` and `:1097`, which lack `public_revision`.
Cleromancy's matching source selection is published as
`3b321539c5bc854403623087d8af18dcef6f2b53`; its final locked library check passes.
Tabletop selects that exact commit rather than following Cleromancy main.

### Consumer verification (2026-09-10)

All three products' all-features metadata pass the shared source-identity
audit across 80 package names: each has at most one reachable
identity per product, with matching identities wherever shared. Legitimate
package absences are informational. The [saved audit output](../testing/platform-alignment/source-identities.txt)
records the selected identities. Tabletop's tracked lockfile is refreshed;
Mesocosm and Paredros keep their existing ignored-lock policy. The audit covers
every resolved Git package from Mere, Genet and Netrender, plus critical names
that may be supplied through registry or local overrides. This broader check
also aligned Mesocosm's Taffy and IPC patches and Paredros's Parley patch with
Genet. Tabletop, Mesocosm and Paredros all pass
`cargo check --workspace --all-features --all-targets --locked -j 2` after these
patches. Checks use the published Git graph without tabletop local overrides.

The separate `shared/wing-integration` lockfile selects the same Mere revision;
all four live cross-product integration tests pass. Mesocosm's camera comparison
example now handles the four terrarium directions added by the camera work.

Run `pwsh -File scripts/audit-source-identity.ps1 -AllFeatures -FailOnDuplicate
-FailOnMismatch` for the locked three-product source gate. Saved metadata may be
provided explicitly for a repeatable audit without resolving again. The audit
validates each workspace root and resolved graph before traversing reachable
packages; it preserves case-distinct JSON feature keys such as `USB` and `usb`.
`pwsh -File scripts/tests/audit-source-identity.ps1` passes eleven regression
cases, including case-distinct features, reachable versus unreachable duplicate
sources, dynamically discovered platform packages, and incomplete or mismatched
metadata. PowerShell 7 is required. The [verification record](../testing/platform-alignment/verification.json)
collects exact pins, metadata hashes, check results and GPU provenance.
