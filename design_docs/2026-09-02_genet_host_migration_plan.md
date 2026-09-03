# Genet host migration

**Status:** M0 through M4 landed and committed (2026-09-03). §6 closed for genet and isometry; mere's pin is blocked on another session's migration and woodshed's bump stays deferred, both recorded in Progress. Founded from the 2026-09-02 wing assessment. First in the audit order ahead of protocol H2, because nothing headed could be receipted until the desktop built.

**Related:** the [runtime profile plan](2026-08-23_runtime_profile_plan.md)
(its R2 desktop gate waits on this), the
[perf and cambification plan](2026-07-20_perf_and_cambification_plan.md)
(its one live item, the search and whisper text lanes, closes here), and
genet's `docs/2026-08-09_cambium_desktop_host_g1_receipt.md` (the host this
plan moves onto).

## 1. The break

From a clean checkout the desktop host does not resolve:

```
error: patch location `https://github.com/merely-made/genet.git?branch=main`
does not contain packages matching `stylo_taffy`
```

Genet retired Stylo and the incumbent layout cone on 2026-08-21
(`55c05d11`, "Retire Stylo and the incumbent layout cone"). At every revision
the family now pins, `components/genet-layout` does not exist. `isometry-genet`
reaches around the shared host to `genet-layout`, `genet-winit-host` and
`netrender` directly and assembles its own `ApplicationHandler` (host.rs,
render.rs, input.rs, about 940 lines plus the `App` struct). None of that can be
repaired by manifest edits: the crate it targets is gone.

Woodshed donated that assembly to genet as `cambium-genet-winit-host` (genet
G1, 2026-08-09) and is its first consumer. Turnstone followed. Isometry's own
manifest comment names woodshed as "the maintained model" and never followed
it. This plan does.

## 2. The boundary, stated once

The host owns window lifecycle, layout, paint, input routing, pointer capture,
wheel, IME, AccessKit sync, frame capture and idle policy. Isometry owns state,
views, the six hooks, the net bridge, self-tests, and the overmap leaf. No
isometry code names `genet-layout`, `genet-winit-host`, `netrender`,
`paint_list_*`, `wgpu` or `winit` after this lands, except through the types
the host re-exports.

Gestures that the host used to hit-test on isometry's behalf move into the
views as `on_pointer`, `on_hover` and `on_wheel` handlers on the board
container. The view already computes the tile under a cursor from coordinates
and pan (`tile_at_cursor`, `hover_needs_update`, `token_drag_candidate`), and
the overmap already drives its canvas drag through `PointerPhase`, so this is
the pattern the views know.

## 3. Decisions recorded 2026-09-02

- **Pin.** Isometry takes every genet crate at the revision mere main pins
  (`eff0cb6` today; `da8762f`, woodshed's pin, is its ancestor and the cambium
  delta between them is a catalog example and frisket). Read from Mark's
  "pin both of them to mere, main" as: isometry and woodshed both follow
  mere's genet reference, which is the rule woodshed's own manifest states.
  Woodshed's bump is deferred: its manifest and lock are mid-edit by another
  session (2026-09-02), so it is not this plan's to touch.
- **Right-click.** The shared host's secondary press serves only the window
  frame (system menu on a drag region); it never reaches a view. The token
  context menu therefore needs a small genet change: route a secondary press
  into the same `on_pointer` capture path as a primary press, distinguishable
  by the event. Genet is a second repo; the change is scoped to
  `cambium-rootstock`, `cambium` and the winit host, and is not committed by
  this plan (see §6).
- **Text lanes.** The whisper composer and the compendium search adopt
  `caret_text_field` in this pass, closing the perf plan's live obviation item.
  The `>` command line already did (2026-07-27). Escape and Enter stay app
  commands through `key_intercept`.

## 4. Gates

**M0 — Manifests resolve from a clean checkout.** Root `Cargo.toml`: genet
crates at mere main's rev; `cambium-genet-winit-host` added; `genet-layout`,
`genet-winit-host`, `netrender`, `paint_list_*`, `wgpu`, `winit` dropped from
`isometry-genet`; `stylo_taffy` patch deleted; `taffy` patch becomes
`package = "genet-taffy"` at `=0.14.0`; `parley` patch added; the patch
comments rewritten to say what is true now. The gitignored
`.cargo/config.toml` loses its dead `taffy` path and gains the
`cambium-genet-winit-host` and `cambium-rootstock` redirects.
**Done when:** `cargo check -p isometry-genet` run from *outside* the repo
(so the local override is skipped) resolves and compiles, and the same check
inside the repo passes against the local checkouts. Zero "patch was not used"
lines in either run.

**M1 — Host rewrite.** `main` becomes `run(options, init, hooks)`. `init`
builds the map, campaign, system, generator catalog and stylesheet as today
and returns `Init { state, logic, sheet }`. Hooks: `frame` (viewport size from
`ctx.logical_size`, animation tick, overmap leaf sync into `ctx.leaves`);
`after_wake` (drain `NetBridge`, whose armillary waker becomes the host's
`HostWake`); `after_dispatch` (source-time attach, storylet rows, sheet
recompute, the things `App::after_dispatch` does now); `after_frame`
(self-tests; capture via `ctx.capture` for `ISOMETRY_CAPTURE_DIR`);
`close_request` (`Exit`); `focused_text` (the active text lane);
`key_intercept` (Escape policy, single-letter verbs, arrows, undo/redo,
Enter). host.rs, render.rs and input.rs are deleted, not shimmed.
**Done when:** `cargo test --workspace --all-features` and clippy with
warnings denied are green; every `ISOMETRY_*_SELFTEST` prints what it printed
before; `ISOMETRY_CAPTURE_DIR` writes a frame.

**M2 — Gestures in the views.** Drag painting, token drag, path-preview
hover, wheel pan and the context menu ride `on_pointer`, `on_hover`,
`on_wheel` on the board container, plus the new secondary-press path for the
menu. The panel-strip exclusion (a drag never spams panel buttons) survives
by construction because the handlers sit on the board, not the window.
**Done when:** a headed run paints a tile run by dragging, drags a token to
a new tile, previews a path on hover, pans on wheel, and opens a token's
menu on right-click; each is exercised by a `Harness` test where the harness
can express it, and by the capture where it cannot.

**M3 — Text lanes on `caret_text_field`.** Whisper composer and compendium
search become `TextInput`s rendered by `caret_text_field` with
`request_focus` on open; `focused_text` reports whichever is active; the
`search_field` widget is deleted. The perf plan's obviation item is ticked.
**Done when:** typing into either lane goes through the field, Escape and
Enter still cancel and submit, and the host's caret and IME work in both.

**M4 — Genet: secondary press reaches views.** In genet: `press_right`
dispatches into the `on_pointer` capture path with the button recorded on
the event, the harness gains a `right_click` that exercises it, and a
cambium-side test proves a view receives it. The window-frame behavior
(system menu on a drag region) is unchanged where it applies.
**Done when:** genet's cambium tests and the host harness tests are green and
isometry's M2 menu test passes against the local checkout.

## 5. Stop rules

- No shim keeps the old host alive beside the new one (DOC_POLICY §3).
- Isometry gains no dependency the shared host does not already carry.
- The migration preserves interaction semantics except where §3 says
  otherwise; a behavior change outside the text lanes stops and is recorded.
- Nothing here touches protocol, replication or the runtime profile crate.

## 6. What this plan cannot close alone

The committed isometry manifest pins a genet revision; M4 lands in a genet
working tree with another session's uncommitted work beside it. Until genet
commits M4 and mere and isometry bump to that revision, a clean checkout of
isometry builds M0 through M3 but not the right-click path. The local path
override hides that split on this machine, which is exactly the hazard the
manifest comments warn about. The commits and the two pin bumps are Mark's
call and are recorded here as the closing step, not performed by this plan.

## Findings

- **2026-09-02.** `components/genet-layout` is absent at `da8762f` and at
  `eff0cb6`; a directory of that name at genet HEAD is empty. The
  compatibility cone is gone, not renamed.
- **2026-09-02.** The shared host's `HostOptions` already defaults
  `NetrenderOptions` to a 1024 tile cache with vello enabled, which is what
  `SurfaceHost::boot` was told by hand in host.rs. Nothing to carry.
- **2026-09-02.** `AppCtx` exposes `leaves`, `wake`, `capture`, `pointer`,
  `logical_size`, `window`, `frame_profile` and `window_commands`. It does not
  expose the cursor; a hook that needs the pointer position gets it from the
  view-side handlers, which is where the gestures move anyway.
- **2026-09-02.** No mere crate isometry consumes names cambium or genet, so
  the two-copies hazard woodshed documents (mere's persona picker naming
  cambium beside a branch-tracking consumer) does not apply here. The pin
  choice is family consistency, not correctness.
- **2026-09-03, Z5: what the fit produced on this display.** With
  `fit_design: Some((1100, 820))` the host opens its clamped 1100x752
  device-logical window, fits at `min(1100/1100, 752/820)` = **0.9171**, and
  hands the application `logical_size` **1199.5x820.0** — the height lands on
  the design, the width keeps its slack, which is the genet plan's own Z1
  arithmetic note seen from the consumer's side. Printed by the whisper
  self-test in a headed run; the physical capture is unchanged at 2200x1504.
  Every pane and pointer number isometry reads is already in that space, so
  `hooks::sync_viewport`, the gestures and the harness needed no arithmetic;
  `hooks::init` did, because it seeds the viewport from `HostWindow` before the
  first frame and the window's scale factor is the *device* scale. It calls the
  host's public `fit_zoom` on the same design rather than dividing by the scale
  alone.
- **2026-09-03, Z5: the design figure is too small for the panel, open.** The
  fit works and 820 is the wrong number. Measured in the harness on the demo
  skirmish: the side panel's last row (the key hint under the status line) ends
  at **1024** logical px and `.side` adds its own 14px bottom padding, so the
  column needs **1038** where `DESIGN_SIZE` names 820. At 820 the fit therefore
  buys 68 logical px of panel and the composer, the status line and the hint are
  still below the fold; the headed capture confirms it, with the composer's own
  green (`#9fd48a`) absent from the frame entirely. A one-off run at
  `DESIGN_SIZE = (1100, 1040)` — zoom 0.7231, `logical_size` 1521.3x1040.0 —
  puts that green at device rows 1378-1464, inside the 1504-tall frame
  (`Code/testing/isometry/images/2026-09-03_isometry_whisper_z5_design1040.png`,
  evidence only; the committed constant is 820 as the host-zoom plan's Z5
  states). Raising it trades a smaller interface for the whole panel, so it is
  **Mark's call**, not this pass's. `host_zoom.rs` carries
  `the_side_panel_is_still_taller_than_the_design`, which fails the day the
  panel fits and says so.
