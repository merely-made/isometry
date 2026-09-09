// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What admission refuses, and what it cannot be made to move.
//!
//! One test per named done-condition. The shipped pack's parity with the
//! native ruleset lives next door in `packed_gland.rs`, because that one is a
//! claim about the game rather than about this crate.
//!
//! Each fixture writes a scratch pack under the target directory and removes
//! it, so nothing here depends on a temp-file crate or on what another test
//! left behind: the directory is named after the test.

use std::path::{Path, PathBuf};

use mesocosm_core::{Registry, Role, Seeding};

use mesocosm_phenotype::*;

/// A scratch pack root, unique to one test and cleaned up by it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("processes")).expect("a scratch directory");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("a scratch directory");
        }
        std::fs::write(path, body).expect("a scratch file");
    }

    /// The shipped pack's five definitions, copied in, with `edit` applied to
    /// the manifest before it is written.
    fn shipped(name: &str, edit: impl FnOnce(&mut Manifest)) -> Self {
        let scratch = Scratch::new(name);
        let source = shipped_root();
        let mut manifest = discover(&source).expect("the shipped pack discovers");
        for relative in &manifest.processes {
            scratch.write(
                relative,
                &std::fs::read_to_string(source.join(relative)).expect("a shipped definition"),
            );
        }
        edit(&mut manifest);
        scratch.write(
            MANIFEST,
            &serde_json::to_string_pretty(&manifest).expect("a manifest encodes"),
        );
        scratch
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `packs/mesocosm`, found from this crate rather than from a working
/// directory, so a test runs the same from anywhere.
fn shipped_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository root")
        .join("packs")
        .join("mesocosm")
}

const GLAND: &str = r#"{
  "namespace": "mesocosm",
  "name": "secrete",
  "expressed_by": ["plate"],
  "seeding": "acquired"
}"#;

