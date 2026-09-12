# Orthographic voxel presentation plan

**Date:** 2026-09-11

**Status:** founded 2026-09-11 from Mark's rulings in the netrender execution
graph review. L0 closed 2026-09-12 with one named gap: netrender holds two
hundred thousand live rectangles through retained fragments, genet's first
frame is superlinear and usable only to about five thousand elements, and
live DOM mutation is unmeasured until a host-side harness exists. Hybrid is
the default. Rulings 10 and 11 (2026-09-12) make L5 body-level with
fragment-backed parts and found genet's L1 plan with the first-frame scaling
slice and the host-side mutation harness as named consumers.

**Owns:** how the three vessels present voxel worlds and bodies through the
web engine, and the unification of Isometry, Paredros, and Mesocosm on one
presentation lane of broader use to the stack.

**Does not own:** world truth (`Ground`, the body document, the core hash),
resident compute (the [resident views plan](2026-08-14_resident_views_composition_plan.md)),
Livery's CSS conformance program (genet's own plans), or netrender's
execution graph (`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`).

**Prior art:**

- [PolyCSS](https://github.com/layoutit/polycss), MIT: polygon meshes and
  `.vox` volumes as DOM elements under `matrix3d`.
- [Zdog](https://zzz.dog), MIT: flat-shaded pseudo-3D drawn as 2D vector
  paths under 3D transforms.
- [Bonsai](https://github.com/scallyw4g/bonsai), WTFPL: already adopted three
  ways on 2026-08-07 (landscape §8.3); its meshing and deferred shading no
  longer apply here.
- GBA-era tactics games (Tactics Ogre, FFTA): many animated bodies as baked
  sprites at a locked angle. Isometry's founding look.

## Rulings (Mark, 2026-09-11)

1. **Paredros is orthographic.** A true isometric game. This closes the camera
   question the [Paredros execution plan](../../paredros/design_docs/2026-08-07_paredros_execution_plan.md)
   left open until the spatial laws could judge it, and it retires the
   Barony/Delver close-perspective reference lane. Camera is not person:
   Paredros stays second person in agency.
2. **The wing presents through genet and netrender.** Bodies and props are
   DOM under CSS; ground is layered. Renderling exits Paredros once L7's
   receipt holds. The engine of broader use is genet plus netrender plus
   conatus, and the three games are DOM consumers of one appearance crate.
3. **Livery gains CSS 3D transforms.** Authorized as genet scope. Genet's own
   plan founds it; this doc names the consumer.
4. **One appearance crate.** `isometry-voxel` and `mesocosm-mesh` merge into a
   wing-neutral crate that emits a sprite sheet or a face list from the same
   volume.
5. **Ground is tile layers, provided underground stays possible.** One layer
   per height step, cutaway by hiding layers above the focus. Layering is also
   the surface-interaction seam.
6. **Live faces are the default to scope first.** Mark wants to feel
   live-all-the-time; the expected landing is a hybrid of live faces in focus
   and during rotation, baked sprites elsewhere. L0 decides which is honest.
7. **HTML is an interchange format.** A body or memorial is a document.
8. **Angles 1-4 are adopted** (§ Novel angles): GPU sprite bake on the resident
   lane, ground as generated tile layers, the hagiograph as a consumer, games
   as documents.
9. **The probe runs first.** No lane past L0 lands before its receipt.
10. **L5 is body-level (Mark, 2026-09-12, from the L0 verdict).** A body or
    part is one DOM element whose paint is a retained netrender fragment
    below it, spliced the way genet splices an iframe's paint list into a
    replaced box. Faces never become elements. This promotes L3's face
    batch from "not yet earned" to the mechanism L5 runs on.
11. **L1's genet plan carries two more consumers (Mark, 2026-09-12):** a
    first-frame style and layout scaling slice, linear and about two orders
    cheaper per element, and a host-side DOM mutation harness with real
    timing. Founded in genet as
    `genet/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md`.

## The picture

```text
Ground / body document / core hash          (authority, unchanged)
    -> appearance crate                     (volume -> sprite sheet | face list)
       -> DOM + CSS                          (genet: layout, classes, hit test, a11y)
          -> netrender Scene                 (Vello affine; retained fragments)
             -> netrender compositor         (external-texture boundaries between layers)
                -> one wgpu device            (netrender_device::WgpuHandles)
```

Orthographic is the load-bearing word. An orthographic voxel face is an
affine quad, so it lowers straight into Vello without a raster tenant. The
tracer's per-pixel depth join (engine review D1) is replaced by interleaving:
ground layers and body layers composite in painter order at netrender's
existing `scene_op_boundary`, which is exact for a fixed camera and quantized
heights.

What this buys the stack rather than the games alone: CSS Transforms Level 2
is a web standard genet lacks and has WPT coverage for, so the Livery work
pays in turnstone and Pelt on real pages and in mere's spatial graph views,
which already carry padded 3D positions. Renderling paid off in one Paredros
crate and two probes, at the cost of a local fork, rust-gpu pins, and a
crabslab fork.

## Authority boundaries

| Layer | Owns | Must not decide |
| --- | --- | --- |
| `Ground`, body document, core | voxel facts, revisions, part placements, the hash | how anything looks |
| appearance crate | volume to sprite or face list; palette; facing; greedy merge | world facts; DOM structure |
| DOM + CSS (genet) | element tree, classes, transforms, animation, hit testing, accessibility | painter lowering; device |
| netrender Scene / compositor | affine rasterization, retained fragments, layer interleave at boundaries | scene meaning; tenant internals |
| conatus / CubeCL | resident planes, GPU bakes into atlas textures | appearance policy |
| tracer (`conatus-brick`) | the optional live-volume lens for any camera that needs it | ground presentation by default |

The execution graph plan is not amended here. Its RG3 second consumer
becomes the tracer as an optional lens rather than renderling as a mesh
tenant, and D1's depth join stops being a promotion gate. That amendment is
Mark's, in netrender's notes, after L0.

## Lanes

Each lane names its owner tree and a done condition. L0 blocks everything.

### L0. The ceiling probes

Two untracked probes under `mere/crates/probes/` (the gitignored probe
workspace), receipts copied into `Code/testing/wing/`.

- **L0a, netrender face throughput.** Bodies as retained fragments of
  rectangles under CPU-side orthographic projection, bypassing Livery.
  Measure CPU scene build, Vello encode, and GPU frame time across a grid of
  body count and rectangles per body, static and with per-part transforms
  changing every frame.
- **L0b, genet element cost.** The same counts as absolutely positioned,
  transformed elements in a document through the Livery route. Measure style,
  layout, and paint per frame, static and with transforms mutated per frame
  where the scripted profile allows it.

**Done when:** both receipts exist with device, driver, and commit named; the
table states the largest static and live element count under a 16 ms frame;
and the doc's § Receipts records whether live faces, the hybrid, or bake-only
is the honest default.

### L1. Livery 3D transforms (genet scope)

CSS Transforms Level 2: `matrix3d`, `translate3d`, `rotate3d`, `scale3d`,
`perspective` and `perspective-origin`, `transform-style`,
`backface-visibility`, 3D `transform-origin`, and the individual `translate`,
`rotate`, `scale` properties. Lowering projects each element to netrender's
existing column-major 4x4 `Transform` and z-sorts within a 3D rendering
context. Netrender's `Transform` is ready; nothing in Livery parses these
today.

Founded in genet on 2026-09-12 as
`genet/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md`
with three lanes: T1 the transforms above; T2 first-frame style and layout
scaling, linear and about two orders cheaper per element, plus phase timing
behind a flag; T3 a host-side DOM mutation harness with real timing and the
fragment-backed replaced element L5 runs on. Ruling 11 names all three as
this plan's consumers. The L0b attribution of the superlinear first frame
lives at `Code/testing/wing/l0b_first_frame_attribution.md` and seeds T2.

**Done when:** genet's plan founds it with its WPT `css/css-transforms`
counts; a wing fixture body of rigid parts renders through Ortet with the
same silhouette the appearance crate's bake produces; individual transform
properties animate a part's yaw without touching the parent matrix; T2's
done condition holds on the L0b grid; T3's harness reports a live per-frame
number for a body whose parts move every frame.

### L2. One appearance crate

Merge `isometry-voxel` (recipe, palette, `bake_facing`, `Sheet`) and
`mesocosm-mesh` (body document, rigid parts, greedy `Quad`, `PartMesh`) into
one wing-neutral crate. Two projections from one volume: a sprite sheet at a
locked angle and facing, or a face list with per-face normal and palette
index. The body document stays the shared organ; nothing already on a body is
remeshed when a part attaches.

**Done when:** Isometry's tileset bake and Mesocosm's live body both build
from the merged crate with pixel-identical sheets and quad-identical meshes
to today's; a face list carries enough for L3 to emit DOM without touching
voxels again; the crate has no engine or wgpu dependency.

### L3. Netrender: face batches and layer interleave

- A part's faces as one retained fragment of N rectangles under one 3D
  transform, placed through `Scene::place_fragment`. L0a proved fragments
  carry per-instance transforms and that this path holds two hundred
  thousand live rectangles; ruling 10 makes it L5's mechanism. A dedicated
  face-batch op inside the fragment is still measured-only: L0a showed
  master compose and Vello render comparable at the largest cell and neither
  constraining below two hundred thousand rectangles.
- The replaced-element splice that lets a genet element carry a host-supplied
  fragment is genet's T3 lane; netrender's side is the fragment API it
  already has.
- Ground layers and body layers interleaved through the existing
  external-texture boundary contract, so a body between two height steps
  composites between their tiles.

**Done when:** a body between two ground layers occludes and is occluded
correctly in a headless readback; the retained fragment for an unchanged body
costs no re-encode; the execution graph's plan dump names each layer.

### L4. Ground as tile layers, underground included

Materialize `Ground` bricks into per-height tile sheets, one DOM layer per
height step, Dwarf Fortress cutaway style: layers above the focus height
hide, the focus layer draws with its surface, layers below draw dimmed. The
tracer remains the lens for the Mesocosm section and any camera that needs a
live volume.

Destructible and constructible ground rides the existing revision contract:
`Ground` bumps its revision and drains dirty bricks; only the tiles of the
dirty bricks on the affected height steps regenerate; retained fragments keep
every other tile. A carve, a burrow, or a placed block is the same path as
generation.

**Done when:** a carved burrow is visible through the cutaway with no
tracer in the frame; a radius-zero carve regenerates only its brick's tiles,
proved by fragment identity; the layer count and tile bytes per revision are
reported.

### L5. Live bodies as fragment-backed elements, the hybrid, and feel

Ruled body-level on 2026-09-12 (ruling 10). A body is one DOM element under
an L1 3D transform; each rigid part is a child element under its own
transform; each part's faces are a retained netrender fragment supplied by
the appearance crate and spliced into the paint list as that element's
replaced content. The DOM never sees a face. Animation is per-part
transform, so a moving body costs one fragment placement per part, which is
the path L0a measured at two hundred thousand live rectangles.

The hybrid stays the default until the genet first-frame slice lands: live
fragment-backed parts for bodies in focus and during rotation, baked sprites
for the rest, switched by class. Live-all-the-time is re-evaluated when
genet's T2 done condition holds.

Destructible and constructible bodies: a part's volume edit remeshes that
part alone and replaces that part's fragment; incorporation adds a part
element with its fragment. Nothing else on the body changes.

**Done when:** a headed run shows a body turning continuously with
fragment-backed parts; the same body swaps to a baked sprite and back by
class with no visible pop at the locked facings; a part edit replaces only
that part's fragment, proved by fragment identity; element count per body
equals part count plus one.

### L6. GPU sprite bake on the resident lane

A CubeCL kernel projects a resident volume into a sprite atlas texture that
netrender samples as a registered image. Zero CPU pixels: a runtime-attached
part is rebaked on the GPU and enters the DOM as a class change. This unifies
the engine review's R2 bake path with Isometry's lane. Downlevel keeps the CPU
bake.

**Done when:** a GPU-baked sheet matches the CPU bake within the coverage
tolerance the render tests already use; one device, one allocation, no
per-frame upload on an unchanged body.

### L7. Renderling exits Paredros

Paredros's room, D1, crossing, lighting, and residency paths move to the
appearance crate plus DOM. The `d1_depth` receipt is retired with rationale,
since the join it proved is replaced by L3 interleave. The renderling fork,
spirv-std pins, and crabslab fork leave the Paredros manifest.

**Done when:** `paredros-room` builds with no renderling dependency; the S0
replay hash is unchanged; the room composes through the same layers as
Mesocosm's section; the probes that still want renderling are archived, not
patched.

### L8. HTML as interchange

A body, prop, or memorial serializes as a self-contained HTML fragment: a
declarative shadow root carrying its own style and its face or sprite markup,
with the body document's identity as data attributes. Genet's Shadow DOM lane
landed 2026-09-07/08, and the parser/script interleaving plan closed
declarative attachment through the custom-element registry on 2026-09-08,
so nothing in genet blocks this lane.

**Done when:** a body round-trips document to HTML to document with the
same part placements and palette; Isometry loads a Mesocosm-authored body
from that fragment over P2P; the hagiograph emits a memorial as one such
fragment.

## CSS features and standards to earmark

Fast-track candidates for genet's
[standards-to-features ledger](../../../genet/design_docs/2026-09-07_standards_to_features_ledger.md),
which is the authority on where Livery stands. The status column here is a
file-mention count over the livery crates on 2026-09-11, a hint and not a
conformance claim; the ledger's census numbers win.

| Tier | Feature | Standard | Livery mentions | Unlocks here |
| --- | --- | --- | --- | --- |
| 0 | 3D transforms, `transform-style`, `backface-visibility`, individual transform properties | CSS Transforms 2 | 0 | L1, L5: every live face and part rotation |
| 0 | `image-rendering: pixelated` | CSS Images 3 | 1 | Isometry's nearest-neighbour lens for sprites and tiles |
| 0 | custom properties, `@property` | CSS Variables 1, Properties and Values API | 11, 1 | palette swap by variable; animatable typed yaw and tint |
| 0 | `color-mix()`, relative colour, `oklch` | CSS Color 4 and 5 | 10, 8 | per-face shading from one material colour, replacing `face_shade` in code |
| 1 | animations and transitions, `animation-composition` | CSS Animations 1, Transitions 1, Animations 2 | 2, 5 | part motion between poses; facing swaps |
| 1 | `clip-path`, `mask-image`, `mask-composite` | CSS Masking 1 | 9, 0 | cutaway of underground layers; damage masks on destructible parts |
| 1 | `content-visibility`, `contain` | CSS Containment 2 | 0 | off-screen layer culling for large maps |
| 1 | `will-change` | CSS Will Change 1 | 0 | the promotion hint for retained fragments |
| 1 | `mix-blend-mode`, `isolation`, `filter`, `backdrop-filter` | Compositing 1, Filter Effects 1 | 1, present | lighting tint, section haze; netrender already rasterizes filters |
| 2 | `offset-path`, `offset-distance` | Motion Path 1 | 0 | bodies moving along paths without script |
| 2 | anchor positioning | CSS Anchor Positioning 1 | 0 | labels and callouts pinned to bodies; Isometry's atlas labels |
| 2 | declarative Shadow DOM, custom elements | HTML, DOM | 6 | L8: a body as a self-contained element |
| 2 | `@scope`, nesting | CSS Cascading 6, Nesting 1 | 0 | per-body style scoping without class prefixes |
| 2 | View Transitions | CSS View Transitions 1 | 0 | epoch board and section transitions |
| 3 | `corner-shape`, `border-shape` | CSS Borders 4 | 0 | non-box shapes only; PolyCSS's triangle path; not needed for voxels |
| 3 | accessibility tree, ARIA | ledger row 3 | ledger | a screen-readable tabletop; the inspector as a test oracle |

Each row that opens a lane gets a ledger row in genet naming this doc as the
consumer. That edit is genet's, in the same session the lane opens.

## Prior art, what still transfers

| Source | Transfers | Does not |
| --- | --- | --- |
| PolyCSS | face culling, greedy rectangle merge, painter sort, baked per-normal shading, atlas slicing, snapshot as standalone HTML | `corner-shape` triangles; a JS dependency |
| Zdog | flat-shaded vector pseudo-3D under affine transforms, exactly the Vello lane | canvas/SVG runtime |
| Bonsai | relief lab loop, decoration tier, SDF layer brush editing, profiler-first culture | meshing, deferred shading, GPU-authoritative generation |
| Renderling | headless image-test pattern, slab allocation ideas for L6 | the runtime and its toolchain |
| GBA tactics games | baked sprites at a locked angle carry many animated bodies | free camera |
| Dwarf Fortress | z-level cutaway as the underground presentation | ASCII |
| Godot Voxel Tools, `block-mesh` | task pools, greedy meshing benchmarks for L2 | engine ownership |

## Novel angles, adopted

1. **GPU sprite bake on the resident lane** (L6). The resident chunk already
   feeds the tracer by GPU copy; the same allocation feeds a bake kernel whose
   output is a sprite atlas, not a mesh.
2. **Ground as generated tile layers** (L4). The version that reaches the
   browser with no GPU features at all.
3. **The hagiograph as a consumer** (L8). `CLAUDE.md` already assigns memorial
   events to the stack's procedural voxel engine; if that engine is genet plus
   the appearance crate, a memorial is a generated, styled, shareable document.
4. **Games as documents.** Save state stays in the core hash. Presentation is
   a DOM that cambium, accessibility, P2P peers, and the inspector already
   understand.

## Deliberate exclusions

- Perspective cameras in any vessel. Vello is affine; the tracer remains the
  only perspective-capable lens and is not load-bearing.
- A per-pixel depth buffer in the DOM. Interleave replaces it.
- Ground as DOM voxels. Ground is tiles per height step or the tracer.
- Per-face animation. Parts are rigid; animation is per part.
- A PolyCSS or Zdog dependency. Their cores are small enough to own.
- Any lane past L0 before the L0 receipt.
- Amending the execution graph plan from this doc.
- Editing genet's ledger or Livery plans from this doc.

## Open decisions for Mark

- Hybrid switch policy: which bodies count as "in focus" for live
  fragment-backed parts while genet's T2 is open.
- Whether the tracer keeps the Mesocosm section by default or the section
  also moves to tile layers once L4 lands.
- Whether L6's GPU bake targets the enhanced capability profile only, with
  CPU bake as the downlevel tier, or replaces the CPU bake outright.
- Where the merged appearance crate lives: the Isometry root workspace, or
  mere as a platform organ once a non-wing consumer appears.

## Receipts

Filled by lanes as they close. L0 first.

### L0a, netrender face throughput (2026-09-11)

Receipt: `Code/testing/wing/l0a_netrender_face_ceiling.{json,md}`; probe at
`mere/crates/probes/netrender-face-ceiling/` (untracked). RTX 4060 Laptop GPU,
Vulkan, driver 610.88, netrender `3961aca91`, 1920x1080 RGBA8, medians over
60 frames after 5 warmup. Three paths were measured: **static** (the same
flat-rectangle scene every frame through the tile cache), **live** (every
body's transform changes every frame, scene rebuilt, tile cache), and
**fragment** (each body a retained fragment placed through
`Scene::place_fragment` with a fresh 4x4 each frame).

| path | rectangles | frame ms |
| --- | ---: | ---: |
| static | 50,000 | 11.3 |
| static | 150,000 | 27.3 |
| live, flat | 5,000 | 6.5 |
| live, flat | 20,000 | 18.4 |
| live, flat | 50,000 | 50.9 |
| live, fragment | 50,000 | 6.8 |
| live, fragment | 200,000 | 10.0 |
| live, fragment | 600,000 | 36.2 |

Findings:

- **Retained fragments are the live path.** Under a 16 ms frame the flat live
  path holds roughly ten to fifteen thousand rectangles; the fragment path
  holds two hundred thousand measured and an interpolated ceiling near three
  hundred thousand. Fragment lowering stayed at one lower per body across
  every transform-changing frame, so a moving body costs a placement, not a
  re-encode. This is a twenty-fold lift and it decides L5: live faces are
  viable at Paredros scale provided every body is a fragment.
- **CPU path overhead dominates, not the GPU.** On the flat live path at the
  largest cell, dirty-tile rebuild is three quarters of netrender's total
  while Vello's render is a small fraction. The tile cache re-lowers every
  tile a moving body touches, which is why the flat path collapses.
- **The fragment path bypasses the tile cache**, so its two halves are master
  compose and Vello render, comparable at the largest cell. A face-batch
  scene op (L3) would attack the compose half; below two hundred thousand
  rectangles neither half is the constraint, so L3's batch op is not yet
  earned.
- **No API refusal.** Fragments already carry a per-instance transform.
- **Probe-workspace blocker, not fixed here:** `mere/.cargo/config.toml`
  patches the genet git source's `servo-paint` to a path that no longer
  exists, so any cargo run with a cwd inside mere fails at resolution. The
  probe was built with `--manifest-path` from outside mere. This will bite
  every probe under `mere/crates/probes` until the stale patch is removed.

L0a's half of the L0 done condition: static ceiling fifty thousand
rectangles through the tile path; live ceiling two hundred thousand through
fragments. The genet half (L0b) decides whether the DOM can supply that
many elements.

### L0b, genet element cost (2026-09-12)

Receipt: `Code/testing/wing/l0b_genet_element_ceiling.{json,md}`; fixtures,
generator, runner, and attribution variants under `Code/testing/wing/l0b/`.
Ryzen 9 7940HS, RTX 4060 Laptop, genet `546201874df` through the Ortet
release host, netrender path-patched at `3961aca91`. Elements are absolutely
positioned faces under 2D `matrix()` transforms (Livery has no 3D transforms
yet), grouped body > faces. Ortet exposes no per-frame timing and presents
with Fifo on a visible window, so steady cost is the slope between a long
and a one-frame run and cannot resolve below the refresh interval.

| elements | first frame over empty baseline | steady per frame |
| ---: | ---: | ---: |
| 1,000 | 0.2 s | 6 to 7 ms (refresh floor) |
| 5,000 | 4.6 to 4.9 s | 8 to 14 ms |
| 20,000 | 60 to 132 s | unresolvable |
| 50,000 | 15 to 17 min | unresolvable |

| variant, 5,000 elements | first frame over baseline |
| --- | ---: |
| faces hidden with `display:none` | 4.0 s |
| faces with transforms (the fixture) | 4.9 s |
| faces with no transform | 10.1 s |

Findings:

- **Steady state is not the problem.** Through five thousand elements the
  redraw sits at the display floor. At twenty and fifty thousand the
  run-to-run variance of a single document exceeds the per-frame slope, so
  steady state is unmeasurable by this method and a host-side harness with
  real timing is needed to say more.
- **The first frame is the wall, and it is superlinear.** Ten times the
  elements from five to fifty thousand costs roughly two hundred times the
  time. This is a genet defect in parse, style, or first layout, not a
  property of the DOM approach; a stock browser lays out fifty thousand
  positioned boxes in well under a second.
- **The quadratic term is absolute positioning, not parse and style.**
  Corrected 2026-09-12 by the one-factor attribution at
  `Code/testing/wing/l0b_first_frame_attribution.md`. Parse, style, and box
  construction are linear at about 0.13 ms per element (the `display:none`
  variant grows with exponent 1.05). Only `position:absolute` changes the
  exponent, to about 2.2: genet's positioned-box pass walks the whole
  fragment tree two to four times per positioned box
  (`apply_absolute_and_fixed_positioning` in genet-livery's
  `layout/positioned.rs` calling `FragmentTree::resize_leaf`,
  `translate_subtree`, and `recompute_overflow` in buckram's
  `fragment_tree.rs`). The untransformed variant was slower because
  `left`/`top` offsets make `translate_subtree` take its whole-document
  scan instead of its zero-offset early return, not because of overdraw.
  The earlier "parse plus style is the larger half" reading in the L0b
  receipt is withdrawn there and here.
- **Live mutation was not measured.** The Boa-scripted fixtures render
  byte-identical frames with no wake events, so this Ortet build does not
  present script mutations. The fifty-thousand live cell timed out at thirty
  minutes on its first frame. This matters less than it looks: live faces in
  this plan are driven by the Rust host mutating the DOM directly, so the
  right live measurement is a host-side harness, not page script.
- **No phase timing exists in genet.** Neither info nor debug logging emits
  parse, style, layout, or paint spans. Attribution had to be done by
  fixture variant.

### L0 verdict

L0's done condition is met with one named gap (live mutation through the
DOM is unmeasured until a host-side harness exists). The ceilings:

| path | largest count under a 16 ms frame |
| --- | ---: |
| netrender, retained fragments, live | 200,000 rectangles (measured), ~290,000 (interpolated) |
| netrender, flat rebuild, live | 10,000 to 15,000 rectangles |
| genet DOM, steady state | at least 5,000 elements, more unmeasurable |
| genet DOM, first frame within 5 s | about 5,000 elements |

The bottleneck is genet's first-frame scaling, not netrender. The honest
default is therefore the **hybrid**: live faces for bodies in focus and
during rotation, baked sprites elsewhere. Live-all-the-time at Paredros scale
is gated on an engine slice that makes genet's first frame linear (the
quadratic term is the per-positioned-box whole-tree rework named above; the
linear parse and style cost is already small) plus a host-side mutation
harness to measure the live path. Both are genet's T2 and T3.

**L5 shape, ruled 2026-09-12 (ruling 10):** put the DOM at body or part
granularity and keep faces below it. A body is one element whose paint is a
retained netrender fragment, the way genet already splices an iframe's paint
list into a replaced box. Element count drops to hundreds, rectangle count
stays with netrender at the level L0a proved, and CSS still owns classes,
transforms, hit testing, and styling per part. Per-face shading and masks
then apply per part rather than per face. This is the shape the hybrid was
always going to converge on; the genet first-frame slice is owed regardless
and belongs in L1's genet plan as a named consumer.
