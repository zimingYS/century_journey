use super::*;

#[test]
fn stack_open_is_unique_and_back_restores_previous_screen() {
    let mut stack = UiScreenStack::default();
    stack.open(UiScreen::Inventory);
    stack.open(UiScreen::Modal);
    stack.open(UiScreen::Modal);

    assert_eq!(
        stack.iter().collect::<Vec<_>>(),
        [UiScreen::Inventory, UiScreen::Modal]
    );
    assert_eq!(stack.back(), Some(UiScreen::Modal));
    assert_eq!(stack.top(), Some(UiScreen::Inventory));
}

#[test]
fn replace_and_close_keep_stack_consistent() {
    let mut stack = UiScreenStack::default();
    stack.open(UiScreen::MainMenu);
    stack.replace(UiScreen::PauseMenu);
    assert_eq!(stack.top(), Some(UiScreen::PauseMenu));
    assert!(stack.close(UiScreen::PauseMenu));
    assert!(stack.top().is_none());
}
