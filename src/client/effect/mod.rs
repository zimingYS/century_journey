//! 屏幕、镜头与方块交互视觉反馈。

use bevy::prelude::*;

use crate::client::ui::resources::ui_font::UiFont;
use crate::game::gameplay::block_action::BlockBreakProgress;
use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::player::components::LocalPlayer;
use crate::game::player::events::DamageEvent;
use crate::shared::components::FpsCamera;
use crate::shared::states::AppState;

const DAMAGE_FLASH_SECONDS: f32 = 0.34;
const NOTICE_SECONDS: f32 = 2.2;

#[derive(Resource, Default)]
struct DamageFeedback {
    flash_remaining: f32,
    trauma: f32,
}

#[derive(Resource, Default)]
struct NoticeFeedback {
    remaining: f32,
}

#[derive(Component)]
struct DamageFlashOverlay;

#[derive(Component)]
struct FeedbackNotice;

#[derive(Component)]
struct FeedbackNoticeText;

pub struct ClientEffectPlugin;

impl Plugin for ClientEffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DamageFeedback>()
            .init_resource::<NoticeFeedback>()
            .add_systems(
                Startup,
                spawn_feedback_ui_system
                    .after(crate::client::ui::resources::ui_font::load_ui_font_system),
            )
            .add_systems(
                Update,
                (
                    receive_damage_feedback_system,
                    receive_notice_feedback_system,
                    update_feedback_ui_system,
                    draw_break_cracks_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), clear_feedback_on_exit_system)
            .add_systems(
                PostUpdate,
                camera_shake_system
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn clear_feedback_on_exit_system(
    mut damage: ResMut<DamageFeedback>,
    mut notice: ResMut<NoticeFeedback>,
    mut flash_query: Query<&mut Visibility, With<DamageFlashOverlay>>,
    mut notice_query: Query<&mut Visibility, (With<FeedbackNotice>, Without<DamageFlashOverlay>)>,
) {
    *damage = DamageFeedback::default();
    *notice = NoticeFeedback::default();
    for mut visibility in &mut flash_query {
        *visibility = Visibility::Hidden;
    }
    for mut visibility in &mut notice_query {
        *visibility = Visibility::Hidden;
    }
}

fn spawn_feedback_ui_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    existing_flash: Query<Entity, With<DamageFlashOverlay>>,
) {
    if !existing_flash.is_empty() {
        return;
    }

    commands.spawn((
        Name::new("DamageFlashOverlay"),
        DamageFlashOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(30_000),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));

    commands
        .spawn((
            Name::new("FeedbackNotice"),
            FeedbackNotice,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(28.0),
                left: Val::Percent(50.0),
                width: Val::Px(220.0),
                min_height: Val::Px(42.0),
                margin: UiRect::left(Val::Px(-110.0)),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(9.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.0)),
            BorderColor::all(Color::srgba(0.82, 0.30, 0.18, 0.0)),
            GlobalZIndex(30_100),
            Pickable::IGNORE,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                FeedbackNoticeText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::NONE),
                Pickable::IGNORE,
            ));
        });
}

fn receive_damage_feedback_system(
    mut reader: MessageReader<DamageEvent>,
    local_player: Query<Entity, With<LocalPlayer>>,
    mut feedback: ResMut<DamageFeedback>,
) {
    let Ok(local_player) = local_player.single() else {
        return;
    };
    for event in reader.read() {
        if event.target != local_player {
            continue;
        }
        let intensity = (event.amount / 6.0).clamp(0.35, 1.0);
        feedback.flash_remaining = DAMAGE_FLASH_SECONDS;
        feedback.trauma = feedback.trauma.max(intensity);
    }
}

fn receive_notice_feedback_system(
    mut reader: MessageReader<InventoryFeedbackEvent>,
    mut feedback: ResMut<NoticeFeedback>,
    mut text_query: Query<&mut Text, With<FeedbackNoticeText>>,
) {
    for event in reader.read() {
        match event {
            InventoryFeedbackEvent::Full => {
                feedback.remaining = NOTICE_SECONDS;
                for mut text in &mut text_query {
                    **text = "背包已满".into();
                }
            }
        }
    }
}

