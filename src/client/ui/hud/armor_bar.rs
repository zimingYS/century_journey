use crate::client::ui::components::HudRoot;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Defense;
use bevy::prelude::*;

#[derive(Component)]
pub struct ArmorBar;

pub fn spawn_armor_bar(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(e) = hud.single() else {
        log::error!("HUD ROOT NOT FOUND — cannot spawn armor bar");
        return;
    };
    commands.entity(e).with_children(|p| {
        p.spawn((
            ArmorBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(46.0),
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
