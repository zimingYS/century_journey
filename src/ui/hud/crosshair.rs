use bevy::prelude::*;
use crate::ui::components::Crosshair;

pub fn setup_crosshair(
    mut commands: Commands,
){
    commands.spawn((
        Node{
            // 绝对定位
            position_type: PositionType::Absolute,
            // 水平居中
            justify_content: JustifyContent::Center,
            // 垂直居中
            align_items: AlignItems::Center,
            // 设置长宽
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Crosshair,
    )).with_children(|parent| {
        // 横线
        parent.spawn((
            Node {
                width: Val::Px(20.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        ));

        // 竖线
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(2.0),
                height: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        ));
    });
}