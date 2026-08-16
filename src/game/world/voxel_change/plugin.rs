//! 组装世界方块变更管道：命令缓冲、来源记录与唯一应用点。

use crate::game::simulation::SimulationSet;
use crate::game::world::voxel_change::apply::apply_voxel_changes;
use crate::game::world::voxel_change::provenance::VoxelProvenance;
use crate::shared::voxel_change::VoxelChangeBuffer;
use bevy::prelude::*;

/// 组装方块变更缓冲、来源记录资源与唯一应用点。
///
/// 所有对世界的方块写入都先进入 [`VoxelChangeBuffer`]，再由本插件的
/// `apply_voxel_changes` 在 `VoxelChange` 阶段统一应用并记录来源，保证
/// 变更的确定性顺序与来源可追溯。
pub struct VoxelChangePlugin;

impl Plugin for VoxelChangePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelChangeBuffer>()
            .init_resource::<VoxelProvenance>()
            .add_systems(
                FixedUpdate,
                apply_voxel_changes.in_set(SimulationSet::VoxelChange),
            );
    }
}
