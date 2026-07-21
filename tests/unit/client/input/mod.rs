use super::*;

#[test]
fn ui_interaction_translates_press_hold_release_and_cancel() {
    assert_eq!(
        interaction_phase(Interaction::Hovered, Interaction::Pressed),
        Some(UiInteractionPhase::Pressed)
    );
    assert_eq!(
        interaction_phase(Interaction::Pressed, Interaction::Pressed),
        Some(UiInteractionPhase::Held)
    );
    assert_eq!(
        interaction_phase(Interaction::Pressed, Interaction::Hovered),
        Some(UiInteractionPhase::Released)
    );
    assert_eq!(
        interaction_phase(Interaction::Pressed, Interaction::None),
        Some(UiInteractionPhase::Cancelled)
    );
}

#[test]
fn inventory_context_cancels_gameplay_and_close_restores_it() {
    let mut inventory = InventoryState::default();
    let focus = InputFocus::default();
    let search = SearchInputState::default();
    let mut context = InputContextState::default();
    let mut blocked = InputBlocked::default();
    let mut actions = PlayerActionState::default();

    actions.update(true, [PlayerAction::MoveForward]);
    inventory.opened = true;
    resolve_context(
        true,
        &inventory,
        &focus,
        &search,
        &mut context,
        &mut blocked,
    );
    actions.update(context.active().allows_gameplay(), []);

    assert_eq!(context.active(), InputContext::Inventory);
    assert!(blocked.0);
    assert!(actions.cancelled(PlayerAction::MoveForward));

    inventory.opened = false;
    resolve_context(
        true,
        &inventory,
        &focus,
        &search,
        &mut context,
        &mut blocked,
    );
    actions.update(
        context.active().allows_gameplay(),
        [PlayerAction::MoveForward],
    );

    assert_eq!(context.active(), InputContext::Gameplay);
    assert!(!blocked.0);
    assert!(actions.just_pressed(PlayerAction::MoveForward));
}

#[test]
fn back_respects_text_inventory_menu_priority() {
    let gamemode = PlayerGameMode::default();
    let mut inventory = InventoryState::default();
    let mut context = InputContextState::default();
    let mut focus = InputFocus::default();
    let mut search = SearchInputState::default();

    inventory.opened = true;
    search.active = true;
    apply_interface_command(
        InterfaceCommand::Back,
        &gamemode,
        &mut inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert!(inventory.opened);
    assert!(!search.active);

    apply_interface_command(
        InterfaceCommand::Back,
        &gamemode,
        &mut inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert!(!inventory.opened);

    apply_interface_command(
        InterfaceCommand::Back,
        &gamemode,
        &mut inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert!(context.menu_open());

    apply_interface_command(
        InterfaceCommand::Back,
        &gamemode,
        &mut inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert!(!context.menu_open());
}
