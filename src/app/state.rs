use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
