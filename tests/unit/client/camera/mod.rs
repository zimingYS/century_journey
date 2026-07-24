use super::*;
use crate::game::player::model::components::PlayerPart;
use crate::game::player::model::config::PlayerModelConfig;
use crate::shared::components::camera::{MAX_CAMERA_PITCH, MIN_CAMERA_PITCH};

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
