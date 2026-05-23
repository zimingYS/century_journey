use bevy::prelude::*;
use bevy::window::WindowResolution;
use CenturyJourney::core::constant::{WINDOW_HEIGHT, WINDOW_WIDTH};
use CenturyJourney::test_setup::setup;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            primary_window: Some(Window{
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: "CenturyJourney".to_string(),
                name: None,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup,setup)
        .run();
}
