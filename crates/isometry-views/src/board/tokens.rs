//! The pieces on the board: tokens, their appearance layers, and markers.
//!
//! A token is a stack of CSS-classed layers over one placed box, which is why
//! a campaign reskins by swapping sheets rather than by changing this.
//!
//! Split out of `board.rs` on 2026-09-04; unchanged.

use super::*;

pub(super) fn token_el(ui: &UiState, token: &Token) -> UiChild {
    let geo = &ui.geo;
    let elev = *ui
        .map
        .elevation
        .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (cx, cy) = geo.tile_to_screen(token.at, elev);
    let z = depth_key(token.at, elev) + 2;
    // 8x12 sprite at 3x (24x36), feet at the diamond center. The anchor takes
    // the board's scale with the box: the feet must stay on the diamond.
    let s = ui.board_scale;
    let (x, y) = (cx - TOKEN_BOX.0 / 2.0 * s, cy - 32.0 * s);
    let mut class = format!("token token-{}", token.sprite);
    // Equipment appearance remains pack CSS: public layer keys become stable
    // token classes, while the future voxel compositor can replace the same
    // projection with a baked recipe without changing campaign data.
    if let Some(inventory) = ui.inventories.get(&token.id) {
        for item_id in inventory.equipped.values() {
            if let Some(item) = inventory.items.get(item_id) {
                for layer in item.appearance_layers() {
                    class.push_str(" token-layer-");
                    class.push_str(&layer_class(layer));
                }
            }
        }
    }
    // One drawn side, mirrored for the other two facings (the GBA
    // economy): E/N flip, S/W stay.
    let flipped = matches!(
        token.facing,
        isometry_core::Facing::East | isometry_core::Facing::North
    );
    if flipped {
        class.push_str(" token-flip");
    }
    let id = token.id;

    // A1: the beat rides on a wrapper, not on the sprite. `.token-flip` already
    // owns the sprite's `transform` for facing, and a CSS animation on
    // `transform` outranks a normal declaration, so a beat on the same box would
    // strip a west/south-facing token of its mirror mid-swing. Two boxes: the
    // wrapper is placed and beats, the sprite inside is drawn and flipped. They
    // compose instead of fighting.
    let mut wrapper = "beat".to_owned();
    if let Some(beat) = ui.beats.get(&id) {
        wrapper.push_str(" beat-");
        wrapper.push_str(beat);
    }
    let down = ui.map.is_defeated(id);
    if down {
        wrapper.push_str(" beat-down");
    }
    // Conditions render as classes, like beats and equipment layers, so a pack
    // can style `cond-prone` the way it styles a swing. A graded condition also
    // emits `cond-frightened-2`, so a pack can show magnitude without the board
    // knowing what the number means.
    if let Some(conditions) = ui.map.conditions.get(&id) {
        for (name, value) in conditions {
            wrapper.push_str(" cond-");
            wrapper.push_str(name);
            if *value > 1 {
                wrapper.push_str(&format!(" cond-{name}-{value}"));
            }
        }
    }
    // A corpse is not a target, so it does not offer itself as one.
    if ui.picking_target() && !down {
        wrapper.push_str(" beat-targetable");
    }
    // The sprite is the one board box whose size is not the whole story: it
    // also carries `.token-flip`'s mirror, and that transform pre-translates by
    // the sprite's own width (genet conjugates at the box origin). A scaled
    // sprite therefore needs a scaled translate, so both ride the same inline
    // declaration rather than the class. At scale 1 nothing is emitted and the
    // sheet's rule stands untouched.
    let sprite_el = el("div", ()).attr("class", class);
    let sprite: Vec<UiChild> = vec![if s == 1.0 {
        Box::new(sprite_el)
    } else {
        let mut style = format!("width: {}px; height: {}px;", TOKEN_BOX.0 * s, TOKEN_BOX.1 * s);
        if flipped {
            style.push_str(&format!(
                " transform: translateX({}px) scaleX(-1);",
                TOKEN_BOX.0 * s
            ));
        }
        Box::new(sprite_el.attr("style", style))
    }];
    standing_on(
        clickable(
            el("div", sprite)
                .attr("class", wrapper)
                .attr("style", placed(ui, (x, y), TOKEN_BOX, z)),
            move |ui: &mut UiState, _| {
                // In target-pick mode a click on a token names the victim rather
                // than selecting it.
                if ui.picking_target() {
                    ui.pick_action_target(id);
                } else {
                    ui.click_token(id);
                }
            },
        ),
        token.at,
    )
}

/// Make a pack layer key safe for the CSS-class vocabulary. Different raw
/// punctuation collapses intentionally: appearance keys are authored ids, and
/// pack validation can reject collisions once packs gain a formal manifest.
fn layer_class(key: &str) -> String {
    key.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// A ground marker diamond under a token (turn-active gold, selection
/// green), one depth step above the tile it stands on.
pub(super) fn marker_el(ui: &UiState, token_id: isometry_core::TokenId, class: &str) -> Option<UiChild> {
    let token = ui.map.token(token_id)?;
    let elev = *ui
        .map
        .elevation
        .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (cx, cy) = ui.geo.tile_to_screen(token.at, elev);
    let z = depth_key(token.at, elev) + 1;
    let s = ui.board_scale;
    let (x, y) = (cx - MARKER_BOX.0 / 2.0 * s, cy - MARKER_BOX.1 / 2.0 * s);
    Some(standing_on(
        el("div", ())
            .attr("class", class.to_owned())
            .attr("style", placed(ui, (x, y), MARKER_BOX, z)),
        token.at,
    ))
}

#[cfg(test)]
mod layer_class_tests {
    use super::layer_class;

    #[test]
    fn equipment_layer_keys_map_to_stable_css_classes() {
        assert_eq!(layer_class("effect:Flame"), "effect-flame");
        assert_eq!(layer_class("weapon/longsword +1"), "weapon-longsword--1");
    }
}
