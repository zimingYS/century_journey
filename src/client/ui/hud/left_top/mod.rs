use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

#[derive(Component)]
pub struct LeftTopHud;

/// 生成左上HUD节点
pub fn spawn_left_top_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("HUD根节点未生成！");
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
