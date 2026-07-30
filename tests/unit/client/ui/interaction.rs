use super::*;

#[test]
fn drag_actions_match_mouse_tweaks_gestures() {
    assert_eq!(drag_action(MouseButton::Left, false), SlotAction::LeftClick);
    assert_eq!(drag_action(MouseButton::Left, true), SlotAction::ShiftClick);
    assert_eq!(
        drag_action(MouseButton::Right, false),
        SlotAction::RightClick
    );
    assert_eq!(
        drag_action(MouseButton::Right, true),
        SlotAction::RightClick
    );
}
