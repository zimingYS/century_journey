use super::*;
use crate::game::gameplay::gamemode::GameMode;

#[test]
fn creative_break_fix_held_click_only_breaks_on_first_frame() {
    let creative = PlayerGameMode {
        mode: GameMode::Creative,
    };
    let survival = PlayerGameMode::default();
    let mut actions = PlayerActionState::default();

    actions.update(true, [PlayerAction::BreakBlock]);
    assert!(break_action_active(&actions, &creative));

    actions.update(true, [PlayerAction::BreakBlock]);
    assert!(!break_action_active(&actions, &creative));
    assert!(break_action_active(&actions, &survival));
}

#[test]
fn requested_fix_block_inside_player_is_rejected() {
    let half = Vec3::new(0.3, 0.9, 0.3);
    let standing_position = Vec3::new(0.5, 10.9, 0.5);

    assert!(voxel_intersects_player(
        IVec3::new(0, 10, 0),
        standing_position,
        half
    ));
    assert!(!voxel_intersects_player(
        IVec3::new(0, 9, 0),
        standing_position,
        half
    ));
}
