# Open Rulings Register (2026-08-29)

**Status: historical register snapshot, partially superseded; refreshed
2026-08-31.** Do not use this file as the current execution order. TD8 through
TD11 and DC1 through DC4 closed or reframed several entries after the snapshot,
and the [playable ecology plan](2026-08-31_playable_ecology_plan.md) now owns
the integration chain. Source plans remain authoritative.

This was a worklist, not a plan. Its numbered entries are retained as a dated
map of the questions active on 2026-08-29, not silently rewritten into a new
snapshot. Writing a decision down here did not rule it, propose it, or schedule
it; where a source document argued for an answer, that is said and attributed,
and the counter-case is stated with it.

**It will go stale.** Six documents were read on 2026-08-29 and four of them
were written or extended that same day; the terrarium round in particular has
ruled every open question it raised within a day of raising it. The source
documents remain authoritative for detail, for numbers, and for anything this
register compresses. Check the source before acting on an entry.

Read from:

- `2026-08-28_played_slice_plan.md`
- `2026-08-29_terrarium_dynamics_plan.md` (TD)
- `2026-08-29_scale_plan.md` (S)
- `2026-08-29_forms_of_life_brief.md`
- `2026-08-29_traits_and_perception_brief.md`
- `2026-08-29_elements_and_traits_memo.md`

Fifty-seven entries in four groups: **Blocking** (something concrete cannot be
scoped or built until this is answered), **Cheap and freeing** (a one-line
answer settles it, naming rounds included), **Structural** (real design
decisions with no deadline), and **Owed housekeeping** (inconsistencies and
errata that need a word rather than a ruling). Several decisions are raised by
more than one document from different angles; each appears once, citing all of
its sources.

---

## 1. Blocking

### 1. Rule what gates reproduction

**Ask.** Breeding is gated on an absolute 80 mg floor plus a gestation clock,
which knows nothing about the adult mass TD6 derived from the body plan. Two
symptoms pull opposite ways: consumers breed at half their death rate and
decomposers essentially never breed (they live at 17–23% of their own adult
mass), while in the first played session the population ran 61 → 8,155 in 49
seconds unchecked.

**Options.** Gate on the plan's own adult mass (TD7 calls this life history's
own answer and the missing half of TD6); gate on the reserve rather than the
body; leave `can_reproduce` alone and fix the mass shortfall elsewhere. No
`rates.rs` constant reaches it. Whatever is chosen has to serve both symptoms.

**Blocks.** `breathes` — unreached for three rounds, and S1's extra room
relieved the symptom without answering this. Also blocks a played session
staying legible.

**Source.** TD, Findings 2026-08-29 (TD7, recruitment); played slice, Findings
2026-08-28 and Progress 2026-08-29 (later).

### 2. Rule how far a decomposer can reach for carrion

**Ask.** Decomposers starve with 12–15 corpses standing in the enclosure: five
founders took 602 mg of scavenging in 3,000 ticks and died 26–106 ticks after
their last meal. The binding constraint is `DECOMPOSE_RANGE` and the search,
not the yield. How far, and by what means, may a scavenger reach a corpse?

**Options.** None named. The yield lever is ruled out by measurement —
quadrupling `DECAYS_BASE_MG` did not rescue them.

**Blocks.** `breathes`. Decomposers are the one kingdom whose failure is a
movement problem.

**Source.** TD, Findings 2026-08-29 (TD7, decomposers).

### 3. Rule whether a body without locomotion machinery may travel

**Ask.** `axis::seed` can draw a limbed line with no `Limb` tagma at all (22 of
160 consumers, 20 of 50 decomposers across ten seeds). Those bodies pay a
plant's rent under TD7 but `dispersal_for` floors locomotion at 1, so they move
anyway. Separately, a starving sessile body still gets one step per tick from
the TD2d hunger bonus. May a body without the machinery move, and what does a
starving sessile one do?

**Options.** Neither document names a menu. The forms brief names the change
for the sessile half (hand `dispersal_for` the unfloored `actuator_span()`) and
says the starving case is the part that needs a ruling.

**Blocks.** TD7's trophic asymmetry actually binding — seed 2's unlimbed
consumer grazing at a producer's price is the one collapse in the receipt. Also
the only remaining piece of the forms brief's Stage 2.

**Source.** TD, Findings 2026-08-29 (TD7, no actuator); forms brief §2.C and §6
Stage 2.

### 4. Ratify or replace the soil-flow answer

**Ask.** At per-voxel grain a producer reaches exactly 1 column of 1,089,
drains it, and thereafter earns exactly the rent it paid — net zero forever,
every seed collapsing with 340,000 mg lying around. Soil percolation was
shipped to stop that and flagged as the one TD6 decision taken without you;
TD7 then added root forage alongside it. Both are in the tree.

**Options.** The forage radius the grain was chosen for; percolation as
shipped; a coarser grain for uptake only; producers that relocate off spent
ground. TD7's ruling took percolation *and* forage, so what is owed now is
ratification of a shipped state as much as a choice among four.

**Blocks.** Decisively load-bearing: percolation off, everything else
identical, every baseline seed ends at ~1 organism; on at divisor 8, 46–100.

**Source.** TD, Findings 2026-08-29 (TD6, point uptake); TD7 §"Soil flow".

### 5. Decide whether new axes wait on the terrarium breathing

**Ask.** The terrarium does not breathe, three rounds running. Do the forms
brief's four stages, and any per-column element field, wait on that — or may
they start alongside?

**Options.** Both documents state the case for waiting (adding axes to an
ecology that does not balance adds dimensions to the search, not answers) and
neither states a case against.

**Blocks.** All four forms stages; every Tier-1 element idea (element-selective
light capture, soil element maps, payloads landing in soil).

**Source.** Forms brief §6 closing; elements memo §0 standing blocker and §7
item 2.

