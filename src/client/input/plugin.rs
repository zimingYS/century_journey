//! 组装界面输入、玩法动作采集和光标同步系统。

use bevy::prelude::*;

use crate::client::input::actions::{ClientActionState, collect_player_actions_system};
use crate::client::input::console::console_keyboard_system;
use crate::client::input::context::{
    InputBlocked, InputSet, refresh_input_context_system, resolve_input_context_system,
};
use crate::client::input::cursor::sync_cursor_state_system;
use crate::client::input::interface::{InterfaceCommand, handle_interface_input_system};
use crate::client::input::pointer::{UiInteractionLifecycleEvent, ui_interaction_lifecycle_system};
use crate::client::input::rebind::{RebindCapture, capture_rebind_input_system};
use crate::shared::states::InputContextState;

/// 组装界面输入、玩法动作采集和光标同步系统。
pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputContextState>()
            .init_resource::<InputBlocked>()
            .init_resource::<ClientActionState>()
            .init_resource::<RebindCapture>()
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
            )
            .add_systems(
                PreUpdate,
                capture_rebind_input_system
                    .in_set(InputSet::Interface)
                    .before(handle_interface_input_system),
            )
            .add_systems(
                PreUpdate,
                console_keyboard_system
                    .in_set(InputSet::Interface)
                    .before(handle_interface_input_system)
                    // 必须晚于焦点按键分发：'/' 打开输入框当帧，该按键事件
                    // 已派发给未聚焦窗口而被丢弃，预填的 '/' 不会被重复插入。
                    .after(bevy::input_focus::InputFocusSystems::Dispatch),
            );
    }
}
