use bevy::prelude::*;
use crate::core::state::app_state::AppState;
use crate::inventory;
use crate::voxel::event::*;
use crate::voxel::sound::BlockSoundEvent;

pub mod registry;
pub mod properties;
pub mod texture_atlas;
pub mod state;
pub mod model;
pub mod sound;
pub mod event;
pub mod behavior;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin{
    fn build(&self, app: &mut App) { app
        .add_message::<BlockBreakEvent>()
        .add_message::<BlockPlaceEvent>()
        .add_message::<BlockInteractEvent>()
        .add_message::<BlockStateChangeEvent>()
        .add_message::<BlockSoundEvent>()
        .add_systems(OnEnter(AppState::Loading), (
            registry::init_block_registry_system,
            inventory::item::texture_registry::load_item_textures_system,
        ));
    }
}