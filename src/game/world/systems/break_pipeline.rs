use crate::content::block::definition::BlockProperty;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::definition::tool::ToolData;
use crate::content::loot::block_registry::BlockLootRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::block::BlockBehaviorRegistry;
use crate::game::gameplay::block_action::{
    block_break_seconds, can_break_block, can_harvest_block,
};
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::block_ops::set_voxel_at_world;
use crate::game::world::entity::dropped_item::spawn_dropped_item;
use crate::game::world::storage::WorldStorage;
use crate::shared::random::RandomSource;
use bevy::prelude::*;

pub fn execute_block_break(
    world_pos: IVec3,
    block_id: u16,
    gamemode: &PlayerGameMode,
    tag_registry: Option<&RuntimeTagRegistry>,
    active_tool: Option<&ToolData>,
    block_registry: &BlockRegistry,
    behavior_registry: &BlockBehaviorRegistry,
    loot_registry: Option<&BlockLootRegistry>,
    loot_rng: &mut dyn RandomSource,
    world_storage: &mut WorldStorage,
    commands: &mut Commands,
) -> bool {
    if !can_break_block(block_id, gamemode, tag_registry) {
        return false;
    }

    let Some(block) = block_registry.get(block_id) else {
        return false;
    };
    if block_break_seconds(block, gamemode, active_tool).is_none() {
        return false;
    }

    let behavior = behavior_registry.get_behavior_by_id(block_id, block_registry);
    behavior.on_break(world_pos, block_id, world_storage, commands);
    set_voxel_at_world(world_pos, 0, world_storage);

    if should_drop_block_loot(gamemode, block, active_tool)
        && let Some(loot_registry) = loot_registry
    {
        let drops = loot_registry.roll(block_id, loot_rng);
        for (i, (item_id, count)) in drops.into_iter().enumerate() {
            let stack = ItemStack::new(item_id, count);
            spawn_dropped_item(commands, block_drop_spawn_position(world_pos, i), stack);
        }
    }

    true
}

fn should_drop_block_loot(
    gamemode: &PlayerGameMode,
    block: &BlockProperty,
    active_tool: Option<&ToolData>,
) -> bool {
    matches!(gamemode.mode, GameMode::Survival) && can_harvest_block(block, active_tool)
}

/// 掉落物生成在刚被清空的体素内部，避免与上方仍存在的树干重叠。
fn block_drop_spawn_position(world_pos: IVec3, drop_index: usize) -> Vec3 {
    let offset = Vec3::new(
        ((drop_index as f32 * 0.37) % 1.0 - 0.5) * 0.3,
        0.0,
        ((drop_index as f32 * 0.73) % 1.0 - 0.5) * 0.3,
    );
    world_pos.as_vec3() + Vec3::splat(0.5) + offset
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/systems/break_pipeline.rs"]
mod tests;
