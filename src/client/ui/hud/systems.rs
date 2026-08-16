//! HUD 根节点生成系统。

use bevy::prelude::*;

use crate::client::ui::hud::components::HudRoot;

/// 生成HUD根节点
pub fn spawn_hud_root_system(mut commands: Commands) {
    commands.spawn((
        HudRoot,
        Name::new("HudRoot"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    ));
}
