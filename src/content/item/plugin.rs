use bevy::prelude::*;

use crate::content::item::model::{ItemModelRegistry, load_item_models_system};
use crate::content::item::registry::registry::{
    ItemRegistry, auto_generate_block_items_system, load_item_definitions_system,
};
use crate::content::item::texture::registry::load_item_textures_system;
use crate::content::lifecycle::{
    ContentReloadSet, ContentStartupSet, content_compilation_valid, content_reload_requested,
};
use crate::shared::states::app_state::AppState;

pub struct ItemContentPlugin;

impl Plugin for ItemContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .init_resource::<ItemModelRegistry>()
            .add_systems(
                OnEnter(AppState::Loading),
                load_item_textures_system
                    .in_set(ContentStartupSet::Assets)
                    .run_if(content_compilation_valid),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    auto_generate_block_items_system,
                    load_item_definitions_system,
                    load_item_models_system,
                )
                    .chain()
                    .in_set(ContentReloadSet::Load)
                    .run_if(content_reload_requested),
            );
    }
}
