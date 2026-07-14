use crate::client::ui::resources::ui_font::UiFont;
use crate::game::player::components::{Player, PlayerLifeState, PlayerLifecycle};
use crate::game::player::events::RespawnRequest;
use crate::game::player::systems::combat::LastDeathInfo;
use bevy::prelude::*;

#[derive(Component)]
pub struct DeathScreenRoot;

#[derive(Component)]
pub struct DeathReasonText;

#[derive(Component)]
pub struct DeathDropText;

#[derive(Component)]
pub struct RespawnButton;

pub fn spawn_death_screen_system(mut commands: Commands, ui_font: Res<UiFont>) {
    commands
        .spawn((
            DeathScreenRoot,
            Name::new("DeathScreen"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.015, 0.015, 0.82)),
            GlobalZIndex(5_000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("你死了"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(52.0),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.94, 0.91)),
            ));
            root.spawn((
                DeathReasonText,
                Text::new("死因：未知"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.82, 0.78)),
            ));
            root.spawn((
                DeathDropText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.69, 0.65)),
            ));
            root.spawn((
                RespawnButton,
                Button,
                Node {
                    width: Val::Px(190.0),
                    height: Val::Px(48.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.22, 0.24, 0.25)),
                BorderColor::all(Color::srgb(0.62, 0.64, 0.63)),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("重生"),
                    TextFont {
                        font: FontSource::from(ui_font.default.clone()),
                        font_size: FontSize::Px(22.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

pub fn sync_death_screen_system(
    player_query: Query<&PlayerLifecycle, With<Player>>,
    last_death: Res<LastDeathInfo>,
    mut root_query: Query<&mut Visibility, (With<DeathScreenRoot>, Without<RespawnButton>)>,
    mut button_query: Query<&mut Visibility, (With<RespawnButton>, Without<DeathScreenRoot>)>,
    mut reason_query: Query<&mut Text, (With<DeathReasonText>, Without<DeathDropText>)>,
    mut drop_query: Query<&mut Text, (With<DeathDropText>, Without<DeathReasonText>)>,
) {
    let state = player_query
        .single()
        .map(|lifecycle| lifecycle.state)
        .unwrap_or(PlayerLifeState::Alive);
    let visible = state != PlayerLifeState::Alive;
    for mut visibility in &mut root_query {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut visibility in &mut button_query {
        *visibility = if state == PlayerLifeState::Dead {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut reason_query {
        *text = Text::new(if state == PlayerLifeState::Respawning {
            "正在重生...".to_string()
        } else {
            format!(
                "死因：{}",
                last_death
                    .source
                    .map_or("未知", |source| source.display_name())
            )
        });
    }
    for mut text in &mut drop_query {
        *text = Text::new(format!("死亡掉落：{} 组物品", last_death.dropped_stacks));
    }
}

pub fn respawn_button_system(
    button_query: Query<&Interaction, (Changed<Interaction>, With<RespawnButton>)>,
    player_query: Query<(Entity, &PlayerLifecycle), With<Player>>,
    mut writer: MessageWriter<RespawnRequest>,
) {
    if !button_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let Ok((entity, lifecycle)) = player_query.single() else {
        return;
    };
    if lifecycle.state == PlayerLifeState::Dead {
        writer.write(RespawnRequest { entity });
    }
}
