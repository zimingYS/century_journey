//! 解析客户端输入上下文，并定义输入管线调度集合和阻断状态。

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::client::ui::state::SearchInputState;
use crate::game::inventory::state::{InventoryState, LocalInventory};
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::shared::states::app_state::AppState;
use crate::shared::states::{InputContext, InputContextState};

/// 表示当前解析出的输入上下文是否阻止玩家操作。
///
/// 该资源由 Client 输入系统写入，并仅供客户端相机与表现系统读取；
/// 权威 Game 规则通过玩家命令接收意图，不直接依赖此资源。
#[derive(Resource, Default, Debug)]
pub struct InputBlocked(pub bool);

/// 客户端单帧输入管线的明确执行阶段。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSet {
    /// 收集菜单、物品栏和文本框命令。
    Interface,
    /// 根据应用与界面状态解析唯一输入上下文。
    ResolveContext,
    /// 把键鼠状态转换为玩家动作和调试命令。
    CollectActions,
    /// 在帧末同步光标等纯表现状态。
    SyncPresentation,
}

/// 在动作采集前解析本帧可用的输入上下文。
pub(super) fn resolve_input_context_system(
    app_state: Res<State<AppState>>,
    inventory: LocalInventory,
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

/// 在帧末根据可能变化的权威界面状态刷新输入上下文。
pub(super) fn refresh_input_context_system(
    app_state: Res<State<AppState>>,
    inventory: LocalInventory,
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

/// 按文本、菜单、物品栏和玩法的优先级解析唯一活动上下文。
pub(super) fn resolve_context(
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