- **2026-09-03, Z5: the board's pixel grid rounds the elevation step, not the
  tile.** `device = BOARD_UNIT * device_scale * ui_zoom`, rounded to the
  nearest whole pixel and divided back, where `BOARD_UNIT` is the projection's
  finest unit — the 8px elevation step, half a tile's height and a quarter of
  its width. Rounding *that* lands all three on whole device pixels; rounding
  the 32px tile width instead leaves the height on a half pixel whenever the
  width comes out odd, and an elevation column would seam where a tile did not.
  On this laptop (device scale 2, zoom 0.9171) the factor is **1.02227**: a tile
  is 60x30 device pixels instead of 58.69x29.35, and the elevation step 15
  instead of 14.67. At zoom 1 the product is already whole on every ordinary
  device scale (1, 1.25, 1.5, 2, 3), so `board_scale` is exactly `1.0` and the
  setting is inert — which is why the existing routing receipts are untouched.
- **2026-09-03, Z5: `AppCtx` carries no laid-out geometry.** The host exposes
  `logical_size`, `ui_zoom`, `zoom_changed`, `window`, `leaves`, `capture`,
  `pointer` and the rest, but nothing that answers "where did this node paint" —
  `painted_rect` lives on `Host`/`Harness` and `LayoutDom` has no geometry at
  all. So a headed self-test can print the layout size (it does) but not an
  element's painted edge; the panel's bottom edges are measured in the harness
  at the same surface instead, which is the same layout because layout runs in
  CSS pixels and the device scale is not in it. Recorded rather than fixed: the
  seam is genet's.
