//! cj-game —— 玩法层
//!
//! 规则：不重新实现底层模拟算法（不依赖 sim / world 的算法）。

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, _app: &mut App) {
        // TODO: 注册玩家 / 物品 / 工艺 / 建筑 / 观测 / UI
    }
}
