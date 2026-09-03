# Side panel diet

**Status:** in progress (2026-09-03) — P0 and P1 landed, both open decisions
closed by Mark (three-column mode grid kept; hints only for a target pick),
one recorded stop (cut 5 needs a cambium `disclosure` that can carry app
state). Founded when host UI zoom (genet's
`2026-09-03_host_ui_zoom_plan.md`, Z5) measured the side panel at 1038
logical pixels against a declared design height of 820.

**Related:** the [genet host migration plan](2026-09-02_genet_host_migration_plan.md)
(M3 moved the text lanes and added the `px grid` row; its 2026-09-03
Progress records the fold); genet's host UI zoom plan (the fit that makes
the design height matter).

## 1. Why

`fit_design` is `(1100, 820)`, decided 2026-09-03: on a small display the
interface scales to fit 820 logical pixels of height and the panel must fit
inside that. Today it does not. The demo board's panel ends at 1024 plus
14 px of padding, so the whisper composer, the status line and the key hints
sit below the fold on any display shorter than about 1040 logical, and the
zoom only shrinks the interface without revealing them. Declaring 1040
instead was rejected: it makes the whole interface small to pay for a panel
that grew past its window one section at a time.

## 2. The target, stated once

On the demo skirmish at zoom 1.0, with every section in its default state,
the side panel's last painted element (the key-hint line under Messages)
ends at or above **800 logical pixels**, leaving 20 for padding and slack
inside the 820 design. The board pane is untouched. Nothing the panel does
today becomes unreachable; a control that moves must still be one click or
one key away.

## 3. The cuts, in order

Measure first: the harness reports each section's painted bottom edge
(`painted_rect`), so the plan records the real per-section heights in
Findings before any cut, and after. Then apply cuts in this order and stop
when the target is met:

1. **Mode list to a two-column grid.** Ten stacked rows (Select, Paint,
   Prop, Fill, Raise, Lower, Token, Play, Measure and the heading) cost about
   300 px; a two-column grid of the same buttons costs about half. Cambium's
   `segmented_control` or a plain grid of the existing clickables; keep the
   keyboard verbs and the selected-state styling.
2. **Map row and `px grid` on one line.** Undo, Redo, Save, Load and the
   pixel-grid toggle wrap to two rows today; a smaller toggle label or a
   `filter_chips`-style row brings them to one.
3. **Dice and Measure side by side.** Two short sections stacked cost two
   headings and two rows; one heading row with the dice buttons and the
   template controls sharing the width costs one.
4. **Key hints only while relevant.** The "arrows: pan / r: face / enter:
   end turn" line shows while composing or when a target pick is armed, not
   always.
5. **Only if still short: a `disclosure` on Turns.** The initiative list can
   collapse to its current entry; the section reopens on click. This is the
   one cut that hides information, so it is last.

## 4. Gates

**P0 — Measure.** Findings table of section heights on the demo board at
zoom 1.0, before cuts. **Done when:** the table is in this plan and a
harness test asserts the panel's last painted bottom edge, so the number
cannot drift silently again.

**P1 — Cuts to target.** Apply §3 in order. **Done when:** the harness
asserts the last painted bottom edge at or above 800; every existing routing
and self-test receipt passes; the whisper self-test capture on this display
shows the composer, caret, status line and hints inside the frame at the 820
fit; a combat capture shows the sheet and board unchanged.

## 5. Stop rules

- No section is removed and no control loses its function.
- The board pane, the overlays and the gestures are not touched.
- A cut that needs a new cambium widget stops and is recorded; the catalog
  is used as it is.

## Findings

### P0 — the panel before the cuts (2026-09-03)

Measured through the harness on the demo skirmish at zoom 1.0, in a
design-sized window (1100x820, no fit), every section in its default state.
Figures are logical pixels in the panel's own column; `top` and `bottom` are
the painted box's edges (`Harness::painted_rect`) and include the 14px of
top padding `.side` opens with. Rows are grouped into the sections the plan
argues about; the row-level detail that matters to a cut is called out under
the section.

| Section | top | bottom | height |
| --- | ---: | ---: | ---: |
| Title block (title, map name, board facts, selection) | 14 | 105 | 91 |
| Command line, idle | 109 | 109 | 0 |
| **Mode** (heading 13, then the nine verbs stacked at 144) | 117 | 278 | **161** |
| Brush (heading 13, swatch row 30) | 290 | 337 | 47 |
| **Map** (heading 13, then Undo/Redo/Save/Load/`px grid` over two rows at 50) | 349 | 416 | **67** |
| Tokens (heading 13, sprite row 44) | 428 | 489 | 61 |
| **Turns** (heading 13, round 16, list 92, three button rows at 25/25/75) | 501 | 759 | **258** |
| **Dice** (heading 13, seven buttons 38, empty roll log 0) | 771 | 830 | **59** |
| **Measure** (heading 13, controls 19, distance line 20) | 838 | 896 | **58** |
| Messages (heading 13, composer row 40) | 908 | 965 | 57 |
| Status line | 975 | 989 | 14 |
| **Key hints** (wraps to two lines at 200px) | 995 | 1024 | **29** |

