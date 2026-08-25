//! Isometry's first product-owned runtime profile.
//!
//! The ordered Isometry event log remains authoritative. This profile accepts
//! only the resulting active map, mirrors its tokens into Conatus, and advances
//! zero spatial steps. It therefore conducts a spatial organ without creating
//! another movement resolver, clock, source identity, or durable record.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use conatus::{
    BodyDesc, BodyError, BodyId, BodyState, ColliderDesc, ColliderShape, Engine, EngineConfig,
    EngineConfigError, EngineError, Transform,
};
use isometry_core::{Facing, MapDocument, Token, TokenId};

#[cfg(feature = "resident-gpu")]
mod resident;

#[cfg(feature = "resident-gpu")]
pub use resident::{IsometryResidentBodies, IsometryResidentError, IsometryResidentUpdate};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapSourceId(String);

impl MapSourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenSourceId {
    pub map: MapSourceId,
    pub token: TokenId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IsometrySpatialConfig {
    /// Conatus world units between adjacent board columns or rows.
    pub tile_span: f32,
    /// Conatus world units for one authored elevation step.
    pub elevation_step: f32,
    /// Token collision half-extents in Conatus world units.
    pub token_half_extents: [f32; 3],
}

impl Default for IsometrySpatialConfig {
    fn default() -> Self {
        Self {
            tile_span: 1.0,
            elevation_step: 0.25,
            token_half_extents: [0.4, 0.5, 0.4],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TokenSpatialState {
    pub source: TokenSourceId,
    pub body: BodyState,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IsometrySpatialFrame {
    pub revision: u64,
    pub changed: Vec<TokenSpatialState>,
    pub removed: Vec<TokenSourceId>,
}

impl IsometrySpatialFrame {
    pub fn is_silent(&self) -> bool {
        self.changed.is_empty() && self.removed.is_empty()
    }
}

pub struct IsometryRuntimeProfile {
    config: IsometrySpatialConfig,
    spatial: Engine,
    bindings: BTreeMap<TokenSourceId, BodyId>,
}

impl IsometryRuntimeProfile {
    pub fn new(config: IsometrySpatialConfig) -> Result<Self, IsometryRuntimeProfileError> {
        validate_config(config)?;
        let spatial = Engine::new(EngineConfig {
            gravity: [0.0; 3],
            ..EngineConfig::default()
        })
        .map_err(IsometryRuntimeProfileError::EngineConfig)?;
        Ok(Self {
            config,
            spatial,
            bindings: BTreeMap::new(),
        })
    }

    pub fn body_for(&self, source: &TokenSourceId) -> Option<BodyId> {
        self.bindings.get(source).copied()
    }

    pub fn body_state(&self, source: &TokenSourceId) -> Option<BodyState> {
        self.body_for(source)
            .and_then(|body| self.spatial.bodies().state(body))
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Mirror one already-accepted active map into the event-driven spatial
    /// runtime and publish its product-local frame.
    ///
    /// This method has no intent or command input. The caller applies and
    /// authorizes Isometry events first; the profile only observes their map.
    pub fn sync_accepted_map(
        &mut self,
        map_source: impl Into<String>,
        map: &MapDocument,
    ) -> Result<IsometrySpatialFrame, IsometryRuntimeProfileError> {
        let map_source = MapSourceId::new(map_source);
        let mut desired = BTreeMap::new();
        for token in &map.tokens {
            let source = TokenSourceId {
                map: map_source.clone(),
                token: token.id,
            };
            let transform = token_transform(self.config, map, token)?;
            if desired.insert(source.clone(), transform).is_some() {
                return Err(IsometryRuntimeProfileError::DuplicateToken(source));
            }
        }

        for (source, transform) in &desired {
            match self.bindings.get(source).copied() {
                Some(body) => {
                    let current = self.spatial.bodies().state(body).ok_or(
                        IsometryRuntimeProfileError::LostBinding {
                            source: source.clone(),
                            body,
                        },
                    )?;
                    if current.transform != *transform {
                        self.spatial
                            .bodies_mut()
                            .set_transform(body, *transform, false)
                            .map_err(IsometryRuntimeProfileError::Body)?;
                    }
                }
                None => {
                    let body =
                        self.spatial
                            .bodies_mut()
                            .spawn(BodyDesc::fixed().at(*transform).with_collider(
                                ColliderDesc::new(ColliderShape::cuboid(
                                    self.config.token_half_extents,
                                )),
                            ))
                            .map_err(IsometryRuntimeProfileError::Body)?;
                    self.bindings.insert(source.clone(), body);
                }
            }
        }

        let desired_sources: BTreeSet<_> = desired.keys().cloned().collect();
        let removed: Vec<_> = self
            .bindings
            .keys()
            .filter(|source| !desired_sources.contains(*source))
            .cloned()
            .collect();
        for source in &removed {
            let body = self
                .bindings
                .remove(source)
                .expect("the removal source came from the binding table");
            self.spatial
                .bodies_mut()
                .despawn(body)
                .map_err(IsometryRuntimeProfileError::Body)?;
        }

        let frame = self
            .spatial
            .advance(0)
            .map_err(IsometryRuntimeProfileError::Engine)?;
        let reverse: BTreeMap<_, _> = self
            .bindings
            .iter()
            .map(|(source, body)| (*body, source.clone()))
            .collect();
        let mut changed: Vec<_> = frame
            .changed
            .into_iter()
            .filter_map(|body| {
                reverse
                    .get(&body.id)
                    .cloned()
                    .map(|source| TokenSpatialState { source, body })
            })
            .collect();
        changed.sort_by(|left, right| left.source.cmp(&right.source));

        Ok(IsometrySpatialFrame {
            revision: frame.revision,
            changed,
            removed,
        })
    }
}

fn token_transform(
    config: IsometrySpatialConfig,
    map: &MapDocument,
    token: &Token,
) -> Result<Transform, IsometryRuntimeProfileError> {
    if !map.ground.in_bounds(token.at.0, token.at.1) {
        return Err(IsometryRuntimeProfileError::TokenOutOfBounds {
            token: token.id,
            at: token.at,
        });
    }
    let elevation = *map
        .elevation
        .get(token.at.0 as u32, token.at.1 as u32)
        .expect("ground and elevation share bounds");
    Ok(Transform {
        translation: [
            token.at.0 as f32 * config.tile_span,
            elevation as f32 * config.elevation_step + config.token_half_extents[1],
            token.at.1 as f32 * config.tile_span,
        ],
        rotation: facing_rotation(token.facing),
    })
}

fn facing_rotation(facing: Facing) -> [f32; 4] {
    const HALF_SQRT_TWO: f32 = std::f32::consts::FRAC_1_SQRT_2;
    match facing {
        Facing::South => [0.0, 0.0, 0.0, 1.0],
        Facing::East => [0.0, HALF_SQRT_TWO, 0.0, HALF_SQRT_TWO],
        Facing::North => [0.0, 1.0, 0.0, 0.0],
        Facing::West => [0.0, -HALF_SQRT_TWO, 0.0, HALF_SQRT_TWO],
    }
}

fn validate_config(config: IsometrySpatialConfig) -> Result<(), IsometryRuntimeProfileError> {
    if !config.tile_span.is_finite() || config.tile_span <= 0.0 {
        return Err(IsometryRuntimeProfileError::InvalidConfig(
            "tile span must be finite and positive",
        ));
    }
    if !config.elevation_step.is_finite() || config.elevation_step < 0.0 {
        return Err(IsometryRuntimeProfileError::InvalidConfig(
            "elevation step must be finite and non-negative",
        ));
    }
    if config
        .token_half_extents
        .iter()
        .any(|extent| !extent.is_finite() || *extent <= 0.0)
    {
        return Err(IsometryRuntimeProfileError::InvalidConfig(
            "token half-extents must be finite and positive",
        ));
    }
    Ok(())
}

#[derive(Debug)]
pub enum IsometryRuntimeProfileError {
    InvalidConfig(&'static str),
    DuplicateToken(TokenSourceId),
    TokenOutOfBounds { token: TokenId, at: (i32, i32) },
    LostBinding { source: TokenSourceId, body: BodyId },
    EngineConfig(EngineConfigError),
    Body(BodyError),
    Engine(EngineError),
}

impl fmt::Display for IsometryRuntimeProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => formatter.write_str(message),
            Self::DuplicateToken(source) => write!(
                formatter,
                "duplicate token {:?} on map {}",
                source.token,
                source.map.as_str()
            ),
            Self::TokenOutOfBounds { token, at } => {
                write!(formatter, "token {token:?} is outside its map at {at:?}")
            }
            Self::LostBinding { source, body } => write!(
                formatter,
                "token {:?} on map {} lost Conatus body {body:?}",
                source.token,
                source.map.as_str()
            ),
            Self::EngineConfig(source) => write!(formatter, "invalid Conatus profile: {source}"),
            Self::Body(source) => write!(formatter, "Conatus body projection failed: {source}"),
            Self::Engine(source) => write!(formatter, "Conatus publication failed: {source}"),
        }
    }
}

impl Error for IsometryRuntimeProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EngineConfig(source) => Some(source),
            Self::Body(source) => Some(source),
            Self::Engine(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_core::{apply, SessionEvent};

    fn token(id: u32, at: (i32, i32)) -> Token {
        Token {
            id: TokenId(id),
            at,
            facing: Facing::South,
            sprite: "knight".to_owned(),
            owner: None,
        }
    }

    fn board() -> MapDocument {
        let mut map = MapDocument::new("field", 4, 4);
        apply(&mut map, &SessionEvent::TokenPlaced(token(1, (0, 0)))).unwrap();
        apply(&mut map, &SessionEvent::TokenPlaced(token(2, (2, 2)))).unwrap();
        map
    }

    fn assert_rotation_close(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-6);
        }
    }

    #[test]
    fn accepted_map_materializes_once_and_an_unchanged_frame_is_silent() {
        let map = board();
        let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();

        let first = profile.sync_accepted_map("field", &map).unwrap();
        let second = profile.sync_accepted_map("field", &map).unwrap();

        assert_eq!(profile.len(), 2);
        assert_eq!(first.changed.len(), 2);
        assert!(first.removed.is_empty());
        assert!(second.is_silent());
    }

    #[test]
    fn an_ordered_event_moves_the_existing_body_after_authority_accepts_it() {
        let mut map = board();
        let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();
        profile.sync_accepted_map("field", &map).unwrap();
        let source = TokenSourceId {
            map: MapSourceId::new("field"),
            token: TokenId(1),
        };
        let body = profile.body_for(&source).unwrap();

        apply(
            &mut map,
            &SessionEvent::TokenMoved {
                id: TokenId(1),
                to: (3, 1),
            },
        )
        .unwrap();
        apply(
            &mut map,
            &SessionEvent::TokenFaced {
                id: TokenId(1),
                facing: Facing::East,
            },
        )
        .unwrap();
        let frame = profile.sync_accepted_map("field", &map).unwrap();

        assert_eq!(profile.body_for(&source), Some(body));
        assert_eq!(frame.changed.len(), 1);
        assert_eq!(frame.changed[0].source, source);
        assert_eq!(frame.changed[0].body.transform.translation, [3.0, 0.5, 1.0]);
        assert_rotation_close(
            frame.changed[0].body.transform.rotation,
            facing_rotation(Facing::East),
        );
    }

    #[test]
    fn a_rejected_event_never_reaches_the_spatial_profile() {
        let mut map = board();
        let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();
        profile.sync_accepted_map("field", &map).unwrap();
        let before = map.clone();

        assert!(apply(
            &mut map,
            &SessionEvent::TokenMoved {
                id: TokenId(1),
                to: (99, 99),
            },
        )
        .is_err());
        assert_eq!(map, before);
        assert!(profile
            .sync_accepted_map("field", &map)
            .unwrap()
            .is_silent());
    }

    #[test]
    fn map_identity_prevents_equal_token_numbers_from_aliasing() {
        let first_map = board();
        let mut second_map = MapDocument::new("cellar", 4, 4);
        apply(
            &mut second_map,
            &SessionEvent::TokenPlaced(token(1, (1, 1))),
        )
        .unwrap();
        let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();
        profile.sync_accepted_map("field", &first_map).unwrap();
        let old_source = TokenSourceId {
            map: MapSourceId::new("field"),
            token: TokenId(1),
        };
        let old_body = profile.body_for(&old_source).unwrap();

        let frame = profile.sync_accepted_map("cellar", &second_map).unwrap();
        let new_source = TokenSourceId {
            map: MapSourceId::new("cellar"),
            token: TokenId(1),
        };
        let new_body = profile.body_for(&new_source).unwrap();

        assert_ne!(new_body, old_body);
        assert!(profile.body_for(&old_source).is_none());
        assert!(frame.removed.contains(&old_source));
        assert_eq!(frame.changed[0].source, new_source);
    }
}
