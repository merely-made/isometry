//! The sandboxed generator runtime: determinism, fuel, and declared assets.
//!
//! `sys/generator.rs`'s side of the lane. A generator is a pack script the
//! host runs under a fuel cap and a declared asset list, and what comes back
//! is a typed proposal plus the entropy trace that produced it.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

fn generator_request() -> GeneratorRequest {
    GeneratorRequest {
        generator: "demo:forge".to_owned(),
        args: GenValue::Text {
            value: "coast".to_owned(),
        },
        locks: BTreeMap::from([(
            "culture".to_owned(),
            GenValue::Text {
                value: "river-clans".to_owned(),
            },
        )]),
    }
}

#[test]
fn generator_is_deterministic_and_records_host_entropy() {
    let script = r#"
        function call_gen(args, entropy)
            return '{"type":"item","item":{"template":"demo:sword","name":"Blade-' .. entropy .. '","tags":["generated"]}}'
        end
    "#;
    let mut first = GeneratorRuntime::load(script, GeneratorLimits::default()).unwrap();
    let mut second = GeneratorRuntime::load(script, GeneratorLimits::default()).unwrap();
    let mut first_tape = EntropyTape::from_seed(7);
    let mut second_tape = EntropyTape::from_seed(7);

    let first_result = first.call(&generator_request(), &mut first_tape).unwrap();
    let second_result = second.call(&generator_request(), &mut second_tape).unwrap();

    assert_eq!(first_result, second_result);
    assert_eq!(first_tape.draws, second_tape.draws);
    assert_eq!(first_tape.draws, vec![first_result.entropy]);
    assert!(matches!(first_result.value, GenValue::Item { .. }));
}

#[test]
fn generator_fuel_cap_stops_unbounded_scripts() {
    let script = r#"
        function call_gen(args, entropy)
            while true do end
        end
    "#;
    let limits = GeneratorLimits {
        fuel: 128,
        ..GeneratorLimits::default()
    };
    let mut runtime = GeneratorRuntime::load(script, limits).unwrap();
    let mut tape = EntropyTape::from_seed(1);
    assert_eq!(
        runtime.call(&generator_request(), &mut tape).unwrap_err(),
        "generator exhausted fuel"
    );
    assert_eq!(tape.draws.len(), 1);
}

#[test]
fn generator_fixture_checks_proposal_and_entropy_trace() {
    let script = r#"
        function call_gen(args, entropy)
            return '{"type":"text","value":"fixed"}'
        end
    "#;
    let mut runtime = GeneratorRuntime::load(script, GeneratorLimits::default()).unwrap();
    let mut expected_tape = EntropyTape::from_seed(99);
    expected_tape.draw();
    let fixture = GeneratorFixture {
        name: "fixed proposal".to_owned(),
        seed: 99,
        request: generator_request(),
        expected: GenValue::Text {
            value: "fixed".to_owned(),
        },
        expected_draws: expected_tape.draws,
    };
    runtime.run_fixture(&fixture).unwrap();
}

#[test]
fn generator_receives_tagged_request_and_locks_as_lua_tables() {
    let script = r#"
        function call_gen(args_json, entropy, request)
            local culture = request.locks.culture
            if request.generator == "demo:forge"
                and request.args.type == "text"
                and request.args.value == "coast"
                and culture.type == "text"
                and culture.value == "river-clans" then
                return '{"type":"text","value":"typed request"}'
            end
            return '{"type":"text","value":"wrong request"}'
        end
    "#;
    let mut runtime = GeneratorRuntime::load(script, GeneratorLimits::default()).unwrap();
    let mut tape = EntropyTape::from_seed(3);
    assert_eq!(
        runtime.call(&generator_request(), &mut tape).unwrap().value,
        GenValue::Text {
            value: "typed request".to_owned()
        }
    );
}

#[test]
fn generator_returns_nested_tagged_lua_tables() {
    let script = r#"
        function call_gen(request_json, entropy, request)
            return {
                type = "object",
                fields = {
                    title = { type = "text", value = "river cache" },
                    contents = {
                        type = "list",
                        values = {
                            {
                                type = "item",
                                item = {
                                    template = "demo:river-blade",
                                    name = "River Blade",
                                    tags = { "weapon", "river" }
                                }
                            }
                        }
                    }
                }
            }
        end
    "#;
    let mut runtime = GeneratorRuntime::load(script, GeneratorLimits::default()).unwrap();
    let mut tape = EntropyTape::from_seed(4);
    let value = runtime.call(&generator_request(), &mut tape).unwrap().value;
    let GenValue::Object { fields } = value else {
        panic!("expected object proposal");
    };
    assert_eq!(
        fields.get("title"),
        Some(&GenValue::Text {
            value: "river cache".to_owned()
        })
    );
    assert!(matches!(
        fields.get("contents"),
        Some(GenValue::List { values }) if matches!(values.as_slice(), [GenValue::Item { .. }])
    ));
}

