use super::*;

#[test]
fn actions_have_press_hold_release_lifecycle() {
    let mut state = PlayerActionState::default();
    state.update(true, [PlayerAction::Jump]);
    assert_eq!(
        state.phase(PlayerAction::Jump),
        Some(PlayerActionPhase::Pressed)
    );

    state.update(true, [PlayerAction::Jump]);
    assert_eq!(
        state.phase(PlayerAction::Jump),
        Some(PlayerActionPhase::Held)
    );

    state.update(true, []);
    assert_eq!(
        state.phase(PlayerAction::Jump),
        Some(PlayerActionPhase::Released)
    );
}

#[test]
fn losing_gameplay_context_cancels_held_actions() {
    let mut state = PlayerActionState::default();
    state.update(true, [PlayerAction::BreakBlock]);
    state.update(false, []);

    assert!(state.cancelled(PlayerAction::BreakBlock));
    assert!(!state.pressed(PlayerAction::BreakBlock));
}
