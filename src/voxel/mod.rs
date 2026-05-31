use bevy::prelude::*;
use crate::core::state::app_state::AppState;

pub mod registry;
pub mod properties;
pub mod texture_atlas;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin{
    fn build(&self, app: &mut App) { app
        .add_systems(OnEnter(AppState::Loading), (
            registry::init_block_registry_system,
        ));
    }
}