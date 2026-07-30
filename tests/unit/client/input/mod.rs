use super::*;
use bevy::input_focus::InputFocus;
use bevy::prelude::Interaction;

use crate::client::ui::state::SearchInputState;
use crate::game::inventory::events::InventoryCommand;
use crate::game::inventory::state::InventoryState;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::shared::states::{InputContext, InputContextState};

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
    let mut inventory = InventoryState::default();
    let mut context = InputContextState::default();
    let mut focus = InputFocus::default();
    let mut search = SearchInputState::default();

    inventory.opened = true;
    search.active = true;
    let command = apply_interface_command(
        InterfaceCommand::Back,
        &inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert_eq!(command, None);
    assert!(inventory.opened);
    assert!(!search.active);

    let command = apply_interface_command(
        InterfaceCommand::Back,
        &inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert_eq!(command, Some(InventoryCommand::Close));

    // Game 会在后续固定步应用关闭命令；测试在此模拟已同步的权威状态。
    inventory.opened = false;
    let command = apply_interface_command(
        InterfaceCommand::Back,
        &inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert_eq!(command, None);
    assert!(context.menu_open());

    let command = apply_interface_command(
        InterfaceCommand::Back,
        &inventory,
        &mut context,
        &mut focus,
        &mut search,
    );
    assert_eq!(command, None);
    assert!(!context.menu_open());
}

#[test]
fn inventory_toggle_is_forwarded_without_client_side_mutation() {
    let inventory = InventoryState::default();
    let mut context = InputContextState::default();
    let mut focus = InputFocus::default();
    let mut search = SearchInputState::default();

    let command = apply_interface_command(
        InterfaceCommand::ToggleInventory,
        &inventory,
        &mut context,
        &mut focus,
        &mut search,
    );

    assert_eq!(command, Some(InventoryCommand::Toggle));
    assert!(!inventory.opened);
}
