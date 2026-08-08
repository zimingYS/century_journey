use super::*;
use crate::client::player::model::components::PlayerPart;
use crate::client::player::model::config::PlayerModelConfig;

#[test]
fn player_visual_camera_pitch_is_clamped_before_world_can_flip() {
    let mut camera = FpsCamera::default();
    camera.add_pitch(10.0);
    assert_eq!(camera.pitch, MAX_CAMERA_PITCH);

    camera.add_pitch(-20.0);
    assert_eq!(camera.pitch, MIN_CAMERA_PITCH);
}

#[test]
fn player_visual_first_person_eye_is_in_front_of_torso() {
    let eye = perspective_offset(CameraPerspective::FirstPerson);
    let torso_front = -PlayerModelConfig::half_dims(PlayerPart::Body).z;

    assert!(eye.z < torso_front);
    assert!(eye.y > PlayerModelConfig::joint_offset(PlayerPart::Body).y);
}

#[test]
fn second_person_camera_is_in_front_and_faces_the_player() {
    let camera = FpsCamera {
        perspective: CameraPerspective::SecondPerson,
        ..default()
    };
    let offset = perspective_offset(camera.perspective);
    let forward = perspective_rotation(&camera) * Vec3::NEG_Z;

    assert!(offset.z < 0.0);
    assert!(forward.z > 0.99);
}

#[test]
fn second_person_pitch_direction_matches_first_person() {
    for pitch in [-0.6f32, -0.2, 0.0, 0.2, 0.6] {
        let first = FpsCamera {
            perspective: CameraPerspective::FirstPerson,
            pitch,
            ..default()
        };
        let second = FpsCamera {
            perspective: CameraPerspective::SecondPerson,
            pitch,
            ..default()
        };
        let first_forward = perspective_rotation(&first) * Vec3::NEG_Z;
        let second_forward = perspective_rotation(&second) * Vec3::NEG_Z;

        // 相同俯仰角下两个视角的垂直视线方向必须一致：正俯仰都向上，负俯仰都向下。
        assert!(
            first_forward.y.signum() == second_forward.y.signum(),
            "pitch={pitch}: first.y={} second.y={}",
            first_forward.y,
            second_forward.y
        );
    }
}

#[test]
fn perspective_cycles_first_second_third() {
    let first = CameraPerspective::FirstPerson;

    assert_eq!(first.next(), CameraPerspective::SecondPerson);
    assert_eq!(first.next().next(), CameraPerspective::ThirdPerson);
    assert_eq!(first.next().next().next(), first);
}
