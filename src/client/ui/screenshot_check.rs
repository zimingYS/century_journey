use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::shared::states::AppState;

const FRAMES_BEFORE_CAPTURE: u32 = 30;

#[derive(Resource, Debug)]
struct UiScreenshotCheck {
    output: PathBuf,
    mode: GameMode,
    prepared: bool,
    ready_frames: u32,
    requested: bool,
}

pub fn configure_ui_screenshot_check(app: &mut App) {
    let Ok(output) = std::env::var("CJ_UI_SCREENSHOT") else {
        return;
    };
    let mode = match std::env::var("CJ_UI_SCREENSHOT_MODE")
        .unwrap_or_else(|_| "survival".to_string())
        .as_str()
    {
        "creative" => GameMode::Creative,
        _ => GameMode::Survival,
    };
    app.insert_resource(UiScreenshotCheck {
        output: PathBuf::from(output),
        mode,
        prepared: false,
        ready_frames: 0,
        requested: false,
    })
    .add_systems(Update, ui_screenshot_check_system);
}

fn ui_screenshot_check_system(
    app_state: Res<State<AppState>>,
    config: Option<ResMut<UiScreenshotCheck>>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut navigation: MessageWriter<UiNavigation>,
    mut commands: Commands,
) {
    let Some(mut config) = config else {
        return;
    };
    if *app_state.get() != AppState::InGame || config.requested {
        return;
    }
    if !config.prepared {
        gamemode.mode = config.mode;
        navigation.write(UiNavigation::Open(UiScreen::Inventory));
        config.prepared = true;
        return;
    }
    config.ready_frames += 1;
    if config.ready_frames < FRAMES_BEFORE_CAPTURE {
        return;
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(config.output.clone()));
    config.requested = true;
}
