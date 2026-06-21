use bevy::prelude::*;

use crate::app::input_block::InputBlocked;
use crate::app::state::CoreStatePlugin;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CoreStatePlugin)
            .init_resource::<InputBlocked>();
    }
}
