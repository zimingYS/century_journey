use super::*;

#[test]
fn player_visual_shoe_soles_rest_on_player_collider_bottom() {
    let foot = PlayerPart::foot_r();
    let sole_y = PlayerModelConfig::joint_offset(PlayerPart::thigh_r()).y
        + PlayerModelConfig::joint_offset(PlayerPart::calf_r()).y
        + PlayerModelConfig::joint_offset(foot).y
        + PlayerModelConfig::mesh_offset(foot).y
        - PlayerModelConfig::half_dims(foot).y;

    assert!((sole_y + 0.9).abs() < 0.0001);
    assert!(
        PlayerModelConfig::half_dims(foot).z > PlayerModelConfig::half_dims(PlayerPart::calf_r()).z
    );
}
