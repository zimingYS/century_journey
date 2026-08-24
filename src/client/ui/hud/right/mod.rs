//! 构建 HUD 右侧区域容器。

use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

/// HUD 右侧区域根节点。
#[derive(Component)]
pub struct RightHud;

/// 生成右边HUD节点
pub fn spawn_right_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("[HUD] 右侧区域挂载失败：HudRoot 未生成");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            RightHud,
            Name::new("RightHud"),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                bottom: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::End,
                ..default()
            },
        ));
    });
}
