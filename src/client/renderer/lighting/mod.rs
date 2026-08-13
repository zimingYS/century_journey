//! 客户端光照表现：光级烘焙与实体光源管理。
//!
//! 光级烘焙把 Game 权威的 `ChunkLight` 数组转换为网格顶点光色；
//! 实体光源池把玩家附近的发光方块映射为有限数量的 Bevy `PointLight`，
//! 用于亚体素几何阴影、法线高光以及与太阳和大气环境光的混合。

use bevy::prelude::*;

use crate::shared::states::AppState;

pub mod bake;
pub(crate) mod material;
mod point_lights;

/// 组装客户端体素光烘焙和实体投影光源。
pub struct VoxelLightingPlugin;

impl Plugin for VoxelLightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::VoxelMaterial>::default())
            .init_resource::<point_lights::BlockPointLightCache>()
            .add_systems(
                Update,
                point_lights::sync_block_point_lights.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnExit(AppState::InGame),
                point_lights::cleanup_block_point_lights,
            );
    }
}
