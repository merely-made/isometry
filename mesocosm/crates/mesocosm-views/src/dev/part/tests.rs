use super::*;

#[test]
fn intake_reading_follows_declared_ports_and_distinguishes_inactive_tissue() {
    use mesocosm_core::{IntakePort, NisKind, Process, Registry, state_hash};
    let (mut world, id, part) = fixture();
    let registry = Registry::native();
    let port = IntakePort::live(NisKind::Consumer)
        .supported_by(registry.of_native(Process::Intake).reference());
    let organism = world.organisms.iter_mut().find(|o| o.id == id).unwrap();
    let geometry = organism.body().clone();
    assert!(organism.phenotype.declare_port(part, port));
    let before = state_hash(&world);
    let reading = part_of(&world, id, part, &History::new()).reading.unwrap();
    assert_eq!(reading.intake, "active: living consumers");
    assert!(reading.feeding.contains("living consumers"));
    assert_eq!(
        state_hash(&world),
        before,
        "inspection changes no world facts"
    );

    let organism = world.organisms.iter_mut().find(|o| o.id == id).unwrap();
    assert_eq!(organism.body(), &geometry);
    assert!(organism.phenotype.declare_port(
        part,
        port.supported_by(registry.of_native(Process::Sense).reference()),
    ));
    let reading = part_of(&world, id, part, &History::new()).reading.unwrap();
    assert_eq!(reading.intake, "inactive: living consumers");
}
use mesocosm_core::{Event, History, PartId, RecordedEvent, World};

fn fixture() -> (World, OrganismId, PartId) {
    let world = World::new(0x00A7_7AC4, 8);
    let organism = world.controlled_id().expect("the world starts embodied");
    let part = world
        .organisms
        .iter()
        .find(|found| found.id == organism)
        .expect("the controlled critter is present")
        .body()
        .root;
    (world, organism, part)
}

#[test]
fn reading_distinguishes_actual_sites_from_unknown_dormant_allocations() {
    let (world, organism, part) = fixture();
    let reading = part_of(&world, organism, part, &History::new())
        .reading
        .expect("the founding root has a phenotype reading");

    assert!(reading.process.contains("dormant: unknown"));
    assert!(reading.process.contains("actual: "));
    if !reading.process.contains("actual: none") {
        assert!(
            reading.process.contains("("),
            "actual sites carry their cause"
        );
    }
    assert!(reading.donor == "founding tissue");
    assert!(reading.discovery_condition == "unknown");
    assert!(reading.history_event == "unknown");
}

#[test]
fn current_condition_and_exact_history_event_are_separate_rows() {
    let (world, organism, part) = fixture();
    let mut history = History::new();
    history.record(RecordedEvent::new(
        7,
        None,
        Event::Expressed {
            organism,
            part,
            cost_mg: 2,
        },
    ));
    let reading = part_of(&world, organism, part, &history)
        .reading
        .expect("the selected part remains readable");

    assert!(
        reading.condition.starts_with("living; ") || reading.condition.starts_with("carcass; ")
    );
    assert_eq!(reading.discovery_condition, "unknown");
    assert_eq!(reading.history_event, "expressed tick 7");
}

#[test]
fn history_for_another_part_does_not_claim_the_selected_part() {
    let (world, organism, part) = fixture();
    let mut history = History::new();
    history.record(RecordedEvent::new(
        10,
        None,
        Event::Grew {
            organism: OrganismId(u32::MAX),
            part,
        },
    ));
    history.record(RecordedEvent::new(
        9,
        None,
        Event::Grew {
            organism,
            part: PartId(part.0.saturating_add(1)),
        },
    ));

    let inspection = part_of(&world, organism, part, &history);
    assert_eq!(
        inspection
            .reading
            .expect("selected part exists")
            .history_event,
        "unknown"
    );
}

#[test]
fn incorporated_tissue_keeps_donor_identity_without_inventing_an_event() {
    let (mut world, organism, part) = fixture();
    let subject = world
        .organisms
        .iter_mut()
        .find(|o| o.id == organism)
        .unwrap();
    let mut body = subject.body().clone();
    body.parts[part.0 as usize].provenance.origin = Origin::Incorporated {
        from_species: mesocosm_core::SpeciesId(42),
        from_part: PartId(17),
    };
    subject.phenotype = mesocosm_core::BodyPhenotype::seed(body);
    let reading = part_of(&world, organism, part, &History::new())
        .reading
        .unwrap();
    assert_eq!(reading.donor, "line 42 part 17");
    assert_eq!(reading.history_event, "unknown");
}

#[test]
fn unavailable_identity_is_explicit() {
    let (world, organism, _part) = fixture();
    assert_eq!(
        part_of(&world, organism, PartId(u32::MAX), &History::new())
            .notice
            .as_deref(),
        Some("the selected part is unavailable")
    );
    assert_eq!(
        part_of(&world, OrganismId(u32::MAX), PartId(0), &History::new())
            .notice
            .as_deref(),
        Some("the selected critter is unavailable")
    );
}
