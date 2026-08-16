//! 客户端反馈音效的播放系统实现。
//!
//! 全部系统只消费消息与只读状态，生成一次性 `AudioPlayer` 实体；
//! 播放不会改变世界、物品栏或动画状态。

use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::app::flow::{DialogKind, DialogState};
use crate::client::player::model::animation::{
    AnimationMarkerEvent, AnimationMarkerKind, PlayerAnimationState, PlayerLocomotionState,
};
use crate::client::sound::resources::{
    AmbientClock, FeedbackAudioAssets, FootstepPlayback, SoundSequence,
};
use crate::client::ui::navigation::UiNavigation;
use crate::client::ui::widgets::common::UiControl;
use crate::client::ui::widgets::slot::InventorySlot;
use crate::content::block::registry::BlockRegistry;
use crate::content::block::sound::{BlockSoundEvent, SoundAction, SoundMaterial};
use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::player::identity::LocalPlayer;
use crate::game::player::interaction::targeting::TargetVoxel;
use crate::game::player::physics::components::PlayerGravity;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;

/// 播放 UI 控件的按下与悬停音效。
///
/// 组合过滤器明确限定可发声控件，避免所有按钮都触发同一音效。
#[allow(clippy::type_complexity)]
pub(super) fn ui_interaction_sound_system(
    query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            Or<(With<UiControl>, With<InventorySlot>)>,
        ),
    >,
    assets: Res<FeedbackAudioAssets>,
    mut commands: Commands,
) {
    for interaction in &query {
        match interaction {
            Interaction::Pressed => play_2d(&mut commands, assets.ui_click.clone(), 0.34, 1.0),
            Interaction::Hovered => play_2d(&mut commands, assets.ui_hover.clone(), 0.16, 1.0),
            Interaction::None => {}
        }
    }
}

/// 播放界面导航的打开与关闭音效。
pub(super) fn ui_navigation_sound_system(
    mut reader: MessageReader<UiNavigation>,
    assets: Res<FeedbackAudioAssets>,
    mut commands: Commands,
) {
    for navigation in reader.read() {
        let clip = match navigation {
            UiNavigation::Open(_) | UiNavigation::Replace(_) | UiNavigation::Reset(_) => {
                assets.ui_open.clone()
            }
            UiNavigation::Back | UiNavigation::Close(_) => assets.ui_close.clone(),
        };
        play_2d(&mut commands, clip, 0.38, 1.0);
    }
}

/// 播放错误对话框提示音效。
pub(super) fn dialog_sound_system(
    dialog: Res<DialogState>,
    assets: Res<FeedbackAudioAssets>,
    mut commands: Commands,
) {
    if !dialog.is_changed() {
        return;
    }
    if matches!(dialog.kind, Some(DialogKind::Error)) {
        play_2d(&mut commands, assets.ui_error.clone(), 0.58, 1.0);
    }
}

/// 播放库存满等反馈音效。
pub(super) fn inventory_feedback_sound_system(
    mut reader: MessageReader<InventoryFeedbackEvent>,
    assets: Res<FeedbackAudioAssets>,
    mut commands: Commands,
) {
    for event in reader.read() {
        match event {
            InventoryFeedbackEvent::Full => {
                play_2d(&mut commands, assets.inventory_full.clone(), 0.62, 1.0);
            }
        }
    }
}

/// 播放进入加载状态时的提示音。
pub(super) fn loading_sound_system(assets: Res<FeedbackAudioAssets>, mut commands: Commands) {
    play_2d(&mut commands, assets.ui_open.clone(), 0.32, 0.92);
}

/// 播放进入游戏时的确认音。
pub(super) fn game_ready_sound_system(assets: Res<FeedbackAudioAssets>, mut commands: Commands) {
    play_2d(&mut commands, assets.ui_confirm.clone(), 0.42, 1.0);
}

