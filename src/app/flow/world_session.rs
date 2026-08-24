//! 世界会话的进入、暂停、保存退出与错误恢复协调。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::contracts::{DialogKind, DialogState, GameSession, LoadingStatus, PendingWorld};
use crate::client::renderer::world::MeshBuildChannel;
use crate::client::ui::hud::HudRoot;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::lifecycle::ContentReloadRequested;
use crate::content::validation::ContentCompilation;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventory;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::RespawnPoint;
use crate::game::player::movement::components::PlayerAim;
use crate::game::player::survival::health::Health;
use crate::game::player::survival::hunger::Hunger;
use crate::game::player::survival::thirst::Thirst;
use crate::game::save::player::{
    PlayerSaveManager, player_backup_available, player_save_path, read_player_data, save_player_now,
};
use crate::game::save::world::metadata::io;
use crate::game::save::world::runtime::world_save::save_entire_world;
use crate::game::save::{LoadQueue, SaveConfig, SaveQueue, SaveWorker, flush_save_queue};
use crate::game::world::chunk::ChunkComponents;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::generation::{StructureGenChannel, TerrainGenChannel};
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::streaming::PlayerChunkCache;
use crate::game::world::time::WorldSimulationClock;
use crate::shared::states::{AppState, InputContextState};

/// 新世界会话进入游戏后请求重建内容消费者缓存。
pub(super) fn request_content_reload_system(
    session: Res<GameSession>,
    mut requests: MessageWriter<ContentReloadRequested>,
) {
    if session.fresh_load {
        requests.write_default();
    }
}

