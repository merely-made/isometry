# Elements and Traits: How a Generated Vocabulary Becomes Real (2026-08-29)

**Status: memo, material scheme ruled 2026-09-02; refreshed 2026-09-01.** Mark
has reaffirmed the direction that world criteria shape generated biology.
Section 7 items 1 through 5 are ruled: storage is scheme A, payloads are
scheme C, fired on provenance, with no fields in PE4's first world and no
per-organism composition vectors. See the ruling note under §7 and the
[playable ecology plan](2026-08-31_playable_ecology_plan.md) §6 ruling 4 for
the full record. Companion to
[forms of life](2026-08-29_forms_of_life_brief.md) and
[traits and perception](2026-08-29_traits_and_perception_brief.md); it does not
restate them. Claims checked against `1b08f6c`.

**2026-08-31 boundary.** A world first realizes the laws, materials, fields,
and schedules that make biological options useful or impossible; generated
developmental candidates then draw from that realized vocabulary. Mechanical
verbs remain authored and bounded, while nouns, parameters, conditions,
placements, costs, and combinations may be generated. The
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns persistence,
reachability, anti-affix validation, and the first ordinary/impossible-world
proof. It deliberately does not choose scheme A, B, or C here.

## The question

> "Maybe worlds should generate a range of elements which are the preconditions
> for traits. More elements, more traits and combinations. Then you could have
> classical elements all the way up to periodic table levels of elements ->
> traits complexity. But it is unclear to me how we translate from or make real
> traits out of those elements… could do a closed set of obvious and necessary
> traits and an open, entirely element contingent set outside that… but it would
> be a mistake to lose out on insane, unexpected, and weird generated
> traits/elements/combinations…"

The hard part is that a generated trait must have **mechanical meaning**.
Generating names and combinations is trivial; generating behaviour is not.

## 0. Two corrections to the framing

**The closed/open seam is one level too late.** A closed set of necessary
traits plus an open contingent set makes two authorities that must agree, and
the closed one becomes a catalog — which `process.rs:13-18` already refuses in
its own comment: "a process vocabulary authored ahead of any consumer becomes a
catalog, which is the Spore failure at a smaller scale." The line belongs
between **verbs and nouns**: verbs closed and hand-audited, elements open and
generated, every trait a verb with element arguments.

**The instinct underneath survives, transformed.** "Obvious and necessary"
becomes a **worldgen reachability guarantee**, not a second mechanism: every
world must be able to make a producer, something that moves, and something that
senses, so the draw is constrained to produce elements satisfying the existing
verbs. One constraint on the sampler. No second rule set.

**And the verb set is not ours to design in advance either.** The engine ships
three processes — Contract, Intake, Sense — and says why: "the fourth arrives
when something asks." The general model plan's five-verb Technique axis is a
*different, unimplemented* set and is already ruled "a world profile, not engine
law." So "fixed verb" here means *the verbs that have consumers*, plus whatever
a played slice forces. (This retires the six-verb sketch — draw/resist/store/
emit/sense/require — offered in conversation the same day: authoring it ahead of
consumers is exactly the catalog error.)

**Standing blocker.** The terrarium does not breathe (TD6's finding, three
rounds running). Every scheme below adds dimensions to a search that has not
converged in the dimensions it has.

## 1. Three schemes

They differ on one question: **where does an element live, and which existing
code reads it?**

- **A — typed matter.** Elements are kinds of milligram, in soil and bodies.
- **B — coefficients.** Elements are property rows read by the rate formulas
  that already run. No new stock.
- **C — exchange payloads.** Elements are what happens when matter changes
  hands. The venom channel, generalized.

Not exclusive. Each is presented as if it were the whole answer, because each
can be.

### Storage shape shared by all three

The generated vocabulary rules out a compile-time Rust enum as material
authority. The saved, immutable material table gives each admitted definition a
world-local `MaterialId` and supplies its physical, visual, chemical, and
process properties. A rule-bearing voxel stores either that id directly or a
compact chunk/volume palette index into it. The current one-byte cell remains
the baseline while the admitted vocabulary fits; if a world exceeds it, the
gate compares a local palette with widening every cell under a measured memory
and lookup-cost receipt.

