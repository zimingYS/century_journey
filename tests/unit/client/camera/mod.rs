use super::systems::{perspective_offset, perspective_rotation};
use super::types::{CameraPerspective, FpsCamera, MAX_CAMERA_PITCH, MIN_CAMERA_PITCH};
use crate::client::player::model::components::PlayerPart;
use crate::client::player::model::config::PlayerModelConfig;
use bevy::math::Vec3;

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
    let eye = perspective_offset(CameraPerspective::FirstPerson, 0.0);
    let torso_front = -PlayerModelConfig::half_dims(PlayerPart::Body).z;

    assert!(eye.z < torso_front);
    assert!(eye.y > PlayerModelConfig::joint_offset(PlayerPart::Body).y);
}

#[test]
fn second_person_camera_is_in_front_and_faces_the_player() {
    let camera = FpsCamera {
        perspective: CameraPerspective::SecondPerson,
        ..Default::default()
    };
    let offset = perspective_offset(camera.perspective, camera.pitch);
    let forward = perspective_rotation(&camera) * Vec3::NEG_Z;

    assert!(offset.z < 0.0);
    assert!(forward.z > 0.98);
}

#[test]
fn second_person_orbit_always_looks_at_the_player() {
    // 球面轨道：任何俯仰角下相机 forward 都精确指向玩家（-offset 方向），
    // 且相机 up 保持朝上（地平线不倾斜、画面不翻转）。
    for pitch in [-0.6f32, -0.2, 0.0, 0.2, 0.6] {
        let camera = FpsCamera {
            perspective: CameraPerspective::SecondPerson,
            pitch,
            ..Default::default()
        };
        let offset = perspective_offset(camera.perspective, camera.pitch);
        let rotation = perspective_rotation(&camera);

        let forward = rotation * Vec3::NEG_Z;
        let to_player = (-offset).normalize();
        assert!(
            forward.dot(to_player) > 0.999,
            "pitch={pitch}: forward 应指向玩家，dot={}",
            forward.dot(to_player)
        );

        // up 不得指向下方：`from_rotation_arc` 在相机背对 -Z 时会把画面上下颠倒。
        let up = rotation * Vec3::Y;
        assert!(up.y > 0.0, "pitch={pitch}: up.y={}", up.y);
    }
}

#[test]
fn orbit_radius_shrinks_and_camera_rises_with_pitch() {
    // 球面轨道：|pitch| 增大时水平距离 r·cos(p) 收缩，正俯仰抬高相机。
    for perspective in [
        CameraPerspective::SecondPerson,
        CameraPerspective::ThirdPerson,
    ] {
        let flat = perspective_offset(perspective, 0.0);
        let steep = perspective_offset(perspective, 0.6);
        let flat_horizontal = Vec3::new(flat.x, 0.0, flat.z).length();
        let steep_horizontal = Vec3::new(steep.x, 0.0, steep.z).length();
        assert!(
            steep_horizontal < flat_horizontal,
            "{perspective:?}: 大俯仰角水平距离应收缩"
        );
        assert!(steep.y > flat.y, "{perspective:?}: 正俯仰应抬高相机");
    }
}

#[test]
fn perspective_cycles_first_second_third() {
    let first = CameraPerspective::FirstPerson;

    assert_eq!(first.next(), CameraPerspective::SecondPerson);
    assert_eq!(first.next().next(), CameraPerspective::ThirdPerson);
    assert_eq!(first.next().next().next(), first);
}
