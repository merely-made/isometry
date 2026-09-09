// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Compatibility serialization for [`Recipe`](super::Recipe).

use std::collections::BTreeSet;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeStruct, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Appendage, AppendageStep, Recipe, Tagma};
use crate::plan::Facing;

/// Where a stretch attaches along its parent segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    Base,
    Middle,
    Tip,
}

/// Placement instruction for one stretch in a recipe's optional layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stretch {
    pub parent: Option<u8>,
    pub anchor: Anchor,
    pub facing: Facing,
    #[serde(default)]
    pub variance: Option<u8>,
}

/// Keeps explicit parent indices coherent when a tagma is divided.
pub(crate) fn divide_layout(layout: &mut Vec<Stretch>, tagma: usize, front: u8, old_segments: u8) {
    let insertion = tagma + 1;
    for stretch in layout.iter_mut() {
        if let Some(parent) = stretch.parent {
            if parent as usize == tagma {
                stretch.parent = match stretch.anchor {
                    Anchor::Base => Some(parent),
                    Anchor::Tip => Some(insertion as u8),
                    Anchor::Middle if (front as u16) * 2 <= old_segments as u16 => {
                        Some(insertion as u8)
                    },
                    Anchor::Middle => Some(parent),
                };
            } else if parent as usize >= insertion {
                stretch.parent = Some(parent.saturating_add(1));
            }
        }
    }
    if let Some(original) = layout.get(tagma).copied() {
        // The new half continues the old stretch in the same direction.
        // Base is the unambiguous attachment point for that continuation;
        // the original anchor remains on the front half.
        let tail = Stretch {
            parent: None,
            anchor: Anchor::Base,
            facing: original.facing,
            variance: original.variance,
        };
        layout.insert(insertion, tail);
    }
}

const VERSION: u8 = 1;
const CHAIN_VERSION: u8 = 2;

impl Serialize for Recipe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let fields = 3
                + usize::from(!self.layout.is_empty())
                + usize::from(!self.appendage_chains.is_empty());
            let mut out = serializer.serialize_struct("Recipe", fields)?;
            out.serialize_field("tagmata", &self.tagmata)?;
            out.serialize_field("variance", &self.variance)?;
            out.serialize_field("lexicon", &self.lexicon)?;
            if !self.layout.is_empty() {
                out.serialize_field("layout", &self.layout)?;
            }
            if !self.appendage_chains.is_empty() {
                out.serialize_field("appendage_chains", &self.appendage_chains)?;
            }
            out.end()
        } else if self.layout.is_empty() && self.appendage_chains.is_empty() {
            let mut out = serializer.serialize_tuple(3)?;
            out.serialize_element(&self.tagmata)?;
            out.serialize_element(&self.variance)?;
            out.serialize_element(&self.lexicon)?;
            out.end()
        } else if self.appendage_chains.is_empty() {
            let mut out = serializer.serialize_tuple(6)?;
            out.serialize_element(&Vec::<Tagma>::new())?;
            out.serialize_element(&VERSION)?;
            out.serialize_element(&self.tagmata)?;
            out.serialize_element(&self.variance)?;
            out.serialize_element(&self.lexicon)?;
            out.serialize_element(&self.layout)?;
            out.end()
        } else {
            let mut out = serializer.serialize_tuple(7)?;
            out.serialize_element(&Vec::<Tagma>::new())?;
            out.serialize_element(&CHAIN_VERSION)?;
            out.serialize_element(&self.tagmata)?;
            out.serialize_element(&self.variance)?;
            out.serialize_element(&self.lexicon)?;
            out.serialize_element(&self.layout)?;
            out.serialize_element(&self.appendage_chains)?;
            out.end()
        }
    }
}

impl<'de> Deserialize<'de> for Recipe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(HumanVisitor)
        } else {
            deserializer.deserialize_tuple(7, BinaryVisitor)
        }
    }
}

fn build(
    tagmata: Vec<Tagma>,
    variance: u8,
    lexicon: BTreeSet<Appendage>,
    layout: Vec<Stretch>,
    appendage_chains: Vec<Vec<AppendageStep>>,
) -> Result<Recipe, String> {
    if tagmata.is_empty() {
        return Err("recipe must contain at least one tagma".into());
    }
    Ok(Recipe {
        tagmata,
        variance,
        lexicon,
        layout,
        appendage_chains,
    })
}

struct BinaryVisitor;

impl<'de> Visitor<'de> for BinaryVisitor {
    type Value = Recipe;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a legacy or versioned Recipe tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Recipe, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let first: Vec<Tagma> = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe tagmata"))?;
        if !first.is_empty() {
            let variance = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe variance"))?;
            let lexicon = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe lexicon"))?;
            return build(first, variance, lexicon, Vec::new(), Vec::new())
                .map_err(de::Error::custom);
        }

