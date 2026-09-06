# Paredros Execution Plan (2026-08-07)

**Status: in progress (2026-09-06); F0-F2 closed, F3 active, F3a landed.**
The dry crossing contact fixture is implemented alongside F3 design. The
borg three-lives/action-query slice is implemented locally; the wider embodied
encounter and graphical character sheet remain proposed.
S0-S3 remain landed foundation receipts, with their stated headed judgments
still open. They do not define a required entourage, sortie loop, or camera.
**R4 was decided and executed
2026-08-10**: see
[the extraction review](2026-08-10_r4_extraction_review.md). The former
S4-S6 future gate line is superseded by the fundamental layers in §4. The
[founding plan](2026-07-30_paredros_founding_plan.md)
remains the charter. Its 2026-08-13 rulings now bind here: one named life in a
persistent generated world; allies are contingent; control changes through
death, an explicit world event, or an optional player rule; culture has
pointable causes; free roster control remains forbidden. Its phase section is
superseded by this plan.

**The wing question, in this vessel** (founding record, ruled 2026-08-07):
Paredros asks whether a **community remains itself as control, bodies, and
generations change**. Every gate below is a partial answer.

**The sortie ruling, retained as proof history (2026-08-07).** S3 was the
wing's extraction trigger and successfully joined negotiated participation,
body facts, terrain, control, deeds, and return. The 2026-08-13 rebase removes
its authority over product shape. A sortie is one possible sequence in the
world, not the game loop, and its tag-in pact is one valid optional or diegetic
control mechanism rather than the ordinary rule.

---

## 1. Identity prerequisites

Control continuity and inheritance require these to remain separate facts:

- **`SubjectId`**: the continuing person, across bodies and control.
- **`BodyRevisionId`**: which body, at which revision, per the wing
  phenotype contract.
- **Controlled-subject session state**: who the player is being, as
  recorded state with the same discipline as Mesocosm's control pointer
  (moves only through a recorded intent; replays).
- **Social identity**: who another subject or institution believes this is.
- **Character, office, and faction facets kept separate**: a role is held,
  not been. Offices survive their holders.
- **Lineage**: biological or constructed descent, independent of player
  control.
- **Player history**: which lives the player has inhabited, without claiming
  that they were one metaphysical person.

Death succession, optional tag-in, possession, domination, transplantation,
cloning, resurrection, chassis replacement, and biological descent are
different recorded operations on these facts. The record says what happened;
cultures and subjects may disagree about what identity survived.

## 2. The two owners, and their joint receipt

The audit's structural correction remains: **social willingness and embodied
action execution are separate owners.** Willingness (agreements, deeds,
confidence, refusal) never reads movement or combat internals; action
adjudication never decides what anyone was willing to do. S3 is the first
joint receipt, not their only future meeting place.

The puppeteering canary binds both: ordinary play embodies one named creature
until death. Other creatures execute their own chosen or agreed acts. An
optional player rule or explicit world event may move control, but never turns
into issuing moment-to-moment orders to several characters.

## 3. Gates

### S0 — The room probe (landed 2026-08-08)

One body, one room, close camera, fixed input trace. The renderling
tenant renders on netrender's device per the cohesion contract
(mesocosm landscape §8.9); movement rides the near-tier kinematics
already landed in mesocosm-core (step, stands, sees). Save, reload,
replay, state hash.

**Done when:** the same input trace replays to the same hash across
save/reload; a headed screenshot receipt exists; frame spans are recorded
beside netrender's.

**Done when, 2026-08-08.** All three hold. The probe is
`crates/paredros-room` (lib plus `src/bin/room.rs`), the repo's first
game code.

Determinism: a 64-tick const trace of per-tick headings drives one body
through `near::step` over `Ground`. Two straight runs produce the same
position log; a save taken at tick 32, restored into a freshly grown
world and run to the end, produces the same log, the same final position
`[39, 2, -39]`, and the same hash. Position-log hash
`0x27a905731c6bfc61`, ground hash `0x728a7687af5408a9`, both FNV-1a
(`mesocosm_core::snapshot::hash_bytes`) over postcard bytes. The save
carries the seed and the ground hash rather than the world, and a restore
that regrows a different ground is refused (`ProbeError::GroundDiverged`)
instead of replayed over. Proven by `tests/replay.rs`; 13 tests green,
`cargo clippy --workspace --all-targets -- -D warnings` clean. The hashes
are dated against mesocosm as of this entry, not pinned in an assertion:
they witness a replay, and relief changes upstream are allowed to move
them.

Picture: `ROOM_TRACE=1 cargo run -p paredros-room --bin room` opens a
winit window presenting netrender's composed master, which is the
renderling room composited at scene-op boundary 0 with a vello chrome bar
over it, both on one device. It drives itself from the trace, captures,
and exits. Receipt at `Code/testing/paredros/s0_room.png`, 1280x720, 243
distinct colours, checked in the capture path so a blank frame fails
rather than writing a file.

Frame spans, this machine: `probe_frame` 14 to 21 ms wall (tick, re-shade,
tenant draw, compose, present) against netrender's own `total` of 1.7 to
2.1 ms, of which `vello_render` 1.5 to 1.9 ms, `master_compose` 70 to
85 µs, `dirty_tile_rebuild` 65 to 85 µs, `tile_invalidate` 48 to 58 µs.
The probe's span
dominates because it re-uploads the room's 2,834 triangles every frame to
move the torch; that is the first thing to fix when a frame budget
matters.

**R1 shared-traversal receipt, 2026-08-20.** The historical receipt command
`ROOM_R1=1 cargo run -p paredros-room --features r1-proof --bin room`
keeps this room, trace, camera policy, netrender master, and replay
discipline, but projects `Ground` through the same `BrickTracer` and brick
DDA used by Mesocosm. Paredros supplies its existing
close-perspective eye and target as a `TraceCamera`; it does not borrow
Mesocosm's camera policy. The 64-frame headed run at 1280×720 on the RTX
4060 Laptop GPU recorded 11.601–34.646 ms overall, 12.552 ms median, zero
steady brick upload, and 2,357 distinct capture colours. The position-log
hash remained `0x27a905731c6bfc61`; the current upstream Ground revision
produced `0x809e3da5b3bd9cf3`, consistent with the dated-hash caveat above.
The receipt and inspected capture live at
`Code/testing/paredros/r1_perspective.{json,png}`.

The same Rust `BrickMap` type proves the ABI directly: origin
`[-9,0,-8]`, pointer extent `[18,3,17]`, atlas extent `[128,16,128]`,
3,672 pointer bytes, and 262,144 atlas bytes. On 2026-08-26 that reusable
organ moved to Mere's `conatus-brick` at commit `28c07fab`. Paredros now owns
its Ground source binding, constructs the shared `BrickMap` directly, and
keeps `r1-proof` in its default compile path. `mesocosm-lens` remains a product
presentation adapter for the headed tenant; it no longer owns the shared ABI
or `BRICK_DDA_WGSL`. The original renderling S0 receipt remains intact;
raymarch-depth composition closed as D1 on 2026-08-26.

**V1 continuous-zoom residency receipt, 2026-08-21.** The opt-in command
`cargo run -p paredros-room --features v1-proof --bin v1_residency`
grows a 256-voxel-half-extent planning region, holds one surface character
as the camera focus, and drives the ratified near-acts / mid-leads /
far-plans camera from distance 8 to 72. The rig rises continuously from 50
to 65 degrees. Its visible range is the four-corner ground-plane footprint
plus one brick, rather than the smaller target-plane width.

The full 6,091-brick Ground refuses the exact tracer's 4,096-brick ceiling.
Paredros therefore selects exact page radii 40, 88, and 128 while the shared
`BrickMap` owns their deterministic pointer/atlas layout. At the far view, visible radius 127 pulls
1,411 bricks and 795,144 logical payload bytes under the 1 MiB budget. Five
page transitions over 96 frames moved 2,250,848 bytes in all. Page
preparation took 230 to 1,463 microseconds after startup.

On the RTX 4060 Laptop GPU, the headed 1280x720 run recorded 4.385 to 32.271
ms frame spans, 5.918 ms median, and 5.876 ms steady median. Warm transition
frames were 5.534, 6.822, 5.995, and 6.724 ms; the 32.271 ms maximum was an
unchanged close page. Rapid far-to-close at frame 48 and close-to-far at
frame 60 both met their profile's 125 percent recovery threshold on the next
frame. The inspected capture contains 63 distinct colours. Full samples and
the capture live at `Code/testing/paredros/v1_residency.{json,png}`.

The receipt's byte count is pointer plus atlas payload. It excludes driver
rounding and the overlap while old GPU resources retire. Its load and
eviction counts are logical set differences. Extent changes still create
replacement textures and upload the complete page; incremental residency
remains open.

**V1a complete, 2026-08-26.** Paredros's working-set policy recomputes
selected keys inside an unchanged zoom band and advances a product-owned
`BrickProjectionRevision` whenever selection changes. The retained Mesocosm
presentation tracer uses that revision to fully republish equal-sized pages without
recreating textures or bind groups, while repeated unchanged pages stay
upload-silent. The tenant admits a newer Ground revision at the same projection
but refuses revision regressions and a map that advances neither identity.
Headless cache/lease coverage and the Paredros policy test are present. The
headed V1 trace moves focus one brick inside the far page band. At frame 72 it
advanced projection revision 4 to 5, exchanged 66 loaded for 33 evicted bricks,
retained the 795,144-byte physical extent, fully republished that payload, and
created zero textures or bind groups. Frame 73 uploaded zero bytes. The real
1280x720 RTX 4060 Laptop GPU run completed 96 frames, preserved one-frame
close/far recovery, and produced a 62-colour capture at
`Code/testing/paredros/v1_residency.{json,png}`. The original four-brick travel
probe changed physical extent and therefore could not prove this gate; the
closing receipt deliberately uses the equal-extent one-brick move.

**Ruling.** This first base-planning view does not force a clipmap: its exact
frustum fits the budget and band changes were not the dominant hitch in this
run. Do not generalize that into an LOD refusal. Larger planning views and
larger travel footprints remain unproved. Traversal ownership is now closed in
`conatus-brick`, raymarch depth composed with Renderling closed as D1, and the
stable `ResidentChunk`-backed brick cache closed as V1b, all 2026-08-26.
Clipmaps or mips become required when a real camera footprint exceeds that
exact cache, not before. Neither gate promotes product projection identity,
frame cadence, or lease scheduling into a cross-product contract.

