# Played Slice Plan (2026-08-28)

**Status: in progress, refreshed 2026-08-31. PS0 and PS1 landed mechanically
and were playtested; their vitals, terrarium-dynamics, roster, and default-body
follow-ups have also landed through DC4. PS2 is now decomposed by the
[playable ecology plan](2026-08-31_playable_ecology_plan.md).** The first
slice of Mesocosm a hand actually plays: the live epoch as the ruled terrarium
section with direct control of your organism. Renderer ruled: the brick-traced
side-on section.

The rulings this plan executes, cited not restated: the terrarium section
and direct control (`2026-08-18_vessel_briefs_and_presentation.md` §2), the
Rain-World pull-back with camera-is-not-the-person, control as a recorded
intent over `World.controlled` (`2026-07-31_phenotype_plan.md`, P1 and the
2026-08-01 correction), Route A's textless HUD lane
(`2026-08-02_views_founding_plan.md`), and metabolize as the single direct
verb (founding plan). The slice depends on no open gate: PD1b/PD2 upgrade
feeding's authority, not playability, and E1's interim feeding is landed.

**D5 (burn vs incorporate) was deliberately not in the slice, and is now
closed.** Its doc named it the recommended first playable proof, but Mark
rejected the hotkey-pair interface (2026-08-28: "framing it as a hotkey is a
bit odd… not a workable ui"), and it waited on an interaction design
consistent with "experienced directly, never through a menu".

**Resolved diegetically, ruled 2026-08-29 and landed the same day:** hunger
routes the meal. A critter inside a documented threshold of an empty budget
burns what it eats; a provisioned one builds with it. There is no second key
and no menu, because there is no question being asked — the answer is the
state you are already in, and the vitals panel is already showing it. The
intent dropped its route accordingly, which also means a replay cannot
disagree about a decision it never carried. See
`2026-08-29_terrarium_dynamics_plan.md` §TD4 and its ruling "Income: the body
routes it".

## PS0 — revive the hand

The playable host already exists: `mesocosm-genet` drives a real `World`
through `Runtime::advance` with WASD/E/Space/Q mapped to recorded intents
and the minimap HUD composited textlessly. It has been dead behind one
call-site type mismatch (`hud.rs` handing `&Vec<PaintCmd>` where netrender
now takes `&[PaintCmd]`).

**Done when:** `mesocosm-genet` builds and runs against current netrender;
a keyboard moves the controlled organism through the live world at fixed
timestep; clippy is clean; and the fix is the call site, not a fork of the
HUD lane.

**Receipt, 2026-08-28.** Landed with zero source changes — the diagnosis
was wrong in an instructive way. The "netrender API drift" was two cargo
source identities for one crate: sprigging (via genet's workspace) pins
the netrender family by rev `6f1a4fe7`, mesocosm rode `branch = "main"`
whose cached fetch was stale at an ancestor commit, and even after
updating, cargo keys git sources by URL *plus reference*, so rev and
branch never unify. Mesocosm's workspace now pins the same rev string
genet uses; the types deduplicated and the host compiled untouched. The
full workspace's tests and `-D warnings` clippy are green with genet
included for the first time since the drift began, and the host launches
and runs its loop. Keyboard play is Mark's half of the receipt.

## PS1 — the ruled look, played

Reunite the live `World` with the terrarium-quality renderer. Today they
have never met: the genet host plays a real world through the rasterized
`mesocosm-render` view with an orbit camera, while the brick tracer — the
ruled side-on orthographic section — has only ever been fed fixture poses.

The work: the genet host's main view becomes the lens brick tracer. The
tracer's `BrickMap` binds the live `Ground` (full build at genesis,
`refresh` from drained dirty bricks as play carves and deposits, revision
advancing with the world). The camera is an orthographic slab section
following the controlled organism — the pulled-back framing G2 ratified —
with the section's few-voxel depth; camera motion is presentation only and
never enters the trace. Bodies ride the lens's existing body-projection
path (the V2 `BodyLensProjection` lineage and the menagerie precedent
decide the exact mechanism; the implementer follows the landed pattern
rather than inventing one). The minimap HUD composites over the section
exactly as it does today. Arrow keys become a small presentation-only
section pan, replacing orbit.

Receipt discipline, per the house pattern: the host writes its intent
trace and `Receipt` (seed, organisms, steps, state hash) on exit; a
`--replay <trace>` mode drives the same headed frames from a recorded
trace with no keyboard and must land the identical state hash, and doubles
as the self-driving receipt (capture written, hash asserted) so the slice
is judged without a human in the loop after the first recording.

**Done when:** a played session in the brick-traced side-on section — move,
metabolize, deposit, carve — writes a trace on exit that replays headless
to the identical state hash and headed to a non-trivial capture; the
section visibly follows the controlled organism with the ruled framing;
the HUD minimap rides Route A unchanged; and clippy stays clean across the
workspace.

**Receipt, 2026-08-28.** All hold mechanically; the human half — Mark's
hand — remains, as with PS0. The host's main view is the brick tracer
over the live `World`: `BrickMap` bound at genesis, dirty bricks drained
per tick through a new documented `&mut`-but-not-a-world-change accessor
(`World::drain_ground_dirty`, invariant proven at runtime: a draining
headed replay and a never-draining headless recording land the same
hash), slab camera following the controlled organism with arrows as
presentation-only pan, the body posed through the landed V2
`BodyLensProjection` path, and the minimap composited unchanged. Traces
and receipts write on exit; `--replay` self-drives headed and asserts
the state hash — and the instrument is proven, since a falsified hash
exits 1. A 200-intent demo trace replayed to `8a101763143e5012` on the
RTX 4060 (ground revision 4 — the carves exercised the refresh path),
and a 369-intent auto-driven session replayed identically. Capture at
`Code/testing/mesocosm/ps1_played.png` (1920×1080, 153 colours): strata,
carve scars, the critter, the minimap.

Two residues. The tracer takes exactly one pose, so only the controlled
organism appears in the section — the other organisms read on the
minimap; a lens roster is the named follow-up. And the G2 slab
(half-height 20) over-frames this 32-voxel enclosure to about half the
frame — the right half-height is nearer 9-11, but the section framing is
a presentation ruling and waits for Mark.

## PS2 — decomposed by the playable ecology plan

The loop's two unwired seams remain real, but reproduction and the epoch are
now explicitly different checkpoints.

- **Individual checkpoint, PE1:** a birth involving the controlled critter
  makes parent, offspring, material cost, and descent inspectable; one recorded
  control choice resumes play. Death enters witnessing and
  `Intent::TakeControl` of an eligible descendant or lineage member. Siblings
  remain in the ecology.
- **Lineage checkpoint, PE3:** an explicit epoch rule drives
  `Runtime::end_epoch`, the world review, and a validated developmental-program
  revision for future descendants. It does not fire silently on every birth.

PE0's first flow reading precedes both so the birth and the epoch review have
the ecological evidence this game's interface requires. The trait-board review
screen stays with Views route B.

**The individual half is wired as of 2026-09-01** (playable ecology plan §9,
PE1). `Runtime` holds the world at a birth involving the critter under your hand
and at the moment that critter's body ends; the host draws the question as a
third chrome lane; `Intent::Resume` and `Intent::TakeControl` are the two
answers and both enter the trace. The lineage half — an explicit epoch rule
driving `Runtime::end_epoch`, the world review, and real developmental
adaptation — remains unwired and is PE3's.

## Findings

- **2026-08-29 (second real playtest, Mark):** 1,819 steps over 5,003
  frames, hash `faea34e116b2e243`, trace preserved as
  `mark_playtest2.*`. Four findings, in his words where they matter:
  - **The loop is "button mashing e to avoid starving to death."** The
    single-loop diagnosis confirmed from the chair, with the standing
    correction restated: "it's the loops that make a game. asking someone
    to playtest a particular loop doesn't make much sense absent the
    context of the rest of them to tune the loops against... the
    pathfinding means nothing without a map/doors/etc., what's the food,
    what's the colonist." Playtests of isolated loops are not a receipt;
    loop composition (PS2 first) is the work.
  - **Input lag, diagnosed in code:** every winit `Pressed` event —
    including OS key auto-repeat, which `intent_for` never filters
    (`event.repeat` unchecked) — queues an intent into an unbounded
    `VecDeque` drained at exactly 10 per second. Holding or mashing keys
    builds a backlog that executes tens of seconds late and grows for the
    whole session. Fix: filter repeats and coalesce/cap the pending
    queue; host-side, cannot touch the trace (the trace records what was
    applied, not what was pressed).
  - **The followed critter disappeared while he was not dead.** Cause
    unknown; his preserved trace is the reproduction fixture. Candidate
    hypotheses to test, not assume: pose lost while `controlled()` still
    live, slab-depth z drift, a severed-parts pose collapse, a lens
    residency seam.
  - **Bodies read as "abstract voxel shapes", not organisms.** His
    direction: "shrink those down and put a few together in the shape of
    a body plan, maybe you have something that looks more like a critter
    or flora then." A real round, not a cosmetic pass — part scale prices
    mass ceilings (`part_ceiling_mg` is voxel volume), so shrinking parts
    while multiplying them per plan touches the economy the way S1's
    analysis warned, and wants its own plan with that arithmetic done
    first.

- **2026-08-28 (first real playtest, Mark):** "no information was
  communicated, i don't think i changed my position? it is hard to tell
  what I was." A step-by-step replay of the session's trace (2,926 steps,
  hash `49c7a47f03895bda`, replay-verified) says he is right, and why:
  - The ecology drives the controlled critter like any other organism,
    spending its energy as it wanders. The starting 1,000mg was gone in
    the first ~166 ticks (~3 seconds); Mark's **first** movement keypress,
    at step 166, was already refused `InsufficientMass`, and no keypress
    ever displaced the critter. He was watching an animal, not driving one.
  - All ten metabolize presses landed (body 52 → 61 parts) — the one verb
    that worked.
  - At tick 999 the critter died, on the same tick its tenth meal landed
    (cause not yet chased). The camera fell back to the world origin and
    the remaining two-thirds of the session every input was refused
    `Disembodied`. This is the PS2 seam, hit in the first minute of the
    first real session.
  - The population ran 61 → 8,155 in 49 seconds, unchecked.
  - None of the above reaches the screen: no energy reading, no refusal
    feedback, no death signal. Every refusal is polite inside `World::apply`
    and silent outside it.
  Four seams, then, all small and all load-bearing: surface refusals and
  vital signs; rule the movement economy (player vs. autopilot over one
  energy pool); handle death (PS2); brake reproduction. The receipts held —
  the hash replayed exactly — but a receipt can be green while the
  experience is empty.

