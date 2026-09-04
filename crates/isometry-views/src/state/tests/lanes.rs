//! Tests for `lanes.rs`: the whisper composer and its log.
//!
//! The field owns the text, so these only check what opening, sending, and
//! receiving do to the state around it — the cap included.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

/// The draft is the field's now, so the edits here are the `TextCommand`s the
/// host dispatches into it rather than the `compose_char` pair that used to
/// stand between them — the same shape the `>` command line has taken since
/// 2026-07-27. What is still this test's business is the lane around the
/// field: opening, the send, the log line, and the outbox.
#[test]
fn compose_whisper_builds_draft_and_logs() {
    let mut ui = UiState::new(demo_map());
    ui.whisper_target = Some("alice".to_owned());
    ui.start_compose();
    assert!(ui.composing);
    ui.whisper_draft
        .apply(cambium::TextCommand::Insert("hi".to_owned()));
    ui.whisper_draft.apply(cambium::TextCommand::Backspace);
    ui.whisper_draft
        .apply(cambium::TextCommand::Insert("ey".to_owned()));
    assert_eq!(ui.whisper_draft.text(), "hey");
    ui.compose_send();
    assert!(!ui.composing);
    assert_eq!(ui.messages, vec!["to alice: hey".to_string()]);
    assert_eq!(
        ui.whisper_outbox,
        vec![("alice".to_string(), "hey".to_string())]
    );
    ui.receive_whisper("dm", "watch out");
    assert_eq!(ui.messages[1], "from dm: watch out");
}

/// The pane shows the last five, so unbounded growth was invisible from the
/// screen: only the cap keeps a long session's log from growing forever.
/// Oldest goes first, so the newest (the ones actually rendered) survive.
#[test]
fn message_log_is_capped_and_drops_oldest() {
    let mut ui = UiState::new(demo_map());
    for i in 0..(MESSAGES_CAP + 20) {
        ui.receive_whisper("dm", &format!("line {i}"));
    }
    assert_eq!(ui.messages.len(), MESSAGES_CAP);
    assert_eq!(
        ui.messages.last().unwrap(),
        &format!("from dm: line {}", MESSAGES_CAP + 19),
        "the newest whisper must survive; the pane renders from the tail"
    );
    assert_eq!(ui.messages.first().unwrap(), "from dm: line 20");
}