**V1b stable-cache receipt, 2026-08-26.** The opt-in command
`cargo run -p paredros-room --features v1b-proof --bin v1b_residency`
holds the exact V1 zoom-and-travel trace over one capacity-fixed cache
(`StableResidency` over `conatus_brick::BrickMap::with_capacity` at
`bd8f0044`): 1,791 slots, 931,376 fixed bytes under the unchanged 1 MiB
budget, pointer box `[34, 3, 34]`, atlas `[128, 56, 128]`, extents never
moving. Retained bricks keep their atlas slots across retargets, so each
of the six transitions uploaded exactly the 13,872-byte pointer volume
plus its loaded slots' 512-byte bricks: the mid band +497 for 268,336
bytes, the far band +793 for 419,888, the rapid close snap pointers alone,
the rapid far reload +1,290 for 674,352, and the travel frame +66 −33 for
47,664 against V1's 795,144-byte full republish. Every frame after the
first asserted zero texture and bind-group creation, `wgpu`'s allocator
report was byte-identical after the first and last frames, both rapid
zooms recovered in one frame, and the RTX 4060 Laptop GPU run's 96 frames
produced a 62-colour capture at
`Code/testing/paredros/v1b_residency.{json,png}`. The V1 bin and receipt
stand unchanged as the pre-stable baseline.

One instrument finding: naively publishing 1,290 loaded slots as
individual `write_texture` calls cost 485 ms of per-call overhead; the
tracer now batches consecutive slots into contiguous strided box writes
from the map's own atlas slice — the same bytes reached the same texels
in 13.5 ms.

The room: `Places::grown(4242, 4, 64)` and `Ground::grow`, then one
`carve` at `[35, 6, -35]` into the first hillside a deterministic outward
ring scan finds with enough overburden to keep a floor, walls, and a
roof. Nine voxels cubed. Nothing about the terrain is Paredros's.

Two deliberate choices are ours and worth naming. The camera backs off
*toward the middle of the room* rather than straight behind the heading:
an over-the-shoulder rig in a nine-voxel chamber puts the eye inside a
wall the moment the body reaches a corner, which the trace does eight
times. And the tenant applies a torch falloff from the eye per vertex,
because greedy meshing turns a wall into one quad and one quad of one
colour has no near side.

### D1 — Raymarch depth composed with Renderling (landed 2026-08-26)

The shared rendering gate named by V1's ruling and by the mesocosm engine
review §5: brick-raymarch ground and Renderling raster geometry must occlude
each other correctly, per pixel, in one headed frame on one device.

**Mechanism (chosen 2026-08-26).** Renderling's stage already stores a
single-sampled standard-z `Depth32Float` depth texture (cleared to 1.0,
compare `Less`) and exposes it; `brick_dda` already returns hit distance
`t`. So the join is a shared depth attachment, not a second compose pass:
renderling draws first (colour and depth), then the brick tracer draws its
fullscreen pass into the same colour target with renderling's depth texture
attached, computing `@builtin(frag_depth)` from
`clip_from_world * (origin + direction * t)` under depth compare
`LessEqual` with depth write on. Occlusion is exact in both directions with
zero additional textures or passes, and Paredros's S0 camera already builds
the raster matrices and the trace rays from identical parameters.

**Ownership, per the R1a ruling.** The camera-neutral half — a
`clip_from_world` uniform, an `fs_depth` entry point, a lazily created
depth-pipeline variant, and `encode_with_depth` — landed in
`mesocosm_lens::BrickTracer`, leaving every existing pipeline and receipt
untouched. Composition policy — draw order, which geometry renderling
owns, the receipt scene — stays here, behind an opt-in `d1-proof` feature
and a `d1_depth` bin. `conatus-brick` did not change; its pinned revision
stands.

**Done when:** a headless tracer test proves the depth join without
renderling by pre-filling a depth texture and asserting the occlusion
split; the headed RTX 4060 run replays the fixed trace to its recorded
hash discipline; the capture shows the witness geometry present where
raster is nearer and absent where raymarched rock covers it, with a
positive control in the same frame; frame spans and tracer diagnostics are
recorded beside the existing receipts at
`Code/testing/paredros/d1_depth.{json,png}`; and no existing S0, R1, or V1
receipt changes behaviour.

