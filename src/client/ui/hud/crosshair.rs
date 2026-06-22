use crate::client::ui::components::{Crosshair, HudRoot};
use bevy::prelude::*;

pub fn setup_crosshair(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(hud_entity) = hud.single() else {
        log::error!("HUD ROOT NOT FOUND — cannot spawn crosshair");
        return;
    };
    commands.entity(hud_entity).with_children(|parent| {
        parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Crosshair,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(2.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                ));
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
    });
}
