//! Isometry's consumer adapter for Cambium's fixed-world graph atlas.

use std::sync::Arc;

use cambium::{
    GraphCanvasAtlas, GraphCanvasAtlasField, GraphCanvasAtlasPaint, GraphCanvasAtlasRoute,
};
use isometry_campaign::{CampaignMap, CampaignWorld, MapScale};
use sceno::{Rect, Size2, Vec2};
use sprigging::ColorF;

use super::atlas::atlas_projection_with_terrain;
use crate::state::UiState;

pub(crate) struct AtlasTerrainCache {
    key: u64,
    paint: Arc<[GraphCanvasAtlasPaint]>,
}

/// Build the atlas for the region containing the party's current world place.
/// Source-time maps and world are read as a pair, so historical terrain cannot
/// be combined with live labels. A missing/ambiguous source leaves the graph
/// fallback in charge.
pub fn overmap_atlas(ui: &UiState) -> Option<GraphCanvasAtlas<String>> {
    let party = ui.viewer.as_deref().unwrap_or("dm");
    let world = ui.overmap_source_world();
    let maps = ui.overmap_source_maps();
    let region = select_region_map(world, maps, party)?;
    let key = terrain_key(region);
    let cached = ui
        .overmap_atlas_terrain_cache
        .borrow()
        .as_ref()
        .filter(|cached| cached.key == key)
        .is_some();
    let projection =
        atlas_projection_with_terrain(Some(region), world, party, cached.then(Vec::new))?;
    if projection.bounds.width <= 0.0 || projection.bounds.height <= 0.0 {
        return None;
    }
    let bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(projection.bounds.width, projection.bounds.height),
    );

    let paint = terrain_paint_cached(ui, key, &projection);
    let fields = projection
        .sites
        .iter()
        .map(|site| GraphCanvasAtlasField {
            id: site.id.clone(),
            anchor: site.anchor,
            footprint: site.footprint.clone(),
            priority: site.priority as i16,
            label: site.label.clone(),
        })
        .collect();
    let discovered = world.overmap_for(party);
    let known = world.party_known.get(party);
    let route_cells: std::collections::BTreeMap<_, _> = world
        .routes
        .values()
        .filter(|route| {
            known.is_some_and(|ids| ids.contains(&route.from) && ids.contains(&route.to))
        })
        .zip(&discovered.edges)
        .enumerate()
        .map(|(index, (route, edge))| (route.id.as_str(), super::overmap_relation_id(index, edge)))
        .collect();
    let routes: Vec<_> = projection
        .routes
        .iter()
        .filter(|route| {
            route_cells
                .get(route.id.as_str())
                .is_none_or(|id| !ui.overmap_hidden_relations.contains(id))
        })
        .map(|route| GraphCanvasAtlasRoute {
            points: vec![route.from, route.to],
            color: ColorF {
                r: 0.55,
                g: 0.36,
                b: 0.18,
                a: 0.9,
            },
            width: if route_cells
                .get(route.id.as_str())
                .is_some_and(|id| ui.overmap_selected_relation.as_ref() == Some(id))
            {
                0.22
            } else {
                0.10
            },
        })
        .collect();
    Some(
        GraphCanvasAtlas::new(bounds)
            .with_paint(paint)
            .with_fields(fields)
            .with_routes(Arc::<[GraphCanvasAtlasRoute]>::from(routes)),
    )
}

fn select_region_map<'a>(
    world: &CampaignWorld,
    maps: &'a std::collections::BTreeMap<String, CampaignMap>,
    party: &str,
) -> Option<&'a CampaignMap> {
    let place_id = world.party_at(party)?;
    let map_id = world.places.get(place_id)?.map.as_deref()?;
    if let Some(map) = maps.get(map_id).filter(|map| map.scale == MapScale::Region) {
        return Some(map);
    }
    let mut candidates = maps.values().filter(|map| {
        map.scale == MapScale::Region
            && map
                .transitions
                .iter()
                .any(|transition| transition.target_map == map_id)
    });
    let candidate = candidates.next()?;
    candidates.next().is_none().then_some(candidate)
}

