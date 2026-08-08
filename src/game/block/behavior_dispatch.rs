//! 将方块变更事件分发到对应方块行为（on_change / on_neighbor_update）。

use crate::content::block::event::{BlockChangedEvent, BlockNeighborChangedEvent};
use crate::content::block::registry::BlockRegistry;
use crate::game::block::behavior_registry::BlockBehaviorRegistry;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use crate::shared::voxel_change::VoxelChangeBuffer;
use bevy::prelude::*;

/// 消费变更事件并分发到行为。
///
/// - `on_change`：方块自身被放置/替换/破坏后，通知**新方块**的行为。
/// - `on_neighbor_update`：六方向邻居变化后，通知受影响位置（neighbor_pos）的方块行为。
///
/// 与支撑检查共用事件源；行为推入 buffer 的命令由下一固定步的
/// `apply_voxel_changes` 统一应用（事件跨帧传播，与支撑检查一致）。
pub(crate) fn dispatch_block_behavior_system(
    mut changed: MessageReader<BlockChangedEvent>,
    mut neighbor_changed: MessageReader<BlockNeighborChangedEvent>,
    behavior_registry: Res<BlockBehaviorRegistry>,
    block_registry: Option<Res<BlockRegistry>>,
    mut buffer: ResMut<VoxelChangeBuffer>,
    world_state: Res<WorldState>,
    mut commands: Commands,
) {
    let Some(block_registry) = block_registry else {
        return;
    };

    // on_change：变化后的方块
    for change in changed.read() {
        let behavior = behavior_registry.get_behavior_by_id(change.new_block_id, &block_registry);
        behavior.on_change(
            change.world_pos,
            change.new_block_id,
            &world_state,
            &mut buffer,
            &mut commands,
        );
    }

    // on_neighbor_update：邻居变了 → 接收者位置的方块
    for n in neighbor_changed.read() {
        let block_id = get_voxel_at_world(n.neighbor_pos, &world_state);
        let behavior = behavior_registry.get_behavior_by_id(block_id, &block_registry);
        behavior.on_neighbor_update(
            n.neighbor_pos,
            block_id,
            n.changed_pos,
            n.new_block_id,
            &world_state,
            &mut buffer,
            &mut commands,
        );
    }
}
