//! 客户端视觉反馈的会话期资源。

use bevy::prelude::*;

/// 受击反馈状态：闪红剩余时间与镜头创伤强度。
#[derive(Resource, Default)]
pub(super) struct DamageFeedback {
    pub(super) flash_remaining: f32,
    pub(super) trauma: f32,
}

/// 顶部提示（如“背包已满”）的剩余显示时间。
#[derive(Resource, Default)]
pub(super) struct NoticeFeedback {
    pub(super) remaining: f32,
}
