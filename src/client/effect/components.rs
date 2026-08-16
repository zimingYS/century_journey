//! 客户端视觉反馈的标记组件。

use bevy::prelude::*;

/// 标记受击时覆盖全屏的红色闪屏节点。
#[derive(Component)]
pub(super) struct DamageFlashOverlay;

/// 标记顶部提示的容器节点。
#[derive(Component)]
pub(super) struct FeedbackNotice;

/// 标记顶部提示内的文本节点。
#[derive(Component)]
pub(super) struct FeedbackNoticeText;
