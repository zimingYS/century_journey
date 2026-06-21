use bevy::prelude::*;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Hunger;
use crate::client::ui::components::HudRoot;

#[derive(Component)]
pub struct HungerBar;

/// 生成饥饿值HUD
pub fn spawn_hunger_bar(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(hud_entity) = hud.single() else {
        log::error!("HUD ROOT NOT FOUND — cannot spawn hunger bar");
        return;
    };
    commands.entity(hud_entity).with_children(|parent| {
        parent.spawn((HungerBar, Node {
            position_type: PositionType::Absolute, left: Val::Px(10.0), top: Val::Px(28.0),
            flex_direction: FlexDirection::Row, column_gap: Val::Px(2.0), ..default()
        }));
    });
}

/// 根据饥饿值同步UI显示
pub fn hunger_bar_sync_system(
    hunger_query: Query<&Hunger, With<Player>>,
    bar_query: Query<Entity, With<HungerBar>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let Ok(hunger) = hunger_query.single() else { return };
    let Ok(bar_entity) = bar_query.single() else { return };

    if let Ok(children) = children_query.get(bar_entity) {
        for child in children.iter() { commands.entity(child).despawn(); }
    }

    let filled = (hunger.current / 2.0).ceil() as usize;
    let max_count = (hunger.max / 2.0).ceil() as usize;

    commands.entity(bar_entity).with_children(|bar| {
        for i in 0..max_count {
            let c = if i < filled { Color::srgb(0.6, 0.4, 0.1) } else { Color::srgb(0.15, 0.08, 0.02) };
            bar.spawn((
                Node { width: Val::Px(9.0), height: Val::Px(9.0), margin: UiRect::all(Val::Px(1.0)), ..default() },
                BackgroundColor(c),
            ));
        }
    });
}