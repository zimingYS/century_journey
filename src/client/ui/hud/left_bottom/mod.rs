use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

#[derive(Component)]
pub struct LeftBottomHud;

/// 生成左下HUD节点
pub fn spawn_left_bottom_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("HUD根节点未生成！");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            LeftBottomHud,
            Name::new("LeftBottomHud"),
            Node {
                left: Val::Px(16.0),
                bottom: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::Start,
                ..default()
            },
        ));
    });
}
