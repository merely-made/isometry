# Dev tools: sitting in a run and interrogating it

**Status (2026-09-02): DT1, DT2, DT3 and DT4 all landed. The plan is
complete.** Both §4 decisions ruled and built. DT4 folded the bespoke replay
and demo harness into genet-probe's `Automatable`/`Driveable`/`Scenario`, and
reconciled the epoch boundary's two disagreeing doors into one on the way.

**Follow-on integration, 2026-09-04:** DT1-DT4 completion does not include
screen-to-part selection. The
[phenotype plan section 8, VB3](2026-07-31_phenotype_plan.md#vb3-point-to-the-body-and-read-what-happened)
owns pointer routing, addressed body/part selection, in-scene highlighting and
the selected-part explanation over these existing tools. The current scenario
pointer hooks report unrouted input; VB3 must complete the real input route
before claiming click-driven acceptance.

## 0. Objective

Mark, 2026-09-01: "We need dev tools to evaluate the game." The base is to sit
in a running game and interrogate it: pause, step, speed, follow any critter,
read its body and flows, force an event, end the epoch now. Watching the whole
ecology on one panel and comparing runs are data, and where those panes live
is not decided, so they are out of scope here.

## 1. What exists (verified 2026-09-01)

- **mesocosm-genet.** Play controls only: WASD, E or Space, Q, C, arrows,
  Escape (`input.rs:144-154`, `app.rs:444`). Headless flags in `main.rs`:
  `--frames`, `--capture`, `--trace`, `--receipt`, `--replay`,
  `--record-demo`, `--auto-eat`, `--slab`, `--seed`. No pause, step, speed,
  inspect, or force.
- **mesocosm-runtime.** `Runtime::advance` runs what the clock authorises and
  `Runtime::step(n)` ignores the clock (`runtime.rs:103`, `:132`), so time
  under the hand is already separable from wall time. A checkpoint holds the
  world without banking time. Readings a lane can draw from: `readings()`,
  `trend()`, `windows()`, `history()`, `last_outcomes()`, `end_epoch()`.
- **Chrome.** Three lanes: the painted HUD (textless by lane discipline), the
  vitals panel, the succession panel. Words go through cambium, per the views
  founding plan §6 as amended 2026-08-29.
- **genet.** `genet-probe` has `Automatable` and `Driveable` and a text
  `Scenario` with the verbs act, click, settle, wait, assert, log, capture
  (`components/genet-probe/scenario.rs:328-336`). `workbench` has `TileTree`
  and `TileEvent` for split-and-tab layouts (`components/workbench/lib.rs`).
  cambium has `CommandItem` (`command_surface.rs:16`), `TreeItem`
  (`disclosure.rs:330`) and sprigging's `GridSpec` (`grid.rs:30`).
- **isometry.** A side panel with edit modes and undo/redo, and an
  `overlay_panel` widget. No inspector, no console.
- **Nowhere in the stack:** pause/step/speed, free follow, click-to-inspect,
  spawn or kill, force-an-event, value editing, or a console.

## 2. Principles

1. **Dev tools are ordinary chrome in the cambium lane.** One more lane, the
   dev lane, built from cambium widgets. Nothing textual enters the paint lane.
2. **Two kinds of dev action, kept apart.** A host-only action (pause, step,
   speed, follow, inspect) never touches the world and lives outside the
   snapshot, so a run paused a hundred times hashes like one never paused. A
   world-changing action (end the epoch, force a birth, kill, place matter)
   is an `Intent` in the trace, so replay reproduces it and the hash stays
   honest. There is no third kind.
3. **The lane invents no readings.** Every value shown is read from `Runtime`
   or `World`. If a fact is not readable, the gap is filled in core with a
   test, not papered over in the panel.
4. **Consolidate into the stack, and grow it when needed.** The bespoke replay
   and demo harness folds toward genet-probe rather than growing. Layout,
   when more than one pane is wanted, uses workbench. No hand-rolled panel
   code where cambium has the widget. Where the stack lacks a piece (a
   time-control strip, an inspector tree over readings), build it as a stack
   component in cambium or genet-probe's own idiom, not as mesocosm-local
   code, so the next game finds one implementation. Ruled by Mark
   2026-09-02: broadening the stack is fine; duplicated or irreconcilable
   runs at the same problem are what the rule forbids.
5. **A played receipt tells the truth.** A run that used a world-changing dev
   action is labelled as such in its receipt, so a playtest cannot be quietly
   assisted.

## 3. Phases

### DT1. Time under the hand

Pause, single step, step N, and a speed multiplier on the clock, host-only,
with the current state shown in the dev lane.

**Done when:** a paused run advances exactly the steps asked; a trace played
with pauses and speed changes produces the same hash as the same trace played
straight; the dev lane shows paused, stepping, or the multiplier; the flag
that enables the lane is recorded in the receipt.

### DT2. Follow and inspect

Follow any critter, by roster cycling or by selecting it in the section, and
read it: parts and roles, sites and cells, its accounts from the flow record,
its discoveries, and its species' current program revision.

**Done when:** following a critter that is not under control does not move
control; every field shown comes from a core query; a test renders the panel
for a fixture critter and compares it against the readings directly; a
critter that dies while followed is reported, not silently dropped.

### DT3. Force and end

World-changing dev intents: end the epoch now (this is PE3's player-triggered
epoch rule, ruled a dev tool on 2026-09-01), force a birth from the followed
critter, kill the followed critter, place matter at a cell.

**Done when:** each is an `Intent` in the trace; replay reproduces it and a
falsified hash still exits 1; matter is conserved through every one (a kill
leaves a corpse, placed matter enters through a named dev source so the flow
record still reconciles); the receipt counts dev intents and labels the run.
The epoch half waits on PE3's rule enum; the rest does not.

### DT4. Harness consolidation

Express `--replay`, `--record-demo` and `--auto-eat` through genet-probe's
`Driveable` and `Scenario`, and drive the existing fixture as a scenario.

**Done when:** the `ps1_played` fixture replays through the scenario driver at
the recorded hash; the bespoke code it replaces is deleted and the deletion is
listed; the headed-verify home under `Code/testing/mesocosm` still receives
receipts and captures.

### DT0. One boundary, one door

Not a phase of its own — a reconciliation DT3 turned up and DT4 carried out.
Two doors ended an epoch and they disagreed: `World::end_epoch(history)`
reckoned, bumped the epoch and restarted the budget but never ran the
adaptation round and left `at_boundary` *false*, while `Intent::EndEpoch` and a
spent `Timed` budget ran the whole PE3a boundary.

**Done when:** the boundary block in `World::apply` is the only door; any
manual door calls exactly it or is gone; the reckoning stays the separate
read-the-past call; a test shows the two remaining routes leave equal worlds;
and the golden fixture's hash has not moved.

**Order.** DT1, then DT2, then DT3. DT4 is independent and can go beside any
of them. Each phase is one agent round on a non-Fable model.

## 4. Decisions, ruled by Mark 2026-09-02

1. **How the lane is enabled.** A `--dev` runtime flag. One binary; the
   receipt records the flag.
2. **Where the lane sits.** Look to cambium, not to isometry. Isometry's side
   panel is app code that happens to sit on cambium, a precedent for "docked"
   and not a component to reuse. The dev lane is a workbench tile holding
   cambium widgets, and its placement is whatever workbench's split-and-tab
   tree gives; no hand-rolled panel frame, no isometry code copied across.

## 5. Stop rules

- No world mutation outside the trace. No second authority over the world.
- No text in the paint lane.
- No reading computed in the lane that core cannot answer.
- No dev-only physics or dev-only rules: a forced birth is the ordinary birth.
- No new harness where genet-probe already has the verb.

## Findings

- **2026-09-01:** the stack has automation (genet-probe), layout (workbench)
  and widgets (cambium) for a dev lane, and no run-time dev tools at all in
  any game. Mesocosm's replay harness is the only headless evaluation tooling
  and it is app-local.
- **2026-09-02 (DT1):** `workbench` (genet, `components/workbench`) is a
  host-owned tree and reducer only — there is no genet-side surface that
  renders a `TileTree` to pixels, draws a tab strip, or turns a drag into a
  `TileEvent`. DT1 never needed one (its tree never holds more than the dev
  lane's one tile), but DT2's second dev pane is what would. See
  `mesocosm-genet/src/dev.rs`'s module docs.
- **2026-09-02 (DT2), three gaps found and each handled differently.**
  1. **Two core readings were genuinely missing** and were added there with
     tests, per principle 3. `flow::Accounts`
     (`mesocosm-core/src/flow/accounts.rs`) reduces one *body's* income, rent
     and outflow off the flow record, making exactly the split `Score` already
     makes for a *line* — written once so the two cannot come to disagree.
     `History::ending` answers when and which way a creature stopped living,
     off the record's own `Died`/`Returned` event and its envelope's tick. The
     driver holds the per-body window (`Runtime::watch`/`::accounts`) because
     it already drains the stream the ecology's own windows reduce; a world
     buffers one tick and the host never sees it.
  2. **A world's discoveries are world-scoped, not per line.** `World::observe`
     records against whoever is under the hand, so a world holds *one*
     discovery list and it is the played line's. The tile shows that list and
     says so; a per-line discovery ledger is reported here rather than
     invented in the panel. It is what DT3 or a later phase would need before
     the row can honestly be labelled "this critter's line".
  3. **Click-to-select in the section needs picking machinery the host does
     not have.** The pieces exist elsewhere — `mesocosm-runtime`'s
     `TactileWorld` raycasts ground and critter capsules, and
     `mesocosm-lens`'s `t1_picking` example judges a sweep on the CPU — but
     `mesocosm-genet`'s `Section` holds no hit data and builds no
     `GroundVoxelProfile`, so a click would need a tactile world synchronized
     with every carve, every roster body presented under its organism id each
     frame, and an unprojection through the orthographic slab. Left out, as
     the slice allows; roster cycling is the path.

- **2026-09-02 (DT3), one ordering fact worth writing down.** A dev intent is
  resolved at the *top* of a tick and the ecology's own passes run after it, so
  "now" means something slightly different at each of the two doors, and the
  two verbs answer it differently.
  - **A forced birth is deferred, deliberately.** The tick's birth pass appends
    its newborns *after* rent, feeding, dispersal and the life-history pass, so
    a natural child is never run through the tick it was born in. A forced
    child appended where it was made would be — aged, rented, possibly walked
    somewhere — and would not be the ordinary child the stop rule requires. So
    `World` holds a one-tick `forced_birth` handoff (`serde(skip)`, empty at
    every tick boundary, so it cannot reach a snapshot or a hash) and the child
    joins the roster at exactly the point `breed`'s newborns do. `PD5`'s filial
    expression covers it too: the `before_births` read moved to the top of
    `apply`, so both routes to a birth are addressable to it.
  - **A dev kill is not deferred, and does differ by an ordering.** The body is
    carrion before the tick's passes see it, so it skips that tick's rent and
    enters the decay arm one tick earlier than a body the ecology takes at the
    end of the same tick. That is the same "now" every acting intent already
    has — `Intent::Metabolize` removes a body at the top of a tick too — and it
    moves no matter: the corpse, the record and the released reserve are
    identical, which is what the tests compare. Recorded here rather than
    engineered away, because deferring the death would have meant a body that
    was dead and still paying rent.

## Progress

- **2026-09-01:** assessment written from a read-only inventory of mesocosm,
  isometry, genet, cambium and mere, verified against file paths above. No
  code dispatched.
- **2026-09-02 (DT1 landed):** `--dev` (off by default; recorded in the
  receipt as `dev: bool`) adds a fifth chrome lane — a cambium detail panel,
  top left, built the way `mesocosm-views::vitals` is — and five keys, live
  only while the flag is set: `P` pauses or unpauses the clock, `.` steps
  once and `,` steps `DEV_STEP_N` (10), both off the clock, and `[`/`]` move
  a five-rung speed ladder (1/4, 1/2, 1, 2, 4) that scales the elapsed
  microseconds the host passes to `Runtime::advance`. All three are host-only
  pacing: pause drops the elapsed time the same way a checkpoint hold already
  does, speed scales it before the clock ever sees it, and step calls
  `Runtime::step` directly — none of the three reaches `Runtime::queue`, so
  none can enter the trace or move a hash. `mesocosm-runtime`'s
  `pauses_speed_changes_and_manual_steps_do_not_change_the_hash` drives the
  same intents through pauses, speed changes and manual steps interleaved
  with `advance` and checks the hash against a straight run; `step`'s
  existing "N unless a checkpoint holds it, then fewer" contract is reused
  unmodified and stated explicitly now in `tests/checkpoint.rs`. Ruled §4.2:
  the dev lane's placement is a `workbench::TileTree`'s own geometry (one
  tile today; see the finding above), not a hand-rolled corner constant like
  the other three lanes'. `--dev --frames 200 --capture` was read: the panel
  reads "state / speed / tick / stepped", sits top left clear of the minimap
  (top right) and vitals (bottom left), and the whole frame is sane.
  `--replay` of the untouched `ps1_played.trace.json` fixture still exits 0
  at `081b4ba4bdc46190` with `--dev` off; a falsified hash still exits 1. The
  receipt's new `dev` field does not move the state hash. `cargo test
  --workspace`, clippy `-D warnings` (both profiles) and fmt are clean.
- **2026-09-02 (DT2 landed):** three more keys, live only under `--dev` and
  clear of everything play owns: `N` follows the next living critter in id
  order, `B` the previous, both wrapping, and `M` snaps the camera back to the
  critter under the hand. **Following moves the section's follow centre and
  nothing else** — `Host::follow` is host state beside the pan, control does
  not move, no intent is queued, and `mesocosm-genet`'s
  `following_does_not_move_control_and_queues_nothing` asserts the controlled
  id, the queue length, the trace and the state hash are all unchanged after a
  follow. The played body is still posed at full fidelity where *it* stands;
  only the slab's centre moves, so `frame` now reads a follow centre and a
  separate played position.

  The one dev tile (DT1's `TileTree` still holds exactly one) shows, for the
  followed critter and entirely from core queries: **id** and whether it is the
  controlled one (`World::controlled_id`), **species** with the registry's name
  (`Lineages::get`), **position**, the two accounts `flow::Account` names —
  **reserve** (`Organism::energy_mg`) and **substance** (`biomass_mg()`) —
  **flows** (income, rent, outflow, from the new `flow::Accounts`) with its
  **window** on a line of its own, the line's **revision**
  (`Species::program().current()`, or `founding`), the world's **discovered**
  conditions by name (`World::discoveries` through `discovery::name_of`), and
  the **parts** count plus one row per part giving its role
  (`plan::classify`), half-extent, and sites with process name and cell count
  (`BodyPhenotype::explain`). Three part rows, then `more +N parts` — the
  roster's own truncate-with-a-count, because the tile is one tile. A
  `mesocosm-views` test puts every one of those lines beside the query it came
  from.

  A followed critter that dies is **reported, then dropped**: `update_follow`
  takes `History::ending`, keeps it as a notice the tile prints
  (`critter 640 died at tick 812`), and snaps follow back to the controlled
  critter; following anybody else clears the notice. The succession lane is
  untouched — a death of the *controlled* critter is still the driver's
  checkpoint and behaves exactly as before.

  **The dock moved, and the tile still takes it off the tree.** DT1 put the
  lane in the top-left corner, which is 196 pixels tall before it reaches the
  vitals panel; an inspector with a dozen rows does not fit there. The dock is
  now the right column under the minimap, the other region no lane claims, and
  `rect_of` still walks the `TileTree` for it. §4.2 rules that the placement is
  the tree's; the corner was never a ruling.

  Two things were left out and are recorded as findings above: click-to-select
  in the section (it needs picking machinery `mesocosm-genet` does not have),
  and a per-line discovery ledger (a world's discovery list is the played
  line's, and the tile shows it as that).

  Receipts (DT2). `--dev --frames 300 --follow 640` was read at
  `Code/testing/mesocosm/dt2_inspect.png`: the tile reads `id 640`,
  `species 5`, `at -10, 22, -57`, `reserve 464 mg`, `substance 590 mg`,
  `flows in 552, rent 101, out 393`, `window 20 ticks`, `revision founding`,
  `discovered none`, `parts 33`, three part rows and `more +30 parts`; it sits
  in the right column clear of the minimap above and the vitals panel opposite,
  and the whole frame is the terrarium DT1's capture shows. A second run with
  `--follow 5` (a critter that died at tick 20) was read too and showed
  `critter 5 died at tick 20` under a tile snapped back to `id 0 (controlled)`.
  `--follow ID` is a new presentation-only flag that only says where the camera
  starts, added because a scripted trace has no dev keys in it. `--replay` of
  the untouched `ps1_played.trace.json` fixture exits 0 at `081b4ba4bdc46190`
  with explicit non-default receipt and capture paths; a falsified recorded
  hash exits 1. `cargo test --workspace`, clippy `-D warnings` (both profiles)
  and fmt are clean. `flow.rs` was split at the six-hundred-line ceiling to
  make room for `flow/accounts.rs`, and the host's follow state lives in
  `app/follow.rs` beside DT1's `app/devtime.rs` for the same reason.
- **2026-09-02 (DT3 landed):** four more keys, live only under `--dev` and
  clear of everything play and the other two phases own: **`X`** ends the epoch
  now, **`F`** forces a birth from the followed critter, **`K`** ends its life,
  and **`G`** puts `DEV_PLACE_MG` (5,000 mg, half the world's per-intent bound)
  into the ground under it. **These four are the other kind of dev action**, and
  the split from DT1's and DT2's eight is principle 2 made structural: those
  eight never reach `Runtime::queue` and this file's `input::DevKey` now says so
  in one predicate (`changes_the_world`), while every one of these does nothing
  but build an ordinary `Intent` and queue it. So all four are in the trace,
  replay reproduces them, refusals come back through `Outcome::Rejected` in the
  vitals lane's own plain words, and there is still no third kind.

  **None of them has its own physics**, which is `world/dev.rs`'s whole job:
  each is a validator in front of a transaction that already existed and is
  already reached by something else. `Intent::EndEpoch` runs the boundary block
  in `World::apply` — the round, `at_boundary`, and the reckoning by whoever
  holds the past — exactly as a spent budget does. `Intent::ForceBirth` calls
  `ecology::bear`, split out of `breeding::breed`'s loop so the birth pass and
  the dev door are one function: same filial seed, same scatter draw, same
  provisioning out of both accounts, same `Event::Born`, and the child joins the
  roster at the point `breed`'s newborns do rather than being run through the
  tick it was born in. `Intent::Kill` calls `ecology::perish`, split out of the
  tick's life-history pass the same way. `Intent::PlaceMatter` is
  `Soil::deposit` plus one recorded transfer. No dev intent writes an `Event`
  variant of its own — a forced birth writes the ordinary `Born` and a dev kill
  the ordinary `Died`, from inside the shared transaction — which is what makes
  a dev-caused death read as a natural one everywhere downstream.

  **The EndEpoch rule choice, and why.** `EpochRule::admits_demand` admits the
  intent under **`PlayerTriggered`**, which is now built and ends its epoch on
  the demand and on nothing else, **and under `Timed`**, which takes it as an
  early end and restarts its budget from that tick. Timed accepts for two
  reasons, both in that function's doc comment: `World::end_epoch` has always
  been able to close a Timed epoch early, so a dev key that could not would be a
  weaker tool under a stricter rule and the two would disagree about what a
  boundary is; and refusing would mean the tool only worked in a world founded
  under a rule the game does not ship, so the boundary it exists to exercise
  could never be reached with it. **`Gated` refuses** — it has no condition
  behind it yet, and a demand standing in for conditions nobody has named would
  make it indistinguishable from `PlayerTriggered`, which the playable ecology
  plan §6 deliberately keeps apart. All three are tested.

  **The dev source account, and how reconciliation closes.** `Account::Dev` is a
  fourth account and it sits *outside* the enclosure rather than in it: a
  placement is a `Process::Place` transfer from `Dev` to `Soil` naming no
  `Subject` on either end, so `tests/flows.rs` claims the soil's gain the way it
  claims every other and attributes nothing to a body (`Account::is_body` is the
  predicate that replaced the soil comparison there). Conservation is therefore
  the three compartments **less what that account issued**, read off the stream
  by `Account::issued_mg` and subtracted with no tolerance — and the control
  `the_check_catches_a_placement_the_dev_source_did_not_account_for` shows an
  uncounted placement reading as conjured matter, so the subtraction is an
  account and not a blanket allowance. Nothing about it is world state: it never
  enters a snapshot, so DT3 moved no state hash, and the golden fixture still
  replays at `081b4ba4bdc46190`.

  Refusals, by name: `EpochNotOnDemand(rule)` (a Gated world), `NotLiving(id)`
  (a corpse can neither bear nor die twice), `NoSuchOrganism(id)`,
  `InsufficientMass` (a parent that cannot provision its line's recipe out of a
  quarter of itself — `bear`'s own gate, not a second one, and the condition a
  natural birth waits on; also a placement of nothing), `OffGrid(at)` (refused
  rather than clamped onto the wall, because `Soil::column_at`'s clamp is
  insurance against a leak and a dev tool leaning on it would pile every mistyped
  coordinate into one edge column) and `OverBound { mass_mg, max_mg }` against
  `PLACE_MATTER_MAX_MG` (10,000 mg: a hundred columns of genesis soil, or ten
  founding bodies).

  **The receipt.** `PlayedReceipt` gained `dev_intents: u64` beside DT1's
  `dev: bool` — the flag says the tools were available, the count says they were
  used — and the host's receipt line leads with `assisted (N dev intents)` where
  it is nonzero, first in the line so it cannot be skimmed past. It is counted
  by the driver off the world's answers rather than off a keyboard, so a refused
  dev intent counts nothing and a `--replay` of an assisted trace reports the
  same number the recording did.

  Receipts (DT3). `mesocosm-genet/examples/dt3_script.rs` records a 64-intent
  trace headlessly — the `--record-demo` arrangement, because dev keys are keys
  and an unattended `--frames` run cannot press them — that eats its way through
  forty ticks, places 5,000 mg, kills the nearest neighbour, forces a birth from
  the next nearest, plays on twenty more ticks and ends with `EndEpoch`. Replayed
  headed with `--dev` and explicit non-default receipt, capture and trace paths,
  it printed `assisted (4 dev intents) replay 64 steps over 16 frames, hash
  5a87b85625e1a3f7 (matches the recorded hash)` and the receipt carries
  `dev: true, dev_intents: 4`. Both are kept beside the capture as
  `dt3_forced.json` and `dt3_forced.trace.json`; a falsified hash on *that*
  trace exits 1 too, still labelled assisted.
  `Code/testing/mesocosm/dt3_forced.png` was read
  whole: the trait board is up in the middle reading **"the epoch is over /
  epoch 1 ended"** with the boundary's four noted marks and the status-quo row —
  the PE3a boundary, opened by a dev key — the dev tile is in the right column
  under the minimap still reading its DT2 inspector (`id 0 (controlled)`,
  `reserve 4263 mg`, `substance 17987 mg`, `flows in 23902, rent 1296, out
  2356`, `parts 84`, `more +81 parts`), the vitals panel is bottom left with
  `energy 4263 mg` and a `grew` notice, the minimap is top right, and the
  terrarium under all of it is the same section the DT1 and DT2 captures show.
  The `247 died in 64 ticks` on the panels is this seed's own churn at 917
  founders — the enclosure's standing verdict, not the one dev kill in it.

  The golden `ps1_played.trace.json` fixture, untouched, still exits 0 at
  `081b4ba4bdc46190` with explicit non-default receipt and capture paths and no
  `assisted` label; a falsified recorded hash exits 1. The `Intent` enum gaining
  four variants moved nothing, as expected: intents are not in the snapshot, and
  a JSON trace is variant-name-tagged, so a recorded trace reads back identically.

  **The population instrument is unmoved.** All 45 runs of `dc4_roster.json`'s
  five batches were re-run against this tree and compared field for field
  against the committed receipt, timing excluded: every batch identical, run for
  run and sample for sample — baseline 0 breathes / 10 thins, archetype 6 thins
  / 4 collapse, roster 2 thins / 8 collapse, stand 10 thins, fauna 2 thins / 8
  collapse, control all collapse. `dc4_roster.json` was restored rather than
  rewritten, so the committed file is not carrying new timing on an unmoved
  result.

  `cargo test --workspace` is green with no thread cap, clippy `-D warnings` is
  clean in both profiles, fmt is clean, and
  `cargo check -p paredros-room --features r1-proof` is clean. Four files were
  split before adding, per the ceiling: `world/dev.rs` holds the four
  transactions in core, `app/devworld.rs` holds the four keys in the host
  beside `app/devtime.rs` and `app/follow.rs`, `flow/accounts.rs` took
  `Account`'s two reductions, and `tests/flows.rs` shed `flows/dev.rs` and
  `flows/refusals.rs` (it was already at 602 lines).
- **2026-09-02 (DT0 landed, the reconciliation DT3 found):** **there is one
  boundary and one door.** `World::end_epoch(history)` is deleted, and so is
  `Runtime::end_epoch` above it. What is left is the boundary block in
  `World::apply` — reached by the world's own epoch rule, or by a hand through
  `Intent::EndEpoch` — plus `World::reckon`, the separate read-the-past call
  PE3a made it.

  **The callers, and what happened to each.** `Runtime::end_epoch` had exactly
  one caller and it was a unit test (`runtime/tests.rs`'s
  `ending_an_epoch_notes_what_the_run_did`), which now queues `Intent::EndEpoch`
  and reads `Runtime::reckoning` — the driver's own `reckon_if_ended` does the
  rest, exactly as it does for a spent budget. `World::end_epoch` had that one
  and eight sites in `mesocosm-core/tests/reckoning.rs`, every one of which
  wanted the *reckoning* rather than the closing and now calls `reckon`; the one
  that also asserted `world.epoch == 1` goes through `apply(Intent::EndEpoch)`
  first, in the order the driver does it. The open rulings register's item 8
  asked in August who would give `Runtime::end_epoch` a production caller: the
  answer is that nobody does, because the door it was is now the intent.

  **What the disagreement actually was.** The manual door bumped the epoch and
  restarted the budget, but it never ran `World::adapt_round` and it left
  `at_boundary` **false** — so an epoch closed through it gave every unplayed
  line no turn and stood the world at no lineage checkpoint. Two answers to one
  question, and the second authority the plan's stop rules forbid.
  `world::dev`'s `the_demand_and_the_spent_budget_leave_the_same_world` is the
  claim now: two worlds, the same intent stream but for the last tick, one
  closed by a spent `Timed` budget and one by the demand, and they are equal to
  the byte the hash reads. It is provable because both `Intent::Resume` and an
  accepted `Intent::EndEpoch` are pure — no state, no event, and both reset the
  idle run — so the only thing that could differ between the two worlds is the
  boundary.

  The golden fixture's hash did not move: the demo never used the manual door,
  and `--replay` of the untouched `ps1_played.trace.json` still lands on
  `081b4ba4bdc46190`.
- **2026-09-02 (DT4 landed): the harness is genet-probe's now.**
  `mesocosm-genet` implements `Automatable` and `Driveable`
  (`app/drive.rs`), and `--scenario <path>` pumps a `genet_probe::Scenario`
  one step per rendered frame. genet was not edited: `genet-probe` is a git
  dependency on the same branch and rev the rest of the cambium family already
  resolves to, so nothing else in the lock moved.

  **The `act` mapping is the host's own key names**, not a second vocabulary of
  intent names and fields. `act e` is the E key, `act x` is the X key, and the
  reason is not brevity: `Host::press_key` is the *same* function the window's
  keyboard handler calls, so a scripted act goes through `input::intent_for`,
  the checkpoint's answers, the board's two keys, the dev-key split and the
  queue's backlog cap in the order a person's keypress does. A mapping that
  built intents directly would have been a second input policy. The twelve dev
  keys and the twelve play keys are all reachable, and
  `app/actions.rs`'s own test asserts every dev name resolves through
  `input::dev_key` rather than a table of its own.

  **Five host-side names for what no key says**, per the plan's instruction to
  express a missing verb through `act` rather than grow genet: `follow <id>`
  (`--follow`'s job; the keys only cycle), `follow-nearest` (the nearest living
  neighbour of the played critter carrying at least `NEIGHBOUR_MIN_MG`, so `act
  f` reaches somebody that can actually bear), `follow-child` (whatever the last
  accepted `ForceBirth` produced — the plan's own example of a verb the stack
  lacks), `hunt <steps>` and `demo <steps>`.

  **`busy` is scripted work in flight**: a replay whose cursor has trace left, a
  pump still running, or an intent still in the queue. **A checkpoint holding
  the world with nothing queued is quiet, not busy**, and that is the decision
  rather than an oversight — nothing about a checkpoint resolves on its own, so
  reporting busy would burn the whole `wait` cap and proceed anyway. It is
  exactly the moment `act enter` is needed, so `wait` hands the script its turn.

  **The two pumps run off the clock, and that is what makes a scripted run
  assertable.** `--auto-eat N` metabolized every N steps while wall time drove
  the ticks, so how much of an enclosure a capture run had eaten depended on the
  frame rate it was captured at. `act hunt 40` takes forty ticks through
  `Runtime::step`, the way DT1's manual step key does. Paired with `act p`,
  nothing but the pump moves the world — which is why the dt3 scenario lands on
  the same hash at 34 frames and at 35.

  **Deletions, every one.** `World::end_epoch` and `Runtime::end_epoch` (above);
  `examples/dt3_script.rs` (176 lines, whole file); the `--record-demo` flag and
  its headless block in `main.rs`; the `--auto-eat` flag and
  `HostConfig::auto_eat_every`; the auto-eat block inside `Host::advance`; the
  hand-written key-routing arm in `Host::window_event` (moved whole into
  `press_key`, which is now the one route); and the duplicated
  `assisted (N dev intents)` and `replay`/`played` strings in `app/receipts.rs`,
  which are `played::assisted_label` and `Host::mode` now because a scenario
  asserts against both. `played::record_demo` was **kept** — it is the golden
  fixture's content, not harness, and all eight `played/tests.rs` claims (PE1's
  loop, PE2's discovery, P3's branch, TD4's hands-off run) rest on it — but its loop
  is now one call to `played::demo_step`, the same function `act demo` pumps a
  frame at a time. `app/actions.rs`'s
  `a_frame_pumped_demo_reaches_the_headless_recordings_hash` is what says the two
  pumps are one driver.

  **`--replay` survives, and deliberately.** A scenario is pumped inside an app
  that already exists, so a trace's seed and roster have to be a flag: there is
  no verb that can found the world the scenario is running in. Three lines in
  `main.rs` is smaller than any sugar that would load a scenario to do the same,
  and the flag alone still works and still exits 1 on a mismatch. What moved is
  that the fixture's *claim* is now a scenario's to make.

  **Stack gaps for genet-probe, reported rather than built.** (1) **Pointer
  delivery.** `Automatable` requires `press`/`moved`/`release` and this host has
  nowhere to route a window point — DT2 already found click-to-select needs
  picking machinery it does not have, and the chrome lanes are rasters
  composited over the frame with no hit-test path back into cambium. The three
  are **attributed no-ops** that record `pointer-unrouted x y`, so a `click`
  resolves its selector correctly and then loudly goes nowhere instead of
  silently passing. That is this host's state, not a defect in genet-probe.
  (2) **No verb founds a world** (above). (3) **No loop or repeat verb**, which
  is why `hunt` and `demo` take a count instead of a scenario saying "again".
  (4) **No `assert snap` against an empty value** — the grammar's `splitn(3)`
  needs three tokens — so the snapshot spells an unaided run `unassisted` rather
  than leaving the field empty.

  **Scenario files.** `Code/testing/mesocosm/ps1_played.scenario` and
  `Code/testing/mesocosm/dt3.scenario`, beside the fixtures they are about, in
  the headed-verify home that still receives every receipt and capture.

  Receipts (DT4). The golden fixture, `--replay ps1_played.trace.json
  --scenario ps1_played.scenario` with explicit non-default receipt and capture
  paths, printed `scenario: waited 772 frames` / `scenario: ok` and
  `replay 3100 steps over 783 frames, hash 081b4ba4bdc46190 (matches the
  recorded hash)`, **exit 0**. The same scenario with the hash literal falsified
  to `...91` (a copy in a scratch directory; the fixture was never touched)
  printed both `FAIL: assert snap hash` and `FAIL: assert snap expected` by name
  and **exit 1**. The same scenario under `--frames 5` printed
  `the frame limit ran out with steps left` and **exit 1** too — a run cut short
  asserted nothing about the rest of itself, and reporting it as ok would be the
  one way a green scenario could mean nothing. `dt3.scenario` under `--dev`, applying the four dev intents
  through `act g`, `act k`, `act f` and `act x` with `follow-nearest` and
  `follow-child` between them, printed `scenario: ok` and
  `assisted (4 dev intents) played 64 steps over 35 frames, hash
  0e4d40ff36fe0566`, **exit 0** — sixty-four steps, exactly the deleted
  example's count. **The hash is re-derived and `5a87b85625e1a3f7` is retired**:
  the example precomputed both neighbours from one snapshot and killed the
  *second* nearest while bearing from the first, where the scenario kills the
  nearest and then bears from whoever is nearest after it, and its `hunt` filler
  walks toward prey where the example's only ever resumed. Same four intents in
  the same order, different neighbours, different trace.
  `Code/testing/mesocosm/dt4_scenario.png` was read whole: the trait board is up
  in the middle reading **"the epoch is over / epoch 1 ended"** with the four
  noted marks and the status-quo row, the dev tile is in the right column under
  the minimap reading `state paused`, `tick 64`, `stepped 4`, `id 0
  (controlled)`, `reserve 3814 mg`, `substance 18330 mg`, `parts 83` and
  `more +80 parts`, the vitals panel is bottom left with `energy 3814 mg` and a
  `burned` notice, the minimap is top right, and the terrarium under all of it is
  the same section DT1, DT2 and DT3's captures show. The scenario asserts
  `follow >= 916` right after `act follow-child`, so the camera provably went to
  the newborn; the tile reads the controlled critter at the end because the child
  did not survive the twenty ticks that followed, which is this enclosure's
  standing churn (`248 died in 64 ticks`) and is what DT3's own capture showed
  too.

  Sixteen new tests carry the slice: eight in `app/drive/tests.rs` (a recorded
  trace replayed through the driver at its hash, a falsified hash failing the
  same scenario, `busy` in each of its states, a checkpoint being quiet, the
  snapshot, an `assert text` against a chrome lane's own retained tree, capture
  naming, and the attributed pointer no-op) and eight in `app/actions.rs` (the
  key table matching `input`'s, an `act` and a press taking the same route to
  the same hash, unknown names refused, `follow-nearest`, `follow-child`,
  `hunt`'s exact tick count, two `follow-nearest` in a row, and the frame-pumped
  demo reaching the headless recording's hash).

  **The population instrument is unmoved, and this time the check has a
  control.** All 55 runs of `dc4_roster.json`'s six batches were re-run against
  this tree and compared field for field against the committed receipt, timing
  excluded: **identical**, and every batch's verdict tally is the one DT3
  recorded — baseline 10 thins, archetype 6 thins / 4 collapse, roster 2 thins /
  8 collapse, stand 10 thins, fauna 2 thins / 8 collapse, control all collapse.
  `dc4_roster.json` was restored rather than rewritten, so the committed file is
  not carrying new timing on an unmoved result.

  The control matters because the first attempt at this **produced a false
  pass**. That run was killed one batch from the end and never wrote
  `dc4_roster.json`, so the naive comparison compared the untouched committed
  file against itself and reported "unmoved" — an absence of difference that was
  really an absence of a run. The comparison now refuses to report anything
  until the file's mtime has actually moved, and the passing run says so
  (`1788377354 -> 1788384722`) before it says anything else.

  Nothing was split at the ceiling this round: `app.rs` came *down* to 574 lines
  because the key-routing arm moved into `press_key`, and the two new files are
  335 and 513. **Three examples were left alone and deliberately**:
  `examples/grow.rs`, `p3_receipt.rs` and `pe2_receipt.rs` are offscreen
  rasterizers that compose contact sheets over a headless device, not ways of
  driving a run — which is what `dt3_script.rs` was and why it went.

- **2026-09-02, ruling: fixture defaults (doc only).** Every DT1-DT4 landing
  entry above records passing explicit non-default trace, receipt and
  capture paths precisely because the headed binary's own defaults are not
  safe to leave alone: an unqualified `--dev` run risks writing over
  `Code/testing/mesocosm/ps1_played.trace.json` and its siblings, the golden
  fixture DT4's scenario replays against. **Ruled by Mark, 2026-09-02:** the
  headed binary's default trace/receipt/capture paths move to a scratch name
  under the testing home; the golden `ps1_played.*` fixture is written only
  by an explicit path. No code changed in this pass; wiring the new defaults
  is a follow-up implementation task.

- **2026-09-04, fixture defaults landed.** The follow-up above, built. The
  three default paths in `mesocosm-genet/src/played.rs` now read
  `<Code>/testing/mesocosm/scratch_played.{trace.json,json,png}` instead of
  `ps1_played.*`, off a named `played::DEFAULT_STEM`; `played::GOLDEN_STEM`
  names the fixture they must not be. `--help` says where the three go and that
  the golden fixture is written only when a flag names it, and the module docs
  of `played.rs` and `main.rs` say why the defaults were unsafe in the first
  place — the discipline every DT1-DT4 entry above kept by hand is now a
  property of the program.

  **Asserted, not remembered.** `defaults_do_not_name_the_golden_fixture`
  (`src/played/tests.rs`) checks all three against `GOLDEN_STEM` and
  `DEFAULT_STEM` together, because they moved together and one left behind is
  the same bug at a third the size; `defaults_stay_under_the_testing_home`
  keeps the other half of what a default is for — scratch, but scratch somebody
  can find.

  **The scenario files were annotated rather than changed.** Both
  `ps1_played.scenario` and `dt3.scenario` already named their receipt, capture
  and trace explicitly, which is exactly the discipline the ruling was about;
  their header comments now record that those names *had* to be explicit before
  today and are merely tidy after it. Nothing about how either scenario runs
  moved, and the golden fixture's own replay still passes explicit scratch
  paths.