Dynamic quantities with their own spatial evolution remain separate planes.
Moisture, nutrient concentration, toxins, temperature, conductivity, and
charge have different update rates, dimensions, conservation laws, and
consumers. Packing every possible scalar into every voxel would make dormant
chemistry cost the same as active chemistry and would tie material identity to
field resolution. Small per-cell flags remain appropriate only when they
genuinely vary at cell frequency.

Body authority stays coarser where the rules are coarser. Parts and phenotype
records own mass, damage/integrity, material/composition references, process
allocation, and provenance;
field planes own spatial concentrations; rendering resolves ids to appearance.
The same fact is not copied into all three stores. Material density may price a
cell edit, but the accepted body transaction reconciles that integer transfer
against the part and world mass ledgers; a render-volume sum does not become a
second biomass authority.

The current projection is already a small proof of the indirection:
`mesocosm-mesh::Volume` stores one byte per cell, where zero is empty and every
other value is a material id, while `BodyDocument` carries only a
content-addressed `VolumeRef`. Teardown’s
[one-byte material palette](https://blog.voxagon.se/2020/12/03/spraycan.html)
is useful precedent for the memory shape and for the pressure created by
runtime variation. Its per-object palettes and 255-entry limit are not adopted
as Mesocosm rules.

### A — typed matter

`Soil.matter_mg` becomes E channels over the same columns; a body's composition
is a material index per part, read the way `Role` already is. A trait is a
`ProcessDef` in the shape the processdef plan ruled — typed input and output
ports, a site requirement, integer cost curves. Elements are the **port type
vocabulary**. There is no effect object: the consequence of a trait is that
matter moved.

Behaviour is automatic because `earn`, `pay_upkeep`, `draw_richest_within`,
`release_reserve` and the eight functions in `rates.rs` already read matter and
already price it. Type the matter and every one becomes selective without being
rewritten. Liebig's minimum — growth capped by the scarcest required element —
is one line, and niche separation, patchiness and succession fall out of
`percolate` plus `FORAGE_RADIUS` moving the shortage around.

*Worked:* an element with low column mobility, required by parts with an
actuator's geometry. Producers where it is thin can fix mass but not build
limbs, so `actuator_span()` reads 0, so the feeding-mode reading comes out
Producer, so crowding bites. A patch went sessile because of one number, through
existing code. *Second:* a decomposer whose intake port accepts an element that
only appears in corpses cannot start until something dies, and its income is
bounded by everything else's mortality — a number nobody set.

**Cost.** Per-tick is the real one: `percolate` is a full pass over every
column, proportional to E **whether or not anything wants the element** — 1,089
column-visits per channel per tick. Snapshot size is *not* the constraint;
every `state_hash` caller is in `examples/` or `tests/`, so it is receipt
cadence. Uptake is *not* proportional to E either: `draw_richest_within` scans
for what a root wants, so read cost tracks appetite, not vocabulary. 400-700
LOC in core.

**Fails by:** breaking the crown receipt first — `tests/matter.rs` is
milligram-exact with no exceptions, and a per-channel leak that another channel
absorbs would pass a total-mass test. **The test must be rewritten before the
feature, not after.** Then: Liebig on an ecology that does not yet close makes
the balance problem E-dimensional.

### B — coefficients

An element is a row of numbers read by kernels that already exist; a lineage's
composition blends them. No new stock anywhere.

*Worked:* an element with high ceiling, high rent and high conspicuousness
coefficients. A lineage concentrating it grows larger, pays more per tick, and
is visible further — so it survives only where income is abundant, and there it
is what everything else can see. Nobody wrote "big animals are conspicuous and
expensive"; three numbers on one row did.

**Cost.** Per-column state zero — the scheme's whole argument. Per-lineage
E=100 × 50 lineages × 2 B ≈ 10 KB, free. **Per-organism vectors are refused:**
at the scale plan's ~4,700 saturation, E=100 per organism is 470,000 serialized
entries inside `state_hash`. If individuals must differ, differ them by which
parts they grew. 250-400 LOC, mostly one file, conservation untouched.

**Fails by:** being an affix system unless §2 is enforced from day one —
coefficients into shared accumulators are commutative, and two elements feeding
the same accumulator *are* one element. Worst discoverability of the three: a
coefficient shift is invisible, and the plan's own rule ("generated rules must
be discoverable or generation has bought nothing a content pack would not
have") bites hardest here. And nothing connects composition to the world unless
it is made downstream of feeding — which drags in a piece of A.

### C — exchange payloads

An element is a **rule that fires when matter changes hands**, plus numbers that
shape it. The engine has exactly three transfer sites and all are written: soil
draw/deposit, the meal, and death/decay. A trait is a payload carried by a
body's matter plus a delivery route from a closed short list (on contact, on
being eaten, on death, into the column). An effect is a typed condition with an
integer envelope — severity, onset, peak, duration — **computed from the
element's own numbers, not authored per element**.

