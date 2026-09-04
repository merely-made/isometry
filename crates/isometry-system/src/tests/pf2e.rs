//! Pathfinder 2e: the four degrees of success, and the rest of the ladder.
//!
//! `pf2e.rs`'s side. The second first-party system is what proves the
//! substrate holds no 5e assumption: degrees, action economy, the multiple
//! attack penalty, and overland travel all come from the pack.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

/// A PF2e Strike against a known AC, with the d20 forced by choosing the
/// attacker's bonus: the skeleton's whole job is proving the four-rung
/// ladder and crit-doubling ride the *same* resolver 5e uses.
fn pf2e_strike(attack_bonus_str: i64, target_ac: i64, seed: u64) -> Resolution {
    let mut sys = pf2e_srd();
    let mut fighter = sys.default_sheet();
    fighter.set_text("name", "Fighter");
    fighter.set_int("str", attack_bonus_str);
    fighter.set_int("level", 1);
    fighter.set_int("rank_attack", 2);
    let mut foe = sys.default_sheet();
    foe.set_text("name", "Foe");
    foe.set_int("ac", target_ac);
    foe.set_int("hp_current", 200); // survives, so defeat never masks a degree
    foe.set_int("hp_max", 200);
    sys.resolve_action(
        "strike",
        KNIGHT,
        &fighter,
        (4, 4),
        GOBLIN,
        &foe,
        (5, 4),
        &mut Rng::new(seed),
    )
    .expect("resolves")
}

#[test]
fn pf2e_strike_reports_four_degrees_of_success() {
    // STR 30 (+10) and trained at level 1 (+3) is 1d20+13, so the lowest
    // possible total (14) still beats AC 1 by 10: always a critical.
    let crit = pf2e_strike(30, 1, 4);
    assert_eq!(crit.degree, 2, "beat the AC by 10 or more");
    assert!(crit.hit);

    // AC 100: even a 20 misses by 10+, so it always critically fails.
    let fumble = pf2e_strike(10, 100, 4);
    assert_eq!(fumble.degree, -1, "missed the AC by 10 or more");
    assert!(!fumble.hit);
    assert!(fumble.damage.is_none());

    // A plain success and a plain failure are the two middle rungs, and
    // `hit` reads them exactly as a binary system always did.
    let (mut saw_success, mut saw_failure) = (false, false);
    for seed in 1..40u64 {
        // STR 10 (+0), level 1, trained (+3) => 1d20+3 against AC 13:
        // rolls 10..19 succeed but never by 10; 1..9 fail but never by 10.
        let r = pf2e_strike(10, 13, seed);
        match r.degree {
            1 => {
                saw_success = true;
                assert!(r.hit);
            }
            0 => {
                saw_failure = true;
                assert!(!r.hit);
            }
            _ => {}
        }
    }
    assert!(saw_success && saw_failure, "the middle rungs are reachable");
}

#[test]
fn a_pf2e_critical_doubles_the_whole_effect() {
    // 1d20+13 against AC 1 always crits (the lowest total, 14, beats it by
    // 10). Re-run with the AC set to exactly the roll: the same seed rolls
    // the same die and the same damage, but the total now *meets* the AC
    // without beating it by 10, so it is a plain success. The only thing
    // that differs between the two is the degree, and therefore the
    // multiplier.
    let crit = pf2e_strike(30, 1, 11);
    assert_eq!(crit.degree, 2);
    let rolled = crit.attack.total as i64;
    let plain = pf2e_strike(30, rolled, 11);
    assert_eq!(plain.degree, 1);
    let (c, p) = (
        crit.damage.as_ref().expect("crit damages").total,
        plain.damage.as_ref().expect("hit damages").total,
    );
    assert_eq!(c, p * 2, "a critical doubles dice and modifiers together");
    // And the log says why, rather than silently reporting a bigger number.
    assert!(crit.damage.unwrap().expr.contains("200%"));
}

