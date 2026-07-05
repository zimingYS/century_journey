use crate::client::ui::hud::bottom::bars::LeftBarsHud;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Defense;
use bevy::prelude::*;

#[derive(Component)]
pub struct ArmorBar;

pub fn spawn_armor_bar(mut commands: Commands, bars_hud: Query<Entity, With<LeftBarsHud>>) {
    let Ok(bars_hud_entity) = bars_hud.single() else {
        log::error!("LEFT BARS HUD NOT FOUND — cannot spawn armor bar");
        return;
    };
    commands.entity(bars_hud_entity).with_children(|p| {
        p.spawn((
            ArmorBar,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            },
        ));
    });
}

pub fn armor_bar_sync_system(
    defense_q: Query<&Defense, With<Player>>,
    bar_q: Query<Entity, With<ArmorBar>>,
    children_q: Query<&Children>,
    mut commands: Commands,
) {
    let Ok(defense) = defense_q.single() else {
        return;
    };
    let Ok(bar) = bar_q.single() else { return };
    if let Ok(c) = children_q.get(bar) {
        for child in c.iter() {
            commands.entity(child).despawn();
        }
    }
    let d = defense.0 as usize;
    if d == 0 {
        return;
    }
    commands.entity(bar).with_children(|b| {
        for _ in 0..d {
            b.spawn((
                Node {
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.5, 0.5, 0.6)),
            ));
        }
    });
}
