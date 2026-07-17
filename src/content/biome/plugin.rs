use crate::content::biome::loader::load_biome_definitions;
use crate::content::biome::registry::BiomeRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::engine::asset::AssetManager;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct BiomeContentPlugin;

impl Plugin for BiomeContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_biomes_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}

fn load_biomes_system(mut registry: ResMut<BiomeRegistry>, asset: Res<AssetManager>) {
    let definitions = load_biome_definitions(&asset);
    match registry.replace_definitions(definitions) {
        Ok(()) => log::info!("[Biome] loaded {} definitions", registry.len()),
        Err(error) => log::error!("[Biome] failed to build registry: {error}"),
    }
}