fn manifest(files: &[&str]) -> String {
    format!(
        r#"{{
  "pack": "scratch",
  "version": "0.0.1",
  "abi": 1,
  "license": "MPL-2.0",
  "processes": [{}]
}}"#,
        files
            .iter()
            .map(|file| format!("\"{file}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

// ---------------------------------------------------------------------------
// The shipped pack, and what it lowers to
// ---------------------------------------------------------------------------

#[test]
fn the_shipped_pack_admits_to_the_native_ruleset() {
    // **PD3's parity receipt, at the crate boundary.** Not merely digest-equal:
    // the admitted registry is `==` to the one the core holds, definition for
    // definition, because both are in canonical order.
    let admitted = admit_dir(&shipped_root()).expect("the shipped pack admits");
    assert_eq!(&admitted, Registry::native());
    assert_eq!(admitted.digest(), Registry::native().digest());
}

#[test]
fn the_manifests_file_order_is_not_rule_bearing() {
    // The shipped manifest deliberately lists its files in build order rather
    // than alphabetically; reversing it must change nothing.
    let scratch = Scratch::shipped("order", |manifest| manifest.processes.reverse());
    let reordered = admit_dir(scratch.path()).expect("it still admits");
    assert_eq!(&reordered, Registry::native());
    assert_eq!(reordered.digest(), Registry::native().digest());
}

#[test]
fn author_facing_text_is_not_rule_bearing() {
    // `label` and `note` are what plan §3 keeps outside rule authority, and
    // this is where that is enforced rather than asserted.
    let scratch = Scratch::new("prose");
    scratch.write("processes/secrete.json", GLAND);
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    let plain = admit_dir(scratch.path()).expect("admits");

    let noisy = Scratch::new("prose_noisy");
    noisy.write(
        "processes/secrete.json",
        r#"{
  "namespace": "mesocosm",
  "name": "secrete",
  "expressed_by": ["plate"],
  "seeding": "acquired",
  "label": "a gland",
  "note": "Every word of this is outside the digest."
}"#,
    );
    noisy.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    let described = admit_dir(noisy.path()).expect("admits");

    assert_eq!(plain.digest(), described.digest());
}

#[test]
fn one_rule_bearing_byte_moves_the_ruleset_digest() {
    // Four fields are rule-bearing, and each of them alone must move it.
    let scratch = Scratch::new("rule_bearing");
    scratch.write("processes/secrete.json", GLAND);
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    let base = admit_dir(scratch.path()).expect("admits").digest();

    for (what, body) in [
        (
            "the seeding byte",
            GLAND.replace("\"acquired\"", "\"geometry\""),
        ),
        (
            "the site requirement",
            GLAND.replace("[\"plate\"]", "[\"limb\"]"),
        ),
        ("the name", GLAND.replace("\"secrete\"", "\"secreted\"")),
        ("the namespace", GLAND.replace("\"mesocosm\"", "\"reef\"")),
    ] {
        let moved = Scratch::new("rule_bearing_moved");
        moved.write("processes/secrete.json", &body);
        moved.write(MANIFEST, &manifest(&["processes/secrete.json"]));
        assert_ne!(
            admit_dir(moved.path()).expect("admits").digest(),
            base,
            "{what} is rule-bearing and did not move the ruleset digest"
        );
    }
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

#[test]
fn a_colliding_namespaced_id_is_refused() {
    // Two files, one qualified id. Neither wins.
    let scratch = Scratch::new("collision");
    scratch.write("processes/secrete.json", GLAND);
    scratch.write("processes/again.json", GLAND);
    scratch.write(
        MANIFEST,
        &manifest(&["processes/secrete.json", "processes/again.json"]),
    );
    assert_eq!(
        admit_dir(scratch.path()),
        Err(Admission::DuplicateId {
            id: "mesocosm:secrete".to_string()
        })
    );
}

#[test]
fn a_namespace_is_what_stops_two_definitions_colliding() {
    // The other half of the same claim: the *same* friendly name under two
    // namespaces is not a collision, which is the whole reason ids are
    // qualified.
    let scratch = Scratch::new("namespaced");
    scratch.write("processes/mine.json", GLAND);
    scratch.write(
        "processes/theirs.json",
        &GLAND.replace("\"mesocosm\"", "\"reef\""),
    );
    scratch.write(
        MANIFEST,
        &manifest(&["processes/mine.json", "processes/theirs.json"]),
    );
    let admitted = admit_dir(scratch.path()).expect("two namespaces, no collision");
    assert_eq!(admitted.len(), 2);
}

#[test]
fn a_path_escape_is_refused() {
    let scratch = Scratch::new("escape");
    scratch.write("processes/secrete.json", GLAND);
    for declared in [
        "../secrete.json",
        "processes/../../secrete.json",
        "/etc/secrete.json",
    ] {
        scratch.write(MANIFEST, &manifest(&[declared]));
        assert_eq!(
            admit_dir(scratch.path()),
            Err(Admission::PathEscape {
                declared: declared.to_string()
            }),
            "{declared} should not be readable from inside a pack"
        );
    }
}

#[test]
fn a_malformed_schema_is_refused() {
    let scratch = Scratch::new("malformed");
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    for body in [
        // Not JSON at all.
        "{",
        // A missing rule-bearing field.
        r#"{"namespace": "mesocosm", "name": "secrete", "seeding": "acquired"}"#,
        // A key nobody declared: a typo is not a comment.
        r#"{"namespace": "mesocosm", "name": "secrete", "expresed_by": ["plate"], "seeding": "acquired"}"#,
        // The right key, the wrong type.
        r#"{"namespace": "mesocosm", "name": "secrete", "expressed_by": "plate", "seeding": "acquired"}"#,
    ] {
        scratch.write("processes/secrete.json", body);
        assert!(
            matches!(
                admit_dir(scratch.path()),
                Err(Admission::MalformedSchema { .. })
            ),
            "admitted {body}"
        );
    }
}

#[test]
fn a_word_this_world_does_not_hold_is_refused_not_approximated() {
    // The same discipline `Registry::resolve` keeps one scale down: a shape or
    // a seeding rule this build does not hold is an answer, never the nearest
    // thing it does hold.
    let scratch = Scratch::new("unknown_words");
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));

    scratch.write(
        "processes/secrete.json",
        &GLAND.replace("[\"plate\"]", "[\"frond\"]"),
    );
    assert_eq!(
        admit_dir(scratch.path()),
        Err(Admission::UnknownRole {
            path: "processes/secrete.json".to_string(),
            word: "frond".to_string()
        })
    );

    scratch.write(
        "processes/secrete.json",
        &GLAND.replace("\"acquired\"", "\"learned\""),
    );
    assert_eq!(
        admit_dir(scratch.path()),
        Err(Admission::UnknownSeeding {
            path: "processes/secrete.json".to_string(),
            word: "learned".to_string()
        })
    );
}