#[test]
fn pf2e_demoralize_frightens_by_degree() {
    let mut sys = pf2e_srd();
    let mut bully = sys.default_sheet();
    bully.set_text("name", "Bully");
    bully.set_int("cha", 30); // +10, trained (+3) at level 1 => 1d20+13
    let mut foe = sys.default_sheet();
    foe.set_text("name", "Foe");
    foe.set_int("hp_current", 200); // no HP change can mask the point
    foe.set_int("hp_max", 200);

    // Will 1: 1d20+13 always beats it by 10, so it always critically
    // succeeds -- and a critical Demoralize inflicts frightened *2*. The
    // magnitude is a number off the degree ladder, not a name and not a
    // constant, and the action deals no damage: fear is the whole effect.
    foe.set_int("will", 1);
    let crit = sys
        .resolve_action("demoralize", KNIGHT, &bully, (4, 4), GOBLIN, &foe, (5, 4), &mut Rng::new(4))
        .expect("resolves");
    assert_eq!(crit.degree, 2);
    assert_eq!(crit.conditions, vec![(GOBLIN, "frightened".to_owned(), 2)]);
    assert!(crit.deltas.iter().all(|d| d.add == 0), "Demoralize deals no damage");

    // Will 14: 1d20+13 still always beats it, but only a natural-ish high
    // roll beats it by 10, so a plain success is reachable -- and a plain
    // success frightens by only 1. Same ladder, a different rung, a
    // different number.
    foe.set_int("will", 14);
    let mut saw_one = false;
    for seed in 1..60u64 {
        let r = sys
            .resolve_action("demoralize", KNIGHT, &bully, (4, 4), GOBLIN, &foe, (5, 4), &mut Rng::new(seed))
            .expect("resolves");
        if r.degree == 1 {
            assert_eq!(r.conditions, vec![(GOBLIN, "frightened".to_owned(), 1)]);
            saw_one = true;
            break;
        }
    }
    assert!(saw_one, "a plain success frightens by 1, not 2");
}

#[test]
fn a_frightened_striker_swings_at_a_penalty() {
    // The read side of the same magnitude. Frightened N is a status penalty
    // to everything, so a frightened Strike is at -N -- and the resolver
    // learns N by reading the injected condition, exactly as it reads any
    // other field. Same seed both times, so the only difference is the fear.
    let mut sys = pf2e_srd();
    let mut fighter = sys.default_sheet();
    fighter.set_text("name", "Fighter");
    fighter.set_int("str", 10); // +0, so the bonus is proficiency alone
    let mut foe = sys.default_sheet();
    foe.set_int("ac", 13);
    foe.set_int("hp_current", 200);
    foe.set_int("hp_max", 200);

    let plain = sys
        .resolve_action("strike", KNIGHT, &fighter, (4, 4), GOBLIN, &foe, (5, 4), &mut Rng::new(7))
        .expect("resolves");
    let afraid_sheet =
        sheet_with_conditions(&fighter, std::iter::once((&"frightened".to_owned(), &2i64)));
    let afraid = sys
        .resolve_action("strike", KNIGHT, &afraid_sheet, (4, 4), GOBLIN, &foe, (5, 4), &mut Rng::new(7))
        .expect("resolves");
    assert_eq!(
        plain.attack.total - afraid.attack.total,
        2,
        "frightened 2 is a -2 status penalty to the Strike"
    );
}

#[test]
fn pf2e_travel_costs_more_when_the_party_loses_the_way() {
    let mut sys = pf2e_srd();
    // A keen navigator (WIS 40, +15) beats any DC on an easy route: smooth
    // travel at the base time, whatever the roll.
    let mut scout = sys.default_sheet();
    scout.set_text("name", "Scout");
    scout.set_int("wis", 40);
    let smooth = sys.resolve_travel(&scout, 2, 100, &mut Rng::new(1));
    assert!(!smooth.lost, "a great navigator does not lose the way");
    assert_eq!(smooth.ticks, 2, "smooth travel is the base (weight 2, normal pace)");

    // A hopeless navigator (WIS 1, -5) on a hard route (weight 20, DC 32)
    // loses the way on any roll, and pays 150% of the base.
    let mut greenhorn = sys.default_sheet();
    greenhorn.set_text("name", "Greenhorn");
    greenhorn.set_int("wis", 1);
    let lost = sys.resolve_travel(&greenhorn, 20, 100, &mut Rng::new(1));
    assert!(lost.lost, "a hopeless navigator on a hard road loses the way");
    assert_eq!(lost.ticks, 30, "lost is 150% of the base 20");
}

