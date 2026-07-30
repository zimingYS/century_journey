//! 根据统一方块变更维护树根与逻辑实例之间的一致性。

use crate::content::block::event::BlockChangedEvent;
use crate::content::vegetation::registry::TreeSpeciesRegistry;
use crate::game::world::state::WorldState;
use bevy::prelude::*;

/// 在树根不再是所属树种的树干时移除逻辑实例。
///
/// 树苗生长产生的“树苗变树干”消息会保留刚创建的实例；玩家或其他规则真正替换树根时，
/// 同一方块消息也会把实例清理。未知树种在没有方块变化时保持休眠，不会因内容包暂时缺失
/// 而丢失存档数据。
pub(in crate::game::world::vegetation) fn track_tree_root_changes_system(
    mut changes: MessageReader<BlockChangedEvent>,
    species_registry: Res<TreeSpeciesRegistry>,
    mut world_state: ResMut<WorldState>,
) {
    for change in changes.read() {
        let Some(instance) = world_state.tree_instance(change.world_pos).cloned() else {
            continue;
        };
        let root_still_matches = species_registry
            .get(instance.species())
            .is_some_and(|species| change.new_block_id == species.trunk_block_id);
        if !root_still_matches {
            world_state.remove_tree_instance(change.world_pos);
        }
    }
}