#[test]
fn an_undeclared_definition_file_is_refused() {
    let scratch = Scratch::new("undeclared");
    scratch.write("processes/secrete.json", GLAND);
    scratch.write(
        "processes/stowaway.json",
        &GLAND.replace("secrete", "sting"),
    );
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    assert!(
        matches!(admit_dir(scratch.path()), Err(Admission::UndeclaredFile { path }) if path.ends_with("stowaway.json")),
        "an unlisted definition must not be admitted or ignored"
    );
}

#[test]
fn a_declared_file_that_is_not_there_is_refused() {
    let scratch = Scratch::new("missing");
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    assert!(matches!(
        admit_dir(scratch.path()),
        Err(Admission::Unreadable { .. })
    ));
}

#[test]
fn a_pack_this_build_cannot_read_is_refused_before_anything_is_lowered() {
    let scratch = Scratch::new("abi");
    scratch.write("processes/secrete.json", GLAND);
    scratch.write(
        MANIFEST,
        &manifest(&["processes/secrete.json"]).replace("\"abi\": 1", "\"abi\": 99"),
    );
    assert_eq!(
        admit_dir(scratch.path()),
        Err(Admission::UnknownAbi {
            found: 99,
            supported: SUPPORTED_ABI
        })
    );
}

#[test]
fn a_pack_that_declares_nothing_is_refused() {
    let scratch = Scratch::new("empty");
    scratch.write(MANIFEST, &manifest(&[]));
    assert!(matches!(
        admit_dir(scratch.path()),
        Err(Admission::EmptyPack { .. })
    ));
    assert!(matches!(
        admit_dir(&shipped_root().with_file_name("there-is-no-such-pack")),
        Err(Admission::NoManifest { .. })
    ));
}

#[test]
fn a_definition_no_shape_can_express_is_refused() {
    let scratch = Scratch::new("no_site");
    scratch.write(
        "processes/secrete.json",
        &GLAND.replace("[\"plate\"]", "[]"),
    );
    scratch.write(MANIFEST, &manifest(&["processes/secrete.json"]));
    assert!(matches!(
        admit_dir(scratch.path()),
        Err(Admission::NoSite { .. })
    ));
}

#[test]
fn a_pack_admits_completely_or_not_at_all() {
    // Four good files and one bad one admit nothing: half a biology is a
    // different biology, not a smaller one.
    let scratch = Scratch::shipped("partial", |_| {});
    scratch.write("processes/secrete.json", "{ this is not json");
    assert!(matches!(
        admit_dir(scratch.path()),
        Err(Admission::MalformedSchema { .. })
    ));
}

// ---------------------------------------------------------------------------
// Lowering
// ---------------------------------------------------------------------------

#[test]
fn a_definition_the_engine_has_no_binding_for_still_lowers() {
    // A pack minting something new is the point of the door, and the native
    // binding is the core's index rather than an author's claim: a foreign
    // definition lowers with `None` and is otherwise an ordinary rule.
    let scratch = Scratch::new("foreign");
    scratch.write(
        "processes/filter.json",
        r#"{
  "namespace": "reef",
  "name": "filter",
  "expressed_by": ["plate", "mass"],
  "seeding": "acquired"
}"#,
    );
    scratch.write(MANIFEST, &manifest(&["processes/filter.json"]));
    let admitted = admit_dir(scratch.path()).expect("admits");
    let def = admitted
        .get(&mesocosm_core::ProcessId::new("reef", "filter"))
        .expect("the id it declared");
    assert_eq!(def.native, None);
    assert_eq!(def.expressed_by, vec![Role::Plate, Role::Mass]);
    assert_eq!(def.seeding, Seeding::Acquired);
    assert!(def.admits(Role::Mass) && !def.admits(Role::Limb));
}
