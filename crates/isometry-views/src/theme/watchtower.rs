//! Starter-campaign appearance, baked once and shared by rebuilt stylesheets.

use std::sync::OnceLock;

pub(super) fn css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| {
        use isometry_voxel::{BakeParams, bake_facing, watchtower::tower_beast};
        let (body, palette) = tower_beast();
        let params = BakeParams { half_w: 2, cube_h: 2, facings: 4, margin: 2 };
        let mut css = String::new();
        for (facing, index) in [("north", 0), ("east", 1), ("south", 2), ("west", 3)] {
            let uri = bake_facing(&body, &palette, index, &params).to_png_data_uri();
            css.push_str(&format!(
                ".token-tower-beast.token-facing-{facing} {{ background-image: url(\"{uri}\"); \
                 background-size: contain; background-position: bottom center; transform: none; }}\n"
            ));
        }
        use isometry_voxel::watchtower::{bell, forest_tree, nest, rubble};
        for (selector, (model, palette)) in [
            (".prop-wall, .prop-tower-wall-east, .prop-tower-wall-west", rubble()),
            (".prop-broken-bell", bell()),
            (".prop-nest-marker", nest()),
            (".prop-forest-tree", forest_tree()),
        ] {
            let uri = bake_facing(&model, &palette, 0, &params).to_png_data_uri();
            css.push_str(&format!(
                "{selector} {{ background-image: url(\"{uri}\"); background-size: contain; \
                 background-position: bottom center; background-repeat: no-repeat; image-rendering: pixelated; }}\n"
            ));
        }
        css.push_str(".tile-rubble { background-color: #81775f; }\n");
        css.push_str(".prop-forest-tree { width: 32px; height: 40px; }\n");
        css.push_str(".tile-face-stone.tile-face-left { background-color: #676c69; }\n.tile-face-stone.tile-face-right { background-color: #515956; }\n");
        css.push_str(".tile-forest-floor { background-color: #405b35; }\n.tile-forest-floor.alt { background-color: #46613a; }\n.tile-forest-path { background-color: #8a7953; }\n");
        css
    })
}
