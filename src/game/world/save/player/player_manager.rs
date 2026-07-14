use super::player_io::{player_save_path, write_player_data};
use super::player_model::{PlayerSaveData, validate_player_data};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::Player;
use crate::game::player::components::stats::{Health, Hunger};
use crate::game::world::save::events::SaveDirtySource;
use crate::game::world::save::system::SaveConfig;
use crate::shared::components::camera::FpsCamera;
use bevy::prelude::*;

/// 玩家位置变化超过此距离才标记Dirty
const POSITION_DIRTY_THRESHOLD_SQ: f32 = 0.25;

// ═══════════════════════════════════════════════════════════════
// 玩家存档状态
// ═══════════════════════════════════════════════════════════════

#[derive(Resource, Debug)]
pub struct PlayerSaveManager {
    pub dirty: bool,
    pub last_dirty_source: Option<SaveDirtySource>,
    pub total_saves: u64,
    pub last_save_time: f64,
    last_saved_position: Vec3,
    auto_save_timer: f32,
}

impl Default for PlayerSaveManager {
    fn default() -> Self {
        Self {
            dirty: false,
            last_dirty_source: None,
            total_saves: 0,
            last_save_time: 0.0,
            last_saved_position: Vec3::ZERO,
            auto_save_timer: 30.0,
        }
    }
}

impl PlayerSaveManager {
    pub const AUTO_SAVE_INTERVAL: f32 = 30.0;

    pub fn set_dirty(&mut self, source: SaveDirtySource) {
        if !self.dirty {
            self.dirty = true;
            self.last_dirty_source = Some(source);
        }
    }

    pub fn check_position_dirty(&mut self, current_pos: Vec3) -> bool {
        let dist_sq = current_pos.distance_squared(self.last_saved_position);
        if dist_sq > POSITION_DIRTY_THRESHOLD_SQ {
            self.set_dirty(SaveDirtySource::Position);
            true
        } else {
            false
        }
    }

    fn reset_timer(&mut self) {
        self.auto_save_timer = Self::AUTO_SAVE_INTERVAL;
    }