/// 播放方块交互（挖掘、放置、踩踏等）音效。
pub(super) fn block_sound_system(
    mut reader: MessageReader<BlockSoundEvent>,
    assets: Res<FeedbackAudioAssets>,
    mut sequence: ResMut<SoundSequence>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let direct_clip = match event.action {
            SoundAction::Interact => Some(assets.ui_click.clone()),
            SoundAction::Open => Some(assets.ui_open.clone()),
            SoundAction::Close => Some(assets.ui_close.clone()),
            _ => None,
        };
        let clip = direct_clip.or_else(|| {
            next_clip(
                clips_for_block(&assets, event.sound_material, event.action),
                &mut sequence,
            )
        });
        let Some(clip) = clip else {
            continue;
        };
        let action_volume = match event.action {
            SoundAction::Step => 0.52,
            SoundAction::Dig => 0.48,
            SoundAction::Place => 0.72,
            SoundAction::FallOn => 0.86,
            _ => 1.0,
        };
        play_spatial(
            &mut commands,
            clip,
            event.position + Vec3::splat(0.5),
            event.volume * action_volume,
            sequence.speed(),
        );
    }
}

fn clips_for_block(
    assets: &FeedbackAudioAssets,
    material: SoundMaterial,
    action: SoundAction,
) -> &[Handle<AudioSource>] {
    if matches!(action, SoundAction::Step | SoundAction::FallOn) {
        return step_clips(assets, material);
    }
    match material {
        SoundMaterial::Wood => &assets.block_wood,
        SoundMaterial::Metal => &assets.block_metal,
        SoundMaterial::Glass => &assets.block_glass,
        SoundMaterial::Dirt
        | SoundMaterial::Grass
        | SoundMaterial::Sand
        | SoundMaterial::Cloth
        | SoundMaterial::Snow
        | SoundMaterial::Water
        | SoundMaterial::Stone => &assets.block_mining,
    }
}

fn step_clips(assets: &FeedbackAudioAssets, material: SoundMaterial) -> &[Handle<AudioSource>] {
    match material {
        SoundMaterial::Grass => &assets.step_grass,
        SoundMaterial::Wood => &assets.step_wood,
        SoundMaterial::Snow => &assets.step_snow,
        SoundMaterial::Dirt | SoundMaterial::Sand | SoundMaterial::Cloth | SoundMaterial::Water => {
            &assets.step_soft
        }
        SoundMaterial::Stone | SoundMaterial::Metal | SoundMaterial::Glass => &assets.step_stone,
    }
}

/// 播放攻击命中与挖掘挥动等动画标记音效。
///
/// 动画标记音效需要同时读取目标、世界和多类内容映射，但不修改权威状态。
#[allow(clippy::too_many_arguments)]
pub(super) fn animation_marker_sound_system(
    mut reader: MessageReader<AnimationMarkerEvent>,
    target: Res<TargetVoxel>,
    world_state: Res<WorldState>,
    registry: Option<Res<BlockRegistry>>,
    player_query: Query<&GlobalTransform, With<LocalPlayer>>,
    assets: Res<FeedbackAudioAssets>,
    mut sequence: ResMut<SoundSequence>,
    mut commands: Commands,
) {
    for marker in reader.read() {
        match marker.marker {
            AnimationMarkerKind::AttackHit => {
                let Ok(transform) = player_query.get(marker.player) else {
                    continue;
                };
                let Some(clip) = next_clip(&assets.combat_hit, &mut sequence) else {
                    continue;
                };
                let position =
                    transform.translation() + transform.forward().as_vec3() * 1.2 + Vec3::Y;
                play_spatial(&mut commands, clip, position, 0.52, sequence.speed());
            }
            AnimationMarkerKind::MiningSwing => {
                let Some(hit) = target.result.as_ref() else {
                    continue;
                };
                let block_id = get_voxel_at_world(hit.hit_pos, &world_state);
                let material = registry
                    .as_deref()
                    .and_then(|registry| registry.get(block_id))
                    .map(|block| block.sound.sound_material)
                    .unwrap_or_default();
                let Some(clip) = next_clip(
                    clips_for_block(&assets, material, SoundAction::Dig),
                    &mut sequence,
                ) else {
                    continue;
                };
                play_spatial(
                    &mut commands,
                    clip,
                    hit.hit_pos.as_vec3() + Vec3::splat(0.5),
                    0.38,
                    sequence.speed(),
                );
            }
            AnimationMarkerKind::PlaceCommit | AnimationMarkerKind::UseCommit => {}
        }
    }
}

