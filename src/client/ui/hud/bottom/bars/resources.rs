//! 底部状态条的资源与显示状态枚举。

use bevy::prelude::*;

/// HUD 状态图标缓存。
///
/// 这里统一保存生命值和饥饿值的 full / half / empty 图标句柄，避免同步系统反复拼路径或直接加载资源。
#[derive(Resource, Default, Clone)]
pub struct HudStatusIconAssets {
    /// 满生命图标。
    pub(super) heart_full: Handle<Image>,
    /// 半生命图标。
    pub(super) heart_half: Handle<Image>,
    /// 空生命图标。
    pub(super) heart_empty: Handle<Image>,
    /// 满饥饿值图标。
    pub(super) hunger_full: Handle<Image>,
    /// 半饥饿值图标。
    pub(super) hunger_half: Handle<Image>,
    /// 空饥饿值图标。
    pub(super) hunger_empty: Handle<Image>,
    /// 满饮水值图标。
    pub(super) thirst_full: Handle<Image>,
    /// 半饮水值图标。
    pub(super) thirst_half: Handle<Image>,
    /// 空饮水值图标。
    pub(super) thirst_empty: Handle<Image>,
}

impl HudStatusIconAssets {
    /// 根据生命格状态取得对应图片。
    pub fn heart_icon(&self, segment: StatusIconSegment) -> Handle<Image> {
        match segment {
            StatusIconSegment::Full => self.heart_full.clone(),
            StatusIconSegment::Half => self.heart_half.clone(),
            StatusIconSegment::Empty => self.heart_empty.clone(),
        }
    }

    /// 根据饥饿格状态取得对应图片。
    pub fn hunger_icon(&self, segment: StatusIconSegment) -> Handle<Image> {
        match segment {
            StatusIconSegment::Full => self.hunger_full.clone(),
            StatusIconSegment::Half => self.hunger_half.clone(),
            StatusIconSegment::Empty => self.hunger_empty.clone(),
        }
    }

    /// 根据饮水格状态取得对应图片。
    pub fn thirst_icon(&self, segment: StatusIconSegment) -> Handle<Image> {
        match segment {
            StatusIconSegment::Full => self.thirst_full.clone(),
            StatusIconSegment::Half => self.thirst_half.clone(),
            StatusIconSegment::Empty => self.thirst_empty.clone(),
        }
    }
}

/// 单个状态格的显示状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIconSegment {
    /// 该格为空。
    Empty,
    /// 该格显示半格。
    Half,
    /// 该格显示满格。
    Full,
}
