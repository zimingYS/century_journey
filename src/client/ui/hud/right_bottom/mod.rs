//! 构建 HUD 右下区域，供快捷提示扩展。

use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

/// HUD 右下区域根节点。
#[derive(Component)]
pub struct RightBottomHud;

/// 生成右下HUD节点
pub fn spawn_right_bottom_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("[HUD] 右下区域挂载失败：HudRoot 未生成");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            RightBottomHud,
            Name::new("RightBottomHud"),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                bottom: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::End,
                ..default()
            },
        ));
    });
}