### 6. Rule the movement economy

**Ask.** In the first playtest the ecology spent the controlled critter's
1,000 mg wandering, and Mark's very first movement keypress was already refused
`InsufficientMass` — he never displaced the critter. TD4 has since given the
hand priority, which flipped the problem: a held critter's dispersal is
suppressed, so it cannot flee, and a fat one is the best meal in the enclosure
(the demo died around tick 134 to up to six predators; `DEMO_STEPS` was capped
at 120 around it). How do the player and the autopilot share one energy pool,
and is being un-fleeable the price of the hand?

**Options.** Neither document names candidate rules.

**Blocks.** Movement being usable at all; a session lasting past ~130 ticks.
One of the three seams the played slice records as untouched.

**Source.** Played slice, Findings 2026-08-28 and Progress 2026-08-29 (later);
TD, Progress "TD4 landed", residues.

### 7. Say which played-slice seam runs next

**Ask.** Three seams remain from the playtest: the movement economy (6),
reproduction's brake (1), and succession (8). Which is dispatched first?

**Options.** Those three. The doc's pattern is that implementation is held
until the seam is ruled — the vitals seam sat "deliberately held; not yet
dispatched" until it was.

**Blocks.** Any further implementation on the played slice.

**Source.** Played slice, Progress 2026-08-29 and 2026-08-29 (later).

### 8. Wire the epoch boundary and succession (PS2)

**Answered 2026-09-02, and the question dissolved rather than being resolved.**
`Runtime::end_epoch` never got a production caller: PE3 made the world end its
own epochs on a versioned world rule, DT3 gave a hand `Intent::EndEpoch`, and
DT4 deleted both `Runtime::end_epoch` and `World::end_epoch` because they ran a
*different* boundary from the one `World::apply` runs — no adaptation round, and
`at_boundary` left false. There is one door now. Succession was wired by PE1.

**Ask.** `Runtime::end_epoch` has exactly one caller and it is a unit test, and
nothing calls `control_lost()` from any host — so `world.epoch` never leaves 0
and `WorldRecord` stays empty in real play. The first playtest hit the death
seam in its first minute: the critter died at tick 999 and every input for the
remaining two-thirds of the session was refused `Disembodied`. Does this get
wired now, and who owns it?

**Options.** Keep PS2 extracted and unscheduled as ruled; or schedule the
death/succession half now with the epoch-boundary half still deferred. Both
seams have landed core machinery and zero host wiring. The trait-board review
screen stays with Views route B (already ruled).

**Blocks.** A session surviving its first death; and any trait bank "weighted
by the world's lifeline", which has no data source until the boundary runs.

**Source.** Played slice PS2, Findings 2026-08-28; traits brief §8 OQ11 and §2.

### 9. Rule the section framing

**Ask.** How far back does the terrarium section's camera sit? The world grew
to 129 voxels across in S1 and the shipped `SLAB_HALF_HEIGHT` of 20 now frames
55% of it with the ground running off both edges. A second question travels
with it: past about half-height 24 the frame's floor dips below bedrock and
shows void, so the follow centre may want clamping to at least `H`.

**Options.** 20.0 shipped (71×40 voxels, a 9-voxel limb is 12.7% of frame);
**28.0 proposed** by S1 (100×56, 77% of world width, limb 9.0%) on the
principle "frame the content's height and let the width follow"; 36.3 (frame
exactly the enclosure, but the capture shows both walls, bedrock and void — the
plan reads it as a slab rather than a terrarium); 48.0, included to show where
the range ends. Captures for all four sit beside the receipt.

**Blocks.** Nothing mechanically — all four replayed the same trace to the same
hash, which is the proof framing cannot reach a trace. The default deliberately
did not move; `--slab 28` exists so one line adopts whichever is ruled.

**Source.** S, Progress "The framing, proposed with its arithmetic" and
Findings (creature:frame ratio). Supersedes the played slice's PS1 residue,
which suggested 9–11 for the old 32-voxel enclosure.

### 10. Decide how to lift the 40-body roster cap (Ruled 2026-09-02)

**Ask.** `mesocosm_lens::MAX_ROSTER` is 40, and every S1 capture at every
half-height reports exactly 40. The slab window now holds more organisms than
the tracer can pose, so what the player sees is a truncation rather than the
enclosure.