fn update_feedback_ui_system(
    time: Res<Time>,
    mut damage: ResMut<DamageFeedback>,
    mut notice: ResMut<NoticeFeedback>,
    mut flash_query: Query<
        (&mut BackgroundColor, &mut Visibility),
        (With<DamageFlashOverlay>, Without<FeedbackNotice>),
    >,
    mut notice_query: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Visibility,
        ),
        With<FeedbackNotice>,
    >,
    mut text_query: Query<&mut TextColor, With<FeedbackNoticeText>>,
) {
    let delta = time.delta_secs();
    damage.flash_remaining = (damage.flash_remaining - delta).max(0.0);
    let flash_strength = (damage.flash_remaining / DAMAGE_FLASH_SECONDS).clamp(0.0, 1.0);
    for (mut color, mut visibility) in &mut flash_query {
        *visibility = if flash_strength > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        color.0 = Color::srgba(0.78, 0.025, 0.015, flash_strength * 0.28);
    }

    notice.remaining = (notice.remaining - delta).max(0.0);
    let elapsed = NOTICE_SECONDS - notice.remaining;
    let fade_in = (elapsed / 0.18).clamp(0.0, 1.0);
    let fade_out = (notice.remaining / 0.30).clamp(0.0, 1.0);
    let alpha = fade_in.min(fade_out);
    for (mut node, mut background, mut border, mut visibility) in &mut notice_query {
        *visibility = if notice.remaining > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.top = Val::Px(28.0 + (1.0 - fade_in) * -8.0);
        background.0 = Color::srgba(0.045, 0.052, 0.06, alpha * 0.94);
        *border = BorderColor::all(Color::srgba(0.88, 0.31, 0.16, alpha));
    }
    for mut color in &mut text_query {
        color.0 = Color::srgba(1.0, 0.92, 0.86, alpha);
    }
}

fn camera_shake_system(
    time: Res<Time>,
    mut feedback: ResMut<DamageFeedback>,
    mut camera_query: Query<&mut Transform, With<FpsCamera>>,
) {
    if feedback.trauma <= 0.001 {
        feedback.trauma = 0.0;
        return;
    }
    let strength = feedback.trauma * feedback.trauma;
    let phase = time.elapsed_secs();
    let offset = Vec3::new(
        (phase * 43.0).sin() * 0.045,
        (phase * 57.0 + 0.8).sin() * 0.035,
        (phase * 31.0 + 1.9).sin() * 0.018,
    ) * strength;
    for mut transform in &mut camera_query {
        transform.translation += offset;
    }
    feedback.trauma = (feedback.trauma - time.delta_secs() * 2.4).max(0.0);
}

fn draw_break_cracks_system(progress: Res<BlockBreakProgress>, mut gizmos: Gizmos) {
    if !progress.visible || progress.progress <= 0.01 {
        return;
    }
    let center = progress.world_pos.as_vec3() + Vec3::splat(0.5);
    let stage = (progress.progress * 5.0).ceil().clamp(1.0, 5.0) as usize;
    let color = Color::srgba(0.025, 0.02, 0.015, 0.92);
    let segments = [
        (Vec2::ZERO, Vec2::new(0.18, 0.10)),
        (Vec2::new(0.18, 0.10), Vec2::new(0.34, 0.25)),
        (Vec2::new(0.18, 0.10), Vec2::new(0.31, -0.08)),
        (Vec2::ZERO, Vec2::new(-0.16, 0.17)),
        (Vec2::new(-0.16, 0.17), Vec2::new(-0.31, 0.34)),
        (Vec2::new(-0.16, 0.17), Vec2::new(-0.37, 0.03)),
        (Vec2::ZERO, Vec2::new(-0.08, -0.20)),
        (Vec2::new(-0.08, -0.20), Vec2::new(0.05, -0.39)),
        (Vec2::new(-0.08, -0.20), Vec2::new(-0.29, -0.34)),
        (Vec2::new(0.31, -0.08), Vec2::new(0.42, -0.22)),
    ];
    let visible_segments = (stage * 2).min(segments.len());
    for face in 0..6 {
        for (from, to) in segments.iter().take(visible_segments) {
            gizmos.line(
                crack_point(center, face, *from),
                crack_point(center, face, *to),
                color,
            );
        }
    }
}

fn crack_point(center: Vec3, face: usize, point: Vec2) -> Vec3 {
    const SURFACE: f32 = 0.506;
    match face {
        0 => center + Vec3::new(SURFACE, point.y, point.x),
        1 => center + Vec3::new(-SURFACE, point.y, -point.x),
        2 => center + Vec3::new(point.x, SURFACE, point.y),
        3 => center + Vec3::new(point.x, -SURFACE, -point.y),
        4 => center + Vec3::new(point.x, point.y, SURFACE),
        _ => center + Vec3::new(-point.x, point.y, -SURFACE),
    }
}
