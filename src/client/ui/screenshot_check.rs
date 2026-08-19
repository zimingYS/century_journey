//! 驱动固定尺寸界面截图场景，供开发期视觉回归检查使用。

use std::path::PathBuf;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::app::flow::{MenuPage, PendingWorld};
use crate::client::camera::{CameraPerspective, FpsCamera};
use crate::client::ui::components::SurvivalInventoryRoot;
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::screens::menu::{PauseSettingsButton, ResumeButton, SaveQuitButton};
use crate::content::block::registry::BlockRegistry;
use crate::game::crafting::grid::ActiveCrafting;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::item::stack::{ItemInstanceData, ItemStack};
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::LocalPlayer;
use crate::game::player::movement::components::PlayerVelocity;
use crate::game::player::physics::components::PlayerGravity;
use crate::game::save::world::metadata::io;
use crate::game::world::time::WorldSimulationClock;
use crate::shared::item_id::ItemId;
use crate::shared::states::AppState;

const FRAMES_BEFORE_CAPTURE: u32 = 30;
const SECONDS_BEFORE_CAPTURE: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenshotTarget {
    MainMenu,
    Settings,
    Pause,
    Inventory,
    Workbench,
    SecondPerson,
    ThirdPerson,
    SkyView,
}

#[derive(Resource, Debug)]
struct UiScreenshotCheck {
    output: PathBuf,
    mode: GameMode,
    target: ScreenshotTarget,
    world_id: String,
    anchor_player: bool,
    world_requested: bool,
    in_game_frames: u32,
    prepared: bool,
    ready_frames: u32,
    ready_seconds: f32,
    capture_delay_seconds: f32,
    requested: bool,
    /// 截图请求后的退出倒计时（帧）：给渲染世界足够时间完成截图并写盘。
    exit_frames: u32,
}

/// 根据截图环境变量配置固定界面状态、相机和自动退出流程。
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
        "workbench" => ScreenshotTarget::Workbench,
        "second-person" => ScreenshotTarget::SecondPerson,
        "third-person" => ScreenshotTarget::ThirdPerson,
        "sky-view" => ScreenshotTarget::SkyView,
        _ => ScreenshotTarget::Inventory,
    };
    let capture_delay_seconds = std::env::var("CJ_UI_SCREENSHOT_DELAY_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(SECONDS_BEFORE_CAPTURE)
        .clamp(0.5, 30.0);
    let requested_world = std::env::var("CJ_UI_SCREENSHOT_WORLD")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let anchor_player = requested_world.is_none();
    let world_id = requested_world.unwrap_or_else(|| "__ui_screenshot".to_string());
    app.insert_resource(UiScreenshotCheck {
        output: PathBuf::from(output),
        mode,
        target,
        world_id,
        anchor_player,
        world_requested: false,
        in_game_frames: 0,
        prepared: false,
        ready_frames: 0,
        ready_seconds: 0.0,
        capture_delay_seconds,
        requested: false,
        exit_frames: 0,
    })
    .add_systems(Update, ui_screenshot_check_system);
}

/// 截图驱动所需的应用流程控制参数聚合，避免 system 参数超过 Bevy 上限。
#[derive(SystemParam)]
struct ScreenshotFlow<'w, 's> {
    navigation: MessageWriter<'w, UiNavigation>,
    pending_world: ResMut<'w, PendingWorld>,
    menu_page: ResMut<'w, MenuPage>,
    next_state: ResMut<'w, NextState<AppState>>,
    commands: Commands<'w, 's>,
    exit_writer: MessageWriter<'w, AppExit>,
}

