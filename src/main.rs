use bevy::prelude::*;
use bevy::window::WindowResolution;
use CenturyJourney::core::constant::window::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use CenturyJourney::core::CorePlugin;
use CenturyJourney::core::state::app_state::AppState;
use CenturyJourney::gameplay::GameplayPlugin;
use CenturyJourney::inventory;
use CenturyJourney::loot::LootPlugin;
use CenturyJourney::player::PlayerPlugin;
use CenturyJourney::tag::TagPlugin;
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
        .add_plugins((
            CorePlugin,
            GameplayPlugin,
            LootPlugin,
            VoxelPlugin,
            TagPlugin,
            PlayerPlugin,
            WorldPlugin,
            UIPlugin,
        ))
        .init_resource::<inventory::item::registry::ItemRegistry>()
        .add_systems(Startup, (
            inventory::item::texture_registry::load_item_textures_system,
        ))
        .add_systems(OnEnter(AppState::InGame), (
            inventory::item::registry::auto_generate_block_items_system,
            inventory::item::registry::load_item_definitions_system,
        ).chain())
        .add_systems(Startup,setup)
        .run();
}
