use bevy::app::App;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

use crate::app::plugin::CorePlugin;
use crate::app::state::AppState;
use crate::engine::constant::window::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory;
use crate::content::loot::LootPlugin;
use crate::game::player::PlayerPlugin;
use crate::client::renderer::RenderingPlugin;
use crate::shared::tag::TagPlugin;
use crate::tests::setup::setup;
use crate::client::ui::UIPlugin;
use crate::content::block::VoxelPlugin;
use crate::game::world::WorldPlugin;

pub fn launch() -> anyhow::Result<()> {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
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
            RenderingPlugin,
            TagPlugin,
            PlayerPlugin,
            WorldPlugin,
            UIPlugin,
        ))
        .init_resource::<inventory::item::registry::ItemRegistry>()
        .add_systems(Startup, (inventory::item::texture_registry::load_item_textures_system,))
        .add_systems(
            OnEnter(AppState::InGame),
            (
                inventory::item::registry::auto_generate_block_items_system,
                inventory::item::registry::load_item_definitions_system,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        .run();

    Ok(())
}