/// 根据步态相位与地面材质播放脚步声。
pub(super) fn footstep_sound_system(
    time: Res<Time>,
    world_state: Res<WorldState>,
    registry: Option<Res<BlockRegistry>>,
    query: Query<(&Transform, &PlayerGravity, &PlayerAnimationState), With<LocalPlayer>>,
    mut playback: Local<FootstepPlayback>,
    mut writer: MessageWriter<BlockSoundEvent>,
) {
    let Ok((transform, gravity, animation)) = query.single() else {
        return;
    };
    let phase_bucket =
        (animation.parameters.locomotion_phase / std::f32::consts::PI).floor() as i64;
    if !playback.initialized {
        playback.initialized = true;
        playback.phase_bucket = phase_bucket;
    }

    let material = || {
        let foot_pos = IVec3::new(
            transform.translation.x.floor() as i32,
            (transform.translation.y - 1.0).floor() as i32,
            transform.translation.z.floor() as i32,
        );
        let block_id = get_voxel_at_world(foot_pos, &world_state);
        registry
            .as_deref()
            .and_then(|registry| registry.get(block_id))
            .map(|block| block.sound.sound_material)
            .unwrap_or_default()
    };

    let locomotion_active = matches!(
        animation.lower_body.current,
        PlayerLocomotionState::Walk | PlayerLocomotionState::Run
    );
    if gravity.is_grounded
        && locomotion_active
        && animation.parameters.horizontal_speed > 0.15
        && phase_bucket != playback.phase_bucket
    {
        writer.write(BlockSoundEvent {
            position: transform.translation - Vec3::Y * 0.9,
            sound_material: material(),
            action: SoundAction::Step,
            volume: if animation.parameters.horizontal_speed > 11.0 {
                0.72
            } else {
                0.58
            },
        });
    }

    if gravity.is_grounded {
        if playback.airborne_seconds > 0.28 {
            writer.write(BlockSoundEvent {
                position: transform.translation - Vec3::Y * 0.9,
                sound_material: material(),
                action: SoundAction::FallOn,
                volume: (playback.airborne_seconds * 0.55).clamp(0.45, 1.0),
            });
        }
        playback.airborne_seconds = 0.0;
    } else {
        playback.airborne_seconds += time.delta_secs();
    }
    playback.phase_bucket = phase_bucket;
}

/// 以随机间隔播放环境音效。
pub(super) fn ambient_sound_system(
    time: Res<Time>,
    assets: Res<FeedbackAudioAssets>,
    mut sequence: ResMut<SoundSequence>,
    mut clock: ResMut<AmbientClock>,
    mut commands: Commands,
) {
    if !clock.timer.tick(time.delta()).just_finished() {
        return;
    }
    if let Some(clip) = next_clip(&assets.ambient, &mut sequence) {
        play_2d(
            &mut commands,
            clip,
            0.08,
            0.96 + (sequence.speed() - 1.0) * 0.5,
        );
    }
    let next_seconds = 11.0 + (sequence.0 % 9) as f32;
    clock
        .timer
        .set_duration(std::time::Duration::from_secs_f32(next_seconds));
    clock.timer.reset();
}

fn next_clip(
    clips: &[Handle<AudioSource>],
    sequence: &mut SoundSequence,
) -> Option<Handle<AudioSource>> {
    (!clips.is_empty()).then(|| clips[sequence.next_index(clips.len())].clone())
}

fn play_2d(commands: &mut Commands, clip: Handle<AudioSource>, volume: f32, speed: f32) {
    commands.spawn((
        AudioPlayer::new(clip),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0)))
            .with_speed(speed),
    ));
}

fn play_spatial(
    commands: &mut Commands,
    clip: Handle<AudioSource>,
    position: Vec3,
    volume: f32,
    speed: f32,
) {
    commands.spawn((
        AudioPlayer::new(clip),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0)))
            .with_speed(speed)
            .with_spatial(true),
        Transform::from_translation(position),
    ));
}
