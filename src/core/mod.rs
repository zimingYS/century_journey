pub mod constant;
pub mod state;
pub mod input_block;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin{
    fn build(&self, app: &mut App) { app
        .add_plugins(state::CoreStatePlugin)
        .init_resource::<input_block::InputBlocked>();
    }
}