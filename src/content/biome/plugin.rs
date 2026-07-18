use crate::content::biome::registry::BiomeRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::validation::ContentCompilation;
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

fn load_biomes_system(mut registry: ResMut<BiomeRegistry>, compilation: Res<ContentCompilation>) {
    let definitions = compilation.content.biomes.clone();
    match registry.replace_definitions(definitions) {
        Ok(()) => log::info!("[Biome] loaded {} definitions", registry.len()),
        Err(error) => log::error!("[Biome] failed to build registry: {error}"),
    }
}
