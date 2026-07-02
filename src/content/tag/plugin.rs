use bevy::prelude::*;

use crate::content::block::registry::BlockRegistry;
use crate::content::tag::block_tags::TagCache;
use crate::content::tag::block_tags::{
    auto_populate_from_block_tags, validate_tags_against_block_registry,
};
use crate::content::tag::cache::CachedTagCache;
use crate::engine::asset::manager::AssetManager;
use crate::shared::states::app_state::AppState;
use crate::shared::tag::loader;

/// Content 层 Tag Plugin。
///
/// 初始化 TagRegistry、CachedTagCache，并执行 BlockRegistry 相关的标签操作。
/// 这些操作依赖 Content 层类型（BlockRegistry），属于 Content 层职责。
pub struct TagContentPlugin;

impl Plugin for TagContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), init_tag_registry_system);
    }
}

fn init_tag_registry_system(
    mut commands: Commands,
    asset: Res<AssetManager>,
    block_registry: Res<BlockRegistry>,
) {
    let mut tag_registry = loader::load_tags_from_assets(&asset);
    auto_populate_from_block_tags(&mut tag_registry, &block_registry);
    validate_tags_against_block_registry(&tag_registry, &block_registry);
    let tag_cache = TagCache::build(&tag_registry, &block_registry);
    commands.insert_resource(tag_registry);
    commands.insert_resource(CachedTagCache(tag_cache));
}
