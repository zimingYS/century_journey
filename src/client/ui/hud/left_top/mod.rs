//! 构建 HUD 左上区域，供调试与状态信息扩展。

use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

/// HUD 左上区域根节点。
#[derive(Component)]
pub struct LeftTopHud;

/// 生成左上HUD节点
pub fn spawn_left_top_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("[HUD] 左上区域挂载失败：HudRoot 未生成");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            LeftTopHud,
            Name::new("LeftTopHud"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..default()
            },
        ));
    });
}
