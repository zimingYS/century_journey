pub mod components;
pub mod systems;
pub mod texture;

use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (systems::setup_sky_system,))
            .add_systems(
                Update,
                (
                    systems::atmosphere_system,
                    systems::celestial_mesh_system,
                    systems::stars_visibility_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
