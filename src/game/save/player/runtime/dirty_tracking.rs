use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::{LocalPlayer, Player};
use crate::game::save::events::SaveDirtySource;
use crate::game::save::player::PlayerSaveManager;
use bevy::prelude::{Changed, DetectChanges, Query, Res, ResMut, Transform, With};

/// 玩家位置变化超过此距离才标记Dirty
pub const POSITION_DIRTY_THRESHOLD_SQ: f32 = 0.25;

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
    inventory: Query<(), (With<LocalPlayer>, Changed<InventoryState>)>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if !inventory.is_empty() {
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
