use crate::client::ui::components::HudRoot;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Health;
use bevy::prelude::*;

#[derive(Component)]
pub struct HealthBar;

/// 生成生命值
pub fn spawn_health_bar(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(hud_entity) = hud.single() else {
        log::error!("HUD ROOT NOT FOUND — cannot spawn health bar");
        return;
    };
    commands.entity(hud_entity).with_children(|parent| {
        parent.spawn((
            HealthBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            },
        ));
    });
}

/// 同步生命值进行UI渲染
pub fn health_bar_sync_system(
    health_query: Query<&Health, With<Player>>,
    bar_query: Query<Entity, With<HealthBar>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let Ok(health) = health_query.single() else {
        return;
    };
    let Ok(bar_entity) = bar_query.single() else {
        return;
    };

    // 清除旧子元素
    if let Ok(children) = children_query.get(bar_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let hearts = (health.current / 2.0).ceil() as usize;
    let max_hearts = (health.max / 2.0).ceil() as usize;

    commands.entity(bar_entity).with_children(|bar| {
        for i in 0..max_hearts {
            let filled = i < hearts;
            let color = if filled {
                Color::srgb(1.0, 0.1, 0.1)
            } else {
                Color::srgb(0.25, 0.05, 0.05)
            };
            bar.spawn((
                Node {
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(color),
            ));
        }
        bar.spawn((
            Text::new(format!("{}", health.current as u32)),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::left(Val::Px(6.0)),
                ..default()
            },
        ));
    });
}
