pub mod cache;
pub mod identifier;
pub mod loader;
pub mod registry;

use crate::app::state::AppState;
use crate::shared::tag::cache::{CachedTagCache, TagCache};
use bevy::prelude::*;

pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), init_tag_registry_system);
    }
}

fn init_tag_registry_system(
    mut commands: Commands,
    block_registry: Res<crate::content::block::registry::BlockRegistry>,
) {
    let tag_registry = loader::load_tags_from_assets();
    loader::validate_tags_against_block_registry(&tag_registry, &block_registry);
    let tag_cache = TagCache::build(&tag_registry, &block_registry);
    commands.insert_resource(tag_registry);
    commands.insert_resource(CachedTagCache(tag_cache));
}