**Receipt, 2026-08-26.** All hold. The headless test
`the_depth_join_settles_pixels_between_tracer_and_raster_depth` refuses a
frame without the matrix, hands the whole frame to either side with
constant-depth matrices, and splits it on a world-z ramp against a
mid-plane raster stand-in; all 33 mesocosm-lens tests are green. The
headed run `cargo run -p paredros-room --features d1-proof --bin d1_depth`
draws the body and three cyan witness pillars through renderling — one
standing before the wall, one with its base a voxel under the floor, one
wholly sunken beneath the surface — and the raymarched room over the
stage's stored depth. Judgment is by named world points on witness faces,
projected through the frame's own matrix (a projected box's screen AABB
necessarily overlaps the visible span under this close a camera, so
regions were abandoned for point probes). The bin selects the first trace
tick whose camera frames every probe with clear sightlines — the carved
chamber is convex, so probe rays cross no rock except the floor meant to
cover sub-floor targets — and tick 1 qualified. On the RTX 4060 Laptop
GPU at 1280×720: standing-pillar probe 9/9 cyan, buried open span 9/9
(the positive control), buried base 0/9, sunken pillar 0/9; 64 frames,
17.767/20.856/70.527 ms min/median/max (the maximum is the judged frame's
readback stall; netrender's own final-frame total was 2.19 ms), steady
brick upload 0 bytes, zero steady resource creations, position-log hash
`0x27a905731c6bfc61` unchanged. Receipt and inspected capture at
`Code/testing/paredros/d1_depth.{json,png}`.

Two findings worth keeping. Renderling's stage *replaces* its depth
texture on size or multisample changes, so the depth view must be fetched
after the raster draw each frame — a held view silently tests the join
against zeroed memory and loses every pixel. And the trace uniform grew
64 bytes (3,280 to 3,344) for the matrix, so refreshed R1/V1 receipts
will record the new `uniform_upload_bytes` without any behaviour change;
the lens tests prove the plain `encode` path renders identically.

### S1 — The refusal scene (landed 2026-08-08, headed judgment open)

Three companions, with reasons. They receive the same offer and respond
differently, and
**every response exposes its premises**. This requires the minimal forms,
present from the start: a deed log (append-only), a relationship fact,
confidence, refusal, and the explanation surface. Minimal is fine;
absent is not — that was the old P1's flaw.

**Done when:** a playtester can say why each of the three answered as
they did, from what the game showed them; and one standing agreement is
formed, exercised, and renegotiated or terminated, all legibly.

**Done when, 2026-08-08: the machine-checkable half holds. The headed
judgment is open.** The scene is `crates/paredros-social` (lib,
`src/bin/refusal.rs`, `tests/refusal.rs`); the §1 identity facts are
`crates/paredros-identity`. 33 tests green across the workspace,
`cargo clippy --workspace --all-targets -- -D warnings` clean.

Aud puts the same offer to three people at tick 8: scout ahead, grade 3,
danger 4, share 2, up to danger 5. Three answers, and the premises say
why. Aud, Bram, Odris and Sela are fixture names for this scene, not
lore.

- **Bram accepts.** Aud stood by him at ticks 1 and 2, which is trust 6
  and liking 4. Scouting asks grade 3 and he holds 4. Danger 4 asks trust
  4 and he holds 6; he would bear 5 for her.
- **Odris refuses.** Aud shared with him at tick 3 and left him at tick 4,
  which is trust -4 against the 4 that danger asks. He is the *best* scout
  of the three, grade 5 against the 3 asked, and refuses anyway. The model
  does not confuse capability with willingness, and
  `the_one_who_refuses_is_the_most_capable_of_the_three` fails if it ever
  starts to.
- **Sela counteroffers**, at share 3 and a danger cap of 3. Three deeds
  (shared, shared, stood by) put her at trust 5 and liking 6, so the ask
  is fine. She is the most careful of the three, and danger 4 is past the
  3 she would carry for anyone. Her answer is the terms she would take
  instead.

The premises are data rather than prose. The test asserts that the three
cited deed sets are non-empty, pairwise disjoint, and made only of deeds
Aud actually did to that person; that the deciding gate is stated; and
that gates never reached are never claimed (Odris stops at trust, so no
danger premise appears in his answer).

The agreement, whole, with the one who accepted. Formed at tick 9 on the
offered terms, after the weighing runs again, so an arrangement nobody
would accept cannot be created. Exercised at tick 10 and performed with no
renegotiation. Renegotiated at tick 11, Bram's proposal, down to share 3
and a danger cap of 3. Asked again at tick 12 at danger 4 and answered
`OutsideTerms`, which is the changed term biting. Ended at tick 13 for
`WorkDone`, with Aud's standing toward Bram and the reason both in the
premises. All four transitions are deeds in the log, and the test walks
the agreement's own history back to each one.

The canary, as a test: `a_standing_agreement_is_not_a_command` forms the
arrangement, records Aud abandoning Bram, and then asks under it. Bram
declines and names the deed that changed. Nothing in the crate can make
anyone act; offers are put and answers come back.

Determinism: building the scene twice and running both the offers and the
whole lifecycle gives equal answers, equal rulings, and equal societies.
Society hash `0x49860851d09fdd84` over 14 deeds, FNV-1a
(`mesocosm_core::snapshot::hash_bytes`) over postcard bytes. Dated here
rather than pinned in an assertion, as S0's are; what the test asserts is
that two runs agree.

**Open: the headed judgment.** Whether a playtester can say why each of
the three answered, from what the game showed them, is not a thing a test
settles. `cargo run -p paredros-social --bin refusal` prints the premises
as lines and is the surface to judge, but nothing has been put in front of
a player and no in-game presentation exists. That half of the
done-condition stays open, to be closed by S2 or by a headed session.

### S2 — The negotiated home (landed 2026-08-08, headed judgment open)

Offer someone housing and work. They accept, refuse, or counteroffer,
from an intelligible history. This is the settlement tested as **peer
agency** before it is tested as production.

**Done when:** at least one refusal and one counteroffer occur for
reasons the player can trace; an accepted agreement changes where someone
lives and what they do daily.

**What landed.** `settlement.rs` in `paredros-social`: dwellings,
tenancies, and the one rule that carries the gate — **residence is
derived, never bookkept**. A tenancy records dwelling, tenant, the
agreement that put them there, and when; it is current exactly while that
agreement stands. `offer_home` weighs the ask like any other offer and
creates a tenancy only on formation, so a home nobody agreed to cannot
exist; ending the agreement *is* moving out, with no second copy of
"who lives where" to fall out of step (`moving_out_is_the_agreement_ending`
ends the arrangement through `Society` alone and reads the move-out off
the settlement). The daily round derives the same way: home from the
current tenancy, work from the agreement under it, so an accepted
agreement is structurally the one thing that changes where a peer lives
and what they do daily.

The settling scene (`settling.rs`) continues S1's story: same four
people, same deeds, asks differing by person because the work does. Bram
takes the gatehouse and the bounds. Odris is offered the night watch, the
one craft he alone holds at a grade he exactly meets, and refuses at the
trust gate citing the abandonment: capability is provably not the reason.
Sela is asked past the danger she would carry, counteroffers, and the
arrangement that forms is her terms with the ward's danger held inside
the cap she named; her daily round is the bargained work, not the work
first asked, which is the counteroffer having mattered
(`an_accepted_agreement_changes_where_someone_lives_and_what_they_do`).
A dwelling under a standing tenancy refuses a second offer before anyone
weighs anything; a vacated one can be offered again. Settlement hash
`0x4087f839d497324a` over 12 deeds and 2 tenancies, dated not pinned.

**Open: the headed judgment**, as S1's. `cargo run -p paredros-social
--bin home` prints the answers with premises and the resulting daily
rounds; nothing has been put in front of a player. What a dwelling
yields, who may offer on the settlement's behalf, and any production are
F5/F6 questions, deliberately absent.

### S3 — One sortie and return (sim half landed 2026-08-08, headed real-time action open)

A bounded expedition: companions negotiate participation under standing
agreements; real-time embodied action for the played subject with
companions executing their agreed parts; tag-in under pressure; bodily
consequence (injury as body-revision fact); scavenged material returns;
deeds are recorded and later *explain* something.

**Done when:** a full sortie-and-return replays to the same hash; a
tag-in occurs mid-action under a pre-agreed condition; an injury
persists as a body fact; one post-sortie offer or refusal is explained
by a deed from the sortie; and the canary holds (no moment-to-moment
multi-character orders anywhere).

**What landed.** `crates/paredros-sortie`, the one crate allowed to read
both owners. The scene is continuity all the way down: S0's seed and
hillside, S2's settled society, departure from the surface above the
carved chamber. At the muster Bram's scouting is negotiated fresh (a
bounded expedition is nobody's daily round) and formed as an agreement
that is exercised by the march and ended `WorkDone` at the return; Sela's
part rides her standing settlement agreement, which also carries the
tag-in pact — "a standing agreement governs tag-in" as data:
`Pact { under, successor }`, firing only while the agreement stands.

The world itself is the hazard. Marching the calibrated heading, Aud and
Bram take a six-voxel scarp the near tier only descends as a forced
drop; both are wounded (the law is uniform), only Aud downs (downing is
a control fact, the thing the pact watches). The pact fires one tick
later, the player becomes Sela, walks four ticks to the ledge, tends
from above (`PerformedUnderAgreement` + `StoodBy`, both deeds), tags
out. The salvage is taken in the trench — and the trench, like every
pocket this terrain drops a walker into, cannot be re-climbed at a
one-voxel lift, so the party **digs**: when the played body makes no
progress for four ticks it carves a head-height notch toward its goal
(mesocosm's own verb, consumed) and climbs into it. 35 voxels hewn come
home with the salvage. Home again, Aud shares with both companions, and
the expedition agreement closes.

Every done-condition is a test in `tests/sortie.rs`: replay to hash
`0x3b7446a25215a0bb` (70 ticks, dated not pinned); tag-in strictly
inside the action with a rescue that takes real ticks; the wound as
`BodyRevisionId(1)` worn in the facets after the run; and the payoff —
the ask S2 saw counteroffered (danger 4 against the 3 Sela would carry)
is put again post-sortie and **accepted**, the premises citing the walk-
home share by deed id: one sortie deed is exactly the margin. The canary
receipt runs the grudged variant: Aud abandons Bram on the eve, Bram
refuses, the sortie completes without a scout, and his trail row never
moves. `Sortie::advance` takes no input at all.

**Open: headed real-time action.** The sim march is goal-seeking from
recorded configuration, which is what makes the receipt a replay; the
played subject under live input, presented, is the headed half —
`cargo run -p paredros-sortie --bin sortie` prints the judgment surface
until then. **R4 is hereby armed**: the extraction review owes its
written decision (extract with two consumers named, or decline in
writing) as its own deliberate pass.

## 4. Fundamental layer ledger (ruled 2026-08-13)

These are game foundations, not a demo itinerary. Each layer establishes
laws and durable facts that later layers consume. A small executable receipt
proves a law; it does not need to resemble a satisfying vertical slice.

S0-S3 map onto several of these layers as preliminary evidence, but none is
closed merely because a fixture exists.

### F0 — Persistent world (closed 2026-08-21)

Own stable place identity, containment and routes; generated settlements,
dungeons, ruins, random-encounter sites, and surface/underground regions;
seeded generation; persistent material edits; save, reload, and replay.
Imported history displaces procedural content at the same slots and never
gates a playable generated world.

**Done when:** two fresh runs generate the same world facts from the same
seed; a journey crosses surface, underground, ruin, and settlement places;
player and non-player edits survive reload; regenerated and inherited places
share one structural slot; replay reaches the same state hash.

**F0a landed 2026-08-21 — structural world and replayed edits.**
`paredros-world` projects Mesocosm's current verb-neutral `Grown` topology and
exact `Ground` into Paredros-owned stable slots. Surface slots carry generated
settlement, ruin, encounter, or wilds meanings; generated nests become
contained underground dungeon slots. Routes are deterministic and keyed by
the structural slot rather than its current meaning. An inherited history fact
replaces the generated meaning in that same slot while retaining its parent
and routes.

One ordered intent path now records material carves by any `SubjectId` and
inherited site replacement. Saves carry seed, configurable world dimensions,
generator version, base hash, and accepted intents. Restore regrows the base,
refuses generator or base drift, then replays to the same whole-state hash.
The receipt proves same-seed equality, different seeded topology, a connected
route plan touching surface, underground, ruin, and settlement, same-slot
inheritance, and surviving player and non-player edits.

**F0b landed 2026-08-21 — exact movement; F0 closed.** `Movement` owns
subject-keyed positions, ordered inputs, resolved moves or holds, and replay.
It has no destination or activity vocabulary. `Navigation` is a derived
exact-ground query with caller-visible search limits; it expands only
Mesocosm's integer `step` transition over the authoritative `Ground` and is
absent from movement saves.

The four-place itinerary exists only in the receipt. The receipt composes
generic slot routes and navigation queries to enter a roofed underground and
reach the generated ruin and settlement. It checks every resolved transition
against `step`, every stance against exact occupancy, the underground waypoint
for a solid roof, and each surface waypoint for its promised `PlaceId`.
World and movement remain separately owned saves. The movement record is only
ordered spawn and step inputs for a subject-keyed position store; restore
replays them against the regrown world. A mid-route restore reaches the same
final state as a straight run. Twin navigation queries return the same advice
without making that advice authority. The same store and transition move a
second, unplayed subject without a control-specific path.

This closes every F0 done-condition. These sites are world structure, not yet
populated lives or mature settlements; those belong to F2 and F6. The traveller
has position and walker height only. Naming, needs, inventory, injury,
capability, rest, recovery, and death now begin at F1.

### F1 — One embodied life (closed 2026-08-21)

One generated creature becomes named and played. The body owns movement,
perception, inventory, needs, rest, injury, recovery, capability, and death.
Naming begins a life rather than selecting a reusable avatar. Neither camera
nor player control exempts the body from world rules.

**Done when:** one named creature can live, travel, gather, carry, rest, be
injured, recover, and die under the same recorded transition rules that an
uncontrolled creature will later use; save/reload preserves its exact body and
history.

**F1 landed 2026-08-21 — body systems and one transition grammar.**
`Bodies`, `Items`, and `Movement` remain separate multi-subject stores.
`GameState` coordinates them with subject-addressed `GameIntent` and
`GameEvent` records; there is no player or camera field and no alternate path
for a controlled subject. `transitions.rs` owns that shared grammar while
`state.rs` owns only system coordination and replay.

A deterministic world, subject, and body seed establish mass, carrying
capacity, sight range, and recovery capacity. Naming starts its durable life.
Hunger and fatigue advance through accepted actions; sight derives from exact
ground; portable items retain stable identity and location; carrying capacity,
wounds, and fatigue gate capability. Falling creates a body revision, dressing
and rest can recover it, starvation or injury can end the life, and a dead body
rejects further action without admitting that refusal to history.

The F1 receipt composes those systems for one named subject and then repeats
the same grammar for a second subject. Its settlement supplies and ruin are
test fixtures, not production state. A mid-life save stores the world record
and accepted game intents, regrows and replays them, then continues in lockstep
to the same body, inventory, event history, and whole-state hash. This closes
every F1 done-condition. F2 begins autonomous intent production and continued
lives while unobserved; it does not require a second body model.

Verification on 2026-08-21: all fourteen `paredros-world` tests pass and
focused plus workspace clippy pass with warnings denied. Full workspace tests
retain the two pre-existing S3 calibration failures
`a_tag_in_occurs_mid_action_under_the_pact` and
`an_injury_persists_as_a_body_revision_fact` against the live Mesocosm core;
the focused F1 receipt is green and does not alter that separate gate.

### F2 — Other lives (closed 2026-08-22)

Generate named creatures from settlements, ruins, dungeons, migrations, and
encounters. They pursue needs, safety, work, curiosity, travel, and projects
without consulting player identity. Meeting the player changes their history,
not their ontological status.

**Done when:** multiple named creatures continue consequential lives while
unobserved; returning later reveals legible changes caused by them; controlling
or following none of them does not suspend their world participation.

**F2 landed 2026-08-22 — population, projects, and unattended rounds.**
`Population` deterministically regrows named life genesis from world facts and
a configurable population seed. Site origins retain their exact `SlotId` and
site kind; configured migration origins retain both endpoints. Settlement,
ruin, encounter, dungeon, migration, and optionally wild origins all produce
the same `Life`, `Body`, and `SubjectId` facts. There is no NPC type.

`Projects` gives goals stable IDs, explicit targets, active or completed state,
and an ordered completion log. The initial baseline assigns each life one
generated visit goal over the structural place graph. This is a real durable
project law, but not yet material work, construction, employment, or cultural
meaning; those remain F5 and F6.

`Simulation` advances every living population member once per round. Its
deterministic policy prioritizes injury and fatigue safety, hunger, available
work, curiosity about another subject, migration travel, active projects, and
ordinary routine. Every accepted act is the existing subject-addressed
`GameIntent`; a `Decision` adds only the pursuit and a checked pointer into
that log. The scheduler accepts rounds only. It has no observer, camera,
selected subject, or control identity.

Simulation saves retain the game record, population configuration and regrown
hash, project completion intents and regrown hash, round, decision pointers,
and whole-state hash. Restore regrows world, population, and projects; replays
their accepted intents; validates each pursuit against the pointed-to game
transition; and refuses reordered project completions or incoherent decision
pointers.

The F2 receipt covers all named origin classes, sixty unattended rounds, every
pursuit class, changed positions and inventories, completed projects, factual
before/after life reports, exact restore and continued decisions, and a
three-round run in which every subject acts without one being selected. All
eighteen `paredros-world` tests pass; focused and workspace clippy pass with
warnings denied. Full workspace tests retain only the two pre-existing S3
calibration failures `a_tag_in_occurs_mid_action_under_the_pact` and
`an_injury_persists_as_a_body_revision_fact` against the live Mesocosm core.

This closes the F2 done-condition, not autonomous society. The policy is a
deterministic baseline rather than final creature intelligence. `LifeReport`
is factual state plus pointable causes, not memory, belief, gossip, standing,
or explanation under a norm. Those begin at F3.

### F3 — Memory, belief, and standing

Observed events become deeds and remembered claims. Subjects may witness
differently, forget, lie, gossip, revise beliefs, and judge the same act under
different norms. Standing and explanations derive from pointable evidence.
The landed S1 fold is a beginning, not the full epistemic model.

**Done when:** two witnesses form different supported beliefs about one event;
one claim travels to an absent creature; a later consequential answer cites
the observations, reports, and norms that produced it; correction or deception
remains visible in history.

#### F3a — Pointable memory and belief

The first slice establishes the durable epistemic record before selecting a
runtime storage engine. `DeedLog` remains the accepted-event authority. F3a
records observer-scoped observations, actor-supported claims, exact report
transmissions, and explicit self-corrections about those deeds. Each current
belief retains pointable support and the record entry for its latest revision.
Reports may address an absent subject; sending a report does not silently make
its recipient believe it.

The record is append-only and replayable. Current belief is a deterministic
fold over it, never a second serialized authority. Self-correction changes the
current fold while the original contradiction remains visible in history.
Same-tick append order is authoritative and pointable. This typed product
vocabulary is deliberately narrower than an ontology or universal claim
schema.

**F3a done when:** one accepted event produces two differently supported
witness claims; one claim is reported to an absent subject; a contradiction
and the claimant's later correction both remain pointable; save/restore admits
the exact same canonical record through live validation and produces the same
belief fold; and rejected operations leave the record, next identifiers, and
derived view unchanged.

**Landed 2026-08-26.** `paredros-social::epistemic` meets that bounded
condition with eight focused receipts. F3 remains open. Forgetting, rejection
and adjudication policy, deception and intent, versioned norms,
observer-relative standing, and a consequential answer citing its complete
observation/report/norm chain remain F3b work.

#### F3b1 — A consequential answer, design proposal (2026-09-05)

Join observer-owned beliefs to a judgment under an explicit norm revision,
then derive that observer's standing and an answer with its complete support
chain. `Relations::derive` currently reads objective deeds directly; the new
path must use what the holder knows and judges. Facts, interpretations, and
normative evaluations need separate meanings: F3a's `Helped`/`Betrayed` reading
is too coarse to be the entire vocabulary for an embodied encounter.

Versioned norm inputs can initially be authored. F6 owns how practices and
institutions produce them. A correction changes later reasoning after it is
received and accepted; preserve the belief and norm revisions used by earlier
answers. Forgetting governs current recall and use, while history persists.

Contradicting one's belief is evidence of insincerity, not sufficient proof
of deception. Distinguish the transmitted claim, the speaker's beliefs at
that time, communicative purpose where actually recorded, and the hearer's
inference. This corrects the earlier conversational suggestion to infer
deception directly from belief mismatch. Public explanations must respect
what the player could learn; an objective debugger is a separate view.

**Done when:** a report reaches an absent creature and changes its answer;
the player can obtain the stated concern and trace available evidence; a
later correction can change the answer while the earlier account remains
pointable; two norms can evaluate the same known facts differently; and
replay reproduces the answers with the exact evidence and norm revisions.
Do not gate this slice on implementing forgetting or deception in full.

#### Post-F3 runtime projection proof — gated, orthogonal to F4

After F3 supplies real product facts, a product-owned world compiler may lower
accepted Paredros history into dense typed tables, relationship indexes, and
later Conatus, scene, audio, or ECS bindings. This is an orthogonal composition
receipt, not a new fundamental layer and not a prerequisite for unrelated F4
work. No ECS dependency or shared compiler contract is selected in advance.

**Done when:** cold rebuild and incremental application agree; recipe and
source revisions refuse stale or skipped input before mutation; removal
precedes replacement; unchanged truth produces no runtime work; disposable
handles never enter saves; dropping and rebuilding the projection leaves the
authoritative product hash unchanged; and one compiled value reaches a real
query or runtime consumer. An embodied spatial consumer is still required to
challenge Isometry R3.

### F4 — Requests and coordination

Generalize offers and agreements beyond companions: help, trade, shelter,
work, travel, rescue, shared construction, information, and combat support.
An ally is a current relationship between autonomous creatures, not a roster
slot. A lone creature retains access to ordinary world verbs; cooperation
changes cost, scale, knowledge, and safety.

**Done when:** the same grammar can ask a stranger, neighbor, ally, or enemy
for materially different acts; refusal and counteroffer remain complete
outcomes; coordinated action follows agreed terms without exposing a party
command surface.

### F5 — Material life

Gather, carry, store, craft, dig, build, repair, maintain, damage, and destroy
through embodied acts over authoritative materials. A creature can build a
modest home alone. Other creatures and institutions make larger works feasible
without becoming construction permissions.

**Done when:** one creature can establish and maintain shelter from world
materials; cooperation changes the attainable work without changing the verb;
the resulting structure retains builders, materials, purpose, maintenance,
beneficiaries, and later alterations as provenance.

### F6 — Settlement and culture

Settlements emerge from inhabitants, places, works, practices, and remembered
choices. Repetition and enforcement turn practices into expectations, norms,
prohibitions, roles, rituals, and institutions. A RimWorld-like ideology is a
legible projection of this history, not an independent random modifier card.
The player may found, join, influence, oppose, or ignore a settlement.

**Done when:** two settlements develop materially and normatively different
responses from their inhabitants' histories; each visible tenet points to
practices, precedents, places, and beneficiaries; a player choice can reinforce,
contest, or violate a norm without opening a sovereign management screen.

### F7 — Danger

Hostile creatures, factions, fauna, environmental hazards, ruins, and
dungeons use the same bodies, places, perception, material facts, standing,
and coordination laws. Combat and escape are situated world processes, not a
separate arena ontology. Injury, death, property loss, rescue, surrender, and
reputation persist.

**Done when:** one conflict can be avoided, negotiated, escaped, won, or lost
through existing facts; allies act from their own perception and agreements;
the consequences alter bodies, property, relationships, and places without
requiring a combat-only duplicate of any of them.

### F8 — Death and control continuity

Ordinary control remains with the named creature until death. On death, an
existing connected life may be available; a generated outsider keeps the
world playable when no connection exists. Offices, structures, deeds,
reputations, remains, enemies, and tools outlive their holder according to
their own facts.

Creative, accessibility, or difficulty settings may permit tag-in. World
mechanics may allow obscure and consequential possession, domination,
transplantation, cloning, resurrection, exchange, or stranger processes. They
use the same explicit transition boundary as death and leave the original
body, copies, witnesses, and social interpretations in the world.

**Done when:** death continues through both an eligible existing life and the
outsider fallback; neither path rewrites prior history; one non-death
body-changing process records exactly what moved, copied, died, remained, or
became disputed; different cultures can recognize that event differently;
replay preserves the complete control history.

### Presentation is a lens, not a foundation

Every layer needs enough projection to inspect and judge its laws. The final
camera is deliberately open. Third-person, over-the-shoulder, top-down, and
first-person references each privilege different information. Refine the camera
as F1 embodiment, F5 construction, and F7 danger establish locomotion,
perception, reach, verticality, scene density, and how much off-body knowledge
the player should receive. S0's close camera is evidence, not a ruling.

### Current design focus: borgs, learning, and body-shaped equipment

**First read-only slice complete locally, 2026-09-06.** Mark redirected
exploration from extending the crossing toward borg generation and RPG growth.
The [founding plan](2026-07-30_paredros_founding_plan.md#borg-generation-techniques-and-the-character-sheet)
owns the proposed rules and three-lives fixture. Historical entry and the
distinct inheritance routes live in
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`, under
"Historical entry and lineage-shaped lives". Neither adds a mandatory
Mesocosm-to-Paredros-to-Isometry progression sequence.

Work through the following acceptance targets before broad procedural generation:

1. **Three explainable lives.** Author the shared wetland lineage and the
   keeper, surveyor, and repairer histories. Done when every starting part,
   technique, attachment, and consequential relationship has a compatible
   source; each life suggests useful work and danger choices; and the same
   line produces distinct possibilities without hardcoded class exceptions.
2. **One technique, alternative bodies.** Describe arresting a fall through
   grip, adhesion, and equipment. Establish load, support, range, supply,
   timing, and control requirements. Done when intact, damaged, occupied,
   unsupported, and exhausted sources produce explainable different results;
   failed binding preserves state; and losing the source preserves learning.
3. **One readable subject sheet.** Prototype part/item selection, equipped
   bindings, action explanations, learning choices, and known history. Done
   when the player can explain an unavailable action, predict an equipment
   tradeoff, find an alternate implementation, and release a held object
   without needing a development inspector. Include non-spatial navigation.
4. **One consequential improvement.** Join a learning opportunity and bounded
   practice evidence to a deliberate choice, then test body change separately.
   Done when the improvement changes an ordinary task and a dangerous one,
   repeating trivial actions cannot farm unbounded progress, and replay
   restores knowledge, condition, bindings, costs, and their source events.
5. **One historical contrast.** Compare the tradition before and after a
   disruption using dated fixtures. Done when available teachers, records,
   materials, and relationships explain changed starts; future facts do not
   leak backward; and a fresh generated history works without prior wing play.
   This does not require implementing a complete historical world generator.

Before code, map these examples onto current subject/body revision and part
identities. Extend existing owners or name the missing owner explicitly;
the contact fixture's numeric body id must not become a competing durable
subject identity. Binding, learning, historical generation, and presentation
are separate concerns even when one receipt exercises them together.
Attribute scales, learning currency, starting-life customization limits,
prepared-action limits, and historical checkpoint UX remain open decisions.

**Bounded first slice:** an authored, runnable three-lives receipt plus a
read-only action-binding query. Reuse `SubjectId`/`BodyRevisionId` from
`paredros-identity` and `BodyDocument`/`PartId` from Mesocosm for anatomical
addresses. Keep named lives, local traditions, dated sources, and authored
scenario values in fixture data, not production class switches. The query
must explain compatible sources and blockers without changing anatomy,
knowledge, inventory, or resources. Admission of those facts into `GameState`,
durable learning, live action execution, and the graphical sheet remain later
joins. This slice is complete when the example explains each life and tests
cover source chronology, alternative bindings, missing/occupied/severed
sources, resource limits, and stale revision rejection. It is partial evidence
for targets 1 and 2, not completion of the entire RPG prototype.

Ownership mapped for this slice:

| Fact | Existing owner / next join |
| --- | --- |
| Continuing subject and body revision | `paredros-identity::{SubjectId, BodyRevisionId}`; the query must reject mismatched subjects and stale revision references. |
| Addressable anatomy and loss | Mesocosm `BodyDocument`/`PartId`; severed parts retain their addresses. Paredros `Body` has not yet joined this document to its numeric profile. |
| Item identity and possession | `paredros-world::Items`; current possession is whole-subject `Carried(SubjectId)`, so attachment/function facts are caller-supplied query inputs until an equipment transition owns them. |
| Technique meaning and availability | Paredros-owned read-only query over explicit learned facts, source functions, occupancy, support, and resource limits. Mesocosm's ecological capability enum stays unchanged. |
| Authored lives and dated transmission | Example/test fixture data. A fixture source check does not implement generated world history or player-relative memory. |

The receipt must distinguish possessing a part from that part supplying a
function, and a supplied function from sufficient capacity in this situation.
The same distinction applies to owning a harness versus having a complete,
attached line with a reachable support. Query answers are previews; future
execution must revalidate against current state before committing costs.

The first query is deliberately concrete: `arrest_fall` explains grip plus
line, adhesion, and harness plus line. Its supplied equivalent arrest demand
and capacities use milligrams, reach uses body-scale voxels, and adhesive
units are fixture doses. It does not calculate fall dynamics or define a
general procedural technique language. A later usable limb/item is selected
when an earlier candidate is occupied, severed, too short, or too weak.

**Read-only sheet implemented locally, 2026-09-06:** uses this same query for the
authored lives. Show named parts, exact sources, costs, knowledge blockers,
and alternative bindings through both a part list and action inspection.
Keep scenario comparisons separate from player control selection. Before
making equipment/learning editable, define their admitted `GameState` facts
and replayed transitions; the query's caller-supplied projections must not
silently become a parallel save authority.

This native prototype is a separate authored scenario inspector, not a
player-control selector. `paredros-world` supplies read-only presentation
rows from body/query facts; `paredros-room` owns selection, navigation,
layout and rendering on the existing Netrender host. The three-lives fixture
has one public demo-data home so tests, the text receipt and native inspector
read the same facts. It is not a new procedural start generator.

Done for this slice when named living/severed parts can be selected without
spatial picking, action selection identifies sources and plain-language
blockers/costs, mouse and keyboard navigation change only view state, long
details remain accessible, and native captures show the keeper, untrained
surveyor and injured repairer correctly. Editable equipment, learning,
authoritative anatomy admission and action execution stay outside this slice.

The dry crossing remains a useful contact fixture. Mark's play feedback found
that the held plank could leave the player stuck in the gap and that the blue
structure's purpose was unclear. The 2026-09-06 narrow fix separates release
from placement: E picks up/places, G lets go even while falling or obstructed.
Released boards retain their location and descend under fixture-scale vertical
gravity with swept landing against solids/bodies. This is not full rigid-body
tumbling or impact damage. Player confirmation, visible exit guidance, and
body-specific affordance explanations remain open acceptance targets.
Earlier automated route checks do not override that feedback.
Charge and inhabitants are deferred while the borg design is explored.

### Embodied prototype alongside F3: proposed 2026-09-05

The [founding plan's player-experience proposal](2026-07-30_paredros_founding_plan.md#player-experience-body-place-and-other-lives)
owns action feel, progression, world differences, and the player-facing
knowledge boundary. F0-F8 remain semantic milestones. A small playable
cross-section may exercise preliminary F4/F5/F7 verbs before those milestones
are complete, so interaction can inform the social design.

1. **Body and contact.** One inhabited location, two materially different body
   forms, a tool, an innate action, obstacles, and a readable opponent. Prototype
   movement, aiming, guarding, recovery, local damage, and one action chain.
   Done when reach and footing change outcomes, an injured capability visibly
   changes available actions, and escape/recovery works. Compare camera distance
   and assistance settings in play; this is an action experiment, not F7 closure.
2. **One rule, two uses.** Start with authored candidate world rules and apply
   the same mechanism to a practical task and a dangerous situation. Done when
   a player can learn the connection from in-world feedback and predict a new
   use. Repeat the location under a second rule whose structure changes the
   solution. Procedural selection waits for these readable authored examples.
3. **One encounter with social consequences.** Add two autonomous inhabitants,
   one disputed resource or passage, and physical signals such as offering an
   object or lowering a weapon. Connect the outcome to F3b1. Done when a player
   can negotiate, help, withdraw, or fight through ordinary actions; a partial
   witness report influences a later request; and the player can learn and
   contest the reason without reading private minds.

The encounter is an acceptance fixture. Its cast and story must not become
special cases in production rules. Each stage needs an interactive receipt and
appropriate replay/invariant checks before its done-condition closes. Shared
spatial and capability machinery should be consumed where compatible; game
timing, damage, knowledge admission, and social interpretation remain owned
by Paredros. Huge organisms and alternative world topology remain separate
later probes with the bounded targets stated in the founding proposal.

#### First encounter: the damaged crossing

**Dry fixture implemented, 2026-09-05; player acceptance open and wider encounter remains a design draft.** A concrete fixture for the embodied prototype,
with authored terrain, bodies, and two candidate world profiles. It tests
whether ordinary actions can produce a situation worth remembering. The
crossing, inhabitants, and outcomes below are scenario data, not production
types or required story beats. This section owns the encounter details; the
founding proposal owns the wider player experience.

**Why you are here.** Rain is approaching. Across a shallow cut is a roofed
resting place beside a workshop. You can see its shelter and dry ground from
the approach. Rest would be useful, but exposure is forgiving in this fixture:
you can stop to experiment or turn away. The player can pursue another goal;
there is no mandatory accept-quest step or completion reward screen.

**The place.** A short deck crosses the cut. A damaged connection has brought
an energy-carrying living fibre into contact with its wet section. An upper
rock lip offers an awkward route around it. The shallow channel offers a
longer route to the far bank, interrupted by the same wet connection. A loose
dry board can span a small section; a sheltered patch on the approach lets
the player recover from a failed attempt. The roof, workshop, and collector
remain ordinary structures after the encounter.

**The inhabitants.** One creature is maintaining the collector and drying
rack that support its work. Another is hauling a load toward the far bank.
Working roles identify the fixture here; names and subject identities are
ordinary generated data. The maintainer wants the live path isolated before
anyone disturbs it. The hauler wants a usable route and may accept losing
power temporarily. Both value their own safety. Neither is automatically an
enemy. The maintainer can signal a warning, intercept interference, accept
help, retreat, or defend itself according to the current situation.

Run their routines without player intervention: inspect damage, fetch
material, wait or seek an alternate route, and resume work when possible.
Their needs and capabilities should determine those transitions. Arrival
does not freeze them into dialogue roles or start a mandatory storm countdown.

##### The rule you can learn

First author a small charge profile: connected wet fibre carries stored
charge; a dry mineral or wooden spacer interrupts that path. A release
consumes a bounded stored quantity. The collector, reservoir, and connected
material patches suffice for the first model; full weather or fluid simulation
is unnecessary. Rain onset is an explicit scenario input with a visible cue.

Before the dangerous section, a harmless charged strand and loose dry spacer
permit observation and experimentation. A travelling pulse, small visible
motion at contact, and a matching sound expose the path. Interrupting the
strand stops those cues. Inspection initially says what the subject can
observe, such as "pulses reach the wet section"; after testing it can retain
"dry spacers interrupt this connection" as a discovered relationship.
Equivalent shape, motion, text, and sound cues avoid reliance on colour alone.

Three plausible approaches emerge from these facts. Isolate and repair the
connection, preserving the workshop supply; drain the reservoir to make a
temporary crossing, interrupting the drying rack; or use the board or upper
lip and leave the repair to its owner. Passing the maintainer is physically
possible. Its warning and preferences do not create an invisible access wall.
Routes have different effort, bodily requirements, exposure, and consequences.

##### Two bodies, shared intentions

| Fixture body | Spatial strengths | Limits and equipment | Local power use |
| --- | --- | --- | --- |
| Low shell crawler with a gripping forelimb | Stable brace, pushes and carries a board, fits under low obstacles | Short reach, poor climb; shell supports a harness and forelimb holds a tool | A carried discharge probe can drain a contacted source if equipped and understood |
| Light climber with anchoring tendrils | Reaches an upper hold, tethers a loose object, can catch a fall | Limited load and stability; taut tendril exposes a vulnerable connection | In the charge profile, a storage organ can accept and release a limited charge through contact |

These are test bodies, not final species or classes. Each gets a viable
low-risk route and a chance to use its strengths. The climber is neither
obliged nor universally safe to conduct charge through its body. Inspecting
its known capacity and current storage must expose overload risk. The crawler
can use mundane material handling without acquiring the climber's anatomy.
Switching test bodies happens by restarting a fixture, not by free switching
in ordinary play.

Begin with move/aim, use equipped action, defend or brace, inspect, and signal.
An action picker exposes only learned, supported actions and their source.
Show an interrupted action's reason, such as lost anchor or occupied grip.
Commitment and recovery are visible in the body; severe damage changes a
specific capability. The first injury target is reduced reach or an impaired
grip with a recoverable path, rather than mandatory limb loss.

##### One possible encounter, not the required sequence

The player sees the warning, experiments with insulation, then drains the
reservoir to clear the deck. The hauler crosses. The maintainer emerges from
behind the workshop and sees the disconnected supply and the player holding
a tool. It did not see the dangerous contact or the safe crossing. It signals
objection and approaches to stop further work.

The player can put the tool away, indicate the damaged section, offer a
spacer, continue working, withdraw, or attack. An offer states an intended
transfer; the recipient may accept, refuse, or suggest different terms.
Pointing directs attention to inspectable evidence. It does not automatically
prove the player's motive. A warning or surrender must be perceived before
it can affect a decision, and remains subject to that creature's willingness.

If fighting occurs, the work site provides the spatial test: the tool's sweep
needs clearance, bracing changes the effect of a shove, and a taut tether can
arrest a fall. The charge path affects everyone exposed to it. An interrupted
strike and a dropped tool should create space to withdraw. Test controlled
attack trajectories with contact and impulse consequences before adding
full-body physical animation. Injury, dropped property, and repairs remain
after the conflict; the fixture does not reset the place when combat ends.

The hauler may later report that the player made the crossing usable; the
maintainer may report damage to the supply. An absent third subject at a later
meeting supplies F3b1's receipt: its response to a tool loan changes only after
information reaches it and it judges that information. Both reports may be
factually compatible. Restoring power need not erase a complaint about acting
without permission, and a safety norm can value the same act differently.
The fixture must also exercise an actual factual correction, with delivery
and belief revision separate from disagreement under different norms.

##### What the player sees

In ordinary movement, keep the body, relevant action readiness, and immediate
physical cues prominent. A brief signal caption might read "warning: wet
deck" when the communication is understood. Before understanding a signal,
show its observed form without asserting its meaning. Common controls and
fixture communication are initially taught; inventing an entire language is
outside the first prototype.

Inspection of the maintainer can say "blocking the deck; carrying an insulating
tool." After conversation, its concern can be recorded as "wants the supply
kept intact." A later refusal can say "I was told you cut their supply," with
the source shown if disclosed. Following the journal entry shows who said
what, when it was learned, and what has subsequently been checked. An inferred
claim remains labelled as inferred. Unheard private reports and objective
intent remain absent from this player view.

The player-facing history might therefore contain "I drained the reservoir,"
"the hauler crossed," and "the maintainer objected." It must not fabricate
"everyone trusts me less." An optional development view exposes the full
causal chain for debugging separately. Pause or slow inspection, target
assistance, camera distance, and damage/recovery settings are fixture options
to compare; opening a diagnostic panel must not be necessary to play.

##### Progression you can feel here

The first gain is a learned interaction and an altered route. A tool found,
borrowed, made, or traded for can carry that interaction elsewhere. Training
can later improve timing and control; acquiring a suitable organ or symbiont
can add storage or sensing. Assistance may make the maintainer willing to
teach, but independent experimentation and other sources remain possible.
Killing the maintainer does not automatically grant its technical knowledge.
Fighting, helping, and bypassing are not assigned universal moral scores.

Repeat the layout in a second authored world profile before procedural
generation. In the resonance profile, dry rigid connected members carry a
pulse, while a compliant joint damps it. A dry rigid spacer that electrically
insulates in the charge profile may complete the resonant path here. Tools, anatomy,
local industry, and cues must be admitted coherently for that profile; an
electrical organ is not silently renamed into a sonic organ. Success means a
player observes the changed rule and finds a different material solution in
both work and danger. The two profiles need no arbitrary mid-world switch.

##### Build order and acceptance

Implement a small encounter harness over the existing Paredros owners after
checking suitable shared spatial/capability APIs. The fixture owns layout,
cast, initial facts, selected rule profile, and replay inputs. `paredros-world`
owns admitted material/body/action outcomes; `paredros-social` owns knowledge,
judgments, requests, and agreements. The host routes observed outcomes to
those owners and projects their results into the existing rendering stack.
Scene objects and displayed text never become save authority.

1. **Dry movement and contact.** Build only the cut, deck, board, recoverable
   landing, and two body configurations. Prove movement, contact, brace,
   tether, one timed attack, and capability injury with an adjustable camera.
   Done when each body has a readable traversal solution, a failed attempt
   permits recovery, and reach/footing changes the same action's outcome.
2. **Local charge interaction.** Add the finite reservoir and a bounded set
   of conductive connections. Done when isolated/reconnected/drained states
   are persistent, the same rule explains a tool use and a dangerous contact,
   and untrained players can predict a new application after experimentation.
3. **Inhabitants and evidence.** Add the maintainer, hauler, and later absent
   recipient fixture. Done when autonomy continues without player action,
   signalling can change an encounter, partial observation yields distinct
   supported accounts, and delivery/correction changes a later answer with
   inspectable evidence. Pair this with F3b1 rather than a complete culture AI.
4. **Second world profile.** Reuse the encounter's constraints under resonance.
   Done when material connectivity changes both the safe route and a combat
   application, with coherent abilities and cues, without scripting outcomes
   by profile name. This is the admission gate for procedural combinations.

Record accepted inputs and resolved effects at the simulation step, with
body/world/rule revisions sufficient to replay them. Establish a pinned
runtime replay baseline before promising cross-device numerical equivalence.
Meaningful automated checks cover interrupted actions, capability limits,
finite resource use, evidence visibility, and save/replay continuity. Headed
play must separately test camera, action readability, recovery, and whether a
player can explain another creature's response. The dry slice's implementation
and verification are recorded below; wider encounter acceptance remains open.
If the dry movement pass is uninteresting, revise its controls and space
before expanding the social or procedural scope.

##### Dry fixture implementation (2026-09-05)

`cargo run -p paredros-room --bin crossing` opens the authored dry cut, loose
board, overhead hold, broad exit stair, and stationary practice body. The
crawler can carry the board; the climber can attach a tether. Both can use the
channel and stairs. Keys 1/2 restart the entire fixture with a different body;
they are development controls, not a body-switching rule for ordinary play.
WASD is camera-relative, arrows/right-drag orbit, wheel adjusts distance,
Shift braces, E handles the board, Q attaches/releases the tether, Space
strikes, R restores an impaired capability, and F requests one explicit
practice counterstrike. This last input is a training fixture, not NPC agency.

The ownership review found `conatus::BodyWorld::move_character` already in
the pinned Mere revision `d82afa17`. The initial local stepping solver was
replaced before acceptance. Conatus now resolves 3D character movement,
grounding, collision, and short steps; `contact/spatial.rs` lowers authored
geometry to that API. For this small fixture the spatial query world is
rebuilt per move. Retained bindings need profiling before this grows to a
larger world. `paredros-world::ContactWorld` owns profile limits, action
timing, carrying/placement, a bounded tether constraint, injury, recovery,
and fixed-step input replay. The host and meshes remain derived.

This contact record is a separate forcing probe, not yet joined to durable
`GameState` subjects, body revisions, needs, death, or epistemic evidence.
The board remains an axis-aligned, kinematically carried/placed collider;
rotational rigid-body dynamics and destructible voxel anatomy are open.
Capability recovery currently repairs impairment without replenishing
integrity. A grounded brace halves strike damage and preserves capability.
The trace limit is 72,000 ticks; the host stops simulation and offers a
fixture restart when it fills. This is not a general long-lived save format.

Before advancing to charge, playtest camera visibility, the usefulness of
tethering, attack timing, and whether the differing routes are interesting.
An overhead traversal solution is not established merely by a tether catch
test. Charge, autonomous inhabitants, witnesses, communication, audio,
generated anatomy, and the resonance profile remain deferred.

Verification: `cargo test -p paredros-world --tests -p paredros-room --test
crossing --bin crossing --lib --offline --locked` passes **53 tests**, with
two existing opt-in physical GPU tests ignored. Coverage includes board
support across the actual cut, both bodies escaping via the stair, full
save/restore state equality, attack facing/obstruction/cooldown, grounded
brace versus impairment, tether fall arrest, and invalid record admission.
The room library also checks with `--no-default-features`.

Headed smoke captures for crawler/climber show 2,839/3,454 distinct colours,
movement, equal replayed state, one graph encoder/submission boundary, zero
Renderling-internal submissions, and no scoped validation error. Local
artifacts are `Code/testing/paredros/crossing/run-1788660172168` and
`run-1788660180412`. These are render/replay receipts, not a complete playtest.
The final host caps smoke at exactly 210 ticks (the earlier captures ended
at 212/213 because a catch-up frame could overrun the stop).

Live window checks verified readable geometry/text, pickup, and whole-fixture
body restart. They exposed a dropped quick movement tap; the host now retains
the released movement state for one simulation tick. The user subsequently
traversed to the practice body during checking, so automated input stopped.
Combat feel, tether-route usefulness, focus/resize behavior under sustained
play, and subjective traversal acceptance remain unclaimed.

## 5. Stop rules

- The canary above all: ordinary play stays with one named creature until
  death. Free roster selection and real-time puppeteering of a party are drift.
- Allies are contingent. Building, exploration, and ordinary survival may be
  harder alone but never require a friendship flag.
- Willingness and embodied execution stay separately owned.
- No universal character schema or `same_person` flag: control, subject, body,
  social identity, role, lineage, and player history remain distinct.
- S0-S3 are receipts. Do not optimize the game around reenacting their fixture
  stories or closing their playtester judgments.
- Nothing here grows an engine organ Mesocosm already owns; the room
  probe *consumes* near-tier kinematics and the tenant seam, it does not
  fork them.
- The consequence envelope (intent, adjudication, explicit consequences,
  durable facts, projections) is **anticipated, not extracted**: Paredros
  implements its own transition layer concretely, and extraction waits
  for the third sovereign proof per the general model's evaluator rule.
- **R4 lives here now** (inherited 2026-08-07 from Mesocosm's archived
  body-pipeline plan): the extraction review — what, if anything, becomes
  a shared runtime crate, with Paredros as the second consumer or not at
  all — fires **after S3**, the sortie being the wing's ruled extraction
  trigger. Done when the seam is either extracted with two consumers
  named, or explicitly declined in writing.
  **DECIDED 2026-08-10**: see
  [the extraction review](2026-08-10_r4_extraction_review.md). Tenancy
  pushed up to netrender (landed there 2026-08-10), identity promoted in
  place, grammar refused on principle, the adventure-mode frame adopted
  symmetrically.

## 6. Findings

- **2026-09-06:** the three-lives query reuses subject/revision and anatomical
  part identities without changing Mesocosm's ecological capability enum.
  Anatomy presence, function, and situational viability need separate checks.
  Selecting the first matching grip or line hides valid later sources; both
  grip and line reach must constrain the result. Learning facts are retained
  when anatomical sources are severed. The authored history fixture checks
  source identity, acquisition chronology and place; it is not a historical
  generator or a player-memory model.

- **2026-09-06:** `contact/mechanics.rs::toggle_board` requires a supported,
  clear placement destination; `contact/spatial.rs` can halt movement when the
  carried board lacks clearance. Release must bypass both placement checks and
  remove the carry constraint before movement. Contact recording version 2 adds
  the release input; older binary fixture recordings are explicitly rejected.

- **2026-09-05 (borg design):** `bodies.rs` has a small generated numeric
  profile and durable condition; `contact.rs` has two authored presets and
  fixture actions. Neither establishes learned techniques with alternative
  anatomical bindings or a body-shaped equipment sheet. The new proposal
  names those missing joins without claiming the contact probe implements them.

- **2026-09-05 (dry contact ownership):** Mere's pinned Conatus supplies the
  3D character controller; Seiche's inspected scene API is Rapier 2D and
  Mesocosm's inspected `places::step` is integer Ground navigation. The dry
  fixture consumes Conatus rather than promoting its experimental local
  movement math. Authored fixtures and consequence rules remain Paredros-owned.

- **2026-09-05 (player-experience design):** live `epistemic.rs` supplies
  observer-scoped evidence and exact report history; `relation.rs` still
  derives standing from objective `DeedLog`, using `DeedKind::weight` in
  `deed.rs`. The missing join is from known facts through judgment to action.
  The existing transition and room receipts do not establish responsive
  spatial combat or player understanding of a generated rule.
- **2026-09-05 (encounter design):** `paredros-world/src/transitions.rs`
  currently admits generate, name, move, observe, take, eat, rest, fall, and
  wait. It does not yet supply timed attacks, anchoring, bracing, equipment
  use, charge connectivity, or communication acts. The crossing plan is a
  proposed extension to those owners, not a claim that its verbs already
  exist behind the room renderer.
- **2026-08-26 (F3a planning):** the existing seams already express the
  authority split. `GameEvent::Observed` records a witness and visible target
  but not a proposition or another subject's belief. `DeedLog` is append-only;
  `Relations::derive` folds one globally visible deed log; and `Premise` cites
  objective deed evidence. F3 therefore needs an observer-scoped epistemic
  record and derived belief fold without rewriting any of those owners.
- **2026-08-26 (F3a implementation):** the admitted record now validates deeds
  against `DeedLog`, scopes direct and reported evidence to the actor who holds
  it, preserves exact report propositions and forwarding chains, and treats
  correction as claimant-owned revision. Restore decodes only ordered entries
  and replays them through live admission, refusing reordered identities,
  noncanonical supports, or changed semantics. Deception was removed from this
  slice because contradiction alone cannot establish intent.
- **2026-08-26 (runtime projection):** Paredros's existing ordered maps and
  replay receipts are the oracle for the first semantic lowering. Bevy, Flecs,
  hecs, SHACL execution, and differential dataflow remain candidates behind a
  later measured profile; choosing one in F3a would test a library before the
  product facts force its shape.
- **2026-08-08 (S3):** the wound threshold is Paredros's own constant
  (`SAFE_FALL = 4`), not an import: the near tier keeps `COMFORT_DROP`
  private, and the ruling is independent anyway — falls hurt here even if
  the walker's willingness to take them changed upstream. If mesocosm
  ever exports the constant, tying them is a decision, not a cleanup.
- **2026-08-08 (S3):** only the home body downs; wounds are uniform.
  Downing is a control fact — it is what the pact watches for — and a
  tagged-in successor or a companion grits through the same wound,
  because two downed bodies at a cliff base with the healer among them is
  a deadlock, not a story. F7/F8 will revisit what another creature's fall,
  injury, and death can mean.
- **2026-08-08 (S3):** the grown terrain's pockets are one-way at a
  one-voxel climb, everywhere the survey looked: what a walker descends
  past comfort it can never re-climb. The dig rule (stuck four ticks →
  carve a head-height notch toward the goal and climb in) turned that
  from a scripted-detour problem into a law: the sortie cannot strand,
  the verb is mesocosm's `carve` consumed as-is, and the hewn rock comes
  home with the salvage. This is also the first time Paredros *writes*
  the world rather than reading it.
- **2026-08-08 (S3):** the sim march is goal-seeking (waypoints and
  parts), not per-tick input, so determinism is structural and the
  replay receipt is exact; "real-time embodied action" is deliberately
  the headed half's burden. The calibration survey (`tests/calibrate.rs`,
  ignored) is kept: `SITE_OFFSET` and `WAY_OFFSETS` are facts about
  seed 4242's terrain, chosen from a printed table, and a regrown world
  that stops having that scarp fails the receipts loudly.
- **2026-08-08 (S3):** `Wound` lives in `paredros-sortie` for now. The
  combat/movement owner is its natural home, but which crate that *is*
  (the room probe grown up, or an extracted seam) is exactly R4's
  question, so the type waits where the joint receipt made it.
- **2026-08-08 (S3):** depending on `paredros-room` brings the render
  stack into the sim crate's build tree unused. Accepted for now — the
  room is the world's owner and S0's path-dep posture was already ruled —
  but a headless split of the room crate is available if it starts to
  cost.
- **2026-08-08 (S2):** residence wanted to be a store and is a derivation
  instead. The first sketch had `offer_home` writing a tenancy and an
  `end_tenancy` closing it, which would have made the settlement a second
  copy of agreement state with a synchronization duty. Deriving currency
  from the agreement (`a tenancy is current while its agreement stands`)
  deleted the duty: ending the arrangement through `Society` alone moves
  the tenant out, and `moving_out_is_the_agreement_ending` exists to keep
  it that way. Same shape as standing folded from the deed log.
- **2026-08-08 (S2):** the settlement lives in `paredros-social` as a
  module, not a crate. It is willingness-adjacent state (what agreements
  change) with no combat surface, so the two-owners wall is not in play;
  a module is the default until an enforced boundary is needed. F5/F6 may
  revisit when material production and culture arrive.
- **2026-08-08 (S2):** dwellings are founded, not negotiated: `found` is
  a world act that moves nobody's standing, and nothing gates who may
  offer a dwelling on the settlement's behalf. That permission question
  belongs to F4/F6, recorded here so its absence reads as a decision
  rather than an oversight.
- **2026-08-08 (S1):** the two-owners rule is a dependency fact, and that
  decided a crate boundary. `SubjectId` cannot live in either owner: if it
  lived in the willingness crate, the combat owner would have to depend on
  willingness to name a person, which is the wall breached from the other
  side. `crates/paredros-identity` is therefore a foundation crate that
  knows about neither, holding `SubjectId`, `BodyRevisionId`, `FactionId`,
  `OfficeId`, `Tick`, the facet stores, and the control pointer. It is the
  one place the four facts are named.
- **2026-08-08 (S1):** storing standing would have made the explanation
  surface a second job. Folding it out of the deed log on each call made
  the surface honest for free: a `Standing` carries the ids of the deeds
  that produced it, so a premise can only ever cite entries that exist.
  The test asserts on ids resolved back through the log, so a rule that
  kept answering correctly while inventing its reasons would fail.
- **2026-08-08 (S1):** refusing weighs (0, 0) in the deed table. That is a
  ruling rather than an omission: refusal is a legitimate answer in this
  vessel, so it costs the refuser nothing, and
  `a_refusal_is_an_outcome_and_costs_the_refuser_nothing` pins it.
- **2026-08-08 (S1):** the played character is admitted to the society the
  same way anyone else is, because succession means the distinction is
  temporary. There is no second record shape for the one being played, and
  the willingness rule never asks who the player is.
- **2026-08-08 (S1):** `paredros-social` takes a **dev**-dependency on
  `mesocosm-core`, for `snapshot::encode` and `hash_bytes` and nothing
  else. Judgment call, recorded because it brushes two rules at once: the
  stop rule forbids growing an organ Mesocosm already owns, so the replay
  receipt consumes the wing's hash rather than writing a second FNV-1a,
  and the shipped dependency list of the willingness owner still contains
  only `serde` and `paredros-identity`.
- **2026-08-08 (S1):** §1's controlled-subject session state landed and S1
  does not use it. `Control` is tested on its own (tag-in, tag-out,
  succession, replay from intents); S3 consumed part of it, and F8 owns its
  general control-continuity role. Recorded so it
  is not rediscovered as missing.
- **2026-08-08 (S0):** the stop rule holds in practice. The room probe
  writes no terrain, no kinematics, and no mesher: `Places::grown`,
  `Ground::grow`, `Ground::carve`, `near::step`, `Ground::stands`, and
  `mesh_volume` are all consumed as they stand. What Paredros wrote is
  the room's siting rule, the trace, the camera, and the save
  discipline.
- **2026-08-08 (S0):** `near::step` refuses a blocked move rather than
  sliding along a full-height wall, so a body in a sealed chamber holds
  position at the wall. The trace relies on this, and
  `probe::tests::the_trace_ends_pressed_against_walls` asserts it, so a
  kinematics change upstream that starts letting bodies through walls
  fails here loudly.
- **2026-08-08 (S0):** the execution plan cited a hillside-hunt
  reference at `mesocosm-core/src/places/hunt.rs` (test `the_chase`)
  that does not exist. The nearest real stage-scans are
  `places/near.rs::tests::a_stance` and
  `places/bricks.rs::tests::hills_block_sight_and_tunnels_grant_it`;
  this probe's hunt is modelled on those.
- **2026-08-08 (S0):** the probe's crate uses path dependencies on
  mesocosm, netrender, and the local renderling fork, against the
  workspace convention of branch-tracked git deps. The fork has no
  published home, and S0 exists to consume all three exactly as they
  stand on the machine. Revisit when the probe grows into a shipped
  target.

## 7. Progress

- **2026-09-06: native Body/Actions inspector.** Luna supplied the read-only
  `SubjectSheet` projection and shared fixture home; Terra supplied the native
  `body_sheet` host. Root integrated them, corrected selection tests, separated
  list/detail scrolling, guarded row hit tests, and repaired inspection layout.
  The combined world/room library, body-sheet/crossing binary, contact,
  crossing and three-lives gate passed **52 tests**. Selection and navigation
  preserve supplied facts; severed parts remain inspectable, stale revisions
  block availability, and the untrained surveyor retains anatomical capability.
  Reproduce with:
  `cargo test -p paredros-world -p paredros-room --lib --bin body_sheet --bin crossing --test three_lives --test crossing --test contact --test contact_actions --locked --offline -j 2 --target-dir target-contact`.
  Launch `target-contact/debug/body_sheet.exe`: 1/2/3 compare lives, Tab switches
  lists, arrows select, mouse clicks select rows, wheel scrolls the hovered
  list/detail area, PgUp/PgDn scroll inspection, and Escape quits.
  `PAREDROS_BODY_SHEET_SMOKE=1` captures three authored states and exits;
  `PAREDROS_BODY_SHEET_OUTPUT` selects its output directory and `PAREDROS_FONT`
  can supply a font file. The prototype scales a fixed 1280x720 canvas.
  It uses list/source cross-highlighting, not a spatial anatomy diagram.
  Player usability acceptance, generated history, action execution, and durable
  equipment/learning remain open. Existing unused-Vello-patch and room
  dead-code warnings remain unrelated. The final gate passed again after label
  cleanup. Native smoke exited successfully and root visually inspected all
  three 1280x720 captures at
  `Code/testing/paredros/body_sheet/run-1788673253414/{sedge,tremor,mend}.png`:
  Sedge shows harness equipment and attached-part sources; Tremor shows missing
  training and line equipment; Mend preserves the selected severed limb while
  grip/adhesion alternatives remain available. These are native rendered-frame
  receipts; physical mouse/keyboard usability and a spatial body view remain
  unverified/unimplemented respectively. Work remains local and uncommitted.

- **2026-09-06: three-lives/action-query local slice.** Terra implemented
  `src/technique.rs` and its tests; Luna authored the three-lives fixture and
  example. Root integrated/reviewed them, repaired the harness blocker type,
  consolidated example/test inputs, and tightened source chronology and
  symbiont donor references. The combined world/room library, contact,
  crossing and three-lives gate passed **43 tests**. The nine new query/fixture
  tests cover stale/mismatched/unlearned context, alternative sources, finite
  line reach, missing support/line, adhesive exhaustion, loss without erased
  knowledge, source chronology/place, and the exact example inputs. Querying
  preserves anatomy, knowledge and resources. This is partial evidence for
  targets 1 and 2; ordinary livelihoods are narrative, and at that gate GUI, execution,
  durable equipment/learning, and historical generation remain unimplemented.
  The final three-lives tests passed again after integration cleanup. Built
  and ran `target-contact/debug/examples/three_lives.exe`: keeper selects a
  free second grip or its harness; surveyor reports `NotLearned`; repairer
  selects a surviving grip or its incorporated adhesive organ. Example build:
  `cargo build -p paredros-world -p paredros-room --example three_lives --locked --offline -j 2 --target-dir target-contact`.
  Existing unused-Vello-patch and room dead-code warnings remain unrelated.

- **2026-09-06: crossing release fix implemented locally.** Added independent
  G/release input and HUD hint while preserving E/pickup-placement. Release
  wins over simultaneous interaction and does not require support or carry
  capability. Released boards use vertical gravity, swept landing and body
  clearance so walking out from underneath remains possible. The focused
  `contact`, `contact_actions`, crossing integration and crossing binary suites
  passed 21 tests, including the actual held-plank trap, subsequent exit route,
  placement regression, airborne release, support removal and replay. Command:
  `cargo test -p paredros-world --test contact --test contact_actions -p paredros-room --test crossing --bin crossing --locked --offline -j 2 --target-dir target-contact`.
  Contact save grammar is now v2. Headed player confirmation remains open.

- **2026-09-05 (borg design):** recorded classless capability foundations,
  historically transmitted traditions, alternative technique implementations,
  deliberate learning, and a body-shaped subject sheet. Added a three-lives
  fixture and acceptance ordering. Recorded Mark's crossing feedback as open
  recovery/legibility failures; paused charge/inhabitant expansion. Doc-only
  design work, not new generation, progression, or historical-entry code.

- **2026-09-05 (dry contact implementation):** added the `crossing` host,
  authored scene/HUD, replayable contact rules, and Conatus movement adapter.
  Automated room-route checks cover the carried board and recovery stairs
  for both bodies; 53 tests pass. Headed render/replay and limited live-input
  checks are recorded above. Replaced the initial local movement solver with
  Conatus, corrected a live-input tap loss, and brightened fixture lighting
  without changing the older room's torch policy. Player acceptance and
  later encounter stages remain open.

- **2026-09-05 (encounter draft):** specified the damaged crossing as the
  first playable design: a practical destination, two body configurations,
  multiple physical approaches, independent inhabitants, partial witness
  knowledge, and persistent consequences. Set dry movement/contact as the
  first implementation slice, then charge, social evidence, and a distinct
  authored resonance profile. Updated the index; implementation remains open.
- **2026-09-05 (design):** recorded the embodied sandbox proposal and F3b1
  judgment/answer seam. Added a proposed playable sequence covering body and
  contact, a world rule used in work and danger, and a witnessed encounter.
  These are design changes; no new gameplay implementation or acceptance
  receipt is claimed.
- **2026-09-05 (RG3e):** Renderling commit `3683dd6` exposed a caller-owned
  complete-stage encoder path for direct draws. The normal room tenant now
  records geometry, the bloom chain, tonemapping, and optional debug work into
  one encoder and Paredros performs exactly one tenant submission. The physical
  RG3c receipt reports 20 render passes, zero copy commands, zero internal
  submissions, one caller submission, and one separate Netrender graph
  submission; its final master still byte-matches the legacy composition and
  contains 466 distinct colours. The D1 shared-depth binary compiles through
  the same tenant draw path. Compute-culling stages remain on Renderling's
  legacy self-submitting path and are explicitly refused by `Stage::encode_into`
  until their preparatory compute work can join the caller encoder.
- **2026-09-05 (later):** the RG3d shared-device recovery lifecycle landed.
  Initial boot and recovery now use one constructor for the compatible adapter,
  device, queue, Renderling tenant, optional DDA tenant, Netrender composer,
  frame-health state, and host callbacks. `RebuildAll` suppresses the faulting
  attempt, drops that complete GPU-owned set, preserves the winit window and
  surface, boots a later device generation, reconfigures the surface, and
  resumes drawing. The headed `PAREDROS_RG3_REBUILD_PROBE=1` receipt latched a
  synthetic uncaptured-error disposition in generation 1: attempt 1 made no
  surface acquisition or presentation call; generation 2 acquired and
  presented attempt 2. The machine-readable receipt is
  `Code/testing/paredros/rg3d_rebuild_all_headed.json`. This exercises the real
  rebuild and surface paths after a synthetic shared fault. It does not claim
  to manufacture or measure physical device loss.
- **2026-09-05:** the RG3b headed frame-health gate closed for the room's real
  winit surface. `PAREDROS_RG3_HEADED_PROBE=awaited` injected one invalid
  buffer copy into the existing tenant validation scope: attempt 1 was
  attributed to `paredros-room` / `renderling::Stage::render (opaque)` and
  suppressed before `surface.get_current_texture`; attempt 2 acquired and
  presented. The `optimistic` run acquired and presented attempt 1, observed
  its validation result without blocking, suppressed attempt 2 before surface
  acquisition, then acquired and presented attempt 3. The machine-readable
  receipts are `Code/testing/paredros/rg3b_awaited_headed.json` and
  `rg3b_optimistic_headed.json`. This proves native acquisition/presentation
  suppression and recovery on the shared device. It does not promise rollback
  of renderer-internal bookkeeping already performed before the host gate.
- **2026-08-28:** the shared brick ABI crate was renamed `conatus-brick` →
  `modulus` (ruled by Mark; the architect's base unit of measure, and the
  layout math is modular arithmetic) and published to crates.io; this repo
  repinned to `33f9b6b6`. Dated receipts above keep the name they were
  taken under.

- **2026-08-26 (latest):** V1b landed — the stable resident brick cache.
  One capacity-fixed `conatus-brick` map (new platform commits at
  `bd8f0044`, consumers repinned) retargets in place with retained slots;
  the headed run held the V1 trace with zero recreation, per-brick
  transition uploads, one-frame recoveries, and zero allocator growth.
  Receipt in the V1b entry above; the shared-engine consolidation chain
  in the mesocosm engine review is now closed.
- **2026-08-26 (later):** D1 landed — brick raymarch and the renderling
  raster tenant now occlude each other per pixel on one shared depth
  surface. `mesocosm_lens::BrickTracer` gained the opt-in
  `encode_with_depth` join; the headed witness-pillar receipt passed on
  the RTX 4060 with the replay hash unchanged, judged by projected point
  probes on a self-selected trace frame. Mechanism, receipt, and the
  depth-view staleness finding recorded in the D1 section above. Noticed
  in passing, not D1's doing: two `paredros-sortie` sim-content tests
  fail against current upstream mesocosm ("the pact never fired", "the
  played body was never wounded"), and `mesocosm-genet` no longer
  compiles at mesocosm `main` against current genet.
- **2026-08-26:** V1a closed equal-sized travel-page cache coherence, then the
  proven traversal organ moved into Mere's `conatus-brick` at `28c07fab`.
  Paredros now owns its Ground binding and compiles the shared ABI/DDA by
  default; the final headed image remained byte-identical after extraction.
  The
  Paredros residency policy owns projection revision, notices changed key sets
  inside one page band, refuses regressing or unchanged replacement maps, and
  preserves unchanged-frame silence. The refreshed headed artifact proves a
  retained 795,144-byte extent, full changed-page publication with zero resource
  recreation, and zero upload on the following frame. Raymarch-depth
  composition, incremental `ResidentChunk` publication, allocator-observed
  bytes, and larger travel receipts remain open.
- **2026-08-26:** F3 became active and F3a landed in `paredros-social`: an
  append-only, actor-scoped epistemic record over accepted deeds, exact report
  provenance, claimant-owned correction, a deterministic belief fold, and
  validated exact replay. All 31 focused social tests pass and focused clippy
  passes with warnings denied. The ECS/world-compiler experiment remains an
  orthogonal post-F3 receipt. Full F3 still requires forgetting, adjudication,
  deception/intent, norms, observer-relative standing, and a consequential
  answer citing its evidence chain.
- **2026-08-08:** S3's sim half landed; headed real-time action open;
  R4 armed. `crates/paredros-sortie` joins the workspace as the one
  crate reading both owners: negotiated participation, agreement-driven
  companion parts, the terrain's own falls as wounds (body-revision
  facts), the pact-governed tag-in, the dig rule, and the sortie deed
  that flips a post-sortie answer with its premises citing it. 48 tests
  green across the workspace, clippy clean. Receipt hashes and the open
  half in the S3 section; six findings recorded.
- **2026-08-08:** S2 landed with its headed judgment open. `settlement.rs`
  and the settling scene join `paredros-social`: homes offered with daily
  work, answered from history, and residence plus the daily round derived
  from agreement state so an accepted agreement is the one thing that
  changes where a peer lives and what they do. 42 tests green across the
  workspace, clippy clean. Receipt and the open half in the S2 section
  above; three findings recorded.
- **2026-08-08:** S1 landed with its headed judgment open.
  `crates/paredros-social` is the willingness owner (deeds, standing,
  confidence, refusal, standing agreements, premises) and
  `crates/paredros-identity` is the foundation both owners share. Three
  companions answer one offer three ways with their reasons attached, and
  one standing agreement is formed, exercised, renegotiated, and ended
  with every transition in the deed log. 33 tests green across the
  workspace, clippy clean. Scene, premises, agreement receipt, and the
  open half recorded in the S1 section above.
- **2026-08-08:** S0 landed. `crates/paredros-room` is the first game
  code in the repo: a room carved into a grown hillside, one body under
  `near::step`, a 64-tick fixed trace, save/reload/replay to a matching
  position-log hash, and a headed winit run presenting netrender's
  composed master with the renderling room inside it. Receipt, hashes,
  and frame spans recorded in the S0 section above.
- **2026-08-07:** founded from the audit (old phase order could not test
  the premise; no action-RPG slice existed; identity facts were missing)
  and Mark's synthesis ruling (the sortie as the wing's extraction
  trigger). Charter rulings preserved; phase section superseded.
- **2026-08-13:** rebased after Mark rejected demo-first and entourage-first
  framing. The ordinary rule is one named life until death; tag-in survives as
  an optional player rule or explicit world process; clever body-changing
  mechanics remain deliberately possible. S0-S3 became foundation receipts.
  F0-F8 now order the game from persistent world through embodied and social
  life, material construction, causal culture, danger, death, and contested
  continuity. Camera choice remains open until the spatial laws can judge it.