#[test]
fn the_navigator_stance_changes_the_travel_outcome() {
    // A borderline navigator (WIS 10, +0) on a weight-4 route (DC 16): the
    // exploration stance is what tips it. Scouting ahead (+3) finds the way
    // on a roll where Searching every thicket (-2) loses it. Find a roll in
    // that flip zone over a fixed seed range.
    let mut sys = pf2e_srd();
    let mut flipped = false;
    for seed in 0..64u64 {
        let mut scout = sys.default_sheet();
        scout.set_int("wis", 10);
        scout.set_text("stance", "scout");
        let scout_lost = sys.resolve_travel(&scout, 4, 100, &mut Rng::new(seed)).lost;

        let mut searcher = sys.default_sheet();
        searcher.set_int("wis", 10);
        searcher.set_text("stance", "search");
        let search_lost = sys.resolve_travel(&searcher, 4, 100, &mut Rng::new(seed)).lost;

        if !scout_lost && search_lost {
            flipped = true;
            break;
        }
    }
    assert!(
        flipped,
        "on some roll, Scouting finds the way where Searching loses it"
    );
}

#[test]
fn a_long_march_tolls_the_party_exhaustion() {
    let mut sys = pf2e_srd();
    let mut scout = sys.default_sheet();
    scout.set_int("wis", 100); // never loses even a hard road, so ticks == base
    // A 20-tick march exhausts the party (level 2), a graded condition; a
    // short hop tires no one.
    let long = sys.resolve_travel(&scout, 20, 100, &mut Rng::new(1));
    assert_eq!(long.ticks, 20);
    assert_eq!(long.exhaustion, 2, "a long march tires the party");
    let short = sys.resolve_travel(&scout, 4, 100, &mut Rng::new(1));
    assert_eq!(short.exhaustion, 0, "a short hop tires no one");

    // 5e declares no toll rule, so its travel never tires.
    let mut plain = srd_5e();
    let sheet = plain.default_sheet();
    assert_eq!(
        plain.resolve_travel(&sheet, 20, 100, &mut Rng::new(1)).exhaustion,
        0,
        "a system with no attrition never tires"
    );
}

#[test]
fn a_long_road_throws_encounters_by_chance() {
    let mut sys = pf2e_srd();
    let mut scout = sys.default_sheet();
    scout.set_int("wis", 100); // never lost, so ticks == base
    // A 30-tick road always runs into something (d20 + 30 clears 25 on any
    // roll); a 1-tick hop never does (it would need a 24 on a d20).
    assert!(
        sys.resolve_travel(&scout, 30, 100, &mut Rng::new(1)).encounter,
        "a very long road always has perils"
    );
    assert!(
        !sys.resolve_travel(&scout, 1, 100, &mut Rng::new(1)).encounter,
        "a short hop is safe"
    );
    // A middling road (15 ticks) is a chance, not a certainty: over seeds,
    // both a safe passage and a peril occur.
    let (mut safe, mut peril) = (false, false);
    for seed in 0..60u64 {
        if sys.resolve_travel(&scout, 15, 100, &mut Rng::new(seed)).encounter {
            peril = true;
        } else {
            safe = true;
        }
        if safe && peril {
            break;
        }
    }
    assert!(safe && peril, "a middling road throws perils by chance, not always");

    // 5e declares no encounter rule, so its roads are safe.
    let mut plain = srd_5e();
    let sheet = plain.default_sheet();
    assert!(!plain.resolve_travel(&sheet, 30, 100, &mut Rng::new(1)).encounter);
}

