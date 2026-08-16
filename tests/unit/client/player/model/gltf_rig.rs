//! glTF 玩家骨架加载器的单元测试。

use super::*;

#[test]
fn part_to_glb_name_maps_every_player_part() {
    use crate::client::player::model::components::PlayerPart;
    let parts = [
        PlayerPart::Head,
        PlayerPart::Body,
        PlayerPart::upper_arm_r(),
        PlayerPart::upper_arm_l(),
        PlayerPart::forearm_r(),
        PlayerPart::forearm_l(),
        PlayerPart::hand_r(),
        PlayerPart::hand_l(),
        PlayerPart::thigh_r(),
        PlayerPart::thigh_l(),
        PlayerPart::calf_r(),
        PlayerPart::calf_l(),
        PlayerPart::foot_r(),
        PlayerPart::foot_l(),
    ];
    for part in parts {
        let name = part_to_glb_name(part);
        assert!(
            !name.is_empty(),
            "PlayerPart {part:?} must map to a non-empty glTF node name"
        );
    }
}

#[test]
fn foot_fallback_uses_calf_node_name() {
    // 无独立 foot 节点，脚绑定到 calf（1:1 对应人体左右）。
    assert_eq!(part_to_glb_name(PlayerPart::foot_r()), "right_calf");
    assert_eq!(part_to_glb_name(PlayerPart::foot_l()), "left_calf");
}

#[test]
fn held_item_anchor_routes_to_right_hand_node() {
    // 玩家右 = Blockbench right_hand（命名已与人体左右一致）。
    assert_eq!(part_to_glb_name(PlayerPart::hand_r()), "right_hand");
}
