//! 组织 HUD 中央准星和目标反馈。

pub mod crosshair;

use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

/// HUD 中央区域根节点。
#[derive(Component)]
pub struct CenterHud;

/// 生成中间HUD节点
pub fn spawn_center_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("[HUD] 中央区域挂载失败：HudRoot 未生成");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            CenterHud,
            Name::new("CenterHud"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ));
    });
}
