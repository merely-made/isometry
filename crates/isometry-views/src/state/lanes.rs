//! The two side-panel text lanes: the `>` command line with its verbs
//! (`>spawn`, `>gen`, `>find`, `>roll`, `>time`) and the whisper composer.
//!
//! Both drafts are `TextInput`s rendered by `caret_text_field` (the command
//! line 2026-07-27, the composer 2026-09-03), so nothing here appends or
//! deletes a character: the field owns text, caret, selection, undo and IME,
//! and these methods only open, cancel and submit the lane.
//!
//! Split out of `play.rs` on 2026-08-08.

use super::*;

impl UiState {
    /// Open the `>` command line. The lane's field takes the caret when it is
    /// built. Entered by the `>` key, the same way `w` opens a whisper.
    pub fn start_command(&mut self) {
        self.command_active = true;
        self.command_draft = cambium::TextInput::default();
        self.command_results.clear();
        self.status = "> command (enter run, esc cancel)".to_owned();
    }

    pub fn command_cancel(&mut self) {
        self.command_active = false;
        self.command_draft = cambium::TextInput::default();
        self.status = "command cancelled".to_owned();
    }

    /// Parse and dispatch the command line, then close it. Every verb routes to
    /// machinery that already exists; the command layer is just the front door.
    pub fn command_submit(&mut self) {
        let input = self.command_draft.text().trim().to_owned();
        self.command_active = false;
        self.command_draft = cambium::TextInput::default();
        if input.is_empty() {
            return;
        }
        match crate::command::parse(&input) {
            crate::command::Command::Spawn(query) => self.spawn_query(&query),
            crate::command::Command::Gen(kind) => self.start_generator(&kind),
            crate::command::Command::Choose {
                seed,
                domain,
                prompt,
            } => self.choose_generator(seed, domain, prompt),
            crate::command::Command::Find(query) => self.find_query(&query),
            crate::command::Command::Roll(expr) => {
                if expr.trim().is_empty() {
                    self.status = "roll what? e.g. >roll 2d6+3".to_owned();
                } else {
                    // Attribute to the actual roller (a joined player rolls as
                    // themselves, not "DM"), the same way `roll_dice` does.
                    self.roll_dice(&expr);
                }
            }
            crate::command::Command::Time(ticks) => self.pass_time(ticks),
            crate::command::Command::Help => {
                self.status = "commands: >spawn >gen >choose >find >roll >time".to_owned();
            }
            crate::command::Command::Unknown(verb) => {
                self.status = format!("unknown command: {verb} (try >help)");
            }
        }
    }

    /// `>spawn <query>`: place a statted creature. Host/DM only, because a
    /// spawn is authoring. Resolves the query to a bestiary entry and reuses
    /// the same path the compendium spawn button takes.
    pub fn spawn_query(&mut self, query: &str) {
        if !self.can_edit_inventory {
            self.status = "spawning requires the host".to_owned();
            return;
        }
        match self.resolve_bestiary(query) {
            Some(key) => self.spawn_monster(&key),
            None => self.status = format!("no monster matches '{query}'"),
        }
    }

