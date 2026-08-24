//! 调试浮层的节点生成、开关切换与每帧文本刷新。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::app::settings::{KeyAction, Keybinds};
use crate::client::camera::FpsCamera;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::game::gameplay::rules::GameRules;
use crate::game::player::identity::LocalPlayer;
use crate::game::world::chunk::ChunkState;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::state::WorldState;
use crate::game::world::streaming::{PlayerChunkCache, WorldStreamingConfig};
use crate::game::world::time::WorldSimulationClock;
use crate::shared::states::app_state::AppState;
use crate::shared::states::input_context::InputContextState;
use crate::shared::voxel::CHUNK_SIZE;

use super::components::{DebugOverlayRoot, DebugOverlayState, DebugOverlayText};
use super::info::{self, ChunkCounts, ChunkInfo, ClockInfo, DebugOverlayData, FacingInfo};

/// 调试浮层单帧文本字号。
const DEBUG_OVERLAY_FONT_SIZE: f32 = 16.0;

/// 在 Startup 生成常驻的调试浮层节点，默认隐藏。
pub fn spawn_debug_overlay_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            DebugOverlayRoot,
            Name::new("DebugOverlayRoot"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                top: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                DebugOverlayText,
                Name::new("DebugOverlayText"),
                Text::default(),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(DEBUG_OVERLAY_FONT_SIZE),
                    ..default()
                },
                TextColor(theme.text_primary),
            ));
        });
}

/// 游戏玩法上下文中按调试浮层键切换开关；与骨架调试的按键门控语义一致。
pub fn toggle_debug_overlay_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keybinds: Res<Keybinds>,
    context: Res<InputContextState>,
    mut state: ResMut<DebugOverlayState>,
) {
    if context.active().allows_gameplay()
        && keybinds.is_just_pressed(KeyAction::ToggleDebugOverlay, &keyboard, &mouse)
    {
        state.visible = !state.visible;
    }
}

/// 同步浮层显隐：开关打开且处于游戏内（含暂停）时可见。
pub fn sync_debug_overlay_visibility_system(
    app_state: Res<State<AppState>>,
    state: Res<DebugOverlayState>,
    mut visibility_query: Query<&mut Visibility, With<DebugOverlayRoot>>,
) {
    let Ok(mut visibility) = visibility_query.single_mut() else {
        return;
    };
    let in_world = matches!(app_state.get(), AppState::InGame | AppState::Paused);
    *visibility = if state.visible && in_world {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// 调试浮层刷新所需的世界与玩家数据源；可选资源在主菜单等阶段缺失。
#[derive(SystemParam)]
pub struct DebugOverlaySources<'w, 's> {
    /// 本地玩家权威变换，坐标与朝向来源。
    player: Query<'w, 's, &'static Transform, With<LocalPlayer>>,
    /// 本地相机，俯仰角来源。
    camera: Query<'w, 's, &'static FpsCamera>,
    /// 世界权威时钟；进入世界前缺失。
    clock: Option<Res<'w, WorldSimulationClock>>,
    /// 会话规则，时间倍率来源。
    rules: Option<Res<'w, GameRules>>,
    /// 世界生成器，种子来源。
    generator: Option<Res<'w, WorldGenerator>>,
    /// 世界状态，已加载区块计数来源。
    world_state: Option<Res<'w, WorldState>>,
    /// 区块流送缓存，预期区块计数来源。
    player_cache: Option<Res<'w, PlayerChunkCache>>,
    /// 区块实体状态，渲染态计数来源。
    chunk_states: Query<'w, 's, &'static ChunkState>,
}

/// 每帧维护帧率滑动平均并重写浮层文本；浮层隐藏时只更新平均不写文本。
pub fn update_debug_overlay_text_system(
    time: Res<Time>,
    mut state: ResMut<DebugOverlayState>,
    sources: DebugOverlaySources,
    mut text_query: Query<&mut Text, With<DebugOverlayText>>,
) {
    let delta = time.delta_secs();
    if delta > 0.0 {
        // 指数滑动平均抑制逐帧抖动；钳制避免异常长帧污染平均。
        let frame_ms = (delta * 1000.0).min(1000.0);
        let fps = (1.0 / delta).min(1000.0);
        state.fps_ema = smooth(state.fps_ema, fps);
        state.frame_ms_ema = smooth(state.frame_ms_ema, frame_ms);
    }
    if !state.visible {
        return;
    }
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    let data = collect_overlay_data(&state, &sources);
    text.0 = info::build_lines(&data);
}

/// 指数滑动平均；首次采样直接采用当前值。
fn smooth(current: f32, sample: f32) -> f32 {
    if current <= 0.0 {
        sample
    } else {
        current * 0.9 + sample * 0.1
    }
}

/// 从数据源采集一帧调试数据；玩家或资源缺失的字段保持 None。
fn collect_overlay_data(
    state: &DebugOverlayState,
    sources: &DebugOverlaySources,
) -> DebugOverlayData {
    let player = sources.player.single().ok();
    let position = player.map(|transform| transform.translation);

    let facing = player
        .zip(sources.camera.single().ok())
        .map(|(transform, camera)| FacingInfo {
            direction: info::compass_direction(*transform.forward()),
            yaw_deg: transform.rotation.to_euler(EulerRot::YXZ).0.to_degrees(),
            pitch_deg: camera.pitch.to_degrees(),
        });

    // 区块坐标取地板除；局部坐标用同一地板值回减，正确处理负坐标。
    let chunk = position.map(|position| {
        let chunk = WorldStreamingConfig::chunk_from_world(position);
        let floor = IVec3::new(
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        );
        ChunkInfo {
            chunk,
            local: floor - chunk * CHUNK_SIZE as i32,
        }
    });

    let clock = sources.clock.as_deref().map(|clock| {
        let snapshot = clock.snapshot();
        ClockInfo {
            game_day: snapshot.game_day,
            hour: snapshot.hour,
            minute: snapshot.minute,
            season: info::season_label(snapshot.season),
            time_scale: sources
                .rules
                .as_deref()
                .map(|rules| rules.time_scale)
                .unwrap_or(1.0),
        }
    });

    let chunk_counts =
        (sources.world_state.is_some() || sources.player_cache.is_some()).then(|| ChunkCounts {
            loaded: sources
                .world_state
                .as_deref()
                .map(|world| world.loaded_chunk_count())
                .unwrap_or(0),
            expected: sources
                .player_cache
                .as_deref()
                .map(|cache| cache.expected_chunk_count())
                .unwrap_or(0),
            rendered: sources
                .chunk_states
                .iter()
                .filter(|chunk_state| **chunk_state == ChunkState::Rendered)
                .count(),
        });

    DebugOverlayData {
        fps: state.fps_ema,
        frame_ms: state.frame_ms_ema,
        position,
        facing,
        chunk,
        clock,
        seed: sources.generator.as_deref().map(|generator| generator.seed),
        chunk_counts,
        simulation_tick: sources
            .clock
            .as_deref()
            .map(|clock| clock.simulation_tick()),
    }
}
