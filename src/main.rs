use bevy::prelude::*;
use bevy::window::WindowResolution;
use CenturyJourney::core::constant::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use CenturyJourney::player::PlayerPlugin;
use CenturyJourney::test_setup::setup;
use CenturyJourney::world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            primary_window: Some(Window{
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: WINDOW_TITLE.to_string(),
                name: None,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            PlayerPlugin,
            WorldPlugin,
        ))
        .add_systems(Startup,setup)
        .run();
}
