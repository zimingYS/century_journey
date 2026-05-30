use bevy::prelude::*;
use bevy::window::WindowResolution;
use CenturyJourney::core::constant::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use CenturyJourney::core::state::AppState;
use CenturyJourney::player::PlayerPlugin;
use CenturyJourney::test_setup::setup;
use CenturyJourney::ui::UIPlugin;
use CenturyJourney::voxel::VoxelPlugin;
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
        .init_state::<AppState>()
        .add_plugins((
            VoxelPlugin,
            PlayerPlugin,
            WorldPlugin,
            UIPlugin,
        ))
        .add_systems(Startup,setup)
        .run();
}