#[test]
fn a_dull_reader_cannot_read_a_map() {
    let mut sys = pf2e_srd();
    // A scholar (INT 40, +15) reads any map: roll + 15 clears DC 15 always.
    let mut scholar = sys.default_sheet();
    scholar.set_int("int", 40);
    assert!(
        sys.read_map(&scholar, &mut Rng::new(1)),
        "a lettered reader makes sense of it"
    );
    // A brute (INT 1, -5) cannot: only a natural 20 would clear the DC, so
    // over a fixed seed range it fails to read the map -- and holds a map it
    // cannot use.
    let mut brute = sys.default_sheet();
    brute.set_int("int", 1);
    let mut failed = false;
    for seed in 0..64u64 {
        if !sys.read_map(&brute, &mut Rng::new(seed)) {
            failed = true;
            break;
        }
    }
    assert!(failed, "a dull-witted reader fails to read a map");

    // 5e declares no reading rule, so anyone can read a map.
    let mut plain = srd_5e();
    let sheet = plain.default_sheet();
    assert!(plain.read_map(&sheet, &mut Rng::new(1)), "no rule means anyone reads it");
}

#[test]
fn foraging_yields_food_only_when_you_forage() {
    let mut sys = pf2e_srd();
    // A capable forager (WIS 40) who took the Forage stance gathers food.
    let mut forager = sys.default_sheet();
    forager.set_int("wis", 40);
    forager.set_text("stance", "forage");
    assert_eq!(
        sys.resolve_travel(&forager, 4, 100, &mut Rng::new(1)).forage,
        2,
        "foraging on the road gathers food"
    );
    // The same navigator just walking (no stance) gathers nothing.
    let mut walker = sys.default_sheet();
    walker.set_int("wis", 40);
    assert_eq!(
        sys.resolve_travel(&walker, 4, 100, &mut Rng::new(1)).forage,
        0,
        "you gather food only if you forage"
    );

    // 5e declares no foraging rule.
    let mut plain = srd_5e();
    let sheet = plain.default_sheet();
    assert_eq!(plain.resolve_travel(&sheet, 4, 100, &mut Rng::new(1)).forage, 0);
}

#[test]
fn pace_feeds_the_travel_base_and_no_nav_rule_never_loses_the_way() {
    // Pace scales the base the system rules against; a keen navigator travels
    // it smoothly, so the ticks track the pace-scaled base directly.
    let mut sys = pf2e_srd();
    let mut scout = sys.default_sheet();
    scout.set_int("wis", 40);
    assert_eq!(sys.resolve_travel(&scout, 4, 50, &mut Rng::new(2)).ticks, 2, "fast halves the base");
    assert_eq!(sys.resolve_travel(&scout, 4, 200, &mut Rng::new(2)).ticks, 8, "slow doubles it");

    // 5e declares no nav rule, so the party always finds its way at base cost.
    let mut plain = srd_5e();
    let sheet = plain.default_sheet();
    let calm = plain.resolve_travel(&sheet, 6, 100, &mut Rng::new(3));
    assert!(!calm.lost, "a system with no nav rule never loses the way");
    assert_eq!(calm.ticks, 6, "and always pays the base");
}

