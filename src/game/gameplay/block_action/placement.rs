//! 校验方块放置条件，并在成功后消费对应背包物品。

use crate::content::block::registry::BlockRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::InventoryState;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use bevy::prelude::*;

/// 检查当前模式、手持物品和支撑方块是否满足放置条件。
pub fn can_place_block(
    block_id: u16,
    place_pos: IVec3,
    gamemode: &PlayerGameMode,
    active_stack: Option<&ItemStack>,
    block_registry: &BlockRegistry,
    tag_registry: Option<&RuntimeTagRegistry>,
    world_state: &WorldState,
) -> bool {
    if block_id == 0 {
        return false;
    }

    // 创造模式不消耗物品，但仍必须遵守方块本身的放置条件。
    if !gamemode.is_creative() && active_stack.is_none_or(|stack| stack.is_empty()) {
        return false;
    }

    let Some(property) = block_registry.get(block_id) else {
        return false;
    };
    let Some(required_tag) = &property.placement.required_support_tag else {
        return true;
    };
    let Some(tag_registry) = tag_registry else {
        return false;
    };

    let support_pos = place_pos + IVec3::new(0, -1, 0);
    let support_block_id = get_voxel_at_world(support_pos, world_state);

    tag_registry.contains(required_tag, support_block_id)
}

/// 在生存模式消费一个已成功放置的手持方块；创造模式不消费。
pub fn consume_placed_block_item(
    inventory: &mut InventoryState,
    gamemode: &PlayerGameMode,
) -> bool {
    if gamemode.is_creative() {
        return true;
    }

    let index = inventory.hotbar.active_index;
    let Some(stack) = inventory.hotbar.get_stack_mut(index) else {
        return false;
    };

    let _ = stack.take(1);
    if stack.is_empty() {
        inventory.hotbar.set_stack(index, ItemStack::empty());
    }
    true
}
