use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

pub mod bars;
pub mod hotbar;

#[derive(Component)]
pub struct BottomHud;

/// 生成底部HUD节点
pub fn spawn_bottom_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("HUD根节点未生成！");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            BottomHud,
            Name::new("BottomHud"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                bottom: Val::Px(5.0),
                row_gap: Val::Px(5.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..default()
            },
        ));
    });
}