- **2026-09-03, Z5 residue: `theme::force_css` bakes an unscaled tile step.**
  The stagger and shove keyframes are generated from
  `IsoGeometry::default()` at build time, so under a board scale the beat slides
  by up to ~3.5% less than a tile. It is a transient animation offset on a
  400-1600ms beat, not geometry anything is measured against, so it is left
  alone; scaling it would mean regenerating the sheet on every zoom change.

## Progress

- **2026-09-02.** Plan founded after the wing assessment; decisions in §3
  taken with Mark. M0 through M4 open.
- **2026-09-02, M4 landed in the genet working tree (uncommitted).**
  `cambium::PointerButton { Primary, Secondary }` is a public field on
  `PointerEvent`; `PointerEvent::new` still means Primary and
  `with_button` marks Secondary, so no existing call site changed. A
  secondary press is one-shot: `dispatch_pointer_down` routes one `Down` to
  the nearest `on_pointer` ancestor and never captures, so no `Move` or `Up`
  follows and a right press during a left drag does not steal it. It never
  dispatches a click. `Harness` gained `right_click_at` and `right_click_on`.
  Deviation from §3 as written, kept because it is the module's own rule
  (view first, host default second): the DOM sees a right press on a drag
  region too, and the system menu runs afterwards unless the view prevented
  the default. Eight new tests; 254 tests green across the three crates.
  Clippy with warnings denied fails on a pre-existing baseline in files M4
  did not touch; the touched files add no warning. `HostPointer` has no
  secondary variant, so a hook cannot self-drive a right press; the menu is
  receipted through the harness, which M2 allows.