`venom_mg` is the existing one-element instance, and `act.rs:283-292` is the
template that makes it legal: what the venom cost the eater is deposited back
into the column under the corpse. Nothing evaporates. Every generated payload
follows that pattern.

*Worked:* a payload that fires on death into the column. A lineage carrying it
poisons the ground where it dies; percolation spreads it over a radius
comparable to the forage radius, so a stand suffering heavy mortality degrades
its own patch and `draw_richest_within` makes producers walk away from it.
Nobody wrote succession. *Second:* `Signal` already splits claim from truth, so
generating payload and signal separately gives Batesian mimicry, aggressive
mimicry and honest advertisement as three points in one space — with no mimicry
system.

**Cost.** Bounded condition slots per organism inside `state_hash` — bound them
hard, 2-4 fixed-width, or it is unbounded state at saturation. Per-column zero
unless a payload lands in soil. Application is at transfer sites, already
O(meals). Observer side rides the traits brief's cheap formulation (a bitset on
`LivingTarget`, which is `Copy` and not `Serialize`, so widening is
hash-neutral); **do not widen `SENSOR_COUNT`** — that re-indexes every evolved
policy. 300-500 LOC, plus closing a live inconsistency: only the *played* meal
charges venom today.

**Fails by:** opacity (DF's failure — symptoms reported, causal chains never;
the audit trail exists but must be surfaced); the pairwise table, unless the
already-ruled asymmetry holds — **payloads act on bodies, payloads do not act
on payloads**; and the UO trap, nearest here — a payload table keyed by element
identity is a lookup in a chemistry costume. The guard: if you can delete the
row and recompute it from the element's properties, it is real.

## 2. What structurally prevents "draw from X / resist Y"

Four tests, cheap and mechanical, applying to all three schemes. Without them
all three become affix systems; with them none can.

1. **Three consumers.** Refuse any element property read by fewer than three
   existing kernels — rent, income, ceiling, soil transport, perception,
   reproduction, decay, return. A property with one consumer *is* the reskin,
   mechanised.
2. **Disjoint subsets.** Each verb reads a *different* subset. If every verb
   reads the same three numbers, that is one axis with several names. Hardness
   matters for structure and not for signalling; density buys ceiling and taxes
   locomotion. That is what makes an element good at one thing and bad at
   another rather than "strong."
3. **A world write-path.** At least half the verbs must change what *leaves* the
   organism. A trait whose only effect is on its own numbers is terminal, and
   terminal is what samey means. B needs this most and has it least.
4. **Fingerprint and reject collisions at worldgen.** Run each generated element
   through a fixed battery on a reference body; two elements with the same
   fingerprint are one element with two names — DF's divine metals, identical at
   1,000,000 kPa each. Reject at generation; deterministic and integer-only.

**The standing instrument, not a one-off check.** Generate N lineages, run a
fixed battery (matter throughput, survival under shading, predation success,
detection range), build the fingerprint matrix, take its **rank**. If nearly all
variance sits in two or three components, it is an affix system in costume. Run
it at the classical setting and the large setting: **if rank does not rise with
element count, the elements are not reaching the formulas.** That is the single
measurement that says whether the idea works.

**The advantage already owned.** Milligram conservation plus ceilings means
expression is paid in mass some other trait then cannot have — `part_ceiling_mg`
per part, `mass_ceiling_mg` per body, the ruled allocation mosaic per organ.
Addition into a shared accumulator is the Diablo failure; a shared conserved
budget under a ceiling is structurally not addition. **This is the largest
structural advantage the design holds. Do not build any expression path that
bypasses it.**

