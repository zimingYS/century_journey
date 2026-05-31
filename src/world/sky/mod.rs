pub mod component;
pub mod system;

use bevy::prelude::*;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) { app
        .add_systems(Startup,(
            system::setup_sky_system,
        ))
        .add_systems(Update,(
            system::atmosphere_system,
            system::setup_player_camera_system,
        ));
    }
}