- **2026-08-28:** the long-flagged "mesocosm-genet does not compile
  against current genet" was neither genet drift nor an API break: it was
  a duplicate-source split (rev-pin vs stale branch fetch of the same
  netrender commit line). The ninth resolution trap; fixed by pinning the
  family to the rev string genet pins.
- **2026-08-28:** two rendering pipelines have never met: genet's host
  (real world, rasterized) and the lens tracer (ruled look, fixture-fed).
  PS1 is their join.
- **2026-08-28:** nothing calls the epoch boundary or `control_lost()`
  from any host; the adaptation phase has only ever run headlessly against
  the bare lineage roster.

## Progress

- **2026-09-01:** PS2's individual half landed as PE1 — the reproduction
  checkpoint, the succession path, and the host composition around both. The
  detail and its receipts live in the playable ecology plan §9; nothing about
  the lineage half moved. The recorded demo is now seed 7 over 3,100 steps
  (hash `f90123db6f2a5ac5`), replacing the 120-step PS1 recording at the same
  path: the old one could not reach a birth or a death, which is what the
  checkpoint needed to demonstrate.

- **2026-08-31:** reconciled PS2 with the playable ecology architecture.
  Reproduction is the individual checkpoint, the epoch boundary is the lineage
  checkpoint, and PE0's first ecological reading precedes their host wiring.
  Documentation only; no implementation dispatched.