// 截图工具在同一帧检查多个界面锚点和状态，参数只存在于开发期验证路径。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn ui_screenshot_check_system(
    real_time: Res<Time<Real>>,
    app_state: Res<State<AppState>>,
    config: Option<ResMut<UiScreenshotCheck>>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut player_query: Query<
        (
            &mut InventoryState,
            &mut ActiveCrafting,
            &mut Transform,
            &mut PlayerGravity,
            &mut PlayerVelocity,
        ),
        With<LocalPlayer>,
    >,
    mut containers: ResMut<WorldContainers>,
    mut camera: Query<&mut FpsCamera, With<Camera3d>>,
    mut clock: ResMut<WorldSimulationClock>,
    mut flow: ScreenshotFlow,
    block_registry: Option<Res<BlockRegistry>>,
    pause_controls: Query<
        (&ComputedNode, &InheritedVisibility),
        Or<(
            With<ResumeButton>,
            With<PauseSettingsButton>,
            With<SaveQuitButton>,
        )>,
    >,
    survival_inventory: Query<(&ComputedNode, &InheritedVisibility), With<SurvivalInventoryRoot>>,
) {
    let Some(mut config) = config else {
        return;
    };
    if config.requested {
        // 截图已请求：给渲染世界留出完成截图写盘的时间后再退出。
        if std::env::var("CJ_UI_SCREENSHOT_EXIT").as_deref() == Ok("1") {
            config.exit_frames += 1;
            if config.exit_frames >= 90 {
                flow.exit_writer.write(AppExit::Success);
            }
        }
        return;
    }
    let state = app_state.get();
    if config.anchor_player
        && state == &AppState::InGame
        && let Ok((_, _, mut transform, mut gravity, mut velocity)) = player_query.single_mut()
    {
        // 锚定在海平面上方：SEA_LEVEL=64，脚底约 69，离水 5m，不在水中也不被地形遮挡。
        transform.translation = Vec3::new(0.0, 70.0, 0.0);
        gravity.velocity_y = 0.0;
        gravity.fall_distance = 0.0;
        velocity.horizontal = Vec3::ZERO;
    }
    if state == &AppState::MainMenu
        && matches!(
            config.target,
            ScreenshotTarget::MainMenu | ScreenshotTarget::Settings
        )
    {
        *flow.menu_page = if config.target == ScreenshotTarget::Settings {
            MenuPage::Settings
        } else {
            MenuPage::Worlds
        };
        if !config.prepared {
            config.prepared = true;
            return;
        }
    } else if state == &AppState::MainMenu && !config.world_requested {
        let screenshot_world = config.world_id.clone();
        if !io::world_exists(&screenshot_world) {
            if !config.anchor_player {
                error!("requested screenshot world does not exist: {screenshot_world}");
                config.requested = true;
                return;
            }
            let Some(block_registry) = block_registry else {
                return;
            };
            if let Err(error) = io::save_level(
                &screenshot_world,
                12_345,
                crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION,
                &crate::game::world::time::WorldSimulationClock::default(),
                Vec3::new(0.0, 70.0, 0.0),
                &block_registry,
            ) {
                error!("创建 UI 截图测试世界失败: {error}");
                return;
            }
        }
        flow.pending_world.0 = Some(screenshot_world);
        config.world_requested = true;
        flow.next_state.set(AppState::WorldLoading);
        return;
    }
    if state == &AppState::InGame && !config.prepared {
        if config.in_game_frames < FRAMES_BEFORE_CAPTURE {
            config.in_game_frames += 1;
            return;
        }
        gamemode.mode = config.mode;
        let Ok((mut inventory, mut active_crafting, _, _, _)) = player_query.single_mut() else {
            return;
        };
        if matches!(
            config.target,
            ScreenshotTarget::Inventory | ScreenshotTarget::Workbench
        ) {
            inventory.hotbar.set_stack(
                0,
                ItemStack::with_instance(
                    ItemId::item("century_journey:wooden_axe"),
                    1,
                    ItemInstanceData {
                        durability: Some(18),
                    },
                ),
            );
        }
        match config.target {
            ScreenshotTarget::Inventory => {
                flow.navigation
                    .write(UiNavigation::Open(UiScreen::Inventory));
            }
            ScreenshotTarget::Workbench => {
                let Some(container_id) = containers.ensure_at(
                    IVec3::ZERO,
                    crate::game::inventory::container::ContainerKind::Workbench,
                ) else {
                    return;
                };
                *active_crafting = ActiveCrafting::workbench(IVec3::ZERO, container_id);
                flow.navigation
                    .write(UiNavigation::Open(UiScreen::Container));
            }
            ScreenshotTarget::SecondPerson => {
                if let Ok(mut camera) = camera.single_mut() {
                    camera.perspective = CameraPerspective::SecondPerson;
                    camera.set_pitch(-0.12);
                }
            }
            ScreenshotTarget::ThirdPerson => {
                if let Ok(mut camera) = camera.single_mut() {
                    camera.perspective = CameraPerspective::ThirdPerson;
                    camera.set_pitch(-0.12);
                }
            }
            ScreenshotTarget::SkyView => {
                if let Ok(mut camera) = camera.single_mut() {
                    camera.perspective = CameraPerspective::FirstPerson;
                    // 仰头约 50°：让近处大型云团（仰角约 40°~80°）落在画面中央。
                    camera.set_pitch(0.88);
                }
                // 正午：太阳直射云顶，底面由自发光保持冷灰亮色，光影最接近参考图。
                *clock = WorldSimulationClock::from_legacy_time_of_day(12.0);
            }
            ScreenshotTarget::Pause => {
                flow.navigation
                    .write(UiNavigation::Open(UiScreen::PauseMenu));
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
        ScreenshotTarget::Workbench => state == &AppState::InGame,
        ScreenshotTarget::SecondPerson
        | ScreenshotTarget::ThirdPerson
        | ScreenshotTarget::SkyView => state == &AppState::InGame,
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
    if config.ready_frames < FRAMES_BEFORE_CAPTURE
        || config.ready_seconds < config.capture_delay_seconds
    {
        return;
    }
    flow.commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(config.output.clone()));
    config.requested = true;
}
