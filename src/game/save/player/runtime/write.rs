//! 从权威玩家组件收集数据，并执行自动保存和退出保存。

use bevy::app::AppExit;
use bevy::math::Vec3;
use bevy::prelude;
use bevy::prelude::{MessageReader, Query, Res, ResMut, Time, Transform, With};

use crate::content::item::ItemRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::{InventoryState, LocalInventory};
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::RespawnPoint;
use crate::game::player::movement::components::PlayerAim;
use crate::game::player::survival::health::Health;
use crate::game::player::survival::hunger::Hunger;
use crate::game::save::SaveConfig;
use crate::game::save::player::io::write_player_data;
use crate::game::save::player::runtime::manager::PlayerSaveManager;
use crate::game::save::player::{PlayerSaveData, player_save_path};

fn perform_save(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    item_registry: &ItemRegistry,
    player_query: &Query<(&Transform, &Health, &Hunger, &RespawnPoint, &PlayerAim), With<Player>>,
    save_manager: &mut PlayerSaveManager,
    time: &Time,
) {
    let (data, player_position) =
        collect_player_save_data(gamemode, inventory, item_registry, player_query);
    let path = player_save_path(world_name);

    match write_player_data(&data, &path) {
        Ok(()) => {
            mark_save_succeeded(save_manager, player_position, time);
            log::info!(
                "玩家数据保存成功：{}，累计保存 {} 次",
                path.display(),
                save_manager.total_saves
            );
        }
        Err(error) => {
            log::error!("玩家数据保存失败：{}，错误：{}", path.display(), error);
        }
    }
}

/// 立即保存玩家数据，供“保存并退出”流程同步确认结果。
pub fn save_player_now(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    item_registry: &ItemRegistry,
    player_query: &Query<(&Transform, &Health, &Hunger, &RespawnPoint, &PlayerAim), With<Player>>,
    save_manager: &mut PlayerSaveManager,
    time: &Time,
) -> prelude::Result<(), String> {
    let (data, player_position) =
        collect_player_save_data(gamemode, inventory, item_registry, player_query);
    let path = player_save_path(world_name);

    write_player_data(&data, &path)?;
    mark_save_succeeded(save_manager, player_position, time);

    log::info!(
        "玩家数据立即保存成功：{}，累计保存 {} 次",
        path.display(),
        save_manager.total_saves
    );
    Ok(())
}

/// 按 SaveConfig 的间隔检查并保存脏玩家数据。
pub fn auto_save_player_system(
    time: Res<Time>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: LocalInventory,
    item_registry: Res<ItemRegistry>,
    player_query: Query<(&Transform, &Health, &Hunger, &RespawnPoint, &PlayerAim), With<Player>>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if !save_manager.tick(time.delta_secs()) {
        return;
    }
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &item_registry,
        &player_query,
        &mut save_manager,
        &time,
    );
}

/// 收到应用退出事件时立即保存尚未落盘的玩家状态。
/// 退出保存显式读取全部权威玩家字段，确保进程结束前收集完整快照。
#[allow(clippy::too_many_arguments)]
pub fn save_on_exit_system(
    mut exit_reader: MessageReader<AppExit>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: LocalInventory,
    item_registry: Res<ItemRegistry>,
    player_query: Query<(&Transform, &Health, &Hunger, &RespawnPoint, &PlayerAim), With<Player>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    time: Res<Time>,
) {
    if exit_reader.read().next().is_none() || !save_manager.dirty {
        return;
    }
    log::info!("[存档系统] 检测到游戏退出，正在保存游戏");
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &item_registry,
        &player_query,
        &mut save_manager,
        &time,
    );
}

/// 从同一玩家实体收集位置、生存状态和权威视角，避免依赖客户端相机。
fn collect_player_save_data(
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    item_registry: &ItemRegistry,
    player_query: &Query<(&Transform, &Health, &Hunger, &RespawnPoint, &PlayerAim), With<Player>>,
) -> (PlayerSaveData, Vec3) {
    let (transform, health, hunger, saturation, respawn_point, pitch) = player_query
        .single()
        .map(|(transform, health, hunger, respawn, aim)| {
            (
                *transform,
                health.current,
                hunger.current,
                hunger.saturation,
                respawn.0,
                aim.pitch,
            )
        })
        .unwrap_or((
            Transform::default(),
            20.0,
            20.0,
            5.0,
            Vec3::new(0.0, 70.0, 0.0),
            0.0,
        ));

    let data = PlayerSaveData::from_runtime(
        transform.translation,
        transform.rotation,
        pitch,
        gamemode,
        inventory,
        item_registry,
        health,
        hunger,
        saturation,
        respawn_point,
    );
    (data, transform.translation)
}

/// 仅在所有玩家数据成功落盘后更新保存管理器快照。
fn mark_save_succeeded(save_manager: &mut PlayerSaveManager, position: Vec3, time: &Time) {
    save_manager.dirty = false;
    save_manager.last_dirty_source = None;
    save_manager.total_saves += 1;
    save_manager.last_save_time = time.elapsed_secs() as f64;
    save_manager.last_saved_position = position;
}
