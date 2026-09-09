# Games wing repository consolidation

**Status: completed locally, 2026-09-09.** Isometry is the repository
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