#[test]
fn declared_pack_fixture_runs_without_opening_undeclared_assets() {
    let root = std::env::temp_dir().join(format!(
        "isometry-generator-pack-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("generators")).unwrap();
    std::fs::create_dir_all(root.join("fixtures")).unwrap();
    std::fs::write(
        root.join(GeneratorPack::MANIFEST_FILE),
        r#"{
  "format": 1,
  "id": "demo",
  "name": "Demo Pack",
  "version": "0.1.0",
  "generators": [{
"id": "forge_item",
"script": "generators/forge_item.lua",
"fixtures": ["fixtures/forge_item.json"]
  }]
}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("generators/forge_item.lua"),
        r#"function call_gen(args_json, entropy)
return '{"type":"text","value":"forge"}'
end"#,
    )
    .unwrap();
    std::fs::write(
        root.join("fixtures/forge_item.json"),
        r#"{
  "name": "declared fixture",
  "seed": 7,
  "request": {
"generator": "demo:forge_item",
"args": { "type": "text", "value": "river" },
"locks": {}
  },
  "expected": { "type": "text", "value": "forge" },
  "expected_draws": [7191089600892374487]
}"#,
    )
    .unwrap();

    let pack = GeneratorPack::load(&root).unwrap();
    assert_eq!(pack.manifest().id, "demo");
    let request = GeneratorRequest {
        generator: "demo:forge_item".to_owned(),
        args: GenValue::Text {
            value: "river".to_owned(),
        },
        locks: BTreeMap::new(),
    };
    let mut tape = EntropyTape::from_seed(7);
    let record = pack
        .generate(
            "generated.forge.1",
            &request,
            &mut tape,
            GeneratorLimits::default(),
        )
        .unwrap();
    assert_eq!(record.request, request);
    assert_eq!(
        record.proposal,
        GenValue::Text {
            value: "forge".to_owned()
        }
    );
    assert_eq!(record.entropy, tape.draws[0]);
    pack.run_fixture(
        "demo:forge_item",
        "fixtures/forge_item.json",
        GeneratorLimits::default(),
    )
    .unwrap();
    assert!(pack
        .run_fixture(
            "demo:forge_item",
            "fixtures/not-declared.json",
            GeneratorLimits::default(),
        )
        .is_err());

    let catalog = GeneratorCatalog::discover([&root]);
    assert!(catalog.diagnostics().is_empty());
    assert_eq!(catalog.choices()[0].id, "demo:forge_item");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_npc_generator_yields_a_bestiary_backed_creature_that_can_be_statted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/demo");
    let pack = GeneratorPack::load(root).unwrap();
    let request = GeneratorRequest {
        generator: "demo:npc".to_owned(),
        args: GenValue::Text { value: "wilds".to_owned() },
        locks: BTreeMap::new(),
    };
    let gen = |seed: u64| {
        let mut tape = EntropyTape::from_seed(seed);
        let record = pack
            .generate("generated.demo.npc.1", &request, &mut tape, GeneratorLimits::default())
            .unwrap();
        match record.proposal {
            GenValue::Npc { npc } => npc,
            other => panic!("expected an npc proposal, got {other:?}"),
        }
    };

    // The proposal's key is a real bestiary slug, so it lowers to a stat
    // block: this is the bridge `>gen npc` relies on.
    let npc = gen(1);
    let bestiary: Vec<_> = srd_bestiary().into_iter().map(|m| m.key).collect();
    assert!(
        bestiary.contains(&npc.key),
        "generated key {:?} is not a bestiary creature",
        npc.key
    );
    assert!(!npc.name.is_empty(), "an NPC needs a name");
    // And that creature really does stat up.
    let monster = srd_bestiary().into_iter().find(|m| m.key == npc.key).unwrap();
    let mut sheet = monster_sheet(&monster);
    sheet.set_text("name", npc.name.clone());
    assert!(sheet.int("hp_current").unwrap() > 0);
    assert_eq!(sheet.text("name"), Some(npc.name.as_str()));

    // Deterministic per seed; a different draw (reroll) can change the pick.
    assert_eq!(gen(1), gen(1), "same seed, same NPC");
    let differs = (1..8).any(|s| gen(s) != npc);
    assert!(differs, "reroll should be able to produce a different NPC");
}
