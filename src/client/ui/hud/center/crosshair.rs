use crate::client::ui::hud::center::CenterHud;
use crate::game::gameplay::block_action::BlockBreakProgress;
use bevy::prelude::*;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct BreakProgressBar;

#[derive(Component)]
pub struct BreakProgressFill;

pub fn spawn_crosshair(mut commands: Commands, hud: Query<Entity, With<CenterHud>>) {
    let Ok(hud_entity) = hud.single() else {
        log::error!("CENTER HUD NOT FOUND - cannot spawn crosshair");
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
                parent
                    .spawn(Node {
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: Val::Px(72.0),
                        height: Val::Px(52.0),
                        ..default()
                    })
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
                        parent
                            .spawn((
                                BreakProgressBar,
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(2.0),
                                    width: Val::Px(46.0),
                                    height: Val::Px(4.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.65)),
                                Visibility::Hidden,
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    BreakProgressFill,
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.9, 0.9, 0.9, 0.95)),
                                ));
                            });
                    });
            });
    });
}

pub fn sync_break_progress_bar(
    progress: Res<BlockBreakProgress>,
    mut bar_query: Query<&mut Visibility, With<BreakProgressBar>>,
    mut fill_query: Query<&mut Node, With<BreakProgressFill>>,
) {
    let Ok(mut visibility) = bar_query.single_mut() else {
        return;
    };
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };

    if progress.visible && progress.progress > 0.0 {
        *visibility = Visibility::Visible;
        fill_node.width = Val::Percent(progress.progress.clamp(0.0, 1.0) * 100.0);
    } else {
        *visibility = Visibility::Hidden;
        fill_node.width = Val::Percent(0.0);
    }
}
