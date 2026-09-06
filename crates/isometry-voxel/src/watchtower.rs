//! Original starter-campaign creature. Geometry is appearance only; its
//! abilities and disposition live in the watchtower content pack.

use crate::{Palette, Voxels};

/// A broad forest crown over a visible trunk. The board binds this to a
/// three-height-unit canopy; the model supplies appearance, not sight rules.
pub fn forest_tree() -> (Voxels, Palette) {
    let palette = Palette::new(vec![
        [92, 66, 43],
        [51, 82, 44],
        [68, 105, 48],
        [99, 132, 61],
    ]);
    let mut model = Voxels::new(16, 22, 16);
    model.fill(7, 9, 0, 14, 7, 9, 0);
    model.fill(4, 12, 9, 11, 7, 9, 0);
    for (lo, hi, bottom, top, color) in [
        (3, 13, 9, 13, 1),
        (0, 16, 12, 16, 1),
        (2, 14, 16, 19, 2),
        (5, 11, 19, 22, 3),
    ] {
        model.fill(lo, hi, bottom, top, lo, hi, color);
    }
    (model, palette)
}

/// A low, moss-backed quadruped with a pale muzzle, ears and a long tail.
/// Facing zero looks toward -Z, matching the existing bake convention.
pub fn tower_beast() -> (Voxels, Palette) {
    let palette = Palette::new(vec![
        [76, 91, 73],    // hide
        [126, 146, 88],  // moss
        [216, 197, 153], // muzzle and claws
        [37, 43, 39],    // feet and nose
        [241, 176, 67],  // eyes
    ]);
    let mut body = Voxels::new(12, 14, 22);
    body.fill(2, 10, 4, 10, 6, 17, 0);
    body.fill(3, 9, 10, 12, 7, 15, 1);
    for x in [2, 8] {
        for z in [7, 14] {
            body.fill(x, x + 2, 0, 6, z, z + 3, 0);
            body.fill(x, x + 2, 0, 2, z - 1, z + 2, 3);
            body.fill(x, x + 2, 0, 1, z - 1, z, 2);
        }
    }
    body.fill(2, 10, 6, 11, 2, 8, 0);
    body.fill(3, 9, 5, 8, 0, 4, 2);
    body.fill(4, 8, 7, 9, 0, 1, 3);
    body.fill(2, 4, 10, 14, 4, 6, 0);
    body.fill(8, 10, 10, 14, 4, 6, 0);
    body.set(2, 9, 2, 4);
    body.set(9, 9, 2, 4);
    body.fill(5, 7, 6, 9, 16, 20, 0);
    body.fill(6, 8, 8, 10, 19, 22, 1);
    (body, palette)
}

/// Fallen masonry, still authored independently of the height-field walls.
pub fn rubble() -> (Voxels, Palette) {
    let palette = Palette::new(vec![[133, 129, 111], [92, 102, 76]]);
    let mut model = Voxels::new(12, 8, 10);
    model.fill(0, 7, 0, 3, 1, 7, 0);
    model.fill(7, 12, 0, 4, 4, 10, 0);
    model.fill(2, 8, 3, 7, 2, 6, 0);
    model.fill(3, 6, 7, 8, 3, 5, 1);
    (model, palette)
}

/// A bronze bell suspended from the surviving wooden support.
pub fn bell() -> (Voxels, Palette) {
    let palette = Palette::new(vec![[113, 79, 49], [191, 139, 65], [71, 85, 66]]);
    let mut model = Voxels::new(14, 20, 8);
    model.fill(0, 2, 0, 20, 3, 5, 0);
    model.fill(12, 14, 0, 20, 3, 5, 0);
    model.fill(0, 14, 18, 20, 3, 5, 0);
    model.fill(6, 8, 14, 18, 3, 5, 2);
    model.fill(5, 9, 9, 14, 2, 6, 1);
    model.fill(4, 10, 6, 10, 1, 7, 1);
    model.fill(3, 11, 5, 7, 0, 8, 1);
    model.fill(6, 8, 2, 5, 3, 5, 2);
    (model, palette)
}

/// A shallow moss-and-twig nest with pale stones at its centre.
pub fn nest() -> (Voxels, Palette) {
    let palette = Palette::new(vec![[115, 87, 57], [91, 116, 63], [216, 197, 153]]);
    let mut model = Voxels::new(14, 4, 12);
    model.fill(1, 13, 0, 1, 1, 11, 1);
    model.fill(0, 14, 1, 3, 0, 2, 0);
    model.fill(0, 14, 1, 3, 10, 12, 0);
    model.fill(0, 2, 1, 3, 2, 10, 0);
    model.fill(12, 14, 1, 3, 2, 10, 0);
    model.fill(5, 7, 1, 4, 4, 7, 2);
    model.fill(8, 10, 1, 3, 6, 9, 2);
    (model, palette)
}