/// 进入暂停状态时暂停 Bevy 虚拟时间。
pub(super) fn pause_virtual_time_system(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

/// 离开暂停状态时恢复 Bevy 虚拟时间。
pub(super) fn resume_virtual_time_system(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

/// 从启动状态进入内容加载阶段并更新加载提示。
pub(super) fn enter_boot_system(
    mut next_state: ResMut<NextState<AppState>>,
    mut loading: ResMut<LoadingStatus>,
) {
    loading.title = "正在启动".into();
    loading.detail = "正在加载方块、纹理和基础资源...".into();
    next_state.set(AppState::Loading);
}

/// 进入主菜单时把内容编译错误转换为玩家可见对话框。
pub(super) fn show_content_errors_system(
    compilation: Option<Res<ContentCompilation>>,
    mut dialog: ResMut<DialogState>,
) {
    let Some(compilation) = compilation.filter(|compilation| !compilation.is_valid()) else {
        return;
    };
    dialog.error(
        "内容编译失败",
        format!(
            "发现 {} 个内容错误，已阻止进入游戏。\n{}",
            compilation.report.errors.len(),
            compilation.error_summary(12)
        ),
    );
}

/// 根据菜单输入上下文同步暂停与游戏状态。
pub(super) fn sync_pause_state_system(
    state: Res<State<AppState>>,
    context: Res<InputContextState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match state.get() {
        AppState::InGame if context.menu_open() => next_state.set(AppState::Paused),
        AppState::Paused if !context.menu_open() => next_state.set(AppState::InGame),
        _ => {}
    }
}

/// 世界装载需要一次性重置的运行时资源集合。
#[derive(SystemParam)]
pub(super) struct PrepareWorldParams<'w, 's> {
    commands: Commands<'w, 's>,
    save_config: ResMut<'w, SaveConfig>,
    world_generator: ResMut<'w, WorldGenerator>,
    climate_sampler: ResMut<'w, ClimateSampler>,
    simulation_clock: ResMut<'w, WorldSimulationClock>,
    world_state: ResMut<'w, WorldState>,
    chunk_runtime: ResMut<'w, ChunkRuntime>,
    player_cache: ResMut<'w, PlayerChunkCache>,
    terrain_channel: ResMut<'w, TerrainGenChannel>,
    structure_channel: ResMut<'w, StructureGenChannel>,
    mesh_channel: ResMut<'w, MeshBuildChannel>,
    save_queue: ResMut<'w, SaveQueue>,
    save_worker: ResMut<'w, SaveWorker>,
    load_queue: ResMut<'w, LoadQueue>,
    chunk_query: Query<'w, 's, Entity, With<ChunkComponents>>,
    session: ResMut<'w, GameSession>,
    loading: ResMut<'w, LoadingStatus>,
    dialog: ResMut<'w, DialogState>,
    next_state: ResMut<'w, NextState<AppState>>,
}

/// 读取待加载世界，并在成功后原子替换本轮世界会话资源。
///
/// 玩家或世界主存档损坏且存在备份时，本系统先返回主菜单等待确认，避免在恢复
/// 决策完成前清理当前运行时状态。
pub(super) fn prepare_world_system(pending: Res<PendingWorld>, mut params: PrepareWorldParams) {
    let Some(world_id) = pending.0.as_deref() else {
        params.dialog.error("加载失败", "没有待加载的世界");
        params.next_state.set(AppState::MainMenu);
        return;
    };
    params.loading.title = "正在加载世界".into();
    params.loading.detail = format!("正在读取 {world_id}...");
    match io::load_level(world_id) {
        Ok(level_data) => {
            let player_path = player_save_path(world_id);
            if player_path.exists() {
                if let Err(error) = read_player_data(&player_path) {
                    if player_backup_available(&player_path) {
                        params.dialog.kind = Some(DialogKind::ConfirmRecoverPlayer {
                            world_id: world_id.to_string(),
                        });
                        params.dialog.title = "玩家存档损坏".into();
                        params.dialog.message =
                            format!("玩家数据无法读取：{error}\n是否恢复最近一次有效备份？");
                    } else {
                        params
                            .dialog
                            .error("玩家存档损坏", format!("{world_id}: {error}"));
                    }
                    params.next_state.set(AppState::MainMenu);
                    return;
                }
            } else if player_backup_available(&player_path) {
                params.dialog.kind = Some(DialogKind::ConfirmRecoverPlayer {
                    world_id: world_id.to_string(),
                });
                params.dialog.title = "发现玩家存档备份".into();
                params.dialog.message = "主存档缺失，是否恢复最近一次有效备份？".into();
                params.next_state.set(AppState::MainMenu);
                return;
            }

            // 切换会话前必须让旧世界所有已接受写入完成；否则旧快照可能进入新世界的
            // 读取屏障，或在新世界开始后继续覆盖旧 Region。
            if !params.save_queue.queue.is_empty() || !params.save_worker.is_idle() {
                let previous_world = params.save_config.world_name.clone();
                if let Err(error) = flush_save_queue(
                    &previous_world,
                    &mut params.save_queue,
                    &mut params.save_worker,
                ) {
                    params
                        .dialog
                        .error("世界切换失败", format!("旧世界存档尚未完成：{error}"));
                    params.next_state.set(AppState::MainMenu);
                    return;
                }
            }
            for entity in &params.chunk_query {
                params.commands.entity(entity).despawn();
            }
            *params.world_state = WorldState::default();
            *params.chunk_runtime = ChunkRuntime::default();
            *params.player_cache = PlayerChunkCache::default();
            *params.terrain_channel = TerrainGenChannel::default();
            *params.structure_channel = StructureGenChannel::default();
            *params.mesh_channel = MeshBuildChannel::default();
            params.save_queue.queue.clear();
            *params.save_worker = SaveWorker::default();
            params.load_queue.queue.clear();
            params.save_config.world_name = world_id.to_string();
            let ore_veins = params.world_generator.pipeline.ore_veins.clone();
            let biomes = params
                .world_generator
                .pipeline
                .biome_registry
                .as_ref()
                .clone();
            *params.world_generator = WorldGenerator::with_generation_version(
                level_data.seed as u32,
                level_data.generation_version,
                biomes,
            );
            *params.climate_sampler =
                ClimateSampler::new(level_data.seed as u32, ClimateConfig::default());
            params
                .world_generator
                .pipeline
                .replace_ore_veins(ore_veins.as_ref().clone());
            *params.simulation_clock = WorldSimulationClock::from_persisted(
                level_data.simulation_tick,
                level_data.game_minute,
                level_data.subminute_tick,
            );
            params.session.fresh_load = true;
            params.session.active_world = Some(world_id.to_string());
            params.loading.detail = "正在生成出生区域...".into();
            params.next_state.set(AppState::InGame);
        }
        Err(error) => {
            if io::level_backup_available(world_id) {
                params.dialog.kind = Some(DialogKind::ConfirmRecoverWorld {
                    world_id: world_id.to_string(),
                });
                params.dialog.title = "世界存档损坏".into();
                params.dialog.message =
                    format!("世界元数据无法读取：{error}\n是否恢复最近一次有效备份？");
                params.next_state.set(AppState::MainMenu);
                return;
            }
            params
                .dialog
                .error("世界加载失败", format!("{world_id}: {error}"));
            params.next_state.set(AppState::MainMenu);
        }
    }
}

/// 保存退出需要共同读取的世界、玩家与表现层上下文。
#[derive(SystemParam)]
pub(super) struct SaveQuitParams<'w, 's> {
    commands: Commands<'w, 's>,
    save_config: Res<'w, SaveConfig>,
    world_state: Res<'w, WorldState>,
    block_registry: Option<Res<'w, BlockRegistry>>,
    world_generator: Res<'w, WorldGenerator>,
    simulation_clock: Res<'w, WorldSimulationClock>,
    save_queue: ResMut<'w, SaveQueue>,
    save_worker: ResMut<'w, SaveWorker>,
    // 玩家存档元组六元素已超 Clippy type_complexity 阈值；共享类型别名会增加模块耦合，这里仅做局部豁免。
    #[allow(clippy::type_complexity)]
    player_query: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Health,
            &'static Hunger,
            &'static Thirst,
            &'static RespawnPoint,
            &'static PlayerAim,
        ),
        With<Player>,
    >,
    gamemode: Res<'w, PlayerGameMode>,
    inventory: LocalInventory<'w, 's>,
    item_registry: Res<'w, ItemRegistry>,
    save_manager: ResMut<'w, PlayerSaveManager>,
    time: Res<'w, Time>,
    chunk_query: Query<'w, 's, Entity, With<ChunkComponents>>,
    hud_query: Query<'w, 's, Entity, With<HudRoot>>,
    dialog: ResMut<'w, DialogState>,
    session: ResMut<'w, GameSession>,
    context: ResMut<'w, InputContextState>,
    next_state: ResMut<'w, NextState<AppState>>,
}

/// 依次落盘队列、完整世界和玩家状态，全部成功后才清理会话并返回主菜单。
pub(super) fn save_and_quit_system(
    mut request: ResMut<super::contracts::SaveAndQuitRequest>,
    mut params: SaveQuitParams,
) {
    if !request.0 {
        return;
    }
    request.0 = false;
    let Some(registry) = params.block_registry.as_deref() else {
        params.dialog.error("保存失败", "方块注册表不可用");
        return;
    };
    let spawn = params
        .player_query
        .single()
        .map(|(transform, _, _, _, _, _)| transform.translation)
        .unwrap_or(Vec3::ZERO);
    if let Err(error) = flush_save_queue(
        &params.save_config.world_name,
        &mut params.save_queue,
        &mut params.save_worker,
    ) {
        params.dialog.error("保存失败", error.to_string());
        return;
    }
    if let Err(error) = save_entire_world(
        &params.save_config.world_name,
        &params.world_state,
        registry,
        params.world_generator.seed as u64,
        params.world_generator.generation_version,
        &params.simulation_clock,
        spawn,
    ) {
        params.dialog.error("保存失败", error.to_string());
        return;
    }
    if let Err(error) = save_player_now(
        &params.save_config.world_name,
        &params.gamemode,
        &params.inventory,
        &params.item_registry,
        &params.player_query,
        &mut params.save_manager,
        &params.time,
    ) {
        params.dialog.error("保存失败", error);
        return;
    }
    for entity in &params.chunk_query {
        params.commands.entity(entity).despawn();
    }
    for entity in &params.hud_query {
        params.commands.entity(entity).despawn();
    }
    params.session.active_world = None;
    params.session.fresh_load = false;
    params.context.set_menu_open(false);
    params.next_state.set(AppState::MainMenu);
}

/// 内容刷新请求发出后结束“刚加载”标记，避免后续帧重复触发。
pub(super) fn finish_fresh_session_system(
    state: Res<State<AppState>>,
    mut session: ResMut<GameSession>,
) {
    if *state.get() == AppState::InGame && session.fresh_load {
        session.fresh_load = false;
    }
}

/// 进入游戏世界时记录会话开始。
pub(super) fn log_enter_world_system(session: Res<GameSession>) {
    let name = session.active_world.as_deref().unwrap_or("未知世界");
    log::info!("[世界] 已进入世界：{name}");
}

/// 离开游戏世界（返回主菜单或退出应用）时记录会话结束。
pub(super) fn log_exit_world_system() {
    log::info!("[世界] 已退出世界");
}
