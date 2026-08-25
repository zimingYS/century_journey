//! 可持久化设置资源的数据模型。

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// 玩家可持久化的客户端运行设置。
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GameSettings {
    /// 水平区块加载与渲染距离。
    pub render_distance: u32,
    /// 全局主音量，取值范围为 `0.0..=1.0`。
    pub master_volume: f32,
    /// 鼠标视角灵敏度倍率。
    pub mouse_sensitivity: f32,
    /// UI 整体缩放倍率。
    pub ui_scale: f32,
    /// 是否使用当前显示器的无边框全屏模式。
    pub fullscreen: bool,
    /// 是否启用垂直同步。
    pub vsync: bool,
    /// 界面语言标识，对应 `assets/locales/` 中的语言文件名。
    pub language: String,
}

/// 默认界面语言。
fn default_language() -> String {
    "zh-CN".to_string()
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            master_volume: 1.0,
            mouse_sensitivity: 1.0,
            ui_scale: 1.0,
            fullscreen: false,
            vsync: true,
            language: default_language(),
        }
    }
}
