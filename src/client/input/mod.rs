//! 采集客户端输入，并把界面意图与玩家动作转换为权威层命令。

use bevy::prelude::*;

use crate::shared::states::InputContextState;

mod actions;
mod context;
mod cursor;
mod interface;
mod pointer;

pub use actions::ClientActionState;
pub use context::{InputBlocked, InputSet};
pub use interface::InterfaceCommand;
pub use pointer::{UiInteractionLifecycleEvent, UiInteractionPhase};

use actions::collect_player_actions_system;
use cursor::sync_cursor_state_system;
use interface::handle_interface_input_system;
use pointer::ui_interaction_lifecycle_system;

#[cfg(test)]
use context::resolve_context;
use context::{refresh_input_context_system, resolve_input_context_system};
#[cfg(test)]
use interface::apply_interface_command;
#[cfg(test)]
use pointer::interaction_phase;

/// 组装界面输入、玩法动作采集和光标同步系统。
pub struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputContextState>()
            .init_resource::<InputBlocked>()
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

#[cfg(test)]
#[path = "../../../tests/unit/client/input/mod.rs"]
mod tests;
