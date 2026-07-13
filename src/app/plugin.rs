use bevy::prelude::*;

use crate::app::flow::GameFlowPlugin;
use crate::app::state::CoreStatePlugin;
use crate::shared::states::input_blocked::InputBlocked;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CoreStatePlugin)
            .add_plugins(GameFlowPlugin)
            .init_resource::<InputBlocked>();
    }
}
