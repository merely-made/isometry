//! Tests for the system-plugin lane.
//!
//! They drive `System` end to end (schema, scripted rules, and the SRD pack
//! together) rather than any one split module, so they sit beside the crate
//! root instead of inside a piece of `sys`. Within that, each module below
//! takes one of the subject's own seams: the generator runtime, pack loading
//! and the 5e schema, the action resolver, and the Pathfinder 2e ladder.
//!
//! `duel` and its two token ids stay here because the resolver tests and the
//! Pathfinder ones both stat their duelists with it.
//!
//! Split out of `lib.rs` on 2026-07-24, and along those seams on 2026-09-04;
//! unchanged both times.

use super::*;

mod actions;
mod generator;
mod packs;
mod pf2e;

/// A knight who reliably hits, and a victim whose AC is the only variable.
fn duel(target_ac: i64, target_hp: i64) -> (System, SheetData, SheetData) {
    let sys = srd_5e();
    let mut knight = sys.default_sheet();
    knight.set_text("name", "Knight");
    knight.set_int("str", 16); // +3, plus prof 2 => 1d20+5
    let mut goblin = sys.default_sheet();
    goblin.set_text("name", "Goblin");
    goblin.set_int("ac", target_ac);
    goblin.set_int("hp_current", target_hp);
    goblin.set_int("hp_max", target_hp);
    (sys, knight, goblin)
}

const KNIGHT: TokenId = TokenId(1);
const GOBLIN: TokenId = TokenId(2);
