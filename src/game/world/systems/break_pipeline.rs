use crate::content::block::registry::BlockRegistry;
use crate::content::loot::block_registry::BlockLootRegistry;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::world::block_ops::set_voxel_at_world;
use crate::game::world::entity::dropped_item::spawn_dropped_item;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

/// 执行方块破坏的全部流程
/// 根据 GameMode 决定是否生成掉落物：
/// - Creative：直接删除方块，不生成掉落物
/// - Survival：删除方块 → 查询 LootTable → 生成 DroppedItem
pub fn execute_block_break(
    world_pos: IVec3,
    block_id: u16,
    gamemode: &PlayerGameMode,
    block_registry: &BlockRegistry,
    loot_registry: &BlockLootRegistry,
    world_storage: &mut WorldStorage,
    commands: &mut Commands,
) {
    // 调用方块行为
    let behavior = block_registry.get_behavior_by_id(block_id);
    behavior.on_break(world_pos, block_id, world_storage, commands);

    // 实际移除方块
    set_voxel_at_world(world_pos, 0, world_storage);

    // GameMode 决定是否掉落
    match gamemode.mode {
        GameMode::Creative => {
            // 创造模式不掉落
        }
        GameMode::Survival => {
            // 查询掉落表
            let drops = loot_registry.roll(block_id);
            let center = world_pos.as_vec3();

            for (i, stack) in drops.into_iter().enumerate() {
                // 略微随机偏移位置，避免掉落物堆叠
                let offset = Vec3::new(
                    (i as f32 * 0.3) % 1.0 - 0.5,
                    0.3,
                    (i as f32 * 0.7) % 1.0 - 0.5,
                );
                spawn_dropped_item(commands, center + offset, stack);
            }
        }
    }
}
