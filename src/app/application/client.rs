use super::application::Application;
use crate::app::config::AppConfig;
use crate::client::plugin_group::ClientPluginGroup;
use crate::engine::constant::window::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::game::world::systems::WorldStreamingConfig;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowPosition, WindowResolution};

const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=warn";

pub struct ClientApplication;

impl Application for ClientApplication {
    fn build(config: AppConfig) -> anyhow::Result<App> {
        let world_streaming_config = WorldStreamingConfig::from_render_config(&config.render);
        let screenshot_mode = std::env::var_os("CJ_UI_SCREENSHOT").is_some();
        let screenshot_width = std::env::var("CJ_UI_SCREENSHOT_WIDTH")
            .ok()
            .and_then(|value| value.parse::<u32>().ok());
        let screenshot_height = std::env::var("CJ_UI_SCREENSHOT_HEIGHT")
            .ok()
            .and_then(|value| value.parse::<u32>().ok());
        let window_resolution = match (screenshot_width, screenshot_height) {
            (Some(width), Some(height)) if width >= 640 && height >= 360 => {
                WindowResolution::new(width, height).with_scale_factor_override(1.0)
            }
            _ => WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        };
        let window_position = if screenshot_mode {
            WindowPosition::At(IVec2::ZERO)
        } else {
            WindowPosition::Automatic
        };
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: window_resolution,
                        title: WINDOW_TITLE.to_string(),
                        name: None,
                        position: window_position,
                        decorations: !screenshot_mode,
                        resizable: !screenshot_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: format!("{DEFAULT_LOG_FILTER},icu_provider=error"),
                    ..default()
                }),
        );
        app.insert_resource(world_streaming_config)
            .add_plugins(ClientPluginGroup);
        Ok(app)
    }
}