- **2026-08-29 (later): the vitals surface landed** — Mesocosm's first
  cambium consumer, and the first words on the screen. Energy as a number
  and a bar, refusals as short plain words for a presentation-timed window,
  a persistent `dead` state when control is lost. Read entirely off
  `Runtime::last_outcomes`, `World::energy_mg` and `World::controlled`; no
  core or runtime change was needed.

  **Genet does not own the window.** The runner diffs the view fn into a
  `ScriptedDom`, Livery styles and lays it out, its paint list lowers
  through `paint_list_render`, and the raster composites over the frame
  exactly as the minimap does. The full host inversion was not reached for
  and is still Mark's to rule. The two lanes now share one netrender
  instance and one blend pass (`mesocosm-genet::chrome`) rather than
  carrying two copies of the texture-and-twin dance.

  Receipts: workspace tests green (463); clippy `-D warnings` clean; Mark's
  own playtest trace replayed headed to its recorded hash
  `49c7a47f03895bda` over 732 frames with the chrome live, exit 0. Captures
  in `Code/testing/mesocosm/`: `ps1_vitals_alive.png` (energy 897 mg, bar
  near full), `ps1_vitals_refused.png` (0 mg, empty bar, "not enough
  energy"), `ps1_vitals_dead.png` (the end of Mark's session: the panel
  reads `state dead` and is otherwise empty).

  Two residues. The bar's denominator is the session's own high-water
  energy, because the world has no capacity to measure against; if a
  capacity ever becomes a real quantity the bar should read it instead. And
  the three remaining seams from the findings below are untouched: the
  movement economy, reproduction's brake, and succession (PS2).

- **2026-08-29:** Mark ruled the next seam from the playtest findings:
  surface the state (energy, refusals, death). Re-scoped the same day
  during the wing GUI conversation: the surface is cambium chrome with
  words and numbers, not a painted-lane workaround — which makes it
  Mesocosm's first cambium consumer (views founding plan §6, vessel
  briefs and presentation record §2 "The shared chrome stack").
  Implementation deliberately held; not yet dispatched.
- **2026-08-28 (later):** PS0 landed: the host builds, tests and clippy
  are green workspace-wide, and it launches and runs. Netrender-family
  pins aligned to `6f1a4fe7`; netrender's registry-published stack noted
  as a possible future retirement of the vello git patch.
- **2026-08-28:** founded; scope, renderer, and the D5 deferral ruled by
  Mark the same day.
