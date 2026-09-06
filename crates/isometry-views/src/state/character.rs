//! State transitions for the bounded character-creation panel.

use super::*;

impl UiState {
    /// Open the host-only creation panel with a useful default name. A joined
    /// peer cannot originate campaign authoring, including a token whose owner
    /// would otherwise grant it command rights.
    pub fn open_character(&mut self) {
        if !self.can_edit_inventory {
            self.status = "character creation requires the host".to_owned();
            return;
        }
        self.character_open = true;
    }

    pub fn close_character(&mut self) {
        self.character_open = false;
    }

    /// Ask the host to validate the current system's default sheet before it
    /// places one token through the normal event path. The request stays local
    /// UI state until then, so an unavailable system cannot leave a sheetless
    /// token behind.
    pub fn create_character(&mut self) {
        if !self.can_edit_inventory {
            self.status = "character creation requires the host".to_owned();
            return;
        }
        let name = self.character_name.text().trim();
        if name.is_empty() {
            self.status = "enter a character name".to_owned();
            return;
        }
        let owner = self.character_owner.text().trim();
        let Some(at) = self.available_spawn_tile() else {
            self.status = "no free tile for a character".to_owned();
            return;
        };
        let token = Token {
            id: self.next_token_id(),
            at,
            facing: Facing::South,
            sprite: self.character_sprite.clone(),
            owner: (!owner.is_empty()).then(|| owner.to_owned()),
        };
        self.character_create_request = Some(CharacterCreateRequest {
            token,
            name: name.to_owned(),
        });
        self.character_open = false;
        self.status = format!("creating {name}");
    }
}
