pub mod components;
pub mod systems;

use bevy::prelude::*;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) { app
        .add_systems(Startup,(
            systems::setup_sky_system,
        ))
        .add_systems(Update,(
            systems::atmosphere_system,
        ));
    }
}