**Options.** Raise the cap, or hand far bodies to silhouettes (S4's own plan).
S1 does not choose between them, only that one must happen before zoom.

**Ruled by Mark, 2026-09-02: far bodies go to silhouettes** rather than
raising the capsule caps; it lands with S4 zoom. See the
[default creatures plan](2026-08-30_default_creatures_plan.md) §3.3 and §7
question 10 for how this composes with that plan's own capsule-budget note.

**Blocks.** S4 — zoom cannot mean anything while the roster is clipped.
Presentation only; it does not touch the trace.

**Source.** S, Findings 2026-08-29 (S1, roster cap). The played slice's PS1
"lens roster" residue is closed by the roster landing; the cap is what remains.

### 11. Decide whether S3 displaces S2 as the next scale rung

**Ask.** S1's measurements argue the region tier is now the load-bearing rung
rather than the windowed atlas: at ±64 the grown graph has diameter 3 against
the `demote_hops = 2` it was tuned for, a region is 43 voxels across rather
than 11, and 43% of the roster (against 19%) now sits in the far tier — whose
target scan is unbounded, and some of it is on screen. Does the ladder reorder?

**Options.** Keep the stated order (S2 next, as the Status line still says); or
run S3 first — a distance cap on far-tier perception, a spatial index for
`Places::at`, cohorts as a real execution path. S1 reports the argument and
explicitly does not rule it.

**Blocks.** Which rung is built next, and whether ±128 (S2's unlock) or the
tier's correctness comes first.

**Source.** S, Findings 2026-08-29 (S1, tier line), against the Status line.

### 12. Accept or revise the four-stage scope in the forms brief

**Ask.** The brief proposes: Stage 1 unbind energy source from symmetry
(mixotrophy), Stage 2 finish sessility, Stage 3 the attached edge then the
internal one, Stage 4 inhabitation as play, deliberately last. Stage 1 is
recommended first by a wide margin. Accept, reorder, or decline?

**Options.** As above. Stage 2 is no longer independent — TD7 built its harder
half — and should be re-scoped rather than presented as standalone. Stage 4 is
last because building the playstyle before the ecology can express the
relationship is called the Spore mistake in miniature. Explicitly *not*
recommended now: horizontal transfer, a second medium for filter feeding,
spores, true viruses.

**Blocks.** Everything else in that brief; nothing in it is scheduled until
this is settled. Gated in turn by entry 5.

**Source.** Forms brief §6 Scope, staged.

### 13. Rule whether a mixotroph reads as prey

**Ask.** If a body both fixes matter from the soil and eats, can grazers still
bite it? Today `LivingTarget.kingdom` makes "what I earn from" and "what I am
edible as" the same enum, so a soil-fixing mixotroph would be invisible to
every grazer — fixing capacity added to the world with no prey added.

**Options.** (a) Edibility follows any fixing part, so mixotrophy costs you
safety; (b) edibility stays with kingdom, so mixotrophs are free riders and the
pyramid tilts; (c) edibility becomes its own reading off the body — a second
unbinding, and a bigger job. The brief calls none of them obviously right.

**Blocks.** Scoping Stage 1, which is the stage recommended first.

**Source.** Forms brief §8 OQ1 (with §0.2, §2.A, §6).

### 14. Rule whether acquisition stays player-only

**Ask.** `learn_from` returns early without `controlled()`, and incorporation
runs only through `Intent::Metabolize` on the controlled body, so no NPC
lineage ever learns a word or keeps a part. Is acquisition the player's verb,
with the surrounding ecology a fixed cast — or does the guard come off?

**Options.** Keep it player-only, or make NPC acquisition a stage of its own,
which costs: every NPC meal touching a lineage recipe, and recipes changing
under bodies that are mid-life. The traits brief adds that keeping it
player-only looks cheaper but is dearer, because a player-only rarity table
deepens the same split.

**Blocks.** Stage 1's payoff and the forms brief's whole §4 synthesis, which
today describes a loop running for exactly one organism; and anything that
weights a trait bank by the world rather than by the player.

**Source.** Forms brief §8 OQ2 (with §0.7, §4, §6); traits brief §7 item 9.

### 15. Rule what an undefined lineage distance costs

**Ask.** The proximity term in the incorporation cost formula is undefined for
essentially every meal: genesis founds only unparented roots and only the
player can create a parent link, so `Lineages::distance` returns `None` for
every cross-founder pair forever. What does an unrelated donor cost?

**Options.** Max cost; outright refusal of the graft; a separate explicit
"unrelated" tier; or make founders related at genesis — which reverses the
standing ruling that `None` is the honest answer and a shared ancestor must not
be invented to make the arithmetic work.

**Blocks.** The incorporation cost formula entirely. The traits brief calls it
the single hardest blocker in the message it was written against.

**Source.** Traits brief §8 OQ1 (with §0.5, §7 item 7).

### 16. Pick which element scheme gets built first

**Ask.** Three ways to make elements real, and the memo makes no
recommendation. Which is built first?

**Options.** **A, typed matter** — elements are kinds of milligram in soil and
bodies; matches the ruled ProcessDef grammar most exactly and gets second-order
effects free, but costs most (400–700 LOC, O(columns × E) per tick, and the
milligram-exact conservation test must be rewritten before the feature).
**B, coefficients** — property rows read by the rate formulas that already run;
cheapest in state (per-column cost zero, 250–400 LOC) but weakest on exclusion
(blending has no refusal), worst discoverability, highest affix risk.
**C, exchange payloads** — rules that fire when matter changes hands; cheapest
overall, half-built (`venom_mg` is the existing one-element instance), fixes a
live bug on the way, 300–500 LOC, but risks opacity and the lookup-table trap.

**Blocks.** Every other element decision in the memo, and which F0-shaped
ancestor gets built.

**Source.** Elements memo §7 item 1, with the case in §1 and §5.

---

## 2. Cheap and freeing

### 17. Play the slice and say whether it passes

**Ask.** PS0 and PS1 are mechanically green — the host builds and runs, a
200-intent and a 369-intent trace both replay to `8a101763143e5012`, capture at
`Code/testing/mesocosm/ps1_played.png`. Both receipts record the human half as
outstanding: keyboard play, and judging the brick-traced side-on section by
hand. Does it pass?

**Blocks.** Calling PS0 and PS1 done.

**Source.** Played slice, PS0 and PS1 receipts 2026-08-28.

### 18. The naming rounds owed

**Ask.** Nine words are wanted and none are coined in the source documents,
which deliberately flag rather than fill. Each needs the usual round —
crates.io, game, studio, trademark — per CLAUDE.md Terminology.

**New names owed:** the fixing/absorbing process (forms §2.A — its geometry is
already named, `Role::Plate`, so only the process word is open); the
organism-to-organism edge (forms §2.D); the influence channel gentler than
`TakeControl` (forms §5); the composed trait unit, the acquisition cost, and
the apparent-kind reading (traits brief front matter).

**Collisions to resolve:** "element" is already used in this repo in the
Genshin/BOTW sense (a field channel reacting with surfaces) and the memo uses
it for a compositional precondition — which mechanic keeps the word? `bank` is
`Lineage::bank`, a budget you spend in the adaptation phase, and the proposed
trait mechanic is a pool you draw from in the same phase — which one moves? And
there is no plant noun, because the bare word *flora* is reserved
platform-side; `Kingdom::Producer` is the code name.

**Blocks.** The process word blocks Stage 1's implementation, the edge word
Stage 3, the channel word Stage 4; the element round blocks writing either
element mechanic down; the plant noun blocks any player-facing plant
vocabulary.

**Source.** Forms brief §8 OQ10; traits brief front matter, §8 OQ4 and OQ12;
elements memo §7 item 7.

### 19. Charge venom on NPC meals

**Ask.** Only the played meal path charges `venom_mg` today, so to an AI
predator a Batesian bluffer and a genuinely armed critter cost the same
(nothing). Make the ecology charge it the way the played path does, with the
spill deposited back to the column?

**Options.** The memo proposes it for acceptance: it is a bug fix, it closes a
live inconsistency, it is exactly the envelope scheme C generalizes, and it
requires choosing nothing.

**Blocks.** Independent of entry 16, so accepting it lets work start before the
scheme is picked. Also a prerequisite for entry 39.

**Source.** Elements memo §6 (C's ancestor); traits brief §5; forms brief §2.G.

### 20. Decide whether burning a meal should teach

**Ask.** `learn_from` runs after the burn/build route is taken, so burning a
meal teaches the same words as grafting it — a starved player who burns
everything raises the complexity frontier as fast as one who builds. Intended?

**Options.** Tie learning to the graft route (one conditional), or accept that
burning teaches. The traits brief calls the current behaviour almost certainly
unintended, because it decouples the reward from the tradeoff `Route` exists to
express.

**Blocks.** Pricing incorporation coherently — the reward is currently on both
routes with a price on neither.

**Source.** Traits brief §3, §7 item 2.

### 21. Commit to near-tier-only trait-relative appearance

**Ask.** Say explicitly that trait-relative appearance is a near-tier
mechanic? `preferred_target` walks the whole living roster with no distance cap
and `can_perceive_position` short-circuits above `Tier::Near`; at 4,700
organisms that is ~22M tuple writes per tick before any appearance work exists.

**Options.** The brief argues it is a natural boundary rather than a
compromise — a far-tier cohort has no individual bodies to be seen as anything
— and asks only that it be a stated commitment rather than a discovery.

**Blocks.** Bounds the cost of trait-relative perception and keeps it inside
the scale plan.

**Source.** Traits brief §4.

### 22. Rule parasite vs mutualist as a flow reading, and set the window

**Ask.** Derive parasite/commensal/mutualist from the sign of net mass flow
across the host–guest edge over a window, rather than storing a label? And if
so, how long is the window?

**Options.** The brief argues for the flow reading: no enum, and the real-
ecology flip (the same partner mutualist in a rich year, parasitic in a lean
one) comes free. The unnamed alternative is an authored label. Cost is one
running net per edge, which must be hashed.

**Blocks.** Unblocks forms §E entirely — given the edge, the brief says §E
possibly needs nothing else.

**Source.** Forms brief §8 OQ4, §2.E.

### 23. Choose budding, spores, both, or neither

**Ask.** Does reproduction get a second mode? Budding routes an at-ceiling
body's overflow into a bud instead of budget; spores are reproduction without
locomotion, scattered further than a sessile parent could carry.

**Options.** Budding is nearly free and conservation-neutral (the overflow had
to go somewhere) and gives determinate growth an outlet, since hitting your
ceiling is currently a dead end; `Route`'s doc comment already anticipates it.
Spores need a bounded scatter rule that is not the parent's step budget —
TD2b's finding is the warning, birth scatter threw offspring through walls with
no bound check — and are on the brief's "not recommended now" list.

**Blocks.** Sessile lineages spreading.

**Source.** Forms brief §8 OQ8, §2.F.

### 24. Rule whether the host–guest edge counts as eating

**Ask.** `axis.rs` rules that a lineage cannot express an appendage it has
never eaten. If a symbiont is metabolically inside you, does living on the edge
count as eating — permitting horizontal trait transfer?

**Options.** Yes (defensible: a symbiont is metabolically inside you), or
horizontal transfer stays out. Worth knowing first: the shipped rule is softer
than it reads — it gates one setter, `Recipe::tagmata` is a public Vec,
`divide` copies appendages unchecked, and it only ever teaches the player's
lineage.

**Blocks.** Horizontal transfer, currently on the "not recommended now" list
pending this.

**Source.** Forms brief §8 OQ9, §2.F, §1.

### 25. Adopt the anti-affix tests and the rank instrument as standing gates

**Ask.** Enforce four structural tests from day one on whichever element scheme
is chosen, and keep the rank measurement as a standing instrument?

**Options.** The tests: refuse any element property read by fewer than three
existing kernels; require each verb to read a disjoint subset of properties;
require at least half the verbs to change what leaves the organism; fingerprint
every generated element on a reference body and reject collisions at worldgen.
The instrument: generate N lineages, run a fixed battery, take the rank of the
fingerprint matrix at both element settings — if rank does not rise with
element count, the elements are not reaching the formulas.

**Blocks.** Without them the memo says all three schemes become affix systems;
B needs test 3 most and has it least. The memo calls the rank reading the
single measurement that says whether the whole idea works.

**Source.** Elements memo §2.

### 26. Accept the admission-by-trace gate on the forms stages

**Ask.** The place-graph plan rules that machinery is admitted by trace, not by
prior art. Stages 1 and 3 are argued by design, not by a receipt. Require each
stage to name the reading that shows the gap it closes, before scheduling?

**Options.** Accept the gate — the brief calls it cheap and names the readings
(for Stage 1, a run where lineages sit at a trophic dead end the enclosure has
capacity for; for Stage 3, a run where every interesting relation in the record
is instantaneous) — or hold that the rule was written about acceleration
structures and adopted dependencies, not an in-crate mechanic.

**Blocks.** Scheduling Stages 1 and 3.

**Source.** Forms brief §6, "Two stop rules the draft did not test itself
against".

### 27. Decide whether to take the kinship first-caller slice

**ANSWERED 2026-08-29, differently than asked:** the first caller arrived as
**TD10's predation discount** (kinship tempers the appetite, ruled by Mark),
not as F0's migration framing. `ecology/kinship.rs` now spends
`Lineages::distance` in prey scoring. The migration slice itself remains
untaken; whether it still wants doing after TD10 is a smaller question than
this entry posed.

**Original ask.** `Lineages::distance` and `World::kinship` are built, correct,
tested, and have zero production callers, and "migration following kinship
rather than distance" is already on F0's sanctioned candidate list. Take it as
the next fantastical slice?

**Options.** The memo offers it as the cheapest such slice in the repo — a
function waiting for its first caller — and names no alternative.

**Blocks.** Nothing. Independent of the scheme choice, so it can run while
entries 16 and the rest are open.

**Source.** Elements memo §6, closing paragraph.

### 28. Decide whether energy gets a real capacity

**Ask.** The vitals bar's denominator is the session's own high-water energy,
because the world has no capacity to measure against. Should a real capacity
become a world quantity for the bar to read?

**Options.** Keep the high-water denominator, or introduce a capacity.
Cosmetic today; the vitals surface landed with no core or runtime change.

**Blocks.** The bar meaning the same thing across sessions.

**Source.** Played slice, Progress 2026-08-29 (later), residues.

### 29. Decide whether to shorten the long matter run

**Ask.** The conservation test file went from 84 s to 570 s in the debug
profile, almost all of it percolation over 16,641 columns for the 4-seed
4,000-tick run. Shorten it, accepting less seam coverage?

**Options.** No specific lengths named. The plan frames it as coverage versus
friction, says shortening is a coverage decision rather than a speed one, and
discloses it rather than trading it away — it becomes Mark's if the friction
proves real.

**Blocks.** Nothing today.

**Source.** S, Progress "Tests".

---

## 3. Structural

### 30. Rule the exclusion relation

**Ask.** What makes two generated things mutually impossible? The general model
plan requires an exclusion relation over any generated combination space, and
neither the trait design nor the element schemes name one. The same question
arrives in three places: trait combinations, element pairs, and axis
combinations generally.

**Options.** The forms brief proposes, explicitly for rejection, that the
conservation economy *is* the exclusion relation — combinations that cannot pay
rent do not persist, and combinations whose readings contradict each other
cannot be built, with authored refusals reserved for the few that would need
new world machinery to represent (today: exactly one, an interior place graph;
possibly a second, a second medium to filter). It states its own two limits:
the economy sorts by gradient, so "derived exclusion" means *loses over time*
rather than *cannot exist*, and the sharpest exclusions come from readings
contradicting themselves, not from the rent bill. The alternative is a
hand-written coherence table, whose maintenance grows as the square of the
axes. For element pairs specifically the memo asks the same question in its
sharpest form: is a disfavoured pair a hard gate, or an expensive but
recoverable graft?

**Blocks.** The trait bank's legality rules; the axis on which the three
element schemes differ most (A has native exclusion via port mismatch, B has
none, C needs the asymmetry law).

**Source.** Forms brief §4; traits brief §8 OQ8; elements memo §7 item 5.

### 31. Rule whether a trait is a reading or a record

**Ask.** Is a composed trait computed from the parts a body carries plus their
provenance, or stored as a record beside the body? If a lineage can hold a
trait no part expresses, there are two answers to what a body can do.

**Options.** A reading over body plan plus provenance — `Origin::Incorporated`
is already durable per-part state and is already filtered on, so geometry stays
the authority. Or a stored record — whose end state `guise` already
demonstrates: a stored claim about appearance no tick rule could honestly
consume, so none does. The brief says the reading version survives its own
challenge and the record version does not.

**Blocks.** The whole trait-unit design and everything downstream — bank, cost,
perception.

**Source.** Traits brief §8 OQ6, §6.

### 32. Rule rarity tiers against the repo's own complexity idiom

**Ask.** Mark proposed a rarity ladder (1 effect common, 2 uncommon, 3 rare, 4
legendary, 5+ epic). Do generated traits and elements carry an authored ladder,
or is legibility expressed through quantities the repo already computes?

**Options.** For: players need a fast read on whether a trait matters, effect
count is an honest proxy, tiers are the standard grammar for a reveal. Against:
the founding plan bars a loot economy by name for this mechanic; pillar 5
already names upkeep as where scarcity lives; "sample constraints, not powers"
rules against ranking by effect count; the five words are five uncleared
coinages; a tier is context-free and so fails the RimWorld criterion (Frame is
worthless in a warm crowded world and decisive under gravity and predation);
and `Recipe::complexity()` is already a legible quantity read off anatomy and
wired to the frontier. Two alternatives are offered: reframe rarity as
*constraint*-count rather than effect-count, which turns it from cutting
against the stop rule to satisfying it; or, for elements, measure interest as
how unlike this world's history a thing is — world-relative and retrospective,
per significance-as-abnormality. The memo argues plainly for the latter and
says do not build the ladder. Side note: the proposed ladder inverts the
genre's own ordering, and "5+ epic?" was written with a question mark.

**Blocks.** The trait bank and anything player-facing that labels a trait or an
element.

**Source.** Traits brief §8 OQ3, §6; elements memo §3 "On rarity tiers".

### 33. Choose where the incorporation cost lands

**Ask.** Charging for a graft needs a destination for the milligrams. Which?

**Options.** (a) Skim the tax out of the meal so it falls to the soil — free,
no new sink, uses existing arithmetic, diegetic ("you waste more of the
carcass"); the brief's recommended shape. (b) Charge the eater's reserve — the
same three lines as the venom debit, but inherits that comment's unresolved
debt-or-damage question and makes incorporation lethal at the margin. (c) A
recurring surcharge on upkeep — the best match for "metabolic cost", but
collides with TD7's rewrite of `upkeep_mg` and breaks the property that rent
derives purely from what a body is.

**Blocks.** Pricing incorporation. Option (c) additionally waits on TD7.

**Source.** Traits brief §8 OQ2, §3.

### 34. Rule whether the level-up stays diegetic

**Ask.** TD4 ruled that the burn/build choice is made by the body from budget
state, never by the player's fingers, "which is why replays cannot disagree
about it". Does a costed incorporation stay inside that — the body pays
automatically — or does it become a cost/benefit prompt at meal time?

**Options.** Body pays automatically, with refusals routed through the existing
`Rejection` gate; or a player-facing prompt, which the brief says must be
argued as a reversal of TD4 rather than slipped in.

**Blocks.** The whole incorporation-cost interaction, and replay determinism.

**Source.** Traits brief §8 OQ5, §3.

### 35. Retain or replace the fixed-verb composition grammar

**Ask.** The general model plan rules "fix the Technique axis; generate the
Form axis from the world's own ontology." The proposed trait design has no
fixed verb axis — one flat generated bank, composed within itself. Does the
existing ruling stand, or is it being replaced?

**Options.** Keep the split, in which case the bank's shape changes to fit it;
or record an explicit replacement.

**Blocks.** The trait bank's shape.

**Source.** Traits brief §8 OQ7.

### 36. Rule apparent kind: one global value, or a function of the observer

**Ask.** Is what a body appears to be one reading, or does it depend on who is
looking? Today `LivingTarget.kingdom` is built from `o.kingdom()` — one true
symmetry reading, identical for every observer — and `guise` is heritable but
inert.

**Options.** A global apparent-kind reading routed through one hinge in
`ecology.rs` (the consumer already exists in `movement.rs`); or a per-observer
function, which forces whatever it needs about the target to be lifted into
`LivingTarget` at its build site. The brief's cheap formulation of the latter
is a fixed-width bitset per target, a channel mask per observer, and a mask
intersect per pair — never an S×T loop. Distinct from entry 13, which asks
whether *edibility* unbinds from kingdom.

**Blocks.** Trait-relative perception and NPC-visible deception. Also waiting
on LOC room — `movement.rs` is 560 against a 600 ceiling; `perception.rs` at
125 is the obvious home.

**Source.** Traits brief §8 OQ9, §4, §5.

### 37. Decide the key and decay for deception memory

**Ask.** "Plus memory" wants a pair-keyed, non-decaying, valenced relation.
`LastSeen` is one slot per organism holding a position, decaying in 8 ticks and
cleared on any tier change, and its doc comment rules that any such thing must
replay with the organism — i.e. live inside `state_hash`. Is remembering that a
lineage lies a per-lineage or a per-individual mechanic?

**Options.** Organism-pair-keyed — O(N²) serialized state, which the brief says
does not get cheap; or lineage-keyed with bounded cardinality and a decay rule,
which is affordable.

**Blocks.** Any memory-of-deception mechanic.

**Source.** Traits brief §8 OQ10, §4.

### 38. Decide where a lifeline-weighted bank would live

**Ask.** If a trait bank is weighted by the world's lifeline, what holds the
counts?

**Options.** Three arms, each with a cost. `WorldRecord` is in `state_hash` and
joins by max — a count or a running total does not join by max, so putting one
there breaks the property the type exists for and trips the "do not break the
semilattice" stop rule. `History` is unbounded and deliberately outside the
hash. `World::apply` does not take one, so no tick rule can read the lifeline
without a signature change. The brief names no winner.

**Blocks.** The trait bank, alongside entry 8.

**Source.** Traits brief §2.

### 39. Decide whether deception is earned or authored

**Ask.** The whole mimicry composition is authored once at genesis: `signal`,
`venom_mg` and `guise` are copied verbatim at breeding with no mutation
operator, while the *response* to a signal does evolve. The mix is fixed at
seeding (10% bluffer, 10% aggressive mimic, 20% honestly armed, 60% honest
plain) and only differential survival moves it. Is deception meant to be
earned?

**Options.** If earned: charging venom on the NPC path (entry 19) and adding a
mutation operator to the inherited traits are prerequisites, not polish. If
authored: leave differential survival as the only mover. Both changes are
small; neither is free. The forms brief calls this the highest
story-per-line item it holds.

**Blocks.** Any mimicry story not authored at world seeding. Independent of the
four stages.

**Source.** Forms brief §2.G, §6; traits brief §5, §7 item 10.

### 40. Rule whether a virus is an organism or a condition

**Ask.** How is a thing with no metabolism represented — it pays no rent and
reproduces out of someone else's synthesis — given TD5's ruling of one economy
for all life?

**Options.** (i) An organism with a tiny body whose rent is paid through the
edge out of the host's budget — stays inside one economy and is the only one of
the three you could inhabit; (ii) a condition carrier on the host — coherent
and cheap, but not a critter, so not playable; (iii) out of scope, germ and
parasite are enough.

**Blocks.** Whether a virus playstyle exists at all, and whether the edge must
carry rent as well as flow.

**Source.** Forms brief §8 OQ3, §0.6, §4.

### 41. Rule whose biomass counts when you play a guest

**Ask.** The goal is your lineage's share of the world's biomass, but a
symbiont's best strategy is to grow a host whose biomass is not yours. Is that
the most interesting tension in the idea, or a broken win condition?

**Options.** None named; the brief says it cannot tell. It is a care-
granularity question as much as a scoring one — the wing invariant is care for
a species, and this asks which species the play grows.

**Blocks.** Stage 4.

**Source.** Forms brief §8 OQ5, §5.

### 42. Rule what the complexity frontier reads for a guest

**Ask.** `Ineligible::AboveTheFrontier` refuses inhabiting something more
elaborate than you have earned, while permitting a step down into a newly
viable niche. A simple guest inside a complex host is a step down by that rule
— but you are steering the complex thing. Does the frontier read the guest, the
host, or the pair?

**Options.** Those three. A separate bound exists independently: `FaunaPolicy`
runs only for Grazer/Predator at `Tier::Near` with ground.

**Blocks.** Stage 4's eligibility rules.

**Source.** Forms brief §8 OQ6.

### 43. Choose the influence channel's shape, as a reversal of TD4

**Ask.** An influence channel means the held body runs its own `disperse` under
a biased policy — the player's keys and the critter's instincts working one
body on the same tick. That is precisely what TD4 arranged them *not* to do
("holding a key moves the critter with no instinct fighting the hand"), because
the drive scoring lives inside the call the ecology skips while a hand is on
the body. If it is reversed, is influence a bias applied before the choice, or
a veto after it?

**Options.** (a) A bias — the concrete candidate is `FaunaPolicy.biases`, an
existing per-drive additive term evolution already inherits and mutates, so a
nudge could be bounded in evolution's own units. (b) A veto or nudge on the
already-chosen drive. Either way, influence over drives does not touch feeding,
because `choose_living_target` never consults the policy. The brief insists the
choice be put as a reversal of TD4, not as the same dial at lower authority.

**Blocks.** Stage 4; and the "why did it do that" readout, since
`last_fauna_decision` is stale or `None` for a held body today.

**Source.** Forms brief §8 OQ7, §0.5, §5.

### 44. Rule the guest-edge tick order

**Ask.** Does a guest on the host–guest edge eat before or after the host pays
rent?

**Options.** Before or after. TD5's implementation note records the analogous
answer (rent before income, so a body is asked whether it is starving after the
day has cost it something) — but that sits in a Progress log as a recorded
consequence, not in a ruled section, so the brief declines to lean on it.

**Blocks.** Stage 3 — TD6's tick restructure settles meals pairwise and the
edge's flow has to slot into that order.

**Source.** Forms brief §2.D.

### 45. Decide how the host–guest edge survives cohort-ization

**Ask.** The general model plan still flags an open individual-to-cohort
storage replacement, and `Cohort` carries no `OrganismId`. An edge keyed on
individual ids would not survive a demotion to the far tier. How is the edge
keyed?

**Options.** Design the key to survive cohort-ization, or accept that holding
an edge is itself a reason to keep both parties near — which the brief stresses
is a cost, not a free choice.

**Blocks.** Stage 3's design. The far tier is where most of the world lives.

**Source.** Forms brief §2.D, §6.

### 46. Rule on the consumer-pull inversion for Stage 3

**Ask.** Stage 3 adds new hashed world state whose only named consumer, Stage
4, is deliberately last. That is the inversion the repo just corrected
elsewhere, where a deferred route sat on a consumer pull nobody scheduled and
operated in practice as a ban. Acceptable here?

**Options.** Take the counter-argument (consumer pull as ruled is about
dependency and lane adoption, not ordering mechanics inside one crate), or hold
Stage 3 until its consumer is real. The brief says the counter-argument is
available but should be made, not assumed.

**Blocks.** Scheduling Stage 3 ahead of Stage 4.

**Source.** Forms brief §6.

### 47. Set the dense field budget

**Ask.** How many dense per-column element channels does a world carry?

**Options.** The arithmetic gives roughly 4 to 8. Against a measured 133 µs per
tick at 75 bodies: 4 channels is noise, 8 is noticeable, 20 is comparable to
the whole tick, 100 is 5–10× it — percolate costs 1,089 column-visits per
channel per tick whether or not anything uses the element. Sparse elements are
a separate case and must be a sorted `Vec<(Column, u64)>`, never a `HashMap`.
The number itself is a call about how coarse the world's chemistry feels.

**Blocks.** The worldgen sampler, which takes the budget as a constraint.

**Source.** Elements memo §7 item 3, §4.

### 48. Choose composition granularity

**Ask.** At what granularity does an organism's element composition live?

**Options.** Per lineage — about 10 KB at E=100 across 50 lineages, free. Per
part — a material index is a byte, read through a walk the tick already
performs; Tier 2 comfortably carries 20–40. Per organism — at ~4,700 saturation
that is 470,000 serialized entries inside `state_hash`; the memo refuses it on
cost and recommends differing individuals by which parts they grew, but says
that if per-organism vectors are wanted anyway it must be said explicitly.

**Blocks.** State layout and the snapshot cost envelope for any scheme; under
scheme B it also decides whether individuals can differ at all.

**Source.** Elements memo §7 item 4, §1B, §4.

### 49. Rule whether "obvious and necessary" becomes a worldgen guarantee

**Ask.** Is the instinct for a closed set of necessary traits satisfied by a
constraint on the worldgen sampler — every world must be able to make a
producer, something that moves, something that senses — or is a genuinely
separate closed trait set wanted?

**Options.** The memo argues for the sampler constraint (one rule, no second
authority) and notes `process.rs`'s own comment refuses the alternative: a
process vocabulary authored ahead of any consumer becomes a catalog, which is
the Spore failure at smaller scale.

**Blocks.** Whether the closed/open seam sits between verbs and nouns or
between two trait authorities; and the sampler constraint spec.

**Source.** Elements memo §7 item 6, §0.

### 50. Accept one element table shared with geology

**Ask.** Do organism-produced materials enter the same element table as
worldgen's, with the same property vector and the same verbs — so a biotic
table grows on top of a static geological basis?

**Options.** The memo presents the shared table as the mechanical version of
second-order elements and calls it unusually cheap, because the return path
already runs (`release_reserve`, carrion decay, the meal's unkept remainder,
percolate). The payoff differs by scheme: free under A and C, meaningless under
B unless composition is made downstream of feeding, which drags in a piece of A.

**Blocks.** Organism-driven soil maps, corpse payloads as properties of the
ground, succession falling out of code that already runs.

**Source.** Elements memo §3.

### 51. Decide whether success-punishing rent is the design

**Ask.** Growth raises rent, so a critter that grows fast crosses
`STARVED_UPKEEP_TICKS` on its own success and its next meals burn instead of
building. Deliberate negative feedback, or an artifact of the m^0.75 scaling?

**Options.** No menu named; the fork is design versus artifact.

**Blocks.** How the played body's growth arc feels on screen.

**Source.** TD, Progress "TD4 landed", residues.

### 52. Rule the full host inversion

**Ask.** The vitals chrome landed with genet still owning the window: the
cambium view is diffed into a `ScriptedDom`, styled by Livery, lowered through
`paint_list_render` and composited over the frame like the minimap. Should the
host be inverted so the chrome stack owns the window, and when?

**Options.** None named — the doc records only that the inversion was not
reached for and is still Mark's.

**Blocks.** Nothing today; the two lanes now share one netrender instance and
one blend pass and work as they are.

**Source.** Played slice, Progress 2026-08-29 (later).

---

## 4. Owed housekeeping

### 53. The repo carries two formulations of the anti-Spore rule

The wing founding record §2 retired "if any stage grows its own engine the wing
hollows out" as forbidding too much, and explicitly permits a vessel its own
renderer, event loop, ECS, camera or physics dimensionality; what is forbidden
is a private answer to identity and history, plus — separately, from the
place-graph plan §0.1 — a second simulation authority. `mesocosm/CLAUDE.md`
still carries the retired phrasing, as "the wing's single most load-bearing
rule" (verified 2026-08-29). Which stands, and does CLAUDE.md get corrected? It
matters to every argument that leans on what a vessel may own privately.
*Source: forms brief §8 OQ11, §3.*

### 54. The feeding and movement paths disagree about who can see a warning

`choose_living_target` reads `target.signal` with no tier gate, no ground gate
and no `Sense`-part gate; the `Sense` gate lives only on the movement path in
`FaunaSenses::read`. So an eyeless grazer walks toward a warning-coloured
target indifferently and then declines to bite it. Resolve it deliberately —
gate feeding as movement is gated, or drop movement's gate — rather than
absorbing it into a per-observer appearance model (entry 36), since that is the
seam the model sits on. Carries an erratum: the forms brief's §1 sentence about
`Signal` gating is true of the movement path only. *Source: traits brief §5,
§6.*

### 55. `attach` and `gain_mass` disagree about the ceiling

`BodyDocument::attach` enforces no mass ceiling while `Organism::gain_mass`
clamps to `mass_ceiling_mg()`, and `land()` gives a new part the meal's whole
`biomass_mg` but only its root's half-extent — so incorporation can create a
part far above its own ceiling. A graft cost expressed as a fraction of mass is
unaffected; one expressed against the ceiling must resolve this first (see
entry 33). Re-verify against TD7's landed `rates.rs`. *Source: traits brief §3.*

### 56. The vello git patch may be retirable

Netrender's stack is published to the registry and the family is pinned to rev
`6f1a4fe7`, so the vello git patch is a possible retirement. Nothing depends on
it; it would simplify the resolution surface that produced the ninth trap.
*Source: played slice, Progress 2026-08-28 (later).*

### 57. The elements memo points at a section it does not have

Its status line sends the reader to "Section 10" for the decisions that are
Mark's; the document has seven sections and the list is §7, "What needs
ruling". *Source: elements memo, status line vs §7.*

---

## Notes on this snapshot

- Nothing recorded as ruled by a source document appears here. In particular:
  the played slice's scope, renderer and D5 deferral; D5 itself, resolved
  diegetically as "hunger routes the meal"; TD5, TD5b, TD6 and TD7; the scale
  plan's founding ruling and its deferral of the world tier above the
  enclosure; mosaic adjacency (2026-08-01); the payload asymmetry; and the
  adoption of a durable organism-to-organism edge as a direction (a ruling on
  the carrier only, not on its shape, key, storage or schedule).
- Two entries are superseded versions of what a reader might expect to find.
  The played slice's PS1 residues asked for a lens roster and a half-height
  near 9–11; the roster has since landed and the enclosure has since grown, so
  what remains are entries 10 and 9 in the scale plan's terms.
- The composable-forms research brief that TD7 asked for exists — it is
  `2026-08-29_forms_of_life_brief.md`. What is still owed on it is entry 12,
  acceptance of its scope.
- Several entries read code that TD7 was editing while the briefs were written
  (`organism.rs`, `ecology.rs`, `ecology/rates.rs`, `axis.rs`,
  `world/genesis.rs`). TD7 has since landed and S1 ran on top of it. Re-verify
  any entry that cites those files before acting on it.
- `DOC_README.md` indexes this file as a historical snapshot and points current
  integration work to the playable ecology plan.

## 2026-09-02 scoping

Mark scoped a deferred bucket on 2026-09-02: not raised again until its own
slice arrives. Listed here verbatim rather than re-litigated.

**Deferred bucket.**

- Forms-of-life brief's open questions (PE5).
- Traits brief cost formula and sub-questions.
- Meal destinations beyond burn and incorporate (D5).
- ProcessDef hybrid identity and portable shape (§12).
- S2 versus S3 order (PE6).
- Collapse terminal condition (PE7).
- Live versus postmortem truth.
- NPC acquisition via evidence.
- Reproduction default.
- Epoch boundary §8 items 2, 3, 5, 6.

**Stack rounds.**

- Workbench rendering surface.
- genet-probe verbs (found a world, repeat, pointer delivery, empty-value
  assert).
- Section picking.
- Banked names deme and borg.
