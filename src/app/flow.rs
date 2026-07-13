use std::time::{SystemTime, UNIX_EPOCH};

use bevy::audio::{GlobalVolume, Volume};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};

use crate::client::renderer::world::MeshBuildChannel;
use crate::client::ui::hud::HudRoot;
use crate::client::ui::theme::scale::UiScaleSettings;
use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadRequested, ContentReloadSet};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::Player;
use crate::game::world::chunk::ChunkComponents;
use crate::game::world::generation::WorldGenerator;
use crate::game::world::save::level;
use crate::game::world::save::player::{PlayerSaveManager, save_player_now};
use crate::game::world::save::region::RegionManager;
use crate::game::world::save::system::{LoadQueue, SaveConfig, SaveQueue, save_entire_world};
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::{
    PlayerChunkCache, StructureGenChannel, TerrainGenChannel, WorldStreamingConfig,
};
use crate::game::world::time::TimeOfDay;
use crate::shared::components::camera::FpsCamera;
use crate::shared::states::{AppState, InputContextState};

#[derive(Resource, Debug, Default)]
pub struct GameSession {
    pub fresh_load: bool,
    pub active_world: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorldSummary {
    pub id: String,
    pub seed: u64,
    pub modified_unix: u64,
}

#[derive(Resource, Debug, Default)]
pub struct WorldCatalog {
    pub worlds: Vec<WorldSummary>,
    pub selected: Option<String>,
}

#[derive(Resource, Debug, Default)]
pub struct PendingWorld(pub Option<String>);

#[derive(Resource, Debug, Clone)]
pub struct LoadingStatus {
    pub title: String,
    pub detail: String,
}

impl Default for LoadingStatus {
    fn default() -> Self {
        Self {
            title: "正在启动".into(),
            detail: "正在加载内容资源...".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    ConfirmDelete { world_id: String },
    Error,
}

#[derive(Resource, Debug, Default)]
pub struct DialogState {
    pub kind: Option<DialogKind>,
    pub title: String,
    pub message: String,
}

impl DialogState {
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.kind = Some(DialogKind::Error);
        self.title = title.into();
        self.message = message.into();
    }

    pub fn clear(&mut self) {
        self.kind = None;
        self.title.clear();
        self.message.clear();
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPage {
    #[default]
    Worlds,
    Settings,
}

#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub render_distance: u32,
    pub master_volume: f32,
    pub mouse_sensitivity: f32,
    pub ui_scale: f32,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            master_volume: 1.0,
            mouse_sensitivity: 1.0,
            ui_scale: 1.0,
            fullscreen: false,
            vsync: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingAction {
    RenderDistance(i32),
    MasterVolume(f32),
    MouseSensitivity(f32),
    UiScale(f32),
    ToggleFullscreen,
    ToggleVsync,
}

#[derive(Message, Debug, Clone)]
pub enum FlowCommand {
    RefreshWorlds,
    SelectWorld(String),
    CreateWorld(String),
    PlaySelected,
    RequestDeleteSelected,
    ConfirmDialog,
    CancelDialog,
    OpenSettings,
    CloseSettings,
    Resume,
    SaveAndQuit,
    QuitApplication,
    AdjustSetting(SettingAction),
}

#[derive(Resource, Default)]
struct SaveAndQuitRequest(bool);

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSession>()
            .init_resource::<WorldCatalog>()
            .init_resource::<PendingWorld>()
            .init_resource::<LoadingStatus>()
            .init_resource::<DialogState>()
            .init_resource::<MenuPage>()
            .init_resource::<GameSettings>()
            .init_resource::<SaveAndQuitRequest>()
            .add_message::<FlowCommand>()
            .add_systems(OnEnter(AppState::Boot), enter_boot_system)
            .add_systems(OnEnter(AppState::MainMenu), refresh_world_catalog_system)
            .add_systems(OnEnter(AppState::WorldLoading), prepare_world_system)
            .add_systems(
                OnEnter(AppState::InGame),
                request_content_reload_system.in_set(ContentReloadSet::Request),
            )
            .add_systems(OnEnter(AppState::Paused), pause_virtual_time_system)
            .add_systems(OnExit(AppState::Paused), resume_virtual_time_system)
            .add_systems(
                Update,
                (
                    handle_flow_commands_system,
                    sync_pause_state_system,
                    save_and_quit_system,
                    apply_settings_system,
                    finish_fresh_session_system,
                )
                    .chain(),
            );
    }
}

fn request_content_reload_system(
    session: Res<GameSession>,
    mut requests: MessageWriter<ContentReloadRequested>,
) {
    if session.fresh_load {
        requests.write_default();
    }
}

fn pause_virtual_time_system(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_virtual_time_system(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn enter_boot_system(
    mut next_state: ResMut<NextState<AppState>>,
    mut loading: ResMut<LoadingStatus>,
) {
    loading.title = "正在启动".into();
    loading.detail = "正在加载方块、纹理和基础资源...".into();
    next_state.set(AppState::Loading);
}

fn refresh_world_catalog_system(mut catalog: ResMut<WorldCatalog>) {
    refresh_world_catalog(&mut catalog);
}

fn refresh_world_catalog(catalog: &mut WorldCatalog) {
    let previous = catalog.selected.clone();
    catalog.worlds.clear();
    let root = std::path::Path::new("saves");
    let Ok(entries) = std::fs::read_dir(root) else {
        catalog.selected = None;
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(data) = level::load_level(&id) else {
            continue;
        };
        let modified_unix = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| duration.as_secs());
        catalog.worlds.push(WorldSummary {
            id,
            seed: data.seed,
            modified_unix,
        });
    }
    catalog
        .worlds
        .sort_by_key(|world| std::cmp::Reverse(world.modified_unix));
    catalog.selected = previous
        .filter(|selected| catalog.worlds.iter().any(|world| &world.id == selected))
        .or_else(|| catalog.worlds.first().map(|world| world.id.clone()));
}

#[allow(clippy::too_many_arguments)]
fn handle_flow_commands_system(
    mut reader: MessageReader<FlowCommand>,
    mut catalog: ResMut<WorldCatalog>,
    mut pending: ResMut<PendingWorld>,
    mut dialog: ResMut<DialogState>,
    mut menu_page: ResMut<MenuPage>,
    mut settings: ResMut<GameSettings>,
    block_registry: Option<Res<BlockRegistry>>,
    mut save_quit: ResMut<SaveAndQuitRequest>,
    mut next_state: ResMut<NextState<AppState>>,
    mut context: ResMut<InputContextState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for command in reader.read() {
        match command {
            FlowCommand::RefreshWorlds => refresh_world_catalog(&mut catalog),
            FlowCommand::SelectWorld(id) => catalog.selected = Some(id.clone()),
            FlowCommand::CreateWorld(name) => {
                let Some(registry) = block_registry.as_deref() else {
                    dialog.error("创建失败", "方块注册表尚未加载完成");
                    continue;
                };
                let id = unique_world_id(&sanitize_world_name(name), &catalog);
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                match level::save_level(&id, seed, Vec3::new(0.0, 70.0, 0.0), 0.25, registry) {
                    Ok(()) => {
                        refresh_world_catalog(&mut catalog);
                        catalog.selected = Some(id);
                    }
                    Err(error) => dialog.error("创建失败", error.to_string()),
                }
            }
            FlowCommand::PlaySelected => {
                if let Some(selected) = catalog.selected.clone() {
                    pending.0 = Some(selected);
                    next_state.set(AppState::WorldLoading);
                } else {
                    dialog.error("无法进入世界", "请先创建或选择一个世界");
                }
            }
            FlowCommand::RequestDeleteSelected => {
                if let Some(world_id) = catalog.selected.clone() {
                    dialog.kind = Some(DialogKind::ConfirmDelete {
                        world_id: world_id.clone(),
                    });
                    dialog.title = "删除世界".into();
                    dialog.message = format!("确定永久删除世界“{world_id}”吗？此操作无法撤销。");
                }
            }
            FlowCommand::ConfirmDialog => {
                if let Some(DialogKind::ConfirmDelete { world_id }) = dialog.kind.clone()
                    && valid_world_id(&world_id)
                {
                    match std::fs::remove_dir_all(RegionManager::save_root(&world_id)) {
                        Ok(()) => refresh_world_catalog(&mut catalog),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            refresh_world_catalog(&mut catalog);
                        }
                        Err(error) => {
                            dialog.error("删除失败", error.to_string());
                            continue;
                        }
                    }
                }
                dialog.clear();
            }
            FlowCommand::CancelDialog => dialog.clear(),
            FlowCommand::OpenSettings => *menu_page = MenuPage::Settings,
            FlowCommand::CloseSettings => *menu_page = MenuPage::Worlds,
            FlowCommand::Resume => {
                context.set_menu_open(false);
                next_state.set(AppState::InGame);
            }
            FlowCommand::SaveAndQuit => save_quit.0 = true,
            FlowCommand::QuitApplication => {
                app_exit.write(AppExit::Success);
            }
            FlowCommand::AdjustSetting(action) => adjust_setting(&mut settings, *action),
        }
    }
}

fn sync_pause_state_system(
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

#[derive(SystemParam)]
struct PrepareWorldParams<'w, 's> {
    commands: Commands<'w, 's>,
    save_config: ResMut<'w, SaveConfig>,
    world_generator: ResMut<'w, WorldGenerator>,
    time_of_day: ResMut<'w, TimeOfDay>,
    world_storage: ResMut<'w, WorldStorage>,
    player_cache: ResMut<'w, PlayerChunkCache>,
    terrain_channel: ResMut<'w, TerrainGenChannel>,
    structure_channel: ResMut<'w, StructureGenChannel>,
    mesh_channel: ResMut<'w, MeshBuildChannel>,
    save_queue: ResMut<'w, SaveQueue>,
    load_queue: ResMut<'w, LoadQueue>,
    chunk_query: Query<'w, 's, Entity, With<ChunkComponents>>,
    session: ResMut<'w, GameSession>,
    loading: ResMut<'w, LoadingStatus>,
    dialog: ResMut<'w, DialogState>,
    next_state: ResMut<'w, NextState<AppState>>,
}

fn prepare_world_system(pending: Res<PendingWorld>, mut params: PrepareWorldParams) {
    let Some(world_id) = pending.0.as_deref() else {
        params.dialog.error("加载失败", "没有待加载的世界");
        params.next_state.set(AppState::MainMenu);
        return;
    };
    params.loading.title = "正在加载世界".into();
    params.loading.detail = format!("正在读取 {world_id}...");
    match level::load_level(world_id) {
        Ok(level_data) => {
            for entity in &params.chunk_query {
                params.commands.entity(entity).despawn();
            }
            *params.world_storage = WorldStorage::default();
            *params.player_cache = PlayerChunkCache::default();
            *params.terrain_channel = TerrainGenChannel::default();
            *params.structure_channel = StructureGenChannel::default();
            *params.mesh_channel = MeshBuildChannel::default();
            params.save_queue.queue.clear();
            params.load_queue.queue.clear();
            params.save_config.world_name = world_id.to_string();
            *params.world_generator = WorldGenerator::new(level_data.seed as u32);
            params.time_of_day.current_time = level_data.time_of_day;
            params.session.fresh_load = true;
            params.session.active_world = Some(world_id.to_string());
            params.loading.detail = "正在生成出生区域...".into();
            params.next_state.set(AppState::InGame);
        }
        Err(error) => {
            params
                .dialog
                .error("世界加载失败", format!("{world_id}: {error}"));
            params.next_state.set(AppState::MainMenu);
        }
    }
}

#[derive(SystemParam)]
struct SaveQuitParams<'w, 's> {
    commands: Commands<'w, 's>,
    save_config: Res<'w, SaveConfig>,
    world_storage: Res<'w, WorldStorage>,
    block_registry: Option<Res<'w, BlockRegistry>>,
    world_generator: Res<'w, WorldGenerator>,
    time_of_day: Res<'w, TimeOfDay>,
    player_query: Query<'w, 's, &'static Transform, With<Player>>,
    camera_query: Query<'w, 's, &'static FpsCamera, With<Camera3d>>,
    gamemode: Res<'w, PlayerGameMode>,
    inventory: Res<'w, InventoryState>,
    save_manager: ResMut<'w, PlayerSaveManager>,
    time: Res<'w, Time>,
    chunk_query: Query<'w, 's, Entity, With<ChunkComponents>>,
    hud_query: Query<'w, 's, Entity, With<HudRoot>>,
    dialog: ResMut<'w, DialogState>,
    session: ResMut<'w, GameSession>,
    context: ResMut<'w, InputContextState>,
    next_state: ResMut<'w, NextState<AppState>>,
}

fn save_and_quit_system(mut request: ResMut<SaveAndQuitRequest>, mut params: SaveQuitParams) {
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
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);
    if let Err(error) = save_entire_world(
        &params.save_config.world_name,
        &params.world_storage,
        registry,
        params.world_generator.seed as u64,
        spawn,
        params.time_of_day.current_time,
    ) {
        params.dialog.error("保存失败", error.to_string());
        return;
    }
    if let Err(error) = save_player_now(
        &params.save_config.world_name,
        &params.gamemode,
        &params.inventory,
        &params.player_query,
        &params.camera_query,
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

fn finish_fresh_session_system(state: Res<State<AppState>>, mut session: ResMut<GameSession>) {
    if *state.get() == AppState::InGame && session.fresh_load {
        session.fresh_load = false;
    }
}

fn adjust_setting(settings: &mut GameSettings, action: SettingAction) {
    match action {
        SettingAction::RenderDistance(delta) => {
            settings.render_distance =
                (settings.render_distance as i32 + delta).clamp(2, 24) as u32;
        }
        SettingAction::MasterVolume(delta) => {
            settings.master_volume = (settings.master_volume + delta).clamp(0.0, 1.0);
        }
        SettingAction::MouseSensitivity(delta) => {
            settings.mouse_sensitivity = (settings.mouse_sensitivity + delta).clamp(0.2, 3.0);
        }
        SettingAction::UiScale(delta) => {
            settings.ui_scale = (settings.ui_scale + delta).clamp(0.6, 1.6);
        }
        SettingAction::ToggleFullscreen => settings.fullscreen = !settings.fullscreen,
        SettingAction::ToggleVsync => settings.vsync = !settings.vsync,
    }
}

fn apply_settings_system(
    settings: Res<GameSettings>,
    mut ui_scale: ResMut<UiScaleSettings>,
    mut streaming: ResMut<WorldStreamingConfig>,
    mut global_volume: ResMut<GlobalVolume>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    ui_scale.user_scale = settings.ui_scale;
    *streaming = WorldStreamingConfig::new(
        settings.render_distance,
        settings.render_distance,
        streaming.data_vertical_radius_above as u32,
        streaming.data_vertical_radius_below as u32,
    );
    global_volume.volume = Volume::Linear(settings.master_volume);
    if let Ok(mut window) = window_query.single_mut() {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        window.present_mode = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
    }
}

fn sanitize_world_name(name: &str) -> String {
    let mut result = String::new();
    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            result.push(character.to_ascii_lowercase());
        } else if character.is_whitespace() && !result.ends_with('_') {
            result.push('_');
        }
    }
    result = result.trim_matches('_').to_string();
    if result.is_empty() {
        format!(
            "world_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    } else {
        result
    }
}

fn unique_world_id(base: &str, catalog: &WorldCatalog) -> String {
    if !catalog.worlds.iter().any(|world| world.id == base) {
        return base.to_string();
    }
    (2..)
        .map(|suffix| format!("{base}_{suffix}"))
        .find(|candidate| !catalog.worlds.iter().any(|world| &world.id == candidate))
        .expect("world suffix space is effectively unbounded")
}

fn valid_world_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_names_are_safe_and_non_empty() {
        assert_eq!(sanitize_world_name(" My World "), "my_world");
        assert!(valid_world_id(&sanitize_world_name("../../")));
        assert!(!valid_world_id("../unsafe"));
    }

    #[test]
    fn settings_are_clamped_to_supported_ranges() {
        let mut settings = GameSettings::default();
        adjust_setting(&mut settings, SettingAction::RenderDistance(-100));
        adjust_setting(&mut settings, SettingAction::MasterVolume(-5.0));
        adjust_setting(&mut settings, SettingAction::UiScale(5.0));
        assert_eq!(settings.render_distance, 2);
        assert_eq!(settings.master_volume, 0.0);
        assert_eq!(settings.ui_scale, 1.6);
    }
}
