//! 组织属于权威世界的动态实体规则。

pub mod dropped_item;

use bevy::prelude::*;

/// 组装世界动态实体的权威游戏规则。
pub struct EntityPlugin;
impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(dropped_item::DroppedItemPlugin);
    }
}
