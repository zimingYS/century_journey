//! 光照截图自检：`--light-check` 自动进入首个存档并截图退出。
//!
//! 用于光照修复的截图自检循环：启动时检测命令行参数 `--light-check`，
//! 自动选择首个世界进入，等待网格构建完成后对相机画面截图保存到
//! `outputs/screenshots/light_check.png`，随后退出。配合外部像素分析脚本
//! 判断光照效果，避免每次修改都全量跑测试。

use std::path::PathBuf;

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowRef;

use crate::app::flow::FlowCommand;
use crate::client::camera::FpsCamera;
use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::Player;
use crate::game::world::chunk::ChunkComponents;
use crate::game::world::lighting::WorldLighting;
use crate::game::world::state::WorldState;
use crate::shared::states::AppState;
use crate::shared::voxel::{CHUNK_SIZE, CHUNK_VOLUME};

/// 进入游戏后等待网格构建的渲染帧数（约 20 秒 @60fps）。
const MESH_WAIT_FRAMES: u64 = 1200;

/// `--light-check` 自检流程状态。
#[derive(Resource, Default)]
struct LightCheckState {
    /// 是否已发送进入世界命令。
    world_selected: bool,
    /// 是否已把玩家传送到陆地。
    teleported: bool,
    /// 是否已触发截图。
    screenshot_queued: bool,
    /// 进入游戏后的渲染帧计数。
    in_game_frames: u64,
}

/// 光照截图自检插件：仅 `--light-check` 参数存在时激活。
pub struct LightCheckPlugin;

impl Plugin for LightCheckPlugin {
    fn build(&self, app: &mut App) {
        if !std::env::args().any(|arg| arg == "--light-check") {
            return;
        }
        info!("光照自检模式：自动进入首个存档并截图");
        let screenshot_path = PathBuf::from("outputs/screenshots/light_check.png");
        if let Some(parent) = screenshot_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        app.init_resource::<LightCheckState>()
            .add_observer(save_to_disk(screenshot_path))
            .add_systems(
                Update,
                light_check_system
                    .run_if(in_state(AppState::MainMenu).or_else(in_state(AppState::InGame))),
            )
            .add_systems(Update, exit_after_screenshot);
    }
}

/// 驱动自检流程：主菜单选世界 → 传送到陆地 → 游戏中定时截图。
#[allow(clippy::too_many_arguments)]
fn light_check_system(
    mut state: ResMut<LightCheckState>,
    app_state: Res<State<AppState>>,
    mut commands: Commands,
    mut flow: MessageWriter<FlowCommand>,
    camera_query: Query<Entity, (With<Camera3d>, With<FpsCamera>)>,
    mut player_query: Query<&mut Transform, With<Player>>,
    chunk_query: Query<&ChunkComponents>,
    world_state: Res<WorldState>,
    registry: Option<Res<BlockRegistry>>,
    lighting: Option<Res<WorldLighting>>,
) {
    match *app_state.get() {
        AppState::MainMenu => {
            if !state.world_selected {
                if let Some(world_id) = first_world_id() {
                    info!("自检进入世界：{world_id}");
                    flow.write(FlowCommand::SelectWorld(world_id));
                    flow.write(FlowCommand::PlaySelected);
                    state.world_selected = true;
                } else {
                    warn!("未找到任何存档，跳过自检");
                }
            }
        }
        AppState::InGame => {
            // 出生点可能在海里：先扫描已加载区块，把玩家传送到最近的陆地上方。
            if !state.teleported {
                let water_id = registry
                    .as_deref()
                    .and_then(|r| r.get_id_by_identifier("century_journey:water"))
                    .unwrap_or(0);
                if let Some(land) = find_land_pos(&world_state, &chunk_query, water_id)
                    && let Ok(mut transform) = player_query.single_mut()
                {
                    transform.translation = land + Vec3::Y * 4.0;
                    transform.rotation = Quat::from_rotation_y(0.0);
                    state.teleported = true;
                    state.in_game_frames = 0;
                    info!("已把玩家传送到陆地：{land:?}");
                }
                return;
            }

            state.in_game_frames += 1;
            if state.in_game_frames >= MESH_WAIT_FRAMES && !state.screenshot_queued {
                print_light_stats(lighting.as_deref());
                if let Ok(camera) = camera_query.single() {
                    commands
                        .entity(camera)
                        .insert(Screenshot(RenderTarget::Window(WindowRef::Primary)));
                    state.screenshot_queued = true;
                    info!("已触发光照截图");
                } else {
                    state.in_game_frames = 0; // 相机未就绪，重新计时
                }
            }
        }
        _ => {}
    }
}

/// 扫描已加载区块，返回第一个「非空气非水」方块地表上方一格的坐标。
///
/// 从每个区块顶面向下扫描，取最先遇到的陆地列，保证自检截图能看到地表。
fn find_land_pos(
    world_state: &WorldState,
    chunk_query: &Query<&ChunkComponents>,
    water_id: u16,
) -> Option<Vec3> {
    for components in chunk_query.iter() {
        let chunk_pos = components.position;
        let data = world_state.chunk(chunk_pos)?;
        let base = chunk_pos * CHUNK_SIZE as i32;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in (0..CHUNK_SIZE).rev() {
                    let id = data.get_voxel(x, y, z);
                    if id != 0 && id != water_id {
                        return Some(Vec3::new(
                            (base.x + x as i32) as f32 + 0.5,
                            (base.y + y as i32 + 1) as f32 + 0.5,
                            (base.z + z as i32) as f32 + 0.5,
                        ));
                    }
                }
            }
        }
    }
    None
}

/// 打印权威光数据的汇总统计，用于区分光数据错误与渲染错误。
fn print_light_stats(lighting: Option<&WorldLighting>) {
    let Some(lighting) = lighting else {
        warn!("WorldLighting 资源不存在");
        return;
    };
    let chunks = lighting.chunk_lights.len();
    if chunks == 0 {
        warn!("WorldLighting 为空：没有任何区块有光数据");
        return;
    }
    let mut lit_sky = 0u64;
    let mut lit_block = 0u64;
    let mut sum_sky = 0u64;
    let total = chunks as u64 * CHUNK_VOLUME as u64;
    for chunk in lighting.chunk_lights.values() {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let cell = chunk.get(x, y, z);
                    sum_sky +=
                        (u64::from(cell.sky.r) + u64::from(cell.sky.g) + u64::from(cell.sky.b)) / 3;
                    if !cell.sky.is_dark() {
                        lit_sky += 1;
                    }
                    if !cell.block.is_dark() {
                        lit_block += 1;
                    }
                }
            }
        }
    }
    info!(
        "光数据统计: 区块数={chunks}, 天空光非零={:.1}%, 方块光非零={:.1}%, 平均天空光={:.1}",
        lit_sky as f64 / total as f64 * 100.0,
        lit_block as f64 / total as f64 * 100.0,
        sum_sky as f64 / total as f64,
    );
}

/// 截图文件已落盘后退出。
fn exit_after_screenshot(mut app_exit: MessageWriter<AppExit>) {
    if std::path::Path::new("outputs/screenshots/light_check.png").exists() {
        info!("光照截图已保存，退出");
        app_exit.write(AppExit::Success);
    }
}

/// 扫描存档目录，返回字典序第一个世界 ID。
fn first_world_id() -> Option<String> {
    let mut entries: Vec<String> = std::fs::read_dir("saves")
        .ok()?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    entries.into_iter().next()
}
