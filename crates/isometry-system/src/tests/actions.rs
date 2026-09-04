//! Adjudication: attacks, effects, conditions, and the social lane.
//!
//! `sys/system_actions.rs`'s side. An intent goes in, the scripted rules
//! decide, and a resolution comes back that the substrate applies verbatim —
//! the mobility projection included, which is Lua and not Rust.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn a_hit_subtracts_from_the_target_and_nothing_else() {
    // AC 1: the attack cannot fail, so this isolates the consequence.
    // Seed 3 rolls a 16: a plain hit, neither a natural 1 (which always
    // misses) nor a natural 20 (which crits). 50 hit points so the blow
    // cannot fell it -- this test is about the delta, not about defeat.
    let (mut sys, knight, goblin) = duel(1, 50);
    let mut rng = Rng::new(3);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng)
        .expect("resolves");

    assert!(r.hit);
    assert_eq!(r.attack.expr, "1d20+5");
    let dmg = r.damage.as_ref().expect("a hit rolls damage");
    assert!(dmg.total > 0, "damage never heals");
    // Exactly one consequence, and it lands on the victim's hit points.
    assert_eq!(r.deltas.len(), 1);
    assert_eq!(r.deltas[0].token, GOBLIN);
    assert_eq!(r.deltas[0].key, "hp_current");
    assert_eq!(r.deltas[0].add, -(dmg.total as i64));
    // And it represents itself. A solid blow (5+) rocks the victim off its
    // feet rather than merely flinching; either way nothing has moved.
    assert_eq!(r.beats.len(), 2);
    assert_eq!(r.beats[0], Beat::new(KNIGHT, "strike"));
    let expected = if dmg.total >= 5 { "staggered-e" } else { "recoil" };
    assert_eq!(r.beats[1], Beat::new(GOBLIN, expected));
    assert!(r.push.is_none(), "a plain attack moves nobody");
}

#[test]
fn a_miss_changes_nothing() {
    // AC 100 is unreachable by 1d20+5.
    let (mut sys, knight, goblin) = duel(100, 7);
    let mut rng = Rng::new(42);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng)
        .expect("resolves");

    assert!(!r.hit);
    assert!(r.damage.is_none());
    assert!(r.deltas.is_empty(), "a miss must not touch game state");
    assert_eq!(r.beats[1], Beat::new(GOBLIN, "dodge"));
}

#[test]
fn a_fixed_entropy_tape_yields_an_identical_resolution() {
    // The property the whole replication model rests on: one machine
    // resolves, every other machine applies, and they agree.
    let (mut a, knight, goblin) = duel(12, 7);
    let (mut b, _, _) = duel(12, 7);
    let first = a
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut Rng::new(7))
        .expect("resolves");
    let second = b
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut Rng::new(7))
        .expect("resolves");
    assert_eq!(first, second);
}

#[test]
fn an_invalid_intent_is_refused_before_any_die_is_rolled() {
    let (mut sys, knight, goblin) = duel(1, 7);
    let mut rng = Rng::new(42);

    // Out of reach: melee has range 1.
    assert_eq!(
        sys.resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (3, 0), &mut rng),
        Err(ActionError::OutOfRange {
            range: 1,
            distance: 3
        })
    );
    // No hitting yourself.
    assert_eq!(
        sys.resolve_action("attack", KNIGHT, &knight, (0, 0), KNIGHT, &knight, (0, 0), &mut rng),
        Err(ActionError::SelfTarget)
    );
    // An ability check names no victim, so it cannot be resolved at one.
    assert_eq!(
        sys.resolve_action("str_check", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng),
        Err(ActionError::NotTargeted("str_check".to_owned()))
    );
    assert!(sys.is_targeted("attack"));
    assert!(!sys.is_targeted("str_check"));

    // The rng was never drawn from, so a refused intent is truly inert.
    let mut fresh = Rng::new(42);
    let a = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng)
        .expect("resolves");
    let b = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut fresh)
        .expect("resolves");
    assert_eq!(a, b);
}

#[test]
fn a_killing_blow_puts_the_target_out_of_play_and_it_falls() {
    // AC 1 so it always lands; 1 hit point so any damage is lethal.
    let (mut sys, knight, goblin) = duel(1, 1);
    let mut rng = Rng::new(3);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng)
        .expect("resolves");

    assert!(r.hit);
    assert_eq!(r.defeated, vec![GOBLIN]);
    // It falls rather than flinching: the beat follows the outcome.
    assert_eq!(r.beats[1], Beat::new(GOBLIN, "fall"));
}

#[test]
fn a_survivable_hit_does_not_defeat() {
    // 50 hit points: a longsword is not going to do it.
    let (mut sys, knight, goblin) = duel(1, 50);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut Rng::new(3))
        .expect("resolves");
    assert!(r.hit);
    assert!(r.defeated.is_empty());
    let dmg = r.damage.as_ref().expect("a hit rolls damage").total;
    let expected = if dmg >= 5 { "staggered-e" } else { "recoil" };
    assert_eq!(r.beats[1], Beat::new(GOBLIN, expected), "still standing");
}