- **2026-09-03, M0 through M2 landed in the working tree (uncommitted).**
  host.rs, render.rs and input.rs are deleted; `hooks.rs` (init plus the
  seven hooks) and `host_routing.rs` (five `Harness` receipts: tile click,
  token right-click menu, wheel pan, Escape cancels a pick, layout geometry)
  replace them. Gestures ride `on_pointer` and `on_wheel` on `.pane` in
  pane-local coordinates; path-preview hover rides a per-element wrapper
  because the host routes hover only on hit-element change. Receipt A from
  outside the repo resolves and, after M2, fails only on `PointerButton`,
  as §6 predicts. `cargo test --workspace`: 293 passed. Verified by the
  reviewer: the five routing tests pass in a fresh run.
  **Not met:** the headed receipt. All three self-test captures are fully
  transparent frames. Layout and hit-testing are correct in the harness;
  with `.pane { overflow: hidden }` removed the app paints completely, and
  with it present the whole window, side panel included, paints nothing.
  Woodshed and turnstone use `overflow-y: auto` and `overflow: scroll` on
  flex children and never `overflow: hidden` on a container of absolutely
  positioned children, so isometry is the first consumer to hit this. The
  overmap self-test also overflows the main thread's stack in the leaf
  paint path while laying out fine in the harness. Both sit below the
  boundary §2 draws; neither is fixed here.
  **Manifest residue:** the old `stylo_taffy` failure was masking a
  p2panda resolution failure (mere-transport now wants
  `mere-p2panda-net =0.7.2`); the lock was refreshed, which pulled sceno
  0.0.4 and a newer fork rev, and `--all-features` now fails with six
  errors in isometry-net's campaign lanes against the fork's API. Isometry
  does not mirror mere's `tag = "mere-p2panda-net-0.7.2"` block.
  **Behavior changes recorded:** a side-panel press no longer dismisses an
  open token menu; a paint drag begun over an overlay paints beneath it;
  beat clearing is a 750 ms wall clock (`AppCtx` exposes no animation
  state); trackpad pixel deltas arrive unscaled from the host; drag
  painting dedupes per tile rather than per element; the net bridge
  drains on wake instead of a 10 Hz poll; wheel over the panel now scrolls.