**Last painted bottom edge: 1024**, and `.side` adds its own 14px of bottom
padding, so the column needs 1038 where the design names 820. That is the
figure `host_zoom::the_side_panel_is_still_taller_than_the_design` recorded.

Two facts the table changes about §3's arithmetic:

- **The mode list is 144px, not "about 300".** §3.1 sized it by eye from the
  stacked-button era; since the M3 migration it is the catalog
  `segmented_control`, whose items are unstyled 16px text rows. Halving it
  therefore buys 49 px, not 150 — a quarter of what §3 needed the first cut
  to find. Everything downstream of that estimate is short by the same
  amount.
- **The mode chips carry no paint at all.** `.selection-item` has no rule
  anywhere in `theme.rs`, so the nine verbs render as bare text with *no
  selected state*: the migration to `segmented_control` dropped the `.btn`
  look and the highlight without anyone noticing, because the panel is below
  the fold on this display. The diet's sheet is where that comes back.

### P1 — the cuts, and what each one bought (2026-09-03)

Applied in §3's order. Every figure is a measured delta on the same board.

| # | Cut | Widget | Before | After | Saved |
| --- | --- | --- | ---: | ---: | ---: |
| 1 | Mode list to a grid | catalog `segmented_control`, re-laid | 144 | 51 | **93** |
| 2 | Map row and `px grid` on one line | the panel's own `.btn` clickables, at the mini scale | 50 | 19 | **31** |
| 3 | Dice and Measure abreast | flex columns under one heading | 137 | 72 | **65** |
| 4 | Key hints only while relevant | none — a conditional row | 35 | 0 | **35** |
| 5 | `disclosure` on Turns | — | 92 | — | **stopped, see below** |

A further 4 px came off the Messages boundary as a margin collapse the merge
in cut 3 changed. **Last painted bottom edge: 796** (810 with the column's
padding, inside the 820 design), against a target of 800.

The panel after the cuts:

| Section | top | bottom | height |
| --- | ---: | ---: | ---: |
| Title block | 14 | 105 | 91 |
| Command line, idle | 109 | 109 | 0 |
| Mode (heading 13, grid 51) | 117 | 185 | 68 |
| Brush | 197 | 244 | 47 |
| Map (heading 13, one row 19) | 256 | 292 | 36 |
| Tokens | 304 | 365 | 61 |
| Turns | 377 | 635 | 258 |
| Dice · Measure (heading 13, both columns 39, roll log 0) | 647 | 707 | 60 |
| Messages | 715 | 772 | 57 |
| Status line | 782 | 796 | 14 |
| Key hints | — | — | 0 while idle |

**Three columns, not two — Mark's to keep or revert.** §3.1 says a
two-column grid. Two columns is five rows of 17 and lands the panel at 859,
59 px over target; three columns is three rows and lands it at 796. It is
the same widget, the same nine verbs, the same order and the same click
path, and nothing is hidden — the only question is whether `Measure`, the
longest label, reads well in a 66px chip. One number in `theme.rs`
(`.mode-grid .selection-item { width: 33.333% }`) reverts it, and the
target then needs the 59 px from somewhere else.

**Cut 5 stops at the catalog boundary, and was not needed.** Cambium's
`disclosure` (and `accordion`) are `View<DisclosureState, ()>`: their
content is a `ViewSequence` over `DisclosureState`, and both `lens` and
`map_state` only project *down* from the parent state
(`Fn(&mut Parent) -> &mut Child`). So a turn row that calls
`ui.select_token(id)` cannot live inside a disclosure panel. Collapsing
Turns behind the catalog widget needs cambium to grow a disclosure generic
over its content's state, which §5 stops on. The rows would fit
`data_grid`, which *is* state-generic, but a grid does not collapse. Turns
is untouched at 258 px, still the panel's largest section, and is where the
next diet would start.

**The transient states still overrun the design, and the hint line is why.**
§2 states the target for "every section in its default state", which is the
796 above. Measured on the same board in the two states cut 4 keeps the
hints for:

| State | last painted bottom edge |
| --- | ---: |
| Idle (the plan's target) | **796** |
| Composing a 19-character whisper | **858** |
| Target pick armed | **831** |

The composer's field wraps to a second line as the draft grows (+21), and
the hint line costs 35 with its margin. So on this laptop's 820-logical fit
the whisper capture shows the composer, the caret and the status line inside
the frame, and the key hints just under it. Dropping the hints from the
composing state alone would not close it either (823), because the wrap is
the composer's own growth. The 69 px cut 5 would free from Turns is what
covers both transients, which is the second argument for it. Worth Mark's
eye separately: while a text field holds the keyboard, `arrows: pan / r:
face / enter: end turn` is not true, and the status line already reads
`whisper (enter send, esc cancel)` — so composing may be the wrong half of
§3.4's "composing or a target pick".

Other choices worth recording:

- **`filter_chips` was considered for the Map row and not used.** It is a
  selection bar over a `SelectionState`, and Undo/Redo/Save/Load are verbs
  rather than a selection — pressing Save is not "Save is now on". §3.2's
  other half (a smaller toggle label) is the honest one, so the toggle reads
  `px` with `btn-active` carrying on/off, and the five run at the mini scale.
- **The mode chips are painted from the panel's own `.btn` vocabulary**,
  scoped under `.mode-grid` so the downtime overlay's pace and stance rows —
  the same component — are left alone.
- **The distance readout stays in the Measure column** rather than moving
  full-width under the pair: at 90px it still fits on one line, and moving it
  cost more than it saved.

### The hints leave the composing state (2026-09-03)

Mark's call on the open question above: the hint line stands only while a
target pick is armed. While a text field holds the keyboard the arrows, `r`
and Enter do not do what the line says, and the status line already reads
`whisper (enter send, esc cancel)`, so §3.4's "composing or a target pick" was
half wrong. `panel.rs` now tests `ui.picking_target()` alone.

Re-measured through the same harness on the same board, at zoom 1 in a
design-sized window, driving the composer through the host's own key path
(`w`, then the self-test's 19-character draft `meet me at the gate`):

| State | before | after |
| --- | ---: | ---: |
| Idle (the plan's target) | 796 | **796** |
| Composing a 19-character whisper | 858 | **823** |
| Target pick armed | 831 | **831** |

The 35 px is the hint row and its margin, and 823 is the figure the P1 note
predicted for dropping it. Composing still overruns the 820 design by 3 px —
the composer's field wraps to a second line as the draft grows, which is the
composer's own growth and not the hint's — so the transient case still wants
the 69 px cut 5 would free from Turns. Target pick is unchanged, as it must
be: nothing about that state's DOM moved. (The 831 is measured with the
panel's default empty status line, the way P1 measured it; with the real
`pick a target (Esc to cancel)` status painted it is 832.)

## Progress

- **2026-09-03.** Plan founded; design height 820 with a panel diet chosen
  by Mark over declaring 1040.
- **2026-09-03, P0 and P1 landed in the working tree (uncommitted).** The
  before table above is the harness's own measurement, not an estimate, and
  it is what corrected §3.1's arithmetic: the mode list was 144 px rather
  than the ~300 the plan sized it at, so §3's first cut was worth a quarter
  of what the target needed. Cuts 1-4 took the last painted bottom edge from
  **1024 to 796** against a target of 800, with the mode grid at three
  columns rather than two (recorded above as Mark's to keep or revert). Cut
  5 was not reached and is stopped on the §5 rule: the catalog's
  `disclosure` cannot carry content that acts on `UiState`. Every control is
  where it was, one click or one key away; nothing is hidden.
  `the_side_panel_is_still_taller_than_the_design` is gone, replaced by
  `host_zoom::the_side_panel_fits_the_design_height`, which asserts the
  figure twice — at zoom 1 in a design-sized window, which is the plan's
  target, and inside the fitted short window this laptop actually opens. It
  measures the maximum over the panel's rows rather than a named last row,
  because which row is last is a matter of state now. Tests 313 under all
  features, 0 failures; `cargo check --workspace --all-features
  --all-targets` green. Behaviour beyond layout: the pixel-grid toggle reads
  `px` instead of `px grid: on`; the key-hint line stands only while a
  composer is open or a target pick is armed; and the nine mode verbs have a
  painted selected state again, which they had lost in the M3 migration to
  `segmented_control`. Two headed self-driven captures in
  `Code/testing/isometry/images/` (`2026-09-03_isometry_whisper_p1.png`,
  `2026-09-03_isometry_combat_p1.png`), both 2200x1504 physical. The whisper
  frame shows the whole panel from the title to the status line inside a
  752-pixel window, which is what the diet was for; the key hints sit just
  under the fold, because the composing state is 858 rather than 796 — see
  the transient table above. **Open, Mark's:** the mode grid's column count,
  and whether the hints belong in the composing state at all.
- **2026-09-03, both open questions closed by Mark.** The three-column mode
  grid stays as it is. The key hints show only while a target pick is armed,
  not while composing: `panel.rs`'s hint predicate lost its `ui.composing`
  half, and `host_zoom::panel_bottom`'s note about which row is last says the
  same. Re-measured: composing falls from **858 to 823**, idle and target pick
  are unchanged at 796 and 831 (see the dated Findings section above). No test
  asserted the hints in the composing state, so none needed adjusting; the
  standing receipt `host_zoom::the_side_panel_fits_the_design_height` is a
  maximum over the panel's rows and is indifferent to which state paints last.
  Tests 313 under all features, 0 failures — the same count as before, since
  the change is a condition rather than a receipt.
