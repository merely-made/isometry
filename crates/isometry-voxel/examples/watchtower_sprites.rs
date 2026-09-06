//! Bake the original watchtower creature with the same parameters as the UI.
//! Usage: cargo run -p isometry-voxel --example watchtower_sprites -- <output-dir>

use isometry_voxel::{BakeParams, bake_facing, bake_strip, watchtower::tower_beast};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args_os()
        .nth(1)
        .ok_or("expected output directory")?;
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory)?;
    let (body, palette) = tower_beast();
    let params = BakeParams {
        half_w: 2,
        cube_h: 2,
        facings: 4,
        margin: 2,
    };
    for (facing, index) in [("north", 0), ("east", 1), ("south", 2), ("west", 3)] {
        let image = bake_facing(&body, &palette, index, &params);
        std::fs::write(
            directory.join(format!("tower-beast-{facing}.png")),
            image.to_png(),
        )?;
    }
    std::fs::write(
        directory.join("tower-beast.png"),
        bake_strip(&body, &palette, &params).to_png(),
    )?;
    Ok(())
}
