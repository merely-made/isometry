# Views Founding: Adapter-First UI, and the Minimap as First Chrome

**Date:** 2026-08-02
**Status:** First slice landed (adapter + leaf, tested). Host embedding open.
Amended 2026-08-29 (§6): the textless guard was lane discipline, not a text ban.
Route B's own subject — the between-epochs screen — landed 2026-09-02 (§7).
**Companions:** mere's projection proofs plan (arrangement register; P4),
isometry's `2026-08-02_overmap_presentation_plan.md` (the second Hulls
consumer), the epoch boundary plan (places, §12).

---

## 1. The rulings (Mark, 2026-08-02)

- **Adapter-first.** `mesocosm-views` discloses world facts as a
  `sceno::Score`, `scenomise` solves placement, leaves realize the solved
  scene. A layout computed in the views crate is in the wrong crate. This
  adopts the posture isometry had to migrate to (P4 deleted its hand-rolled
  layout); mesocosm never incurs the debt.
- **Hulls first** among the reserved arrangements (Mosaic, Atlas, Hulls,
  Armature): the most versatile, with inference, metadata, and scripting
  applicable via fields. Landed in mere as `Arrangement::Hulls` plus
  `numen::FieldExtent::Polygon` (both 2026-08-02).
- **First chrome is the minimap**: which lineage dominates each region.
- **Region meaning is per-vessel.** The contract carries geometry and source
  refs; this vessel decides a cell's tint is the dominant lineage.
- **Backdrop is a DOM layer, dynamically generated** — not static, not merely
  dynamic. Mesocosm's natural backdrop is the world rendering its own
  enclosure top-down (the capture path already renders offscreen); painted-in
  effects stay available via sprigging's vello scene without committing now.
- **Dependencies**: this work is mesocosm's first coupling to genet
  (sprigging) and mere (sceno, scenomise) — git deps, branch-tracked, the
  woodshed pattern.

## 2. What landed

`crates/mesocosm-views`, two modules under the ceiling:

- **`minimap.rs`** — the adapter. `minimap_score` discloses places as
  coordinate-placed sites in a Hulls score (units are voxels, untransformed);
  `dominant_lineages` reads biomass of the living per region at projection
  time (derived, never stored — the same discipline as capability and
  temperament); `lineage_tint` gives every lineage a deterministic
  golden-angle colour.
- **`leaf.rs`** — `MinimapLeaf`, a sprigging `Leaf`: region cells filled
  translucent and stroked in the holder's tint, site dots, a player marker.
  Paints what it is handed, computes nothing, testable without a GPU.

**The congruence test is the load-bearing one**: `Places::at` and the Hulls
solver are the same nearest-site rule, so a sampled position lands in the
scene cell of exactly the place the simulation says it is in. The minimap
draws the world's own regions, not a cartographer's approximation.

## 3. The hosts are staged (Mark, 2026-08-02)

Two host routes, ruled complementary rather than competing, with the game's
own structure picking the boundary:

- **Route A, landed same day: the HUD lane.** During an epoch you are a
  critter; the chrome that belongs there is a textless HUD. The leaf paints
  `PaintCmd`s, `paint_list_render` lowers them, netrender's vello path
  rasterizes into a transparent texture on the game's own device (the
  arrangement the workspace's wgpu-29 pin exists for), and an overlay pass
  blends it over the frame. Vello overwrites its whole target, so the
  texture-then-composite indirection is forced, not chosen.
