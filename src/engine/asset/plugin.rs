use bevy::prelude::*;

use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::registry::AssetRegistry;

/// Engine Asset Plugin。
///
/// 初始化整个 Asset Pipeline：
/// - AssetManager（资源管理入口）
/// - AssetCache（Handle 缓存）
/// - AssetRegistry（生命周期跟踪）
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AssetManager>()
            .init_resource::<AssetCache>()
            .init_resource::<AssetRegistry>();
    }
}
