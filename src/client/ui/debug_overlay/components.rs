//! 定义调试浮层的标记组件与开关状态。

use bevy::prelude::*;

/// 调试浮层根节点标记；控制整块浮层的显隐。
#[derive(Component, Debug, Default)]
pub struct DebugOverlayRoot;

/// 调试浮层正文文本节点标记；每帧由文本刷新系统整行重写。
#[derive(Component, Debug, Default)]
pub struct DebugOverlayText;

/// 调试浮层开关与帧率平滑状态。
#[derive(Resource, Debug)]
pub struct DebugOverlayState {
    /// 是否显示调试浮层；游戏内按 F3 切换。
    pub visible: bool,
    /// 帧率指数滑动平均（FPS）；尚未采样时为 0。
    pub fps_ema: f32,
    /// 单帧耗时指数滑动平均（毫秒）。
    pub frame_ms_ema: f32,
}

impl Default for DebugOverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
        }
    }
}