## 3. Second-order: elements organisms make

Most of biology's weirdness is organism-produced, not geological. The mechanical
version is one decision: **organism-produced material enters the same element
table as worldgen's, with the same property vector and the same verbs.** Geology
gives a static basis; the biotic table grows and moves.

Unusually cheap here, because the return path already runs: `release_reserve`
deposits a corpse's reserve into its column, carrion decay returns substance,
the meal's unkept remainder is deposited, and `percolate` spreads all of it. So
under A the soil's element map is downstream of who lived and died where; under
C a corpse's payload is a property of the ground; under B it means nothing
unless composition is made downstream of feeding.

**Mosaic adjacency — ruled 2026-08-01 and uncited by either 08-29 brief.**
Neighbouring sites in a part's allocation mosaic "can cooperate, interfere, or
expose a new transformation without rewriting either parent," and use plus
survival plus repetition makes the compound eligible for the adaptation bank.
That is unexpected combination **generated by play and gated by survival**,
committed at the epoch boundary. It is strictly better than a rarity-weighted
draw table because it cannot be farmed and it is already tied to living. If
"insane, unexpected, weird" wants a designed source rather than a hoped-for one,
this is it, and it is already ruled.

**On rarity tiers.** Do not build a 1/2/3/4/5 ladder. The epoch boundary plan's
significance-as-abnormality is world-relative and retrospective — the world gets
harder to impress and nobody authors that. A generated element's interest is how
unlike this world's history it is, not a tier assigned to it.

## 4. The element-count arithmetic

The 4-to-100+ range is not one number. It is a budget across three tiers.

**Tier 1 — fields (expensive).** Per-column state, and 1,089 column-visits per
channel per tick in `percolate` whether or not anything uses it. Against a
measured 133 µs/tick at 75 bodies: 4 channels is noise, 8 is noticeable, 20 is
comparable to the whole tick, 100 is 5-10x the whole tick. **Affordable dense
fields: about 4 to 8.** Sparse changes this — a rare element in 20 columns costs
20 entries — but sparse must be a sorted `Vec<(Column, u64)>`, never a
`HashMap`: iteration order reaches `state_hash`.

**Tier 2 — body elements (moderate).** A material index per part is a byte,
read through a walk the tick already performs. Per-lineage composition is free.
**Per-organism vectors: refuse.** Tier 2 comfortably carries 20-40.

**Tier 3 — world constants (free).** Genesis-time coefficients, hashed once.
Unlimited for these purposes.

**So a hundred-element world is affordable** — as ~4-8 fields, 20-40 body
materials, and the rest constants and payload participants. What is not
affordable is a hundred elements that are all fields. Which tier an element
lands in is drawn at worldgen, with the budget given to the sampler as a
constraint.

**And light is not a field today.** Producer income is allometric on body mass
divided by a crowding count; there is no per-column intensity for an absorption
band to read. Element-selective light capture must first build a light field —
Tier 1, with Tier 1's cost, on top of whatever it enables.

### Field dimensionality is part of admission

