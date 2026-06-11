pub mod app_state;

use bevy::prelude::*;
pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin{
    fn build(&self, app: &mut App) { app
        .init_state::<app_state::AppState>();
    }
}
