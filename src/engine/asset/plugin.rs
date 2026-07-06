use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::pipeline::sync_texture_metadata_system;
use bevy::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManager>()
            .add_systems(PostUpdate, sync_texture_metadata_system);
    }
}