- **2026-09-03, the blank frame was genet's and is fixed in the genet
  working tree (uncommitted).** `genet-livery` `emit_stacking_item`
  re-pushed a flattened item's whole ancestor clip stack around each item,
  so 692 absolutely positioned tiles inside one `overflow: hidden` pane
  emitted 692 clip pairs per frame, each a vello compositing layer, and the
  scene came back alpha zero everywhere. It now holds one clip scope across
  a run of adjacent items with matching ancestor stacks (9 pairs for the
  same frame); regression test asserts scope count is independent of item
  count. Combat and overmap captures are now real frames (pixel variance
  about 1000, alpha 255). Recorded in genet's host receipt doc.
  **The overmap abort is a stack budget, not recursion:** livery's layout
  needs about 450 KiB plus 127 KiB per DOM nesting level in a debug build
  and a Windows MSVC main thread has 1 MiB; plain nested divs reproduce it
  and release builds clear it. Latent for every genet consumer; the repair
  (stacker in the layout recursion, an off-main-thread host, or a
  documented `/STACK` link flag) is a genet policy decision, open.
  **Isometry-side:** board tiles carry inline z-index up to 2945 while
  `.overmap` sits at 503, so tiles paint over the overmap panel; genet's
  order is CSS-correct and `.overmap` must be raised in M3. The host passes
  trackpad pixel deltas unscaled where cursor moves are scaled; a one-line
  genet fix that changes scroll feel for woodshed and turnstone, open.
- **2026-09-03, decisions taken with Mark:** the stack budget is repaired
  with `stacker` inside livery's layout recursion (every consumer covered,
  no link flags); the trackpad wheel scaling is fixed in the shared host.
  Both are genet work under this plan's authorization, uncommitted. Noted
  in passing: another session's relicense commit in genet (`957926e`,
  2026-09-03) swept in M4's one-line `PointerButton` export in
  `cambium/src/lib.rs`, so that line is committed while the rest of M4 is
  not; harmless, but the closing genet commit must carry the remainder.
  M3 also absorbs the overlay z-index fix: the combat capture shows the
  Knight sheet overpainted by tiles on its right edge, so every overlay
  panel (sheet, overmap, compendium, and the rest) must sit above the
  tile range, not only `.overmap`.
- **2026-09-03, both genet fixes landed in the genet working tree
  (uncommitted).** The stack budget turned out to be five per-level
  descents across two crates, not one: cascade and box-tree collection and
  the algorithm-tree projection in genet-livery, box-tree normalize and
  materialize and the taffy `compute_node` callback in buckram. Seven guard
  sites behind two helpers (`genet_livery::with_recursion_stack`,
  `buckram::box_tree::with_box_tree_stack`), `stacker::maybe_grow` with a
  256 KiB red zone and 2 MiB growth (stacker's Windows backend commits,
  not reserves, so larger numbers bought nothing). Debug ceiling on a
  256 KiB thread went from 8 nested levels to about 192; the residue is
  inside published genet-taffy's own recursion. Regression test
  `deep_nesting.rs` overflows without the guards (negative control run).
  wasm32 builds with stacker in the graph. The overmap self-test now
  reaches its capture in a plain debug build (alpha 255 everywhere, luma
  variance 1559). Trackpad pixel deltas are now divided by the scale
  factor at the host's wheel arm through a new
  `wheel_delta_from_winit_logical`; line notches unchanged; harness test
  asserts 60 physical px arrives as 30 at 2x. Two things for Mark's eye:
  buckram is `publish = true` and now depends on stacker and psm (a build
  script and a small C/asm component), and genet's root Cargo.toml gained
  one `stacker` workspace-dependency line beside another session's edits.
