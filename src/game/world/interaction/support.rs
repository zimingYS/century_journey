//! 处理方块支撑关系，移除失去有效支撑的可附着方块。

use crate::content::block::event::{BlockChangedEvent, BlockNeighborChangedEvent};
use crate::content::block::registry::BlockRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::entity::dropped_item::spawn_dropped_item;
use crate::game::world::state::WorldState;
use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::{Commands, IVec3, MessageReader, MessageWriter, Res, ResMut, Vec3};

/// 在支撑方块发生变化后移除失去支撑的方块。
///
/// 规则由 `BlockProperty::placement.required_support_tag` 声明，因而植物、挂墙物和未来
/// 的附着方块可复用同一系统。标签注册表未就绪时不作删除，避免内容重载窗口误删世界状态。
pub(super) fn remove_unsupported_blocks_system(
    mut neighbor_changes: MessageReader<BlockNeighborChangedEvent>,
    mut changed_blocks: MessageWriter<BlockChangedEvent>,
    block_registry: Option<Res<BlockRegistry>>,
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    mut world_state: ResMut<WorldState>,
    mut commands: Commands,
) {
    let (Some(block_registry), Some(tag_registry)) = (block_registry, tag_registry) else {
        return;
    };

    for neighbor_change in neighbor_changes.read() {
        let support_pos = neighbor_change.neighbor_pos + IVec3::NEG_Y;
        if neighbor_change.changed_pos != support_pos {
            continue;
        }

        let block_id = get_voxel_at_world(neighbor_change.neighbor_pos, &world_state);
        let Some(block) = block_registry.get(block_id) else {
            continue;
        };
        let Some(required_support_tag) = &block.placement.required_support_tag else {
            continue;
        };

        let support_block_id = get_voxel_at_world(support_pos, &world_state);
        if has_required_support(required_support_tag, support_block_id, &tag_registry) {
            continue;
        }

        let item_id = ItemId::new(block.identifier.clone());
        if let Some(change) = set_voxel_at_world(neighbor_change.neighbor_pos, 0, &mut world_state)
        {
            changed_blocks.write(change);
            spawn_dropped_item(
                &mut commands,
                neighbor_change.neighbor_pos.as_vec3() + Vec3::splat(0.5),
                ItemStack::new(item_id, 1),
            );
        }
    }
}

/// 判断支撑方块是否满足内容定义要求；缺失标签成员按不支持处理。
fn has_required_support(
    required_support_tag: &TagId,
    support_block_id: u16,
    tag_registry: &RuntimeTagRegistry,
) -> bool {
    tag_registry.contains(required_support_tag, support_block_id)
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/interaction/support.rs"]
mod tests;
