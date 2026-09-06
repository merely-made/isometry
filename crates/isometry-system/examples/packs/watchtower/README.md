# Ash and Bells Watchtower

This original 5e-SRD content pack opens on a ruined frontier watchtower. A
party approaches over broken ground, meets the scavengers Brin and Otta, and
finds the stranded traveler Elian below a broken bell. A tower-beast has made
its nest on either the north or south upper ledge; the other ledge holds an
abandoned scrape. The breach also shifts east or west, while the west approach
remains usable in every generation.

Beyond the tower, the campaign includes the Bellwood Reach region map, a forest
road entry, and an Alder Stream clearing. Their transitions form a small loop:
each local site returns to the region, and the region points back to all three.
Forest trees are explicit canopy props on traversable forest floor; stone
crossing cells keep the stream from becoming a movement dead end.
The tower's rear lookout rises to height 6, with steps from the bell platform
through heights 4 and 5. It clears the height-3 canopy. Trees and intervening terrain still
hide lower ground where the sight line passes through them.

The scene supports a parley with the scavengers or a fight among the stones.
The conclusion storylet leaves the choice of what happens at the bell to the
table. The generator proposes a campaign; it never resolves an encounter.

## Generate and commit

The pack is bundled and available as `>gen watchtower` (or by selecting
`watchtower:ruined_tower`). Set `ISOMETRY_GEN_SEED` when a repeatable seed is
needed, or reroll the preview before committing to try another layout. The
`Breach on the east` lock preset fixes the breach while leaving the nest seeded. Use **Generate** to
create the proposal, then **Commit** when the DM accepts that proposal into the
campaign.

Commit once in a fresh session. A later different layout uses the same map
identity and is refused; it does not replace the campaign you have played.
Mira belongs to `player`, matching the default joining client. To use another
player name, change the owner before hosting the scene.

The generated map is 15 by 15. Start at the `party` spawn zone and climb the
stone approach toward the breach. All five inhabitants use the existing
`5e-srd` sheet vocabulary. `tower-beast` is an appearance key for a separately
baked quadruped sprite.

The region and site maps use the existing `CampaignDraft` region/local map and
transition metadata. Movement evaluates ground kinds; trees mark traversable
forest cells rather than solid trunk collisions. Desktop fog uses finite prop
heights and terrain elevation. These remain Isometry maps; cross-game world
and body import are not implemented by this pack.

## Create a character

Select a free tile, click **Character**, enter a name and optional player owner,
choose an appearance, then **create** (or Enter). The host attaches the loaded
system's default sheet and opens it for stat adjustments. Blank owner means
DM-controlled. A full board, water or unavailable system cannot leave a partial
character. Characters currently belong to their map and travel through the
region doors with their sheets; a reusable character library is a later step.
Hosted creation replicates token and sheet together in protocol v4. All peers
must run a matching updated build.

## GM hooks

Brin wants water and a safe road for the camp. Otta wants the party kept away
from the nest until Elian is out of danger. Elian can point out which fallen
stones make the safest approach. The tower-beast can be lured away from the
inner ledge with food or a convincing noise if the table invents a way to do
it; the pack leaves that adjudication to the GM.

In Play mode, click a token and then a highlighted reachable tile to move it.
Click a target token to choose it for an action, or right-click a token for its
context menu. Arrow keys pan, `r` changes facing, `Enter` ends the active turn,
and `f` cycles the fog viewer.
