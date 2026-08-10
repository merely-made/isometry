//! The narrow local boundary from a GM's disclosed choice request to one
//! existing Isometry generator.
//!
//! Cleromancy seals the choice. Isometry still owns pack execution, entropy,
//! preview, and commit; this module never writes campaign state or evaluates a
//! generator.

use cleromancy::{
    Candidate, ContextSnapshot, DerivedSelection, Field, Reading, ReadingEngine, UNIFORM_RULE,
};
use isometry_campaign::{GenValue, GeneratorChoice, GeneratorLockPreset};
use serde::Serialize;

use isometry_views::GeneratorSelectionRequest;

pub const GENERATOR_SELECTION_SCHEMA: &str = "isometry.generator-selection/v1";
pub const GENERATOR_SELECTION_FIELD: &str = "isometry.generator-selection-field/v1";

/// The portable result of one host-local choice. Keeping the context and field
/// beside the sealed reading is intentional: they are what make receipt replay
/// meaningful without coupling Isometry to Cleromancy storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GeneratorSelection {
    pub choice_index: usize,
    pub context: ContextSnapshot,
    pub field: Field,
    pub reading: Reading,
}

#[derive(Serialize)]
struct ChoiceDeclaration<'a> {
    id: &'a str,
    name: &'a str,
    default_args: &'a GenValue,
    lock_presets: &'a [GeneratorLockPreset],
}

/// Derive one declared generator choice from public GM input.
pub(crate) fn select_generator(
    choices: &[GeneratorChoice],
    request: &GeneratorSelectionRequest,
) -> Result<GeneratorSelection, String> {
    if choices.is_empty() {
        return Err("no generator packs loaded".to_owned());
    }
    if request.prompt.trim().is_empty() {
        return Err("choose needs a prompt".to_owned());
    }

    let context = ContextSnapshot::new("Isometry generator selection", GENERATOR_SELECTION_SCHEMA)
        .with_fact("prompt", request.prompt.clone())
        .with_fact("candidate_count", choices.len().to_string());
    let field = Field::new(
        GENERATOR_SELECTION_FIELD,
        UNIFORM_RULE,
        choices.iter().map(|choice| {
            let declaration = serde_json::to_string(&ChoiceDeclaration {
                id: &choice.id,
                name: &choice.name,
                default_args: &choice.default_args,
                lock_presets: &choice.lock_presets,
            })
            .expect("Isometry generator declarations always serialize");
            Candidate::new(&choice.id, &choice.name, declaration)
        }),
    );
    let derivation =
        DerivedSelection::new(&request.seed, &request.domain).map_err(|error| error.to_string())?;
    let reading =
        ReadingEngine::derive(&context, &field, &derivation).map_err(|error| error.to_string())?;
    let choice_index = choices
        .iter()
        .position(|choice| choice.id == reading.candidate_id)
        .ok_or_else(|| "selected generator is not declared".to_owned())?;

    Ok(GeneratorSelection {
        choice_index,
        context,
        field,
        reading,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_campaign::{EntropyTape, GeneratorRequest};
    use isometry_system::{GeneratorCatalog, GeneratorLimits};

    use crate::generator_pack_roots;

    #[test]
    fn receipt_selects_a_real_loaded_generator_before_the_existing_preview_call() {
        let catalog = GeneratorCatalog::discover(generator_pack_roots());
        let choices = catalog.choices();
        assert!(!choices.is_empty(), "bundled generator packs must load");
        let request = GeneratorSelectionRequest {
            seed: "session-4".to_owned(),
            domain: "isometry.generator-preview/v1".to_owned(),
            prompt: "What should I prepare for the next scene?".to_owned(),
        };

        let selection = select_generator(&choices, &request).unwrap();
        let replayed = ReadingEngine::replay(
            &selection.context,
            &selection.field,
            &selection.reading.receipt,
        )
        .unwrap();
        assert_eq!(replayed, selection.reading);

        let choice = &choices[selection.choice_index];
        assert_eq!(choice.id, selection.reading.candidate_id);
        let mut tape = EntropyTape::from_seed(7);
        let record = catalog
            .generate(
                "cleromancy.selected.preview",
                &GeneratorRequest {
                    generator: choice.id.clone(),
                    args: choice.default_args.clone(),
                    locks: Default::default(),
                },
                &mut tape,
                GeneratorLimits::default(),
            )
            .unwrap();
        assert_eq!(record.request.generator, selection.reading.candidate_id);
    }

    #[test]
    fn empty_prompt_does_not_queue_a_choice() {
        let choice = GeneratorChoice {
            id: "demo:npc".to_owned(),
            name: "NPC".to_owned(),
            default_args: GenValue::Text {
                value: "example".to_owned(),
            },
            lock_presets: Vec::new(),
        };
        let request = GeneratorSelectionRequest {
            seed: "seed".to_owned(),
            domain: "isometry.generator-preview/v1".to_owned(),
            prompt: " ".to_owned(),
        };
        assert_eq!(
            select_generator(&[choice], &request).unwrap_err(),
            "choose needs a prompt"
        );
    }
}