/// The action economy and the multiple-attack penalty, both proven against
/// the one per-turn counter primitive. The host would inject the running
/// counters into the sheet; here the test does it by hand, so a Strike sees
/// how many actions it has spent and how many times it has struck.
#[test]
fn pf2e_action_economy_and_map_ride_the_turn_counters() {
    let mut sys = pf2e_srd();
    // A quickened fighter (5 actions) so the multiple-attack penalty can be
    // watched past the point the three-action budget would cut it off --
    // proving the penalty and the economy are independent counters.
    let mut base = sys.default_sheet();
    base.set_int("actions_per_turn", 5);
    let foe = {
        let mut f = sys.default_sheet();
        f.set_int("ac", 10);
        f.set_int("hp_current", 500);
        f.set_int("hp_max", 500);
        f
    };
    // A per-turn counter ledger the test advances as the host would.
    let mut counters: std::collections::BTreeMap<String, i64> = Default::default();
    let strike = |sys: &mut System, counters: &std::collections::BTreeMap<String, i64>| {
        let sheet = sheet_with_turn_counters(&base, counters.iter());
        sys.resolve_action(
            "strike",
            KNIGHT,
            &sheet,
            (4, 4),
            GOBLIN,
            &foe,
            (5, 4),
            &mut Rng::new(7),
        )
    };

    // The multiple-attack penalty is in the *bonus*, so the same die gives a
    // lower total on each successive Strike: 0, -5, -10, then -10 (capped).
    let mut totals = Vec::new();
    for _ in 0..4 {
        let r = strike(&mut sys, &counters).expect("affordable while actions remain");
        totals.push(r.attack.total);
        // Apply this Strike's counter effect, as the host would.
        for (_, key, delta) in &r.turn_counters {
            *counters.entry(key.clone()).or_insert(0) += delta;
        }
    }
    assert_eq!(
        totals[0] - totals[1],
        5,
        "the second Strike takes -5 from the multiple-attack penalty"
    );
    assert_eq!(totals[1] - totals[2], 5, "the third takes -10");
    assert_eq!(totals[2], totals[3], "the penalty caps at -10");

    // The economy, now with a plain three-action fighter. Spend down from
    // an empty turn: three Strikes are affordable, the fourth is not.
    let plain = sys.default_sheet(); // actions_per_turn defaults to 3
    let strike3 = |sys: &mut System, spent: &std::collections::BTreeMap<String, i64>| {
        sys.resolve_action(
            "strike",
            KNIGHT,
            &sheet_with_turn_counters(&plain, spent.iter()),
            (4, 4),
            GOBLIN,
            &foe,
            (5, 4),
            &mut Rng::new(7),
        )
    };
    let mut spent: std::collections::BTreeMap<String, i64> = Default::default();
    for i in 0..3 {
        let r = strike3(&mut sys, &spent);
        assert!(r.is_ok(), "action {i} is within the three-action budget");
        for (_, key, delta) in &r.unwrap().turn_counters {
            *spent.entry(key.clone()).or_insert(0) += delta;
        }
    }
    // The fourth is refused before any die: out of actions.
    assert_eq!(
        strike3(&mut sys, &spent),
        Err(ActionError::CannotAfford("strike".to_owned())),
        "the fourth Strike has no action to pay for it"
    );

    // A fresh turn (counters cleared) affords a Strike again.
    assert!(strike3(&mut sys, &Default::default()).is_ok());
}

#[test]
fn a_system_with_no_action_economy_pays_nothing_for_one() {
    // 5e's attack declares no afford rule and no turn effect, so it is always
    // affordable and touches no counter -- the primitive is opt-in.
    let (mut sys, knight, goblin) = duel(1, 50);
    let r = sys
        .resolve_action("attack", KNIGHT, &knight, (4, 4), GOBLIN, &goblin, (5, 4), &mut Rng::new(3))
        .expect("resolves");
    assert!(r.turn_counters.is_empty(), "5e spends no per-turn counter");
}

#[test]
fn each_system_picks_its_own_rungs_on_the_ladder() {
    // The ladder is opt-in. 5e uses three rungs -- crit on a natural 20,
    // hit, miss -- and never a critical failure, because a natural 1 in 5e
    // simply misses rather than fumbling. PF2e uses all four. Neither pays
    // for the other's complexity, and both ride one resolver.
    let (mut sys, knight, goblin) = duel(1, 50);
    let hit = sys
        .resolve_action("attack", KNIGHT, &knight, (4, 4), GOBLIN, &goblin, (5, 4), &mut Rng::new(3))
        .expect("resolves"); // seed 3 rolls a 16
    assert_eq!(hit.degree, 1, "a plain 5e hit");
    assert!(hit.damage.unwrap().expr.contains("1d8"), "unscaled");

    // A natural 1 misses even against AC 1, and is a plain failure, never
    // the fourth rung.
    let (mut sys, knight, goblin) = duel(1, 50);
    let fumble = sys
        .resolve_action("attack", KNIGHT, &knight, (4, 4), GOBLIN, &goblin, (5, 4), &mut Rng::new(42))
        .expect("resolves"); // seed 42 rolls a natural 1
    assert_eq!(fumble.degree, 0, "5e has no critical-failure rung");
    assert!(!fumble.hit, "a natural 1 always misses, whatever the AC");

    // 5e never reaches -1; PF2e does.
    let pf2e_fumble = pf2e_strike(10, 100, 4);
    assert_eq!(pf2e_fumble.degree, -1, "the fourth rung is PF2e's");
}
