//! 组装通用资产管理资源和同步系统。

use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::pipeline::sync_texture_metadata_system;
use bevy::prelude::*;

/// Engine 层通用资产基础设施插件。
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManager>()
            .add_systems(PostUpdate, sync_texture_metadata_system);
    }
}
