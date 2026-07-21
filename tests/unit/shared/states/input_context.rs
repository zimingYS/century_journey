use super::*;

#[test]
fn highest_priority_context_wins() {
    assert_eq!(
        InputContext::resolve([
            InputContext::Gameplay,
            InputContext::Inventory,
            InputContext::TextInput,
            InputContext::Menu,
        ]),
        InputContext::TextInput
    );
}