fn terrain_key(map: &CampaignMap) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    map.id.hash(&mut hasher);
    map.document.ground.width().hash(&mut hasher);
    map.document.ground.height().hash(&mut hasher);
    map.document.tile_kinds.hash(&mut hasher);
    for row in 0..map.document.ground.height() {
        for col in 0..map.document.ground.width() {
            map.document.ground.get(col, row).hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn terrain_paint_cached(
    ui: &UiState,
    key: u64,
    projection: &super::AtlasProjection,
) -> Arc<[GraphCanvasAtlasPaint]> {
    let mut cache = ui.overmap_atlas_terrain_cache.borrow_mut();
    if let Some(cached) = cache.as_ref().filter(|cached| cached.key == key) {
        return Arc::clone(&cached.paint);
    }
    let paint: Arc<[GraphCanvasAtlasPaint]> =
        super::atlas_terrain::terrain_paint(&projection.terrain).into();
    *cache = Some(AtlasTerrainCache {
        key,
        paint: Arc::clone(&paint),
    });
    paint
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_campaign::{CampaignMap, MapPoint, MapTransition, WorldPlace};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn selection_follows_party_place_and_region_scale() {
        let mut world = CampaignWorld::default();
        world.places.insert(
            "site".into(),
            WorldPlace {
                id: "site".into(),
                name: "Site".into(),
                tags: vec![],
                map: Some("region".into()),
                position: None,
            },
        );
        world.party_node.insert("party".into(), "site".into());
        let mut maps = BTreeMap::new();
        maps.insert(
            "region".into(),
            CampaignMap {
                id: "region".into(),
                scale: MapScale::Region,
                document: isometry_core::MapDocument::new("r", 1, 1),
                spawn_zones: vec![],
                transitions: vec![],
                encounter_anchors: vec![],
            },
        );
        assert_eq!(
            select_region_map(&world, &maps, "party").map(|map| map.id.as_str()),
            Some("region")
        );
    }

    #[test]
    fn missing_party_region_is_graph_fallback() {
        let world = CampaignWorld::default();
        assert!(select_region_map(&world, &BTreeMap::new(), "party").is_none());
    }

    fn cached_ui() -> UiState {
        let mut ui = UiState::new(isometry_core::MapDocument::new("local", 1, 1));
        let mut document = isometry_core::MapDocument::new("region", 2, 2);
        let forest = document.intern_tile_kind("forest-floor");
        for row in 0..2 {
            for col in 0..2 {
                document.ground.set(col, row, forest);
            }
        }
        ui.campaign_maps.insert(
            "region".into(),
            CampaignMap {
                id: "region".into(),
                scale: MapScale::Region,
                document,
                spawn_zones: vec![],
                transitions: vec![],
                encounter_anchors: vec![],
            },
        );
        ui.world.places.insert(
            "region-place".into(),
            WorldPlace {
                id: "region-place".into(),
                name: "Region".into(),
                tags: vec!["region".into()],
                map: Some("region".into()),
                position: Some((0, 0)),
            },
        );
        ui.world
            .party_node
            .insert("dm".into(), "region-place".into());
        ui.world
            .party_known
            .insert("dm".into(), BTreeSet::from(["region-place".into()]));
        ui
    }

    #[test]
    fn terrain_arc_is_reused_and_invalidated_by_source_content() {
        let mut ui = cached_ui();
        let first = overmap_atlas(&ui).unwrap();
        let second = overmap_atlas(&ui).unwrap();
        assert_eq!(Arc::as_ptr(&first.paint), Arc::as_ptr(&second.paint));
        ui.campaign_maps
            .get_mut("region")
            .unwrap()
            .document
            .tile_kinds
            .push("water".into());
        let third = overmap_atlas(&ui).unwrap();
        assert_ne!(Arc::as_ptr(&first.paint), Arc::as_ptr(&third.paint));
    }

    #[test]
    fn local_place_selects_unique_containing_region_and_rejects_ambiguity() {
        let mut world = CampaignWorld::default();
        world.places.insert(
            "tower".into(),
            WorldPlace {
                id: "tower".into(),
                name: "Tower".into(),
                tags: vec![],
                map: Some("tower-map".into()),
                position: None,
            },
        );
        world.party_node.insert("party".into(), "tower".into());
        let region = |id: &str| CampaignMap {
            id: id.into(),
            scale: MapScale::Region,
            document: isometry_core::MapDocument::new(id, 2, 2),
            spawn_zones: vec![],
            transitions: vec![MapTransition {
                id: "tower".into(),
                at: MapPoint { col: 0, row: 0 },
                target_map: "tower-map".into(),
                target_entry: None,
            }],
            encounter_anchors: vec![],
        };
        let mut maps = BTreeMap::new();
        maps.insert("north".into(), region("north"));
        assert_eq!(
            select_region_map(&world, &maps, "party").map(|map| map.id.as_str()),
            Some("north")
        );
        maps.insert("south".into(), region("south"));
        assert!(select_region_map(&world, &maps, "party").is_none());
    }
}