The 4-to-8 estimate above is
for full per-column sweeps in the current terrarium. It does not authorize four
to eight dense 3D volumes. Every field states its consumer, domain, resolution,
cadence, interpolation, sources and sinks, boundary conditions, numeric range,
conservation rule, far-tier reduction, and cost receipt. Soil nutrients and
moisture can begin as columns or shallow layers; light can be a derived
attenuation reading; gas, clouds, or submerged chemistry earn full 3D only when
vertical transport changes play; roots and mycelia remain explicit transport
networks when direction and connection matter. The
[VDB paper](https://doi.org/10.1145/2487228.2487235) is a storage reference if
a sparse, dynamic 3D field eventually binds, not a reason to introduce one.

## 5. Scoreboard

| | A: typed matter | B: coefficients | C: exchange payloads |
|---|---|---|---|
| Fixed-verb grammar | native (`ProcessDef` ports) | verbs = whatever `rates.rs` holds | native (closed route, open payload) |
| Exclusion relation | native (port mismatch) | weakest — blending has no refusal | partial — needs the asymmetry law |
| UO comprehensibility | passes both halves | passes computation, fails discovery | passes if computed, fails if looked up |
| F0 proof-first | worst — a vocabulary layer | poor | closest — half-built |
| Conservation | hardest — per-channel receipt | untouched | template exists |
| Per-tick cost | O(columns × E), unavoidable | ~free | O(meals), bounded |
| Second-order free | yes | no | yes |
| Affix risk without §2 | low | **high** | medium |

## 6. The first proof, per scheme

F0 is one unusual carrier state, one cost, one application route, one
discoverable consequence — **no registry, no shared type**. None of the three is
F0; each has an F0-shaped ancestor, and building it is what says whether the
vocabulary layer is worth it.

- **A's ancestor:** one second matter channel, hardcoded. Done when the
  conservation receipt passes per-channel and a probe shows lineages sorting
  spatially by which columns hold it.
- **B's ancestor:** one coefficient wired into three named kernels, with the
  fingerprint battery built at the same time. Done when the rank measurement
  distinguishes two populations in the sim, not on paper.
- **C's ancestor:** **fix venom first.** Make the ecology charge it on NPC meals
  the way the played path does, with the spill deposited to the column. It is a
  bug fix, it closes a live inconsistency, and it is exactly the envelope C
  generalizes. **Worth doing regardless of which scheme is chosen**, which is
  the strongest sequencing argument available and requires choosing nothing.

Also: `Lineages::distance` / `World::kinship` are built, correct, tested, and
have **zero production callers**, and "migration following kinship rather than
distance" is already on F0's sanctioned candidate list. The cheapest fantastical
slice in the repo is a function waiting for its first caller.

## 7. What needs ruling

**Items 1 through 5 ruled by Mark, 2026-09-02.** Matter is typed by
provenance: a milligram carries where it came from, kingdom first (flora,
fauna, myco, micro, and the meso/macro scale words the world already uses)
and lineage under it. Storage is scheme A's (typed stock in soil and
bodies, per-channel conservation, the matter test rewritten first) with the
type vocabulary world-derived from the roster rather than an authored
element table. Payloads are scheme C's and fire on provenance at the three
transfer sites. Payloads are part of the generative pipeline: by the time a
world has a roster it has its payloads. No fields in PE4's first world.
Composition is not per-organism; see the [playable ecology plan](2026-08-31_playable_ecology_plan.md)
§6 ruling 4 for the full ruling, including the two composition layers and
the disfavoured-element-pair answer. The small typed chunk of matter this
section calls "element" is under a naming round; do not write "element" as a
new term. Full detail below is kept for the record; the numbered questions
are answered, not deleted.

1. ~~Which seam first: A, B, or C?~~ **Ruled: both, not exclusive.** Storage
   is scheme A's, payloads are scheme C's, firing on provenance.
2. ~~May elements be fields at all before the terrarium breathes?~~ **Ruled:
   no fields in PE4's first world.**
3. ~~The field budget: how many dense channels?~~ **Ruled: moot per item 2**
   until fields are introduced.
4. ~~Composition per lineage, per part, or per organism?~~ **Ruled: two
   layers, lineage and part, no per-organism vectors.** See the playable
   ecology plan §6 ruling 4.
5. ~~The processdef plan's open ruling: is a disfavoured element pair a hard
   gate or an expensive but recoverable graft?~~ **Ruled: a graft with
   conditioned limits**: a small milligram allowance with penalties that
   trait conditions raise, composing with PE2's condition table.
6. **Does "obvious and necessary" become a worldgen reachability guarantee**, or
   is a genuinely separate closed set wanted? `process.rs`'s own comment refuses
   the latter.
7. **The word.** "Element" is already used in this repo in the Genshin/BOTW
   sense — a field channel reacting with surfaces. This is a compositional
   precondition. Two mechanics, one word, one repo: **needs a naming round**
   before either is written down. No name coined here.
8. **Sub-part body mutation.** If a bite or developmental change must alter
   cells inside one part, does the body transaction write a new immutable
   content-addressed volume or a deterministic per-body patch over its base?
   Shared `VolumeRef` content cannot be edited in place. Whole-part severing
   remains the cheaper incumbent until a played case proves it insufficient.
