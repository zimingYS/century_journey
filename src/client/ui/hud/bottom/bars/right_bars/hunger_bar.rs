use crate::client::ui::hud::bottom::bars::RightBarsHud;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Hunger;
use bevy::prelude::*;

#[derive(Component)]
pub struct HungerBar;

/// 生成饥饿值HUD
pub fn spawn_hunger_bar(mut commands: Commands, bars_hud: Query<Entity, With<RightBarsHud>>) {
    let Ok(bars_hud_entity) = bars_hud.single() else {
        log::error!("BARS HUD NOT FOUND — cannot spawn hunger bar");
        return;
    };
    commands.entity(bars_hud_entity).with_children(|parent| {
        parent.spawn((
            HungerBar,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            },
        ));
    });
}

/// 根据饥饿值同步UI显示
pub fn hunger_bar_sync_system(
    hunger_query: Query<&Hunger, With<Player>>,
    bar_query: Query<Entity, With<HungerBar>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let Ok(hunger) = hunger_query.single() else {
        return;
    };
    let Ok(bar_entity) = bar_query.single() else {
        return;
    };

    if let Ok(children) = children_query.get(bar_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let filled = (hunger.current / 2.0).ceil() as usize;
    let max_count = (hunger.max / 2.0).ceil() as usize;

    commands.entity(bar_entity).with_children(|bar| {
        for i in 0..max_count {
            let c = if i < filled {
                Color::srgb(0.6, 0.4, 0.1)
            } else {
                Color::srgb(0.15, 0.08, 0.02)
            };
            bar.spawn((
                Node {
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(c),
            ));
        }
    });
}