        let version: u8 = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe wire version"))?;
        if version == CHAIN_VERSION {
            let tagmata = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe tagmata"))?;
            let variance = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe variance"))?;
            let lexicon = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe lexicon"))?;
            let layout = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe layout"))?;
            let appendage_chains = seq
                .next_element()?
                .ok_or_else(|| de::Error::custom("missing recipe appendage chains"))?;
            return build(tagmata, variance, lexicon, layout, appendage_chains)
                .map_err(de::Error::custom);
        }
        if version != VERSION {
            return Err(de::Error::custom(format!(
                "unknown Recipe wire version {version}"
            )));
        }
        let tagmata = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe tagmata"))?;
        let variance = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe variance"))?;
        let lexicon = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe lexicon"))?;
        let layout = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing recipe layout"))?;
        build(tagmata, variance, lexicon, layout, Vec::new()).map_err(de::Error::custom)
    }
}

struct HumanVisitor;

impl<'de> Visitor<'de> for HumanVisitor {
    type Value = Recipe;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a Recipe object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Recipe, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut tagmata = None;
        let mut variance = None;
        let mut lexicon = None;
        let mut layout = None;
        let mut appendage_chains = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "tagmata" => {
                    if tagmata.is_some() {
                        return Err(de::Error::duplicate_field("tagmata"));
                    }
                    tagmata = Some(map.next_value()?);
                },
                "variance" => {
                    if variance.is_some() {
                        return Err(de::Error::duplicate_field("variance"));
                    }
                    variance = Some(map.next_value()?);
                },
                "lexicon" => {
                    if lexicon.is_some() {
                        return Err(de::Error::duplicate_field("lexicon"));
                    }
                    lexicon = Some(map.next_value()?);
                },
                "layout" => {
                    if layout.is_some() {
                        return Err(de::Error::duplicate_field("layout"));
                    }
                    layout = Some(map.next_value()?);
                },
                "appendage_chains" => {
                    if appendage_chains.is_some() {
                        return Err(de::Error::duplicate_field("appendage_chains"));
                    }
                    appendage_chains = Some(map.next_value()?);
                },
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                },
            }
        }
        let tagmata = tagmata.ok_or_else(|| de::Error::missing_field("tagmata"))?;
        let variance = variance.ok_or_else(|| de::Error::missing_field("variance"))?;
        let lexicon = lexicon.ok_or_else(|| de::Error::missing_field("lexicon"))?;
        build(
            tagmata,
            variance,
            lexicon,
            layout.unwrap_or_default(),
            appendage_chains.unwrap_or_default(),
        )
        .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_fixture_is_stable_and_decodes() {
        let bytes = [1, 3, 0, 0, 0, 0, 1, 2, 0, 4];
        let recipe: Recipe = postcard::from_bytes(&bytes).expect("legacy recipe");
        assert_eq!(recipe, Recipe::founding(3));
        assert_eq!(postcard::to_allocvec(&recipe).unwrap(), bytes);
    }

    #[test]
    fn legacy_recipe_followed_by_sentinel_roundtrips() {
        let recipe = Recipe::founding(3);
        let mut bytes = postcard::to_allocvec(&recipe).unwrap();
        bytes.extend_from_slice(&[0xA5, 0x5A]);
        let (decoded, rest) = postcard::take_from_bytes::<Recipe>(&bytes).unwrap();
        assert_eq!(decoded, recipe);
        assert_eq!(rest, &[0xA5, 0x5A]);
    }

    #[test]
    fn versioned_recipe_roundtrips() {
        let recipe =
            Recipe::of(vec![Tagma::bare(2), Tagma::new(1, Appendage::Limb)]).with_layout(vec![
                Stretch {
                    parent: None,
                    anchor: Anchor::Base,
                    facing: crate::plan::Facing::Back,
                    variance: None,
                },
                Stretch {
                    parent: Some(0),
                    anchor: Anchor::Tip,
                    facing: crate::plan::Facing::Above,
                    variance: Some(1),
                },
            ]);
        let bytes = postcard::to_allocvec(&recipe).unwrap();
        assert_eq!(bytes[0], 0, "versioned marker is an empty tagmata vector");
        assert_eq!(bytes[1], VERSION);
        assert_eq!(postcard::from_bytes::<Recipe>(&bytes).unwrap(), recipe);
    }

    #[test]
    fn literal_v1_fixture_is_stable() {
        let bytes = [0, 1, 1, 1, 0, 0, 0, 0, 1, 2, 0, 4, 1, 0, 2, 4, 0];
        let recipe: Recipe = postcard::from_bytes(&bytes).expect("V1 recipe");
        let expected = Recipe::founding(1).with_layout(vec![Stretch {
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Above,
            variance: None,
        }]);
        assert_eq!(recipe, expected);
        assert_eq!(postcard::to_allocvec(&recipe).unwrap(), bytes);
    }

    #[test]
    fn nested_v2_recipe_roundtrips_and_leaves_sentinel() {
        let recipe = Recipe::of(vec![Tagma::new(1, Appendage::Limb), Tagma::bare(2)])
            .with_appendage_chains(vec![
                vec![AppendageStep {
                    role: crate::plan::Role::Limb,
                    shape: 3,
                    facing: super::super::ChainFacing::Outward,
                    distal: false,
                }],
                vec![],
            ]);
        let mut bytes = postcard::to_allocvec(&recipe).unwrap();
        bytes.extend_from_slice(&[0xA5, 0x5A]);
        let (decoded, rest) = postcard::take_from_bytes::<Recipe>(&bytes).unwrap();
        assert_eq!(decoded, recipe);
        assert_eq!(rest, &[0xA5, 0x5A]);
        assert_eq!(postcard::to_allocvec(&recipe).unwrap()[1], CHAIN_VERSION);
    }

    #[test]
    fn unknown_or_truncated_versioned_recipe_is_rejected() {
        assert!(postcard::from_bytes::<Recipe>(&[0]).is_err());
        assert!(postcard::from_bytes::<Recipe>(&[0, 2]).is_err());
        assert!(postcard::from_bytes::<Recipe>(&[0, 3]).is_err());
    }

    #[test]
    fn nested_recipe_leaves_following_value_intact() {
        let recipe = Recipe::of(vec![Tagma::bare(1)]).with_layout(vec![Stretch {
            parent: None,
            anchor: Anchor::Middle,
            facing: crate::plan::Facing::Back,
            variance: None,
        }]);
        let mut bytes = postcard::to_allocvec(&recipe).unwrap();
        bytes.extend(postcard::to_allocvec(&77u8).unwrap());
        let (decoded, rest) = postcard::take_from_bytes::<Recipe>(&bytes).unwrap();
        assert_eq!(decoded, recipe);
        assert_eq!(postcard::from_bytes::<u8>(rest).unwrap(), 77);
    }

    #[test]
    fn dividing_a_branch_preserves_layout_parents() {
        let mut recipe = Recipe::of(vec![Tagma::bare(4), Tagma::bare(2), Tagma::bare(1)])
            .with_layout(vec![
                Stretch {
                    parent: None,
                    anchor: Anchor::Base,
                    facing: Facing::Back,
                    variance: None,
                },
                Stretch {
                    parent: Some(0),
                    anchor: Anchor::Tip,
                    facing: Facing::Above,
                    variance: None,
                },
                Stretch {
                    parent: Some(1),
                    anchor: Anchor::Middle,
                    facing: Facing::Below,
                    variance: None,
                },
            ]);
        assert_eq!(recipe.divide(0, 2), Ok(1));
        assert_eq!(recipe.layout.len(), 4);
        assert_eq!(recipe.layout[1].parent, None);
        assert_eq!(recipe.layout[1].facing, Facing::Back);
        assert_eq!(recipe.layout[2].parent, Some(1));
        assert_eq!(recipe.layout[3].parent, Some(2));
    }

    #[test]
    fn dividing_preserves_base_and_selects_the_half_containing_the_middle() {
        for split in [1, 3] {
            let mut layout = vec![
                Stretch {
                    parent: None,
                    anchor: Anchor::Base,
                    facing: Facing::Above,
                    variance: Some(1),
                },
                Stretch {
                    parent: Some(0),
                    anchor: Anchor::Base,
                    facing: Facing::Left,
                    variance: None,
                },
                Stretch {
                    parent: Some(0),
                    anchor: Anchor::Middle,
                    facing: Facing::Right,
                    variance: None,
                },
            ];
            divide_layout(&mut layout, 0, split, 4);
            assert_eq!(layout[1].variance, Some(1));
            assert_eq!(layout[2].parent, Some(0));
            assert_eq!(layout[3].parent, Some(if split == 1 { 1 } else { 0 }));
        }
    }

    #[test]
    fn dividing_clones_appendage_chain_and_assignment_clears_one_chain() {
        let step = AppendageStep {
            role: crate::plan::Role::Limb,
            shape: 1,
            facing: super::super::ChainFacing::Inward,
            distal: false,
        };
        let mut recipe = Recipe::of(vec![Tagma::new(4, Appendage::Limb), Tagma::bare(1)])
            .with_appendage_chains(vec![vec![step], vec![]]);
        assert_eq!(recipe.divide(0, 2), Ok(1));
        assert_eq!(recipe.appendage_chain_for(0), Some([step].as_slice()));
        assert_eq!(recipe.appendage_chain_for(1), Some([step].as_slice()));
        recipe.assign(0, Appendage::Limb).unwrap();
        assert_eq!(recipe.appendage_chain_for(0), Some([step].as_slice()));
        recipe.assign(0, Appendage::None).unwrap();
        assert_eq!(recipe.appendage_chain_for(0), Some([].as_slice()));
        assert_eq!(recipe.appendage_chain_for(1), Some([step].as_slice()));
    }
}