- **Route B, deferred to its consumer: the cambium lane.** Between epochs is
  where text lives: readings, records, naming, adopt-or-branch. That screen
  is panels and fields, the catalog's home turf, and the honest trigger for
  standing up the full genet host (isometry-genet's shape) with the game
  view embedded as an external texture.
- **The guard**: route A never grows a text hack. The moment chrome wants a
  word, that is the consumer pull for route B, not a reason to teach the
  HUD lettering.

### What route A shipped

`mesocosm-render::overlay` (the premultiplied blend pass),
`Renderer::render_scene_with` (a caller pass between scene and readback, so
captures show what windows show), and `mesocosm-genet::hud` (netrender on
the shared device, refresh gated by the leaf's own retention). Verified at
`testing/mesocosm/14_minimap.png`: nine cells tinted by dominant lineage,
site dots, the player marker in its cell.

### The stack bug this shook out

`paint_list_render` copied unpremultiplied `ColorF` into netrender's
premultiplied scene fields, whose consumers divide by alpha on the way to a
brush. Every translucent fill over-brightened toward white; opaque content
passed untouched, which is why genet's mostly opaque DOM never surfaced it.
The minimap's 35%-alpha cells rendered as a white sheet. Fixed at the owning
layer (netrender, with a regression test); `hud_raster.rs` keeps a headless
probe of the exact leaf-to-texture path with a premultiply bound.

### Layer vocabulary, checked against the stack (2026-08-02)

The words were spent before they were checked, so here they are, grounded:

| Word | Meaning in the stack | Owner |
| ---- | -------------------- | ----- |
| **underlay** | the painted scene content beneath host chrome | mere `canvas::underlay` |
| **backdrop** | a layer behind the actors; **may be interactive** | family word, ruled 2026-08-03 |
| **ambient background** | the non-interactive backdrop subtype: pure context | mere `canvas::ambient` |
| **overlay** | anchored floating UI above content | cambium `overlay_at` / `OverlaySurface` |
| **composite** | the act of blending layers into a frame | netrender `Compositor`, `paint_list_render::composite` |

**Backdrop names where a layer sits, not whether it acts** (Mark,
2026-08-03). A backdrop may carry props with hulls and fields with physical
implications; the non-interactive kind is the *ambient* subtype, which is
what mere's Game of Life tier holds and what the minimap's ground is. The
earlier table conflated the family with its passive subtype; mere's
`ambient` module docs now carry the distinction at the source.

`mesocosm-render::Overlay` was separately wrong: cambium's overlay is
anchored floating UI, a thing rather than an act, and the operation's
existing name is **composite**. Renamed.

The split this taxonomy makes clean: an **interactive backdrop projects
world truth** (props, hulls, fields the simulation reasons about) and
enters through a scene lane; an ambient background is pure presentation and
never touches the world. The inhabited-environment lane is therefore "build
the world truth, then back-drop it", not "enrich the ambient sim". It needs
no new primitive, since the hulls lane already rules that a hull composes
**group + region + optional field**; a prop is a region with a field. What
the *assembly* is called still awaits a naming round.

### The backdrop, landed 2026-08-02

The world renders its own enclosure top-down (an overhead orthographic
camera aligned with the minimap's mapping: world +x right, +z down), on a
step cadence rather than per frame, into the Hud's own small renderer.
Composited under the cells, whose 35% translucency existed for exactly
this. Verified at `testing/mesocosm/15_backdrop.png`: terrain under
territory, the dense centre reading light because the world's centre is
dense.

Two colour-space facts the capture chain forced into the open:

- **Vello's target must be plain `Rgba8Unorm`** (it writes through a
  storage binding, which sRGB formats cannot be), and its output is
  display-encoded. Sampling those bytes as linear and writing to an sRGB
  target encodes twice and brightens everything, so each raster is
  byte-copied into a copy-compatible sRGB twin whose decode-on-sample
  cancels the target's encode. The probe test walks the backdrop chain and
  pins the ground colour through the overlay.
- The residual: vello premultiplies in display space, so alpha compositing
  of its output is colourimetrically approximate. Deterministic and fine
  for a HUD; noted so nobody chases it as a bug later.

## 4. Not yet built

- **Interactivity.** Cells as hit targets wait for the route B DOM host.
- **Fields over cells.** `FieldExtent::Polygon` exists; nothing evaluates a
  field over a place yet. Consumer-pull holds.
- **Dominance-shift receipt.** Two captures of one long run showing
  different holders, once a scenario reaches far enough to flip a region.

## 5. Done conditions

- ~~The minimap visible in the windowed host, over a generated backdrop~~
  **done 2026-08-02**: `15_backdrop.png`.
- Dominance shifts on screen when the ecology shifts (verifiable with a
  scenario run: two captures, different holders).
- ~~A `testing/mesocosm/` capture showing it~~ **done**: `14_minimap.png`.

## 6. Amendment: the guard was lane discipline (2026-08-29, Mark)

The route A guard — never teach the HUD lane lettering — was about
which lane renders text, not whether the game has text. With route B
deferred on consumer pull and no consumer ever scheduled, it operated
in practice as a text ban: the first real playtest (played slice plan,
2026-08-28 findings) ran with energy, refusals, and death all
unsurfaced, partly because the only live lane had no words.

Ruled 2026-08-29: the cambium lane is ordinary game GUI for the whole
wing — title menus, status bars, floating panels, text — per the
shared-chrome ruling in the vessel briefs and presentation record §2.
Route A stays what it is: painted in-scene marks, one realization
option, not a text firewall. The guard survives in its true form —
text still never enters the painted lane as a hack; it enters through
cambium, which no longer waits for a consumer pull to exist here.

**Landed the same day.** `mesocosm-views::vitals` is the first cambium
chrome: energy, refusals and death in words over the traced section
(played slice plan, 2026-08-29 progress). The "textless by decree"
comment in `mesocosm-genet/src/hud.rs` is corrected accordingly. The
cambium lane arrived **without** the host inversion route B named: the
runner's `ScriptedDom` is styled and laid out by Livery, and its paint
list rasterizes into a texture composited over the game's own frame,
the same path the minimap takes. Genet owning the window with the game
view embedded stays available and unspent, and is still a scope
decision rather than a technical requirement.

## 7. Progress

- **2026-09-02: route B's own subject finally arrived** (playable ecology plan
  PE3b). §3 named between-epochs as where text lives — *readings, records,
  adopt-or-branch* — and deferred the lane to its consumer. The consumer is the
  lineage review: `mesocosm-views::review` draws the reckoning, a candidate
  table and its three answers as the fourth cambium surface, and
  `mesocosm-genet::review` is its lane. **The host inversion is still unspent**,
  exactly as the 2026-08-29 amendment left it: the board rasterizes into a
  texture composited over the game's own frame, the same path the vitals panel
  and the minimap take, and genet owning the window remains a scope decision
  rather than a technical requirement.

