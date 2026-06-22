use crate::app::state::AppState;
use crate::content::block::event::*;
use crate::content::block::sound::BlockSoundEvent;
use bevy::prelude::*;

pub mod behavior;
pub mod event;
pub mod model;
pub mod properties;
pub mod registry;
pub mod sound;
pub mod state;
pub mod texture_atlas;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BlockBreakEvent>()
            .add_message::<BlockPlaceEvent>()
            .add_message::<BlockInteractEvent>()
            .add_message::<BlockStateChangeEvent>()
            .add_message::<BlockSoundEvent>()
            .add_systems(
                OnEnter(AppState::Loading),
                (registry::init_block_registry_system,),
            );
    }
}
