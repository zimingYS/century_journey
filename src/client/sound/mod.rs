//! 客户端声音反馈。
//!
//! 该模块只消费游戏消息和只读状态；声音播放不会改变世界、物品栏或动画状态。

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::app::flow::{DialogKind, DialogState};
use crate::client::ui::navigation::UiNavigation;
use crate::client::ui::widgets::common::UiControl;
use crate::client::ui::widgets::slot::InventorySlot;
use crate::content::block::registry::BlockRegistry;
use crate::content::block::sound::{BlockSoundEvent, SoundAction, SoundMaterial};
use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::player::components::{LocalPlayer, PlayerGravity};
use crate::game::player::model::animation::{
    AnimationMarkerEvent, AnimationMarkerKind, PlayerAnimationState, PlayerLocomotionState,
};
use crate::game::player::systems::raycast::TargetVoxel;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::storage::WorldStorage;
use crate::shared::states::AppState;

#[derive(Resource)]
struct FeedbackAudioAssets {
    ui_click: Handle<AudioSource>,
    ui_hover: Handle<AudioSource>,
    ui_confirm: Handle<AudioSource>,
    ui_error: Handle<AudioSource>,
    inventory_full: Handle<AudioSource>,
    ui_open: Handle<AudioSource>,
    ui_close: Handle<AudioSource>,
    block_mining: Vec<Handle<AudioSource>>,
    block_wood: Vec<Handle<AudioSource>>,
    block_metal: Vec<Handle<AudioSource>>,
    block_glass: Vec<Handle<AudioSource>>,
    step_grass: Vec<Handle<AudioSource>>,
    step_stone: Vec<Handle<AudioSource>>,
    step_wood: Vec<Handle<AudioSource>>,
    step_snow: Vec<Handle<AudioSource>>,
    step_soft: Vec<Handle<AudioSource>>,
    combat_hit: Vec<Handle<AudioSource>>,
    ambient: Vec<Handle<AudioSource>>,
}

impl FromWorld for FeedbackAudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ui_click: assets.load("sounds/kenney/ui/click.ogg"),
            ui_hover: assets.load("sounds/kenney/ui/hover.ogg"),
            ui_confirm: assets.load("sounds/kenney/ui/confirm.ogg"),
            ui_error: assets.load("sounds/kenney/ui/error.ogg"),
            inventory_full: assets.load("sounds/kenney/ui/inventory_full.ogg"),
            ui_open: assets.load("sounds/kenney/ui/open.ogg"),
            ui_close: assets.load("sounds/kenney/ui/close.ogg"),
            block_mining: load_series(assets, "sounds/kenney/block/mining", 3),
            block_wood: load_series(assets, "sounds/kenney/block/wood", 3),
            block_metal: load_series(assets, "sounds/kenney/block/metal", 2),
            block_glass: load_series(assets, "sounds/kenney/block/glass", 2),
            step_grass: load_series(assets, "sounds/kenney/footstep/grass", 3),
            step_stone: load_series(assets, "sounds/kenney/footstep/stone", 3),
            step_wood: load_series(assets, "sounds/kenney/footstep/wood", 3),
            step_snow: load_series(assets, "sounds/kenney/footstep/snow", 3),
            step_soft: load_series(assets, "sounds/kenney/footstep/soft", 3),
            combat_hit: load_series(assets, "sounds/kenney/combat/hit", 3),
            ambient: vec![
                assets.load("sounds/kenney/ambient/creak.ogg"),
                assets.load("sounds/kenney/ambient/rustle.ogg"),
            ],
        }
    }
}

fn load_series(assets: &AssetServer, stem: &str, count: usize) -> Vec<Handle<AudioSource>> {
    (0..count)
        .map(|index| assets.load(format!("{stem}_{index}.ogg")))
        .collect()
}

#[derive(Resource, Default)]
struct SoundSequence(u64);

impl SoundSequence {
    fn next_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0 = self.0.wrapping_add(1);
        (self.0 as usize) % len
    }

    fn speed(&self) -> f32 {
        0.94 + ((self.0.wrapping_mul(37) % 13) as f32 * 0.01)
    }
}

#[derive(Resource)]
struct AmbientClock {
    timer: Timer,
}

impl Default for AmbientClock {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

#[derive(Default)]
struct FootstepPlayback {
    initialized: bool,
    phase_bucket: i64,
    airborne_seconds: f32,
}

pub struct ClientSoundPlugin;

impl Plugin for ClientSoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackAudioAssets>()
            .init_resource::<SoundSequence>()
            .init_resource::<AmbientClock>()
            .add_systems(
                Update,
                (
                    ui_interaction_sound_system,
                    ui_navigation_sound_system,
                    dialog_sound_system,
                    inventory_feedback_sound_system,
                ),
            )
            .add_systems(
                Update,
                (
                    block_sound_system,
                    animation_marker_sound_system,
                    footstep_sound_system,
                    ambient_sound_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::Loading), loading_sound_system)
            .add_systems(OnEnter(AppState::WorldLoading), loading_sound_system)
            .add_systems(OnEnter(AppState::InGame), game_ready_sound_system);
    }
}

fn ui_interaction_sound_system(
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

fn ui_navigation_sound_system(
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

fn dialog_sound_system(
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

fn inventory_feedback_sound_system(
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

fn loading_sound_system(assets: Res<FeedbackAudioAssets>, mut commands: Commands) {
    play_2d(&mut commands, assets.ui_open.clone(), 0.32, 0.92);
}

fn game_ready_sound_system(assets: Res<FeedbackAudioAssets>, mut commands: Commands) {
    play_2d(&mut commands, assets.ui_confirm.clone(), 0.42, 1.0);
}

fn block_sound_system(
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

fn animation_marker_sound_system(
    mut reader: MessageReader<AnimationMarkerEvent>,
    target: Res<TargetVoxel>,
    world: Res<WorldStorage>,
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
                let block_id = get_voxel_at_world(hit.hit_pos, &world);
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

fn footstep_sound_system(
    time: Res<Time>,
    world: Res<WorldStorage>,
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
        let block_id = get_voxel_at_world(foot_pos, &world);
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

fn ambient_sound_system(
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
