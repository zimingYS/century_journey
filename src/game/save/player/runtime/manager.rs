use crate::game::save::events::SaveDirtySource;
use crate::game::save::player::runtime::dirty_tracking::POSITION_DIRTY_THRESHOLD_SQ;
use bevy::prelude::*;

/// 管理玩家存档的脏状态、自动保存计时和保存统计信息
#[derive(Resource, Debug)]
pub struct PlayerSaveManager {
    pub(in crate::game) dirty: bool,
    pub(in crate::game) last_dirty_source: Option<SaveDirtySource>,
    pub(in crate::game) total_saves: u64,
    pub(in crate::game) last_save_time: f64,
    pub(in crate::game) last_saved_position: Vec3,
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
            auto_save_timer: Self::AUTO_SAVE_INTERVAL,
        }
    }
}

impl PlayerSaveManager {
    /// 自动保存检查间隔,单位为秒
    pub(in crate::game) const AUTO_SAVE_INTERVAL: f32 = 30.0;

    /// 标记玩家数据已经发生需要持久化的变化
    pub(in crate::game) fn set_dirty(&mut self, source: SaveDirtySource) {
        if !self.dirty {
            self.dirty = true;
            self.last_dirty_source = Some(source);
        }
    }

    /// 玩家移动超过阈值时，将位置标记为待保存
    pub(in crate::game) fn check_position_dirty(&mut self, current_position: Vec3) -> bool {
        let distance_squared = current_position.distance_squared(self.last_saved_position);

        if distance_squared > POSITION_DIRTY_THRESHOLD_SQ {
            self.set_dirty(SaveDirtySource::Position);
            true
        } else {
            false
        }
    }

    /// 推进自动保存计时，仅在存在脏数据且计时到期时触发保存
    pub(in crate::game) fn tick(&mut self, delta_seconds: f32) -> bool {
        if !self.dirty {
            return false;
        }

        self.auto_save_timer -= delta_seconds;

        if self.auto_save_timer <= 0.0 {
            self.auto_save_timer = Self::AUTO_SAVE_INTERVAL;
            true
        } else {
            false
        }
    }
}
