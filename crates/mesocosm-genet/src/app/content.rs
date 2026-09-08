// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Admit immutable voxel content before founding, and restore recorded bytes.

use mesocosm_mesh::{VolumeMap, content::ContentPack};
use mesocosm_runtime::Runtime;

use super::HostConfig;

pub(super) fn with_authored(runtime: Runtime) -> Runtime {
    match mesocosm_runtime::Authored::load(&super::pack_root()) {
        Ok(authored) => runtime.with_authored(authored),
        Err(why) => {
            eprintln!("pack: {}", why.words());
            runtime
        },
    }
}

fn runtime(
    config: &HostConfig,
    founding: mesocosm_core::Founding,
    palette: mesocosm_core::PartPalette,
) -> Result<Runtime, String> {
    if let Some(selection) = config.effective_start() {
        if config.effective_scene() != crate::played::SceneMode::Ecology {
            return Err("generated starts require the ecology scene".into());
        }
        return Runtime::generated_start(selection, palette, config.ticks_per_second)
            .map_err(|why| format!("generated start refused: {why:?}"));
    }
    let result = match config.effective_scene() {
        crate::played::SceneMode::Ecology => Runtime::with_founding_palette(
            config.seed,
            config.organisms,
            config.ticks_per_second,
            founding,
            palette,
        ),
        crate::played::SceneMode::Terrarium => {
            Runtime::terrarium(config.seed, config.ticks_per_second, founding, palette)
        },
        crate::played::SceneMode::GraftPractice => {
            Runtime::graft_practice(config.seed, config.ticks_per_second, founding, palette)
        },
        crate::played::SceneMode::ExpressionPractice => {
            Runtime::expression_practice(config.seed, config.ticks_per_second, founding, palette)
        },
    };
    result.map_err(|why| format!("founding refused: {why:?}"))
}

pub(super) fn start(
    config: &HostConfig,
) -> Result<(Runtime, Option<ContentPack>, VolumeMap), String> {
    if config.creator_request.is_some()
        && (config.replay.is_some()
            || config.effective_scene() != crate::played::SceneMode::Ecology)
    {
        return Err("the creator requires a new ecology session".into());
    }
    if let Some(trace) = &config.replay {
        trace.validate_rules()?;
    }
    let founding = if config.effective_start().is_some() {
        mesocosm_core::Founding::Drawn
    } else {
        config.effective_body_layout().founding()
    };
    let pack = match &config.replay {
        Some(trace) => trace.content.clone(),
        None if config.generated_content || config.effective_start().is_some() => Some(
            ContentPack::generate(founding.palette())
                .map_err(|why| format!("generation refused: {why:?}"))?,
        ),
        None => None,
    };
    if let Some(pack) = pack {
        let volumes = pack
            .resolve()
            .map_err(|why| format!("pack refused: {why:?}"))?;
        let runtime = runtime(config, founding, pack.palette)?;
        Ok((runtime, Some(pack), volumes))
    } else {
        let runtime = runtime(config, founding, founding.palette())?;
        let volumes = crate::fixture::volumes_for(runtime.world());
        Ok((runtime, None, volumes))
    }
}

#[cfg(test)]
mod tests;
