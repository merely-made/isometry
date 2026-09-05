use super::*;

/// The companion rule ruled with the half-height: the frame's floor is the
/// world's floor, whatever the body it is following is standing on.
#[test]
fn the_follow_centre_never_frames_below_bedrock() {
    let pan = Pan::default();
    for mode in CameraMode::ALL {
        for half in [20.0, SLAB_HALF_HEIGHT, 36.3, 48.0] {
            for y in [0, 3, 12, 24, 40] {
                let centre = centre_on([7, y, -5], pan, half, mode);
                // The camera's own reach, not the half-height: a tilted
                // section frames more than its half-height and the clamp
                // has to know it.
                let reach = mode.vertical_half(half);
                assert!(
                    centre[1] - reach >= 0.0,
                    "{} at half {half}, y {y} framed {} below bedrock",
                    mode.name(),
                    reach - centre[1]
                );
                assert_eq!(
                    [centre[0], centre[2]],
                    [7.0, -5.0],
                    "the clamp moved x or z"
                );
            }
        }
    }
}

/// It is a floor, not a lock: a body standing above it is still followed,
/// and the pan still pans.
#[test]
fn a_body_above_the_floor_is_followed_and_panned() {
    let high = centre_on(
        [0, 60, 0],
        Pan { x: 2.0, y: 3.0 },
        SLAB_HALF_HEIGHT,
        CameraMode::Side,
    );
    assert_eq!(high, [2.0, 63.0, 0.0]);
    // Panning down into the void stops at the floor rather than showing it.
    let low = centre_on(
        [0, 30, 0],
        Pan { x: 0.0, y: -20.0 },
        SLAB_HALF_HEIGHT,
        CameraMode::Side,
    );
    assert_eq!(low[1], SLAB_HALF_HEIGHT);
}

/// A host that names no half-height frames with the ruled default, and the
/// clamp reads the same number the camera does.
#[test]
fn an_unset_half_height_falls_back_to_the_ruled_default() {
    assert_eq!(half_height_or_default(0.0), SLAB_HALF_HEIGHT);
    assert_eq!(half_height_or_default(-1.0), SLAB_HALF_HEIGHT);
    assert_eq!(half_height_or_default(36.3), 36.3);
}

/// The vanish, retired at the boundary that used to swallow it. Mark's
/// second playtest reached 304 living parts; the host now poses that body
/// and says what it could not carry.
#[test]
fn a_body_of_the_playtest_size_is_still_posed() {
    use mesocosm_core::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2, 2, 2]);
    for index in 0..303 {
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [2, 1, 1],
            Attachment {
                parent: body.root,
                offset: [index + 4, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    }
    assert_eq!(body.living().count(), 304);
    let (pose, dropped) = pose_at(&body, [0, 8, 0], [0.4, 0.6, 0.4], 1.0).expect("a posed body");
    assert_eq!(pose.capsules.len(), mesocosm_lens::MAX_CAPSULES);
    assert_eq!(dropped, 48);
}
