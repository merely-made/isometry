//! Source-derived geographic projection for a campaign region.
//!
//! The region map is the authority for terrain and bounds. World-place
//! coordinates and region transitions are the authority for site anchors;
//! this adapter does not manufacture Voronoi geography from the discovered
//! graph. Hosts can hand these owned records to the shared atlas canvas.

use isometry_campaign::{CampaignMap, CampaignWorld, MapScale};
use sceno::{Footprint, Vec2};
use std::collections::BTreeMap;

/// A fixed map-space rectangle. Its origin is always `(0, 0)` and its extent
/// is the complete retained region map, even when the party knows one site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasBounds {
    pub width: f32,
    pub height: f32,
}

/// One retained region cell, including the open tile vocabulary used by the
/// campaign pack. Hosts choose its appearance; this projection stays neutral.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasTerrainCell {
    pub col: u32,
    pub row: u32,
    pub kind: String,
}

/// A geographic site area. The footprint is deliberately a source-bound cell
/// footprint around a region transition, rather than a graph-layout hull.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasSite {
    pub id: String,
    pub label: String,
    pub anchor: Vec2,
    pub footprint: Footprint,
    pub priority: u32,
}

/// A route in region coordinates. The route remains world-owned by its ids;
/// this record only supplies endpoints for drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasRoute {
    pub id: String,
    pub from: Vec2,
    pub to: Vec2,
}

/// Complete source-derived atlas data for one selected region.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasProjection {
    pub region_id: String,
    pub bounds: AtlasBounds,
    pub terrain: Vec<AtlasTerrainCell>,
    pub sites: Vec<AtlasSite>,
    pub routes: Vec<AtlasRoute>,
    /// Normalized home positions for the existing graph-node API. These use
    /// the same fixed bounds as terrain and remain stable as discovery grows.
    pub positions: BTreeMap<String, (f32, f32)>,
}

/// Project a retained region map and campaign world into geographic atlas
/// data. `party` is explicit so a generic adapter never assumes a watchtower
/// id or a particular owner. `None` is the intentional graph fallback: callers
/// can continue using the ordinary discovered overmap when no region is
/// selected or its retained map is unavailable.
pub fn atlas_projection(
    region: Option<&CampaignMap>,
    world: &CampaignWorld,
    party: &str,
) -> Option<AtlasProjection> {
    atlas_projection_with_terrain(region, world, party, None)
}

pub(crate) fn atlas_projection_with_terrain(
    region: Option<&CampaignMap>,
    world: &CampaignWorld,
    party: &str,
    retained_terrain: Option<Vec<AtlasTerrainCell>>,
) -> Option<AtlasProjection> {
    let region = region.filter(|map| map.scale == MapScale::Region)?;
    let known = world.party_known.get(party);
    let width = region.document.ground.width();
    let height = region.document.ground.height();
    if width == 0 || height == 0 {
        return None;
    }

    let terrain = retained_terrain.unwrap_or_else(|| {
        let mut terrain = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            for col in 0..width {
                let kind = region
                    .document
                    .ground
                    .get(col, row)
                    .and_then(|id| region.document.tile_kinds.get(id.0 as usize))
                    .cloned()
                    .unwrap_or_else(|| "empty".to_owned());
                terrain.push(AtlasTerrainCell { col, row, kind });
            }
        }
        terrain
    });

    let mut sites = Vec::new();
    for transition in &region.transitions {
        let mut places = world
            .places
            .values()
            .filter(|place| place.map.as_deref() == Some(transition.target_map.as_str()));
        let Some(place) = places.next() else {
            continue;
        };
        if places.next().is_some() || sites.iter().any(|site: &AtlasSite| site.id == place.id) {
            continue;
        }
        if !known.is_some_and(|ids| ids.contains(&place.id)) {
            continue;
        }
        let anchor = place
            .position
            .map(|(col, row)| Vec2::new(col as f32 + 0.5, row as f32 + 0.5))
            .unwrap_or_else(|| {
                Vec2::new(
                    transition.at.col as f32 + 0.5,
                    transition.at.row as f32 + 0.5,
                )
            });
        sites.push(AtlasSite {
            id: place.id.clone(),
            label: place.name.clone(),
            anchor,
            footprint: cell_footprint(),
            priority: 2,
        });
    }
    // The region itself is a place too. It has no transition to itself, so add
    // it from the authored world coordinate when it is known.
    if let Some(place) = world
        .places
        .values()
        .find(|place| place.map.as_deref() == Some(region.id.as_str()))
    {
        if known.is_some_and(|ids| ids.contains(&place.id)) {
            if let Some((col, row)) = place.position {
                let anchor = Vec2::new(col as f32 + 0.5, row as f32 + 0.5);
                sites.push(AtlasSite {
                    id: place.id.clone(),
                    label: place.name.clone(),
                    anchor,
                    footprint: cell_footprint(),
                    priority: 1,
                });
            }
        }
    }

    let routes = world
        .routes
        .values()
        .filter_map(|route| {
            let from = sites.iter().find(|site| site.id == route.from)?.anchor;
            let to = sites.iter().find(|site| site.id == route.to)?.anchor;
            Some(AtlasRoute {
                id: route.id.clone(),
                from,
                to,
            })
        })
        .collect();

    let positions = sites
        .iter()
        .map(|site| {
            (
                site.id.clone(),
                (site.anchor.x / width as f32, site.anchor.y / height as f32),
            )
        })
        .collect();
    Some(AtlasProjection {
        region_id: region.id.clone(),
        bounds: AtlasBounds {
            width: width as f32,
            height: height as f32,
        },
        terrain,
        sites,
        routes,
        positions,
    })
}