    fn tick(&mut self, dt: f32) -> bool {
        if !self.dirty {
            return false;
        }
        self.auto_save_timer -= dt;
        if self.auto_save_timer <= 0.0 {
            self.reset_timer();
            true
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 保存逻辑
// ═══════════════════════════════════════════════════════════════

fn perform_save(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    player_query: &Query<(&Transform, &Health, &Hunger), With<Player>>,
    camera_query: &Query<&FpsCamera, With<Camera3d>>,
    save_manager: &mut PlayerSaveManager,
    time: &Time,
) {
    let (transform, health, hunger) = player_query
        .single()
        .map(|(transform, health, hunger)| (*transform, health.current, hunger.current))
        .unwrap_or((Transform::default(), 20.0, 20.0));
    let pitch = camera_query.single().map(|c| c.pitch).unwrap_or(0.0);
    let data = PlayerSaveData::from_runtime(
        transform.translation,
        transform.rotation,
        pitch,
        gamemode,
        inventory,
        health,
        hunger,
    );

    let path = player_save_path(world_name);
    match write_player_data(&data, &path) {
        Ok(()) => {
            save_manager.dirty = false;
            save_manager.last_dirty_source = None;
            save_manager.total_saves += 1;
            save_manager.last_save_time = time.elapsed_secs() as f64;
            save_manager.last_saved_position = transform.translation;
            log::info!(
                "[存档系统] 玩家已保存(共计: {}),保存到{:?}",
                save_manager.total_saves,
                path
            );
        }
        Err(e) => {
            log::error!("[存档系统] 保存失败: {e} !");
        }
    }
}

/// 立即保存玩家数据，供“保存并退出”流程同步确认结果。
pub fn save_player_now(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    player_query: &Query<(&Transform, &Health, &Hunger), With<Player>>,
    camera_query: &Query<&FpsCamera, With<Camera3d>>,
    save_manager: &mut PlayerSaveManager,
    time: &Time,
) -> Result<(), String> {
    let (transform, health, hunger) = player_query
        .single()
        .map(|(transform, health, hunger)| (*transform, health.current, hunger.current))
        .unwrap_or((Transform::default(), 20.0, 20.0));
    let pitch = camera_query
        .single()
        .map(|camera| camera.pitch)
        .unwrap_or(0.0);
    let data = PlayerSaveData::from_runtime(
        transform.translation,
        transform.rotation,
        pitch,
        gamemode,
        inventory,
        health,
        hunger,
    );
    let path = player_save_path(world_name);
    write_player_data(&data, &path)?;
    save_manager.dirty = false;
    save_manager.last_dirty_source = None;
    save_manager.total_saves += 1;
    save_manager.last_save_time = time.elapsed_secs() as f64;
    save_manager.last_saved_position = transform.translation;
    log::info!("[存档系统] 玩家已同步保存到 {path:?}");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// ECS 系统
// ═══════════════════════════════════════════════════════════════

/// 进入游戏时加载玩家存档
pub fn load_player_on_enter_system(
    save_config: Res<SaveConfig>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut inventory: ResMut<InventoryState>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut crate::game::player::components::stats::Health,
            &mut crate::game::player::components::stats::Hunger,
        ),
        With<Player>,
    >,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    _time: Res<Time>,
) {
    use super::player_io::read_player_data;

    let save_path = player_save_path(&save_config.world_name);
    let raw_data = if save_path.exists() {
        match read_player_data(&save_path) {
            Ok(data) => {
                log::info!("[存档系统] 从 {:?} 加载数据成功", save_path);
                data
            }
            Err(e) => {
                log::warn!("[存档系统] 从 {} 加载失败, 已使用默认值", e);
                PlayerSaveData::default()
            }
        }
    } else {
        log::info!("[存档系统] 存档不存在，正在已默认值创建");
        PlayerSaveData::default()
    };

    let save_data = validate_player_data(&raw_data);
    *gamemode = save_data.restore_gamemode();

    let restored = save_data.restore_inventory();
    inventory.hotbar = restored.hotbar;
    inventory.survival = restored.survival;

    if let Ok((mut transform, mut health, mut hunger)) = player_query.single_mut() {
        *transform = save_data.restore_transform();
        save_manager.last_saved_position = transform.translation;

        health.current = save_data.health.clamp(0.0, health.max);
        hunger.current = save_data.hunger.clamp(0.0, hunger.max);
        hunger.saturation = 5.0;
    }

    if let Ok(mut fps_camera) = camera_query.single_mut() {
        fps_camera.pitch = save_data.camera_pitch();
    }

    save_manager.set_dirty(SaveDirtySource::Position);
    inventory.set_changed();

    log::info!(
        "[存档系统] 玩家已生成,位置:{:?},游戏模式:{}",
        save_data.position,
        save_data.gamemode
    );
}

/// 玩家位置脏数据追踪
pub fn player_position_dirty_system(
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    for transform in &player_query {
        save_manager.check_position_dirty(transform.translation);
    }
}

/// 背包变化脏数据追踪
pub fn inventory_dirty_tracking_system(
    inventory: Res<InventoryState>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if inventory.is_changed() {
        save_manager.set_dirty(SaveDirtySource::Inventory);
    }
}

/// 游戏模式变化脏数据追踪
pub fn gamemode_dirty_tracking_system(
    gamemode: Res<PlayerGameMode>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if gamemode.is_changed() {
        save_manager.set_dirty(SaveDirtySource::GameMode);
    }
}

/// 自动保存系统
pub fn auto_save_player_system(
    time: Res<Time>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<(&Transform, &Health, &Hunger), With<Player>>,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if !save_manager.tick(time.delta_secs()) {
        return;
    }
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &player_query,
        &camera_query,
        &mut save_manager,
        &time,
    );
}

/// 收到应用退出事件时立即保存。
pub fn save_on_exit_system(
    mut exit_reader: MessageReader<AppExit>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<(&Transform, &Health, &Hunger), With<Player>>,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    time: Res<Time>,
) {
    if exit_reader.read().next().is_none() {
        return;
    }
    if !save_manager.dirty {
        return;
    }
    log::info!("[存档系统] 检测到游戏退出,正在保存游戏...");
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &player_query,
        &camera_query,
        &mut save_manager,
        &time,
    );
}