- **2026-09-03, the p2panda residue is closed.** The manifest mirrors
  mere: the family at `tag = "mere-p2panda-net-0.7.2"`, the net crate as
  the published `mere-p2panda-net =0.7.2` with mere-transport's exact
  feature set, no `p2panda-net` or `p2panda-stream` patch (neither is in
  the graph; verified with `cargo tree`). One copy of every family crate.
  The six campaign-lane errors became two manifest-only fixes (one
  `Endpoint` type once the net crate matched) and the `Header::builder`
  idiom gemot uses (`Body::from_bytes`, builder signs at `build`). One
  test moved its tamper from a header extension to the payload, because
  in this fork the in-memory extensions are a decoded view of the signed
  bytes and `validate_operation`'s signature check is compile-time true
  outside `test_utils`; authenticity lives at `AnyHeader::decode`. That
  finding is recorded in the stickleback migration plan for K0.
  `cargo check --workspace --all-features --all-targets` green; tests 302
  under all features against 293 default; zero unused-patch lines; from
  outside the repo `isometry-net --all-features` compiles clean.
- **2026-09-03, M3 and the three follow-ons landed in the working tree
  (uncommitted).** Whisper composer and compendium search are `TextInput`s
  under `caret_text_field` (in-flow caret, matching the command line);
  `focused_text` and `key_intercept` share one `focused_lane` helper that
  asks the caret rather than three state flags; the per-character
  `compose_*` and `search_*` paths and `widgets::search_field` are gone;
  the perf plan's obviation item is ticked. The token menu rides
  `overlay_surface` as a third child of `.app` (a window-sized dismissal
  layer cannot live inside the `overflow: hidden` pane), so a press
  anywhere outside it, or Escape, closes it, and the anchor is unchanged
  to the pixel. Overlays stop drags reaching the board through a no-op
  `on_pointer` on the shared `overlay_panel` root; `stop_propagation`
  would have done nothing since the host routes to the nearest handler
  only. Every overlay sits at `theme::OVERLAY_Z` (1,000,000) above the
  tile range; `.storylet` and `.downtime` had no rule at all and now share
  the panel chrome. Tests 306 under all features; four new harness
  receipts with negative controls; five self-driven captures, all real
  frames. Behavior changes beyond §3: a press while the menu is open only
  dismisses it; Escape closes it; the filter no longer resets the grid
  scroll; Ctrl-Z/Y in a focused field undo the field. Unverified headed:
  no self-test opens the compendium or the composer, so the two fields'
  appearance rests on the harness's non-zero-box assertion alone.
- **2026-09-03, headed coverage closed for the text lanes.**
  `ISOMETRY_COMPENDIUM_SELFTEST` and `ISOMETRY_WHISPER_SELFTEST` drive the
  two new fields through the host's own key seam (`key_intercept`, then
  `dispatch_key`, so the field makes the `TextCommand`), and both captures
  are real frames. The compendium frame shows the field, caret, filter and
  clear affordance as intended. The whisper composer is below the fold:
  this laptop's display is 1280x800 logical and the shared host clamps
  the window to monitor height less 48, so 752 is the ceiling here
  whatever isometry asks for (the July receipts at 820 were taken on a
  different display). `initial_logical_size` was already 820. The
  composer sits after the Dice section in the side panel; whether it
  moves above it, or the host learns a `size_env` for receipts, is a
  product call outside this plan. Independent rerun: 306 tests, 0
  failures, under all features.
