//! 组装库存资源、领域消息和固定步命令处理系统。

use bevy::prelude::*;

use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::events::{
    DropItemEvent, InventoryCommand, InventoryFeedbackEvent, SlotInteractionEvent,
};
use crate::game::inventory::runtime;
use crate::game::player::control::command::apply_player_command_system;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// 库存领域在权威固定步命令阶段内的执行顺序。
///
/// 合成等相邻领域只能依赖这些阶段，不应直接依赖库存系统的私有实现。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InventorySet {
    /// 应用打开、关闭、整理等库存命令。
    Commands,
    /// 路由来自界面的槽位交互。
    SlotInteraction,
    /// 应用数字键和滚轮产生的快捷栏选择。
    HotbarSelection,
}

/// Game 层 Inventory 模块 Plugin。
///
/// 只负责 Game 层运行时系统。
/// Definition/Registry/Loader/Texture 已在 Content 层的 ItemContentPlugin 中注册。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::game::inventory::state::AccessorySlotDefinitions>()
            .init_resource::<WorldContainers>()
            .add_message::<SlotInteractionEvent>()
            .add_message::<DropItemEvent>()
            .add_message::<InventoryCommand>()
            .add_message::<InventoryFeedbackEvent>()
            .configure_sets(
                FixedUpdate,
                (
                    InventorySet::Commands,
                    InventorySet::SlotInteraction,
                    InventorySet::HotbarSelection,
                )
                    .chain()
                    .in_set(SimulationSet::Commands),
            )
            .add_systems(
                FixedUpdate,
                runtime::handle_inventory_command_system
                    .in_set(InventorySet::Commands)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                runtime::handle_slot_interaction_system
                    .in_set(InventorySet::SlotInteraction)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                runtime::handle_hotbar_command_system
                    .in_set(InventorySet::HotbarSelection)
                    .after(apply_player_command_system)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
