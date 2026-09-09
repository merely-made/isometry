// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Captures actual founded relatives, plus their admitted content and identities.
//! Usage: cargo run --release -p mesocosm-genet --example voxel_families -- OUT_DIR [spaced|jointed|branching|axial]

use mesocosm_core::{World, axis::archetype};
use mesocosm_mesh::{content::ContentPack, mesh_body};
use mesocosm_render::{Camera, Renderer, SceneItem, kingdom_colour};
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("an output directory is required"),
    );
    std::fs::create_dir_all(&out).unwrap();
    let layout = mesocosm_genet::played::BodyLayout::parse(
        &std::env::args().nth(2).unwrap_or_else(|| "spaced".into()),
    )
    .expect("layout wants spaced, jointed, branching or axial");
    let pack = ContentPack::generate(layout.founding().palette()).unwrap();
    let volumes = pack.resolve().unwrap();
    let world = World::founded_with_palette(7, 120, layout.founding(), pack.palette).unwrap();
    std::fs::write(
        out.join("world.snapshot"),
        mesocosm_core::snapshot(&world).unwrap(),
    )
    .unwrap();
    mesocosm_genet::played::write_json(
        &out.join("world.json"),
        &serde_json::json!({
            "layout": layout.name(), "seed":7, "founders":120,
            "total_matter_mg":world.total_matter_mg(),
            "all_founders_stand":world.organisms.iter().all(|o| o.walker_shape().stands(world.ground(), o.position)),
            "organisms": world.organisms.iter().map(|o| serde_json::json!({
                "id":o.id.0, "lineage":o.species.0, "kingdom":o.kingdom(),
                "feeding":o.feeding_mode(), "mass_mg":o.biomass_mg(),
                "ceiling_mg":o.mass_ceiling_mg(), "parts":o.body().living().count(),
                "actuator_span":o.actuator_span(), "sensor_span":o.sensor_span(),
            })).collect::<Vec<_>>()
        }),
    )
    .unwrap();
    let renderer = Renderer::headless(720, 480).expect("a real GPU is required");
    mesocosm_genet::played::write_json(&out.join("content.json"), &pack).unwrap();
    let mut subjects = Vec::new();
    let mut sheet = vec![0u8; 2160 * 1440 * 4];
    let recipes = if layout == mesocosm_genet::played::BodyLayout::Spaced {
        [
            ("shrub", archetype::spaced::producer_shrub()),
            ("browser", archetype::spaced::consumer_browser()),
            ("armoured", archetype::spaced::consumer_armoured()),
        ]
    } else if layout == mesocosm_genet::played::BodyLayout::Jointed {
        [
            ("shrub", archetype::jointed::producer_shrub()),
            ("browser", archetype::jointed::consumer_browser()),
            ("armoured", archetype::jointed::consumer_armoured()),
        ]
    } else if layout == mesocosm_genet::played::BodyLayout::Branching {
        [
            ("shrub", archetype::branching::producer_shrub()),
            ("browser", archetype::branching::consumer_browser()),
            ("armoured", archetype::branching::consumer_armoured()),
        ]
    } else {
        [
            ("shrub", archetype::producer_shrub()),
            ("browser", archetype::consumer_browser()),
            ("armoured", archetype::consumer_armoured()),
        ]
    };
    for (row, (name, recipe)) in recipes.into_iter().enumerate() {
        let family: Vec<_> = world
            .organisms
            .iter()
            .filter(|o| {
                world
                    .lineages()
                    .get(o.species)
                    .is_some_and(|line| line.recipe == recipe)
            })
            .take(3)
            .collect();
        assert_eq!(family.len(), 3, "three actual relatives of {name}");
        for (index, organism) in family.into_iter().enumerate() {
            let mesh = mesh_body(organism.body(), &volumes).unwrap();
            let bounds = organism.body().aabb();
            let camera = Camera::framing(bounds.min, bounds.max, 1.5);
            let mut item = SceneItem::new(&mesh, [0; 3]);
            // This atlas reads existing guise, like the terrarium. It does not
            // invent anatomy or mark a process as expressed.
            item.recolour = Some(kingdom_colour(organism.guise as u8));
            let frame = renderer.render_scene(&[item], &camera).unwrap();
            for y in 0..480usize {
                let dest = ((row * 480 + y) * 2160 + index * 720) * 4;
                sheet[dest..dest + 720 * 4]
                    .copy_from_slice(&frame.pixels[y * 720 * 4..(y + 1) * 720 * 4]);
            }
            let filename = format!("{name}_{index}.png");
            mesocosm_genet::played::write_png(
                &out.join(&filename),
                frame.width,
                frame.height,
                &frame.pixels,
            )
            .unwrap();
            subjects.push(serde_json::json!({
                "family": name, "organism": organism.id.0, "lineage": organism.species.0,
                "layout": layout.name(), "development_seed": organism.development_seed,
                "recipe": recipe,
                "parts": organism.body().living().count(), "capture": filename,
                "mass_mg": organism.biomass_mg(), "ceiling_mg": organism.mass_ceiling_mg(),
                "kingdom": organism.kingdom(), "feeding": organism.feeding_mode(),
                "unique_volumes": mesh.mesh_count(),
                "drawn_quads": mesh.drawn_quads(),
                "camera": {"target":camera.target, "extent":camera.extent, "yaw":camera.yaw, "pitch":camera.pitch},
            }));
        }
    }
    mesocosm_genet::played::write_json(&out.join("subjects.json"), &subjects).unwrap();
    mesocosm_genet::played::write_png(&out.join("families.png"), 2160, 1440, &sheet).unwrap();
}
