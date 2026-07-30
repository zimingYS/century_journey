use super::*;
use crate::game::gameplay::gamemode::GameMode;
use crate::game::player::interaction::voxel::{break_action_active, voxel_intersects_player};

#[test]
fn creative_held_break_only_triggers_on_initial_press() {
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
fn block_inside_player_is_rejected() {
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