#[test]
fn a_corpse_is_not_a_target() {
    // Already at zero: the system says it is out of play.
    let (mut sys, knight, goblin) = duel(15, 0);
    assert!(sys.is_defeated(&goblin));
    let mut rng = Rng::new(42);
    assert_eq!(
        sys.resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &goblin, (1, 0), &mut rng),
        Err(ActionError::AlreadyDefeated)
    );
    // Refused before any die is rolled, so the swing costs nothing.
    let a = sys
        .resolve_action("attack", KNIGHT, &knight, (0, 0), GOBLIN, &sys_sheet_alive(), (1, 0), &mut rng)
        .expect("a living target still resolves");
    let b = sys
        .resolve_action(
            "attack",
            KNIGHT,
            &knight,
            (0, 0), GOBLIN,
            &sys_sheet_alive(),
            (1, 0),
            &mut Rng::new(42),
        )
        .expect("resolves");
    assert_eq!(a, b, "the refused swing must not have drawn from the rng");
}

/// A living stand-in victim (AC 1, plenty of hit points).
fn sys_sheet_alive() -> SheetData {
    let mut s = srd_5e().default_sheet();
    s.set_text("name", "Goblin");
    s.set_int("ac", 1);
    s.set_int("hp_current", 50);
    s.set_int("hp_max", 50);
    s
}

/// The distinction the whole force design rests on. Both come out of one
/// resolution, and only one of them is allowed to touch the game.
#[test]
fn a_stagger_is_a_flourish_and_a_shove_is_the_truth() {
    // A solid hit staggers: the victim is rocked off its feet, in the
    // direction the blow came from, and *nothing moves*.
    let (mut sys, knight, goblin) = duel(1, 50);
    let hit = sys
        .resolve_action(
            "attack",
            KNIGHT,
            &knight,
            (4, 4),
            GOBLIN,
            &goblin,
            (5, 4), // due east of the knight
            &mut Rng::new(3),
        )
        .expect("resolves");
    assert!(hit.hit);
    assert_eq!(hit.beats[1], Beat::new(GOBLIN, "staggered-e"), "shoved east");
    assert!(
        hit.push.is_none(),
        "a stagger must not move anybody: it is representation, and state that \
         came out of a flourish could not be agreed on"
    );

    // A shove is the other thing entirely: real forced movement, one tile,
    // and the rules only say how far and which way.
    let (mut sys, knight, goblin) = duel(1, 50);
    let shove = sys
        .resolve_action(
            "shove",
            KNIGHT,
            &knight,
            (4, 4),
            GOBLIN,
            &goblin,
            (5, 4),
            &mut Rng::new(3),
        )
        .expect("resolves");
    assert!(shove.hit);
    assert_eq!(shove.push, Some(((1, 0), 1)), "one tile, due east");
    assert_eq!(shove.beats[1], Beat::new(GOBLIN, "shoved-e"));
    // And it does no damage, so it changes position and nothing else.
    assert!(shove.deltas.iter().all(|d| d.add == 0));
}

#[test]
fn the_board_rules_on_where_a_shove_lands() {
    // The system says "one tile east". The substrate is what knows there is
    // a wall there, so `push_path` is where the shove actually stops.
    let blocked = isometry_core::push_path((5, 4), (1, 0), 1, |_| false);
    assert_eq!(blocked, None, "shoved into a wall: nobody moves");
    let clear = isometry_core::push_path((5, 4), (1, 0), 2, |_| true);
    assert_eq!(clear, Some((7, 4)), "two clear tiles east");
    // Stopped short by an obstacle on the second tile.
    let short = isometry_core::push_path((5, 4), (1, 0), 2, |at| at == (6, 4));
    assert_eq!(short, Some((6, 4)));
}

#[test]
fn a_trip_inflicts_prone_and_the_rules_recompute_mobility() {
    // AC 1: the trip cannot miss, so this isolates the consequence.
    let (mut sys, knight, goblin) = duel(1, 50);
    let r = sys
        .resolve_action("trip", KNIGHT, &knight, (4, 4), GOBLIN, &goblin, (5, 4), &mut Rng::new(3))
        .expect("resolves");
    assert!(r.hit);
    // No damage: prone IS the consequence.
    assert!(r.deltas.iter().all(|d| d.add == 0));
    assert_eq!(r.conditions, vec![(GOBLIN, "prone".to_owned(), 1)]);
    // The projection travels with the change: base speed 5 halves to 2,
    // sight untouched. Rules ran once, on the resolver.
    assert_eq!(r.mobility, vec![(GOBLIN, Some((2, 6)))]);
}

