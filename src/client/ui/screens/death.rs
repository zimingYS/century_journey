//! 构建死亡界面，并把重生和返回操作转换为领域请求。

use crate::client::ui::localization::LocalizedText;
use crate::client::ui::resources::ui_font::UiFont;
use crate::engine::localization::Localization;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::{PlayerLifeState, PlayerLifecycle};
use crate::game::player::lifecycle::events::RespawnRequest;
use crate::game::player::lifecycle::rules::LastDeathInfo;
use bevy::prelude::*;

/// 死亡屏幕根节点。
#[derive(Component)]
pub struct DeathScreenRoot;

/// 显示玩家死亡原因的文本节点。
#[derive(Component)]
pub struct DeathReasonText;

/// 显示死亡掉落规则摘要的文本节点。
#[derive(Component)]
pub struct DeathDropText;

/// 请求玩家重生的按钮。
#[derive(Component)]
pub struct RespawnButton;

/// 创建默认隐藏的死亡屏幕。
pub fn spawn_death_screen_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    localization: Res<Localization>,
) {
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
                LocalizedText::new("death.title"),
                Text::new(localization.get("death.title")),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(52.0),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.94, 0.91)),
            ));
            root.spawn((
                DeathReasonText,
                Text::new(""),
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
                    LocalizedText::new("death.respawn"),
                    Text::new(localization.get("death.respawn")),
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

/// 根据玩家生命周期和最近死亡信息同步死亡屏幕内容。
pub fn sync_death_screen_system(
    player_query: Query<&PlayerLifecycle, With<Player>>,
    last_death: Res<LastDeathInfo>,
    localization: Res<Localization>,
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
            localization.get("death.respawning").to_owned()
        } else {
            let cause = last_death.source.map_or_else(
                || localization.get("death.unknown"),
                |source| localization.get(source.cause_key()),
            );
            localization.format("death.reason", &[("cause", cause)])
        });
    }
    for mut text in &mut drop_query {
        *text = Text::new(localization.format(
            "death.drops",
            &[("count", &last_death.dropped_stacks.to_string())],
        ));
    }
}

/// 将重生按钮点击转换为权威重生请求。
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
