use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::app::flow::{MenuPage, PendingWorld};
use crate::client::ui::components::SurvivalInventoryRoot;
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::screens::menu::{PauseSettingsButton, ResumeButton, SaveQuitButton};
use crate::content::block::registry::BlockRegistry;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::world::save::level;
use crate::shared::states::AppState;

const FRAMES_BEFORE_CAPTURE: u32 = 30;
const SECONDS_BEFORE_CAPTURE: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenshotTarget {
    MainMenu,
    Settings,
    Pause,
    Inventory,
}

#[derive(Resource, Debug)]
struct UiScreenshotCheck {
    output: PathBuf,
    mode: GameMode,
    target: ScreenshotTarget,
    world_requested: bool,
    in_game_frames: u32,
    prepared: bool,
    ready_frames: u32,
    ready_seconds: f32,
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
    let target = match std::env::var("CJ_UI_SCREENSHOT_TARGET")
        .unwrap_or_else(|_| "inventory".to_string())
        .as_str()
    {
        "main-menu" => ScreenshotTarget::MainMenu,
        "settings" => ScreenshotTarget::Settings,
        "pause" => ScreenshotTarget::Pause,
        _ => ScreenshotTarget::Inventory,
    };
    app.insert_resource(UiScreenshotCheck {
        output: PathBuf::from(output),
        mode,
        target,
        world_requested: false,
        in_game_frames: 0,
        prepared: false,
        ready_frames: 0,
        ready_seconds: 0.0,
        requested: false,
    })
    .add_systems(Update, ui_screenshot_check_system);
}

fn ui_screenshot_check_system(
    real_time: Res<Time<Real>>,
    app_state: Res<State<AppState>>,
    config: Option<ResMut<UiScreenshotCheck>>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut navigation: MessageWriter<UiNavigation>,
    mut pending_world: ResMut<PendingWorld>,
    block_registry: Option<Res<BlockRegistry>>,
    mut menu_page: ResMut<MenuPage>,
    mut next_state: ResMut<NextState<AppState>>,
    pause_controls: Query<
        (&ComputedNode, &InheritedVisibility),
        Or<(
            With<ResumeButton>,
            With<PauseSettingsButton>,
            With<SaveQuitButton>,
        )>,
    >,
    survival_inventory: Query<(&ComputedNode, &InheritedVisibility), With<SurvivalInventoryRoot>>,
    mut commands: Commands,
) {
    let Some(mut config) = config else {
        return;
    };
    if config.requested {
        return;
    }
    let state = app_state.get();
    if state == &AppState::MainMenu
        && matches!(
            config.target,
            ScreenshotTarget::MainMenu | ScreenshotTarget::Settings
        )
    {
        *menu_page = if config.target == ScreenshotTarget::Settings {
            MenuPage::Settings
        } else {
            MenuPage::Worlds
        };
        if !config.prepared {
            config.prepared = true;
            return;
        }
    } else if state == &AppState::MainMenu && !config.world_requested {
        const SCREENSHOT_WORLD: &str = "__ui_screenshot";
        if !level::world_exists(SCREENSHOT_WORLD) {
            let Some(block_registry) = block_registry else {
                return;
            };
            if let Err(error) = level::save_level(
                SCREENSHOT_WORLD,
                12_345,
                Vec3::new(0.0, 70.0, 0.0),
                0.25,
                &block_registry,
            ) {
                error!("创建 UI 截图测试世界失败: {error}");
                return;
            }
        }
        pending_world.0 = Some(SCREENSHOT_WORLD.into());
        config.world_requested = true;
        next_state.set(AppState::WorldLoading);
        return;
    }
    if state == &AppState::InGame && !config.prepared {
        if config.target == ScreenshotTarget::Pause && config.in_game_frames < FRAMES_BEFORE_CAPTURE
        {
            config.in_game_frames += 1;
            return;
        }
        gamemode.mode = config.mode;
        match config.target {
            ScreenshotTarget::Inventory => {
                navigation.write(UiNavigation::Open(UiScreen::Inventory));
            }
            ScreenshotTarget::Pause => {
                navigation.write(UiNavigation::Open(UiScreen::PauseMenu));
            }
            ScreenshotTarget::MainMenu | ScreenshotTarget::Settings => {}
        }
        config.prepared = true;
        return;
    }
    let ready = match config.target {
        ScreenshotTarget::MainMenu | ScreenshotTarget::Settings => state == &AppState::MainMenu,
        ScreenshotTarget::Pause => state == &AppState::Paused,
        ScreenshotTarget::Inventory => {
            state == &AppState::InGame
                && survival_inventory.single().is_ok_and(|(node, inherited)| {
                    node.size().min_element() > 0.0 && inherited.get()
                })
        }
    };
    if !ready || !config.prepared {
        config.ready_frames = 0;
        config.ready_seconds = 0.0;
        return;
    }
    if config.target == ScreenshotTarget::Pause
        && (pause_controls.iter().count() != 3
            || pause_controls
                .iter()
                .any(|(node, inherited)| node.size().min_element() <= 0.0 || !inherited.get()))
    {
        config.ready_frames = 0;
        config.ready_seconds = 0.0;
        return;
    }
    config.ready_frames += 1;
    config.ready_seconds += real_time.delta_secs();
    if config.ready_frames < FRAMES_BEFORE_CAPTURE || config.ready_seconds < SECONDS_BEFORE_CAPTURE
    {
        return;
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(config.output.clone()));
    config.requested = true;
}
