use std::collections::HashMap;

use bevy::input::mouse::MouseWheel;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::app::flow::{DialogState, FlowCommand, MenuPage};
use crate::client::ui::navigation::UiNavigation;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::command::{PlayerCommand, PlayerCommandBuffer};
use crate::game::player::components::{LocalPlayer, Player, PlayerLifecycle};
use crate::game::world::time::WorldSimulationClock;
use crate::shared::components::FpsCamera;
use crate::shared::states::app_state::AppState;
use crate::shared::states::{InputBlocked, InputContext, InputContextState, InputSet};
use crate::shared::ui_types::SearchInputState;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceCommand {
    OpenInventory,
    CloseInventory,
    ToggleInventory,
    OpenMenu,
    CloseMenu,
    Back,
    ClearTextFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInteractionPhase {
    Hovered,
    Pressed,
    Held,
    Released,
    Cancelled,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct UiInteractionLifecycleEvent {
    pub entity: Entity,
    pub phase: UiInteractionPhase,
}

pub struct ClientInputPlugin;

#[derive(Resource, Debug, Clone, Default)]
pub struct ClientActionState(PlayerActionState);

impl std::ops::Deref for ClientActionState {
    type Target = PlayerActionState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ClientActionState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputContextState>()
            .init_resource::<ClientActionState>()
            .add_message::<InterfaceCommand>()
            .add_message::<UiInteractionLifecycleEvent>()
            .configure_sets(
                PreUpdate,
                (
                    InputSet::Interface,
                    InputSet::ResolveContext,
                    InputSet::CollectActions,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                handle_interface_input_system.in_set(InputSet::Interface),
            )
            .add_systems(
                PreUpdate,
                resolve_input_context_system.in_set(InputSet::ResolveContext),
            )
            .add_systems(
                PreUpdate,
                collect_player_actions_system.in_set(InputSet::CollectActions),
            )
            .add_systems(Update, ui_interaction_lifecycle_system)
            .add_systems(
                PostUpdate,
                (refresh_input_context_system, sync_cursor_state_system)
                    .chain()
                    .in_set(InputSet::SyncPresentation),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_interface_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut commands: MessageReader<InterfaceCommand>,
    gamemode: Res<PlayerGameMode>,
    mut inventory: ResMut<InventoryState>,
    mut context: ResMut<InputContextState>,
    mut input_focus: ResMut<InputFocus>,
    mut search_state: ResMut<SearchInputState>,
    mut navigation: MessageWriter<UiNavigation>,
    dialog: Res<DialogState>,
    menu_page: Res<MenuPage>,
    mut flow: MessageWriter<FlowCommand>,
) {
    for command in commands.read() {
        apply_interface_command(
            *command,
            &gamemode,
            &mut inventory,
            &mut context,
            &mut input_focus,
            &mut search_state,
        );
    }

    let text_active = input_focus.get().is_some() || search_state.active;
    if keyboard.just_pressed(KeyCode::Escape) {
        if text_active {
            apply_interface_command(
                InterfaceCommand::ClearTextFocus,
                &gamemode,
                &mut inventory,
                &mut context,
                &mut input_focus,
                &mut search_state,
            );
        } else if dialog.kind.is_some() {
            flow.write(FlowCommand::CancelDialog);
        } else if *menu_page == MenuPage::Settings {
            flow.write(FlowCommand::CloseSettings);
        } else if matches!(
            app_state.get(),
            AppState::Boot | AppState::Loading | AppState::MainMenu | AppState::WorldLoading
        ) {
        } else {
            navigation.write(UiNavigation::Back);
        }
    } else if keyboard.just_pressed(KeyCode::Enter) && text_active {
        apply_interface_command(
            InterfaceCommand::ClearTextFocus,
            &gamemode,
            &mut inventory,
            &mut context,
            &mut input_focus,
            &mut search_state,
        );
    } else if keyboard.just_pressed(KeyCode::KeyE)
        && *app_state.get() == AppState::InGame
        && !text_active
        && !context.menu_open()
    {
        apply_interface_command(
            InterfaceCommand::ToggleInventory,
            &gamemode,
            &mut inventory,
            &mut context,
            &mut input_focus,
            &mut search_state,
        );
    }
}

fn apply_interface_command(
    command: InterfaceCommand,
    gamemode: &PlayerGameMode,
    inventory: &mut InventoryState,
    context: &mut InputContextState,
    input_focus: &mut InputFocus,
    search_state: &mut SearchInputState,
) {
    match command {
        InterfaceCommand::OpenInventory => open_inventory(inventory, context),
        InterfaceCommand::CloseInventory => {
            close_inventory(gamemode, inventory, input_focus, search_state)
        }
        InterfaceCommand::ToggleInventory => {
            if inventory.opened {
                close_inventory(gamemode, inventory, input_focus, search_state);
            } else {
                open_inventory(inventory, context);
            }
        }
        InterfaceCommand::OpenMenu => context.set_menu_open(true),
        InterfaceCommand::CloseMenu => context.set_menu_open(false),
        InterfaceCommand::ClearTextFocus => clear_text_focus(input_focus, search_state),
        InterfaceCommand::Back => {
            if input_focus.get().is_some() || search_state.active {
                clear_text_focus(input_focus, search_state);
            } else if inventory.opened {
                close_inventory(gamemode, inventory, input_focus, search_state);
            } else if context.menu_open() {
                context.set_menu_open(false);
            } else {
                context.set_menu_open(true);
            }
        }
    }
}

fn open_inventory(inventory: &mut InventoryState, context: &mut InputContextState) {
    context.set_menu_open(false);
    inventory.opened = true;
}

fn close_inventory(
    gamemode: &PlayerGameMode,
    inventory: &mut InventoryState,
    input_focus: &mut InputFocus,
    search_state: &mut SearchInputState,
) {
    inventory.opened = false;
    clear_text_focus(input_focus, search_state);
    if gamemode.is_creative() {
        inventory.cursor.clear();
    } else {
        crate::client::ui::screens::survival_inventory::handle_inventory_close(inventory);
    }
}

fn clear_text_focus(input_focus: &mut InputFocus, search_state: &mut SearchInputState) {
    input_focus.clear();
    search_state.active = false;
}

fn resolve_input_context_system(
    app_state: Res<State<AppState>>,
    inventory: Res<InventoryState>,
    input_focus: Res<InputFocus>,
    search_state: Res<SearchInputState>,
    mut context: ResMut<InputContextState>,
    mut blocked: ResMut<InputBlocked>,
    player_query: Query<&PlayerLifecycle, With<Player>>,
) {
    let player_alive = player_query.single().is_ok_and(PlayerLifecycle::is_alive);
    resolve_context(
        *app_state.get() == AppState::InGame && player_alive,
        &inventory,
        &input_focus,
        &search_state,
        &mut context,
        &mut blocked,
    );
}

fn refresh_input_context_system(
    app_state: Res<State<AppState>>,
    inventory: Res<InventoryState>,
    input_focus: Res<InputFocus>,
    search_state: Res<SearchInputState>,
    mut context: ResMut<InputContextState>,
    mut blocked: ResMut<InputBlocked>,
    player_query: Query<&PlayerLifecycle, With<Player>>,
) {
    let player_alive = player_query.single().is_ok_and(PlayerLifecycle::is_alive);
    resolve_context(
        *app_state.get() == AppState::InGame && player_alive,
        &inventory,
        &input_focus,
        &search_state,
        &mut context,
        &mut blocked,
    );
}

fn resolve_context(
    app_in_game: bool,
    inventory: &InventoryState,
    input_focus: &InputFocus,
    search_state: &SearchInputState,
    context: &mut InputContextState,
    blocked: &mut InputBlocked,
) {
    let mut candidates = vec![InputContext::Gameplay];
    if !app_in_game {
        candidates.push(InputContext::Menu);
    }
    if inventory.opened {
        candidates.push(InputContext::Inventory);
    }
    if context.menu_open() {
        candidates.push(InputContext::Menu);
    }
    if input_focus.get().is_some() || search_state.active {
        candidates.push(InputContext::TextInput);
    }
    let active = InputContext::resolve(candidates);
    context.set_active(active);
    blocked.0 = !active.allows_gameplay();
}

fn collect_player_actions_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    context: Res<InputContextState>,
    mut state: ResMut<ClientActionState>,
    clock: Option<Res<WorldSimulationClock>>,
    command_buffer: Option<ResMut<PlayerCommandBuffer>>,
    player_query: Query<&Transform, With<LocalPlayer>>,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
) {
    let mut actions = Vec::with_capacity(16);
    if context.active().allows_gameplay() {
        push_pressed(
            &keyboard,
            KeyCode::KeyW,
            PlayerAction::MoveForward,
            &mut actions,
        );
        push_pressed(
            &keyboard,
            KeyCode::KeyS,
            PlayerAction::MoveBackward,
            &mut actions,
        );
        push_pressed(
            &keyboard,
            KeyCode::KeyA,
            PlayerAction::MoveLeft,
            &mut actions,
        );
        push_pressed(
            &keyboard,
            KeyCode::KeyD,
            PlayerAction::MoveRight,
            &mut actions,
        );
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            actions.push(PlayerAction::Sprint);
        }
        push_pressed(&keyboard, KeyCode::Space, PlayerAction::Jump, &mut actions);
        if mouse.pressed(MouseButton::Left) {
            actions.extend([PlayerAction::BreakBlock, PlayerAction::Attack]);
        }
        if mouse.pressed(MouseButton::Right) {
            actions.extend([PlayerAction::PlaceBlock, PlayerAction::Use]);
        }
        if keyboard.just_pressed(KeyCode::KeyQ) {
            actions.push(PlayerAction::DropItem);
        }
        if keyboard.just_pressed(KeyCode::F5) {
            actions.push(PlayerAction::TogglePerspective);
        }

        let hotbar_keys = [
            PlayerAction::Hotbar1,
            PlayerAction::Hotbar2,
            PlayerAction::Hotbar3,
            PlayerAction::Hotbar4,
            PlayerAction::Hotbar5,
            PlayerAction::Hotbar6,
            PlayerAction::Hotbar7,
            PlayerAction::Hotbar8,
            PlayerAction::Hotbar9,
        ];
        let key_codes = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ];
        for (key, action) in key_codes.into_iter().zip(hotbar_keys) {
            if keyboard.just_pressed(key) {
                actions.push(action);
            }
        }
        for event in mouse_wheel.read() {
            if event.y > 0.0 {
                actions.push(PlayerAction::HotbarPrevious);
            } else if event.y < 0.0 {
                actions.push(PlayerAction::HotbarNext);
            }
        }
    } else {
        mouse_wheel.clear();
    }
    state.update(context.active().allows_gameplay(), actions);

    let (Some(clock), Some(mut command_buffer)) = (clock, command_buffer) else {
        return;
    };
    let yaw = player_query
        .single()
        .map(|transform| transform.rotation.to_euler(EulerRot::YXZ).0)
        .unwrap_or(0.0);
    let pitch = camera_query
        .single()
        .map(|camera| camera.pitch)
        .unwrap_or(0.0);
    command_buffer.enqueue(PlayerCommand::from_action_state(
        clock.simulation_tick().saturating_add(1),
        &state,
        yaw,
        pitch,
    ));
}

fn push_pressed(
    keyboard: &ButtonInput<KeyCode>,
    key: KeyCode,
    action: PlayerAction,
    actions: &mut Vec<PlayerAction>,
) {
    if keyboard.pressed(key) {
        actions.push(action);
    }
}

fn sync_cursor_state_system(
    context: Res<InputContextState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_query.single_mut() else {
        return;
    };
    let gameplay = context.active().allows_gameplay();
    cursor.visible = !gameplay;
    cursor.grab_mode = if gameplay {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

fn ui_interaction_lifecycle_system(
    query: Query<(Entity, &Interaction), With<Button>>,
    mut previous: Local<HashMap<Entity, Interaction>>,
    mut writer: MessageWriter<UiInteractionLifecycleEvent>,
) {
    previous.retain(|entity, _| query.get(*entity).is_ok());
    for (entity, interaction) in &query {
        let old = previous.get(&entity).copied().unwrap_or(Interaction::None);
        if let Some(phase) = interaction_phase(old, *interaction) {
            writer.write(UiInteractionLifecycleEvent { entity, phase });
        }
        previous.insert(entity, *interaction);
    }
}

fn interaction_phase(previous: Interaction, current: Interaction) -> Option<UiInteractionPhase> {
    match (previous, current) {
        (Interaction::Pressed, Interaction::Pressed) => Some(UiInteractionPhase::Held),
        (_, Interaction::Pressed) => Some(UiInteractionPhase::Pressed),
        (Interaction::Pressed, Interaction::Hovered) => Some(UiInteractionPhase::Released),
        (Interaction::Pressed, Interaction::None) => Some(UiInteractionPhase::Cancelled),
        (Interaction::None, Interaction::Hovered) => Some(UiInteractionPhase::Hovered),
        (Interaction::Hovered, Interaction::None) => Some(UiInteractionPhase::Cancelled),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
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
}