#[test]
fn tripping_the_already_prone_is_not_a_new_condition() {
    let (mut sys, knight, goblin) = duel(1, 50);
    // The caller passes the target sheet with its condition booleans on it,
    // which is how the resolver can tell "apply" from "already there".
    let prone = sheet_with_conditions(&goblin, std::iter::once((&"prone".to_owned(), &1i64)));
    let r = sys
        .resolve_action("trip", KNIGHT, &knight, (4, 4), GOBLIN, &prone, (5, 4), &mut Rng::new(3))
        .expect("resolves");
    assert!(r.hit);
    assert!(r.conditions.is_empty(), "already prone: nothing new to apply");
    assert!(r.mobility.is_empty());
}

#[test]
fn the_projection_is_lua_not_rust() {
    // Blinded is nowhere in the Rust: the system script owns what a
    // condition does to the numbers.
    let mut sys = srd_5e();
    let sheet = sys.default_sheet();
    let blinded = sheet_with_conditions(&sheet, std::iter::once((&"blinded".to_owned(), &1i64)));
    assert_eq!(sys.mobility_for(&blinded, true), Some((5, 0)), "dark, not slow");
    let immobilized =
        sheet_with_conditions(&sheet, std::iter::once((&"immobilized".to_owned(), &1i64)));
    assert_eq!(sys.mobility_for(&immobilized, true), Some((0, 6)), "slow, not dark");
    // No conditions: no override at all; the sheet's base numbers stand.
    assert_eq!(sys.mobility_for(&sheet, false), None);
}

#[test]
fn convince_wins_a_creature_over_when_the_pitch_beats_its_resolve() {
    // The recruit is the system's to *report*, not to apply: it names the
    // target won over and leaves the owner change (and the cap) to the host.
    let mut sys = srd_5e();
    let mut bard = sys.default_sheet();
    bard.set_text("name", "Bard");
    bard.set_int("cha", 18); // +4, plus prof 2 => 1d20+6 to persuade
    bard.set_int("prof", 2);
    let mut goblin = sys.default_sheet();
    goblin.set_text("name", "Goblin");
    goblin.set_int("will", 1); // a pushover: the pitch cannot fail

    let r = sys
        .resolve_action("convince", KNIGHT, &bard, (4, 4), GOBLIN, &goblin, (6, 4), &mut Rng::new(9))
        .expect("resolves");
    assert!(r.hit);
    assert_eq!(r.recruited, Some(GOBLIN), "won over");
    // A social action does no harm.
    assert!(r.deltas.iter().all(|d| d.add == 0));
    assert!(r.defeated.is_empty());

    // A resolute creature (will 99) cannot be talked around.
    let mut wall = goblin.clone();
    wall.set_int("will", 99);
    let miss = sys
        .resolve_action("convince", KNIGHT, &bard, (4, 4), GOBLIN, &wall, (6, 4), &mut Rng::new(9))
        .expect("resolves");
    assert!(!miss.hit);
    assert!(miss.recruited.is_none(), "a failed pitch wins no one");
}

#[test]
fn convince_falls_back_to_the_default_resolve_when_the_sheet_predates_will() {
    // A sheet saved before `will` existed (a pre-C5 campaign) has no such
    // field. The hit rule must resolve against the schema default (12), not
    // error on nil -- otherwise convince silently fails against every legacy
    // token.
    let mut sys = srd_5e();
    let mut bard = sys.default_sheet();
    bard.set_int("cha", 20); // +5, plus prof 2 => 1d20+7
    bard.set_int("prof", 2);
    // A bare sheet with only a name: no `will`, as an old save would be.
    let mut legacy = SheetData::new("5e-srd");
    legacy.set_text("name", "Old Goblin");
    assert!(legacy.int("will").is_none(), "the legacy sheet has no will");

    // Must resolve (not ScriptFailed) and behave as DC 12.
    let r = sys
        .resolve_action("convince", KNIGHT, &bard, (4, 4), GOBLIN, &legacy, (5, 4), &mut Rng::new(1))
        .expect("resolves against the default DC, not an error");
    // The roll landed or missed against 12; either way, no script failure.
    assert_eq!(r.recruited.is_some(), r.hit);
}

#[test]
fn only_a_recruit_action_reports_a_recruit() {
    // A plain attack must never set `recruited`, or the host would change
    // ownership on every hit.
    let (mut sys, knight, goblin) = duel(1, 50);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (4, 4), GOBLIN, &goblin, (5, 4), &mut Rng::new(3))
        .expect("resolves");
    assert!(r.hit);
    assert!(r.recruited.is_none());
}

#[test]
fn a_spawned_goblin_arrives_statted() {
    let goblin = srd_bestiary()
        .into_iter()
        .find(|m| m.name == "Goblin")
        .expect("goblin in the SRD bestiary");
    let sheet = monster_sheet(&goblin);
    // The stat block reaches the sheet, which is what makes it attackable.
    assert_eq!(sheet.int("hp_current"), Some(7));
    assert_eq!(sheet.int("hp_max"), Some(7));
    assert_eq!(sheet.int("ac"), Some(15));
    assert_eq!(sheet.text("name"), Some("Goblin"));
}
