//! 客户端反馈音效资源与播放状态。

use bevy::audio::AudioSource;
use bevy::prelude::*;

/// 已加载的反馈音效资产集合。
///
/// 所有音频句柄在进入应用时一次性加载；播放系统只克隆句柄生成一次性实体，
/// 不改变权威世界或界面状态。
#[derive(Resource)]
pub(super) struct FeedbackAudioAssets {
    pub(super) ui_click: Handle<AudioSource>,
    pub(super) ui_hover: Handle<AudioSource>,
    pub(super) ui_confirm: Handle<AudioSource>,
    pub(super) ui_error: Handle<AudioSource>,
    pub(super) inventory_full: Handle<AudioSource>,
    pub(super) ui_open: Handle<AudioSource>,
    pub(super) ui_close: Handle<AudioSource>,
    pub(super) block_mining: Vec<Handle<AudioSource>>,
    pub(super) block_wood: Vec<Handle<AudioSource>>,
    pub(super) block_metal: Vec<Handle<AudioSource>>,
    pub(super) block_glass: Vec<Handle<AudioSource>>,
    pub(super) step_grass: Vec<Handle<AudioSource>>,
    pub(super) step_stone: Vec<Handle<AudioSource>>,
    pub(super) step_wood: Vec<Handle<AudioSource>>,
    pub(super) step_snow: Vec<Handle<AudioSource>>,
    pub(super) step_soft: Vec<Handle<AudioSource>>,
    pub(super) combat_hit: Vec<Handle<AudioSource>>,
    pub(super) ambient: Vec<Handle<AudioSource>>,
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

/// 按 `{stem}_{index}.ogg` 约定加载一组连续编号音效。
fn load_series(assets: &AssetServer, stem: &str, count: usize) -> Vec<Handle<AudioSource>> {
    (0..count)
        .map(|index| assets.load(format!("{stem}_{index}.ogg")))
        .collect()
}

/// 同一类音效内部的轮换序列与轻微变速，避免重复播放听感单调。
#[derive(Resource, Default)]
pub(super) struct SoundSequence(pub(super) u64);

impl SoundSequence {
    /// 返回下一次播放使用的索引，并在序列内循环。
    pub(super) fn next_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0 = self.0.wrapping_add(1);
        (self.0 as usize) % len
    }

    /// 返回当前序列对应的轻微变速倍率。
    pub(super) fn speed(&self) -> f32 {
        0.94 + ((self.0.wrapping_mul(37) % 13) as f32 * 0.01)
    }
}

/// 环境音效的随机间隔计时器。
#[derive(Resource)]
pub(super) struct AmbientClock {
    pub(super) timer: Timer,
}

impl Default for AmbientClock {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

/// 脚步声播放的跨帧状态：首次相位校准、空中累计与落地音量。
#[derive(Default)]
pub(super) struct FootstepPlayback {
    pub(super) initialized: bool,
    pub(super) phase_bucket: i64,
    pub(super) airborne_seconds: f32,
}
