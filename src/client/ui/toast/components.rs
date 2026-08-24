//! 定义 Toast 的标记组件与单条通知的生命周期状态。

use bevy::prelude::*;

/// Toast 堆叠容器标记；锚定屏幕右上角，Startup 常驻。
#[derive(Component, Debug, Default)]
pub struct ToastRoot;

/// 单条 Toast 的生命周期状态：显示计时到淡出，淡出结束由系统回收实体。
#[derive(Component, Debug)]
pub struct ToastItem {
    /// 完整显示阶段的倒计时。
    pub timer: Timer,
    /// 是否已进入淡出阶段。
    pub fading: bool,
    /// 淡出过渡的倒计时。
    pub fade_timer: Timer,
    /// 生成时的背景基准透明度；淡出按该基准等比衰减。
    pub base_bg_alpha: f32,
}

impl ToastItem {
    /// 以默认时长创建通知生命周期状态。
    pub fn new(visible_seconds: f32, fade_seconds: f32, base_bg_alpha: f32) -> Self {
        Self {
            timer: Timer::from_seconds(visible_seconds, TimerMode::Once),
            fading: false,
            fade_timer: Timer::from_seconds(fade_seconds, TimerMode::Once),
            base_bg_alpha,
        }
    }
}