fn cell_footprint() -> Footprint {
    Footprint::Polygon {
        points: vec![
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, -0.5),
            Vec2::new(0.5, 0.5),
            Vec2::new(-0.5, 0.5),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_campaign::{MapPoint, MapTransition, WorldPlace, WorldRoute};
    use isometry_core::MapDocument;
    use std::collections::BTreeSet;

    fn fixture() -> (CampaignMap, CampaignWorld) {
        let mut document = MapDocument::new("Reach", 4, 3);
        let forest = document.intern_tile_kind("forest-floor");
        let water = document.intern_tile_kind("water");
        for row in 0..3 {
            for col in 0..4 {
                document.ground.set(col, row, forest);
            }
        }
        document.ground.set(3, 0, water);
        let map = CampaignMap {
            id: "region".into(),
            scale: MapScale::Region,
            document,
            spawn_zones: vec![],
            transitions: vec![MapTransition {
                id: "east".into(),
                at: MapPoint { col: 3, row: 1 },
                target_map: "east".into(),
                target_entry: None,
            }],
            encounter_anchors: vec![],
        };
        let mut world = CampaignWorld::default();
        world.places.insert(
            "east".into(),
            WorldPlace {
                id: "east".into(),
                name: "East site".into(),
                tags: vec![],
                map: Some("east".into()),
                position: Some((3, 1)),
            },
        );
        world.places.insert(
            "hidden".into(),
            WorldPlace {
                id: "hidden".into(),
                name: "Hidden site".into(),
                tags: vec![],
                map: Some("hidden".into()),
                position: Some((1, 1)),
            },
        );
        world.places.insert(
            "region".into(),
            WorldPlace {
                id: "region".into(),
                name: "Reach".into(),
                tags: vec!["region".into()],
                map: Some("region".into()),
                position: Some((1, 1)),
            },
        );
        world.routes.insert(
            "east-route".into(),
            WorldRoute {
                id: "east-route".into(),
                from: "region".into(),
                to: "east".into(),
                tags: vec![],
                weight: 1,
            },
        );
        world.party_known.insert(
            "party".into(),
            BTreeSet::from(["region".into(), "east".into()]),
        );
        (map, world)
    }

    #[test]
    fn fixed_bounds_and_terrain_survive_discovery_filter() {
        let (map, world) = fixture();
        let projection = atlas_projection(Some(&map), &world, "party").unwrap();
        assert_eq!(
            projection.bounds,
            AtlasBounds {
                width: 4.0,
                height: 3.0
            }
        );
        assert_eq!(projection.terrain.len(), 12);
        assert_eq!(
            projection
                .terrain
                .iter()
                .filter(|cell| cell.kind == "water")
                .count(),
            1
        );
        assert_eq!(
            projection
                .sites
                .iter()
                .map(|site| site.id.as_str())
                .collect::<Vec<_>>(),
            ["east", "region"]
        );
    }

    #[test]
    fn unknown_site_details_and_routes_are_filtered() {
        let (map, mut world) = fixture();
        world.routes.insert(
            "hidden-route".into(),
            WorldRoute {
                id: "hidden-route".into(),
                from: "region".into(),
                to: "hidden".into(),
                tags: vec![],
                weight: 1,
            },
        );
        let projection = atlas_projection(Some(&map), &world, "party").unwrap();
        assert!(!projection.sites.iter().any(|site| site.id == "hidden"));
        assert_eq!(
            projection
                .routes
                .iter()
                .map(|route| route.id.as_str())
                .collect::<Vec<_>>(),
            ["east-route"]
        );
    }

    #[test]
    fn site_footprints_pick_area_without_rectangle_corners() {
        let (map, world) = fixture();
        let projection = atlas_projection(Some(&map), &world, "party").unwrap();
        let east = projection
            .sites
            .iter()
            .find(|site| site.id == "east")
            .unwrap();
        assert!(east.footprint.contains(Vec2::ZERO));
        assert!(!east.footprint.contains(Vec2::new(0.6, 0.6)));
    }

    #[test]
    fn absent_region_preserves_graph_fallback() {
        let (_, world) = fixture();
        assert!(atlas_projection(None, &world, "party").is_none());
    }

    #[test]
    fn transition_target_resolves_place_by_map_id() {
        let (mut map, mut world) = fixture();
        map.transitions[0].target_map = "map:east".into();
        world.places.get_mut("east").unwrap().map = Some("map:east".into());
        world
            .party_known
            .get_mut("party")
            .unwrap()
            .insert("east".into());
        let projection = atlas_projection(Some(&map), &world, "party").unwrap();
        assert!(projection.sites.iter().any(|site| site.id == "east"));
    }

    #[test]
    fn discovering_another_site_does_not_move_existing_home_position() {
        let (map, mut world) = fixture();
        let before = atlas_projection(Some(&map), &world, "party").unwrap();
        world
            .party_known
            .get_mut("party")
            .unwrap()
            .insert("hidden".into());
        let after = atlas_projection(Some(&map), &world, "party").unwrap();
        assert_eq!(before.positions["east"], after.positions["east"]);
        assert_eq!(before.bounds, after.bounds);
    }
}
