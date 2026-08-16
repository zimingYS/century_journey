//! 底部状态条的标记组件。

use bevy::prelude::*;

/// 底部状态条总容器。
#[derive(Component)]
pub struct BarsHud;

/// 左侧状态条容器，目前承载护甲和生命值。
#[derive(Component)]
pub struct LeftBarsHud;

/// 右侧状态条容器，目前承载饥饿值。
#[derive(Component)]
pub struct RightBarsHud;