    /// Resolve a free-text query to a bestiary key, most specific first: an
    /// exact key, then an exact name, then a name substring, then a key
    /// substring. Deterministic first-match; `>find` is for browsing.
    pub(crate) fn resolve_bestiary(&self, query: &str) -> Option<String> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return None;
        }
        let by = |pred: &dyn Fn(&MonsterRow) -> bool| {
            self.bestiary
                .iter()
                .find(|m| pred(m))
                .map(|m| m.key.clone())
        };
        by(&|m| m.key.to_ascii_lowercase() == q)
            .or_else(|| by(&|m| m.name.to_ascii_lowercase() == q))
            .or_else(|| by(&|m| m.name.to_ascii_lowercase().contains(&q)))
            .or_else(|| by(&|m| m.key.to_ascii_lowercase().contains(&q)))
    }

    /// `>gen <kind>`: select a matching generator and open the existing
    /// generator overlay on a fresh preview.
    pub fn start_generator(&mut self, kind: &str) {
        if !self.can_edit_inventory {
            self.status = "generation requires the host".to_owned();
            return;
        }
        let k = kind.trim().to_ascii_lowercase();
        if k.is_empty() {
            self.status = "generate what? e.g. >gen npc".to_owned();
            return;
        }
        // Match by the id's trailing segment (`demo:npc` -> `npc`), then by a
        // substring of the id or the friendly name.
        let idx = self.generator_choices.iter().position(|c| {
            let suffix =
                c.id.rsplit(':')
                    .next()
                    .unwrap_or(&c.id)
                    .to_ascii_lowercase();
            suffix == k || suffix.contains(&k) || c.name.to_ascii_lowercase().contains(&k)
        });
        match idx {
            Some(i) => {
                self.generator_selected = i;
                self.generator_preview = None;
                self.generator_locks.clear();
                self.generator_open = true;
                // Fire the first preview immediately, so `>gen npc` shows a
                // candidate the DM can reroll or commit at once.
                self.generation_request = Some(GenerationRequest::Generate);
                self.status = format!("generating {}", self.generator_choices[i].name);
            }
            None => self.status = format!("no generator matches '{kind}'"),
        }
    }

    /// `>choose <seed> <domain> <prompt>`: ask the local host to select one of
    /// the already loaded generators, then use the normal preview path. The
    /// view owns only the request; the receipt and generator runtime stay host
    /// concerns.
    pub fn choose_generator(&mut self, seed: String, domain: String, prompt: String) {
        if !self.can_edit_inventory {
            self.status = "generation requires the host".to_owned();
            return;
        }
        if self.generator_choices.is_empty() {
            self.status = "no generator packs loaded".to_owned();
            return;
        }
        self.generator_selection_request = Some(GeneratorSelectionRequest {
            seed,
            domain,
            prompt,
        });
        self.status = "selecting a generator from the local receipt".to_owned();
    }

    /// `>find <query>`: a unified substring search over the compendium
    /// (monsters, items, spells), shown as a list under the command line. Pure
    /// view-side and read-only, so any peer may browse.
    pub fn find_query(&mut self, query: &str) {
        let q = query.trim().to_ascii_lowercase();
        self.command_results.clear();
        if q.is_empty() {
            self.status = "find what? e.g. >find sword".to_owned();
            return;
        }
        const CAP: usize = 12;
        let mut out = Vec::new();
        for m in &self.bestiary {
            if m.name.to_ascii_lowercase().contains(&q) || m.key.to_ascii_lowercase().contains(&q) {
                out.push(format!("monster · {} ({})", m.name, m.key));
            }
        }
        for i in &self.items {
            if i.name.to_ascii_lowercase().contains(&q) {
                out.push(format!("item · {}", i.name));
            }
        }
        for s in &self.spells {
            if s.name.to_ascii_lowercase().contains(&q) {
                out.push(format!("spell · {}", s.name));
            }
        }
        let total = out.len();
        out.truncate(CAP);
        if total > CAP {
            out.push(format!("… and {} more", total - CAP));
        }
        self.command_results = out;
        self.status = if total == 0 {
            format!("no matches for '{query}'")
        } else {
            format!(
                "{total} match{} for '{query}'",
                if total == 1 { "" } else { "es" }
            )
        };
    }

    /// Open the whisper composer. The lane's field takes the caret when it is
    /// built, and owns every key but the two the host keeps (Enter sends,
    /// Escape cancels).
    pub fn start_compose(&mut self) {
        self.composing = true;
        self.whisper_draft = cambium::TextInput::default();
        self.status = "whisper (enter send, esc cancel)".to_owned();
    }

    /// Cancel composing, discarding the draft.
    pub fn compose_cancel(&mut self) {
        self.composing = false;
        self.whisper_draft = cambium::TextInput::default();
        self.status = "whisper cancelled".to_owned();
    }

    /// Send the composed whisper to the current target: log it, and (as a
    /// networked host) queue it for directed delivery.
    pub fn compose_send(&mut self) {
        let text = self.whisper_draft.text().trim().to_owned();
        self.composing = false;
        self.whisper_draft = cambium::TextInput::default();
        if text.is_empty() {
            return;
        }
        let target = self
            .whisper_target
            .clone()
            .unwrap_or_else(|| "table".to_owned());
        self.push_message(format!("to {target}: {text}"));
        self.whisper_outbox.push((target, text));
        self.status = "whisper sent".to_owned();
    }

    /// Record a whisper received from the DM.
    pub fn receive_whisper(&mut self, from: &str, text: &str) {
        self.push_message(format!("from {from}: {text}"));
        self.status = format!("whisper from {from}");
    }

    /// Cycle the whisper target through the connected player names.
    pub fn cycle_whisper_target(&mut self) {
        let names = &self.connected_players;
        if names.is_empty() {
            self.whisper_target = None;
            return;
        }
        self.whisper_target = match &self.whisper_target {
            None => Some(names[0].clone()),
            Some(cur) => {
                let i = names.iter().position(|n| n == cur);
                match i {
                    Some(i) if i + 1 < names.len() => Some(names[i + 1].clone()),
                    _ => None,
                }
            }
        };
    }
}
