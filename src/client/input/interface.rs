//! 解析菜单、物品栏和文本输入命令，并向其权威拥有者转发意图。

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::app::flow::{DialogState, FlowCommand, MenuPage};
use crate::app::settings::{KeyAction, Keybinds};
use crate::client::ui::navigation::UiNavigation;
use crate::client::ui::state::SearchInputState;
use crate::game::inventory::events::InventoryCommand;
use crate::game::inventory::state::{InventoryState, LocalInventory};
use crate::shared::states::InputContextState;
use crate::shared::states::app_state::AppState;

/// 客户端界面输入所表达的导航意图。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceCommand {
    /// 打开本地玩家物品栏。
    OpenInventory,
    /// 关闭本地玩家物品栏。
    CloseInventory,
    /// 在打开与关闭物品栏之间切换。
    ToggleInventory,
    /// 打开暂停菜单。
    OpenMenu,
    /// 关闭暂停菜单。
    CloseMenu,
    /// 按界面优先级返回上一层。
    Back,
    /// 释放当前文本输入焦点。
    ClearTextFocus,
}

/// 处理键盘和界面消息产生的导航意图，并把权威命令转发给所属领域。
///
/// 该系统是界面输入的单一装配点；参数保持显式以便审查每个可写资源和消息出口。
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_interface_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keybinds: Res<Keybinds>,
    app_state: Res<State<AppState>>,
    mut commands: MessageReader<InterfaceCommand>,
    inventory: LocalInventory,
    mut inventory_commands: MessageWriter<InventoryCommand>,
    mut context: ResMut<InputContextState>,
    mut input_focus: ResMut<InputFocus>,
    mut search_state: ResMut<SearchInputState>,
    mut navigation: MessageWriter<UiNavigation>,
    dialog: Res<DialogState>,
    menu_page: Res<MenuPage>,
    mut flow: MessageWriter<FlowCommand>,
) {
    for command in commands.read() {
        if let Some(command) = apply_interface_command(
            *command,
            &inventory,
            &mut context,
            &mut input_focus,
            &mut search_state,
        ) {
            inventory_commands.write(command);
        }
    }

    let text_active = input_focus.get().is_some() || search_state.active;
    let command = if keyboard.just_pressed(KeyCode::Escape) {
        if text_active {
            Some(InterfaceCommand::ClearTextFocus)
        } else if dialog.kind.is_some() {
            flow.write(FlowCommand::CancelDialog);
            None
        } else if *menu_page == MenuPage::Settings {
            flow.write(FlowCommand::CloseSettings);
            None
        } else if matches!(
            app_state.get(),
            AppState::Boot | AppState::Loading | AppState::MainMenu | AppState::WorldLoading
        ) {
            None
        } else {
            navigation.write(UiNavigation::Back);
            None
        }
    } else if keyboard.just_pressed(KeyCode::Enter) && text_active {
        Some(InterfaceCommand::ClearTextFocus)
    } else if keybinds.is_just_pressed(KeyAction::ToggleInventory, &keyboard, &mouse)
        && *app_state.get() == AppState::InGame
        && !text_active
        && !context.menu_open()
    {
        Some(InterfaceCommand::ToggleInventory)
    } else {
        None
    };

    if let Some(command) = command
        && let Some(command) = apply_interface_command(
            command,
            &inventory,
            &mut context,
            &mut input_focus,
            &mut search_state,
        )
    {
        inventory_commands.write(command);
    }
}

/// 更新客户端界面上下文，并返回需要由 Game 固定步消费的库存命令。
pub(super) fn apply_interface_command(
    command: InterfaceCommand,
    inventory: &InventoryState,
    context: &mut InputContextState,
    input_focus: &mut InputFocus,
    search_state: &mut SearchInputState,
) -> Option<InventoryCommand> {
    match command {
        InterfaceCommand::OpenInventory => {
            context.set_menu_open(false);
            Some(InventoryCommand::Open)
        }
        InterfaceCommand::CloseInventory => {
            clear_text_focus(input_focus, search_state);
            Some(InventoryCommand::Close)
        }
        InterfaceCommand::ToggleInventory => {
            if inventory.opened {
                clear_text_focus(input_focus, search_state);
            } else {
                context.set_menu_open(false);
            }
            Some(InventoryCommand::Toggle)
        }
        InterfaceCommand::OpenMenu => {
            context.set_menu_open(true);
            None
        }
        InterfaceCommand::CloseMenu => {
            context.set_menu_open(false);
            None
        }
        InterfaceCommand::ClearTextFocus => {
            clear_text_focus(input_focus, search_state);
            None
        }
        InterfaceCommand::Back => {
            if input_focus.get().is_some() || search_state.active {
                clear_text_focus(input_focus, search_state);
                None
            } else if inventory.opened {
                clear_text_focus(input_focus, search_state);
                Some(InventoryCommand::Close)
            } else {
                context.set_menu_open(!context.menu_open());
                None
            }
        }
    }
}

fn clear_text_focus(input_focus: &mut InputFocus, search_state: &mut SearchInputState) {
    input_focus.clear();
    search_state.active = false;
}
