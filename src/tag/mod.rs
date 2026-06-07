pub mod identifier;
pub mod registry;
pub mod loader;
pub mod cache;

use bevy::prelude::*;
use crate::core::state::app_state::AppState;
use crate::tag::cache::{CachedTagCache, TagCache};

pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            init_tag_registry_system,
        );
    }
}

fn init_tag_registry_system(
    mut commands: Commands,
    block_registry: Res<crate::voxel::registry::BlockRegistry>,
) {
    let tag_registry = loader::load_tags_from_assets();

    // 验证标签中的条目是否在方块注册表中存在
    loader::validate_tags_against_block_registry(&tag_registry, &block_registry);
    // 构建标签缓存
    let tag_cache = TagCache::build(&tag_registry, &block_registry);

    commands.insert_resource(tag_registry);
    commands.insert_resource(CachedTagCache(tag_cache));
}