- **2026-09-03, Z5 of the host-zoom plan landed in the working tree
  (uncommitted).** Isometry consumes genet's new host zoom.
  `HostOptions::fit_design` is `DESIGN_SIZE` (1100x820), the one constant
  `initial_logical_size` is also built from, so the window it asks for and the
  design it is fitted to cannot drift. The board takes its own scale from a
  hook — `hooks::sync_board_scale` compares `(window.scale_factor(),
  ctx.ui_zoom)` against `UiState::pixel_grid` each frame and writes only on a
  change, which covers `zoom_changed` and a drag to a monitor of another
  density with one compare; `hooks::init` seeds the same pair through
  `fit_zoom` before the first frame. `UiState::set_pixel_grid` recomputes
  `board_scale` and rewrites `geo` from it, so emission and hit testing are one
  projection and cannot disagree — `a_click_still_lands_on_the_tile_under_it_at_
  a_fractional_zoom` is the receipt. The board's element boxes ride the same
  scale through `board::placed`, which appends `width`/`height` to the inline
  style **only** when the scale is not 1, so at zoom 1 the emitted DOM is
  character-for-character what it was; the token sprite additionally carries a
  scaled `translateX` because `.token-flip` pre-translates by its own width.
  `depth_key` z-indexes and `OVERLAY_Z` are untouched. The setting is
  `UiState::integer_pixel_rounding` (default on) with a keyboard-free `px grid`
  toggle in the panel's Map row: isometry has no settings surface and persists
  no application settings, so mere's configuration-ownership pattern has no
  counterpart here to hang it on, and that is recorded rather than invented.
  Seven new harness receipts in `host_zoom.rs` (the fit arithmetic; a fitted
  short window laying out identically to a full-height one, with the unfitted
  window as the negative control; whole device pixels on and fractional off at
  device scale 2; the zoom-1 identity across five device scales; the panel
  toggle through the shipping click path; hit testing under the scale; and the
  open design-height receipt). Tests 313 under all features against 306, 0
  failures; `cargo check --workspace --all-features --all-targets` green;
  clippy adds no warning on a line this pass wrote. Two headed self-driven
  captures in `Code/testing/isometry/images/`
  (`2026-09-03_isometry_whisper_z5.png`, `2026-09-03_isometry_combat_z5.png`),
  both 2200x1504 physical with alpha 255 everywhere. **Nothing changes at zoom
  1**: `board_scale` is exactly `1.0` there, `placed` emits nothing extra, and
  the five M0-M3 routing receipts pass unchanged. **Open, Mark's:** the design
  height, per the Findings above.
- **2026-09-03, §6's closing steps taken.** Genet committed as `86019ea`
  ("Scale the interface: UI zoom for the Cambium desktop host") and pushed;
  the push carried a peer session's 08:41 document-lane move, which was
  committed on main and only unpushed. Isometry's eleven genet references
  moved from `eff0cb6` to `86019ea` in one step, since cargo keys a git
  source by URL plus reference and two references land two copies of
  Cambium. The receipt §6 asked for is now green: from outside the repo,
  where the machine-local path override does not apply,
  `cargo check -p isometry-genet` **compiles** rather than merely resolving,
  in 9m16s, with zero "was not used" lines and one pre-existing dead-code
  warning. In-repo: 313 tests under all features, 0 failures.
  **Mere's pin cannot follow yet, and that is not this plan's to force.**
  Mere pins genet at `388d89c3` and names twelve genet crates; genet's
  2026-09-03 document-lane move retired four of them from the tree
  (`knot-editor-host`, `scrying-engine`, `graft-engine`, `weld-engine`), so
  bumping mere to `86019ea` would break its build on all four. That
  migration is another session's active work under mere's platform boundary
  plan, moving consumers first. The two references reconverge when those
  crates land in mere. Woodshed's bump stays deferred for the reason
  recorded above, and now also owes the `Accessibility::sync` signature
  change and the new trackpad wheel feel.
