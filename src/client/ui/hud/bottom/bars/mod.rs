use crate::client::ui::hud::bottom::BottomHud;
use bevy::prelude::*;

pub mod left_bars;
pub mod right_bars;

#[derive(Component)]
pub struct BarsHud;

#[derive(Component)]
pub struct LeftBarsHud;
#[derive(Component)]
pub struct RightBarsHud;
pub fn spawn_bars_hud_system(mut commands: Commands, bottom_hud: Query<Entity, With<BottomHud>>) {
    let Ok(bottom_entity) = bottom_hud.single() else {
        log::error!("BOTTOM HUD NOT FOUND — cannot spawn BarsHud");
        return;
    };

    commands.entity(bottom_entity).with_children(|parent| {
        parent
            .spawn((
                BarsHud,
                Name::new("BarsHud"),
                Node {
                    width: Val::Percent(40.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|bars| {
                bars.spawn((
                    LeftBarsHud,
                    Name::new("LeftBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
                bars.spawn((
                    RightBarsHud,
                    Name::new("RightBarsHud"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ));
            });
    });
}
