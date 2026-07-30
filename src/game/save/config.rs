//! 定义自动保存间隔及当前存档会话使用的路径配置。

use bevy::prelude::{Resource, Timer};

/// 未显式选择存档时使用的世界名称。
const DEFAULT_WORLD_NAME: &str = "NEW WORLD";
/// 自动保存的默认现实时间间隔。
const DEFAULT_AUTO_SAVE_INTERVAL_SECONDS: f64 = 60.0;

/// 保存配置
#[derive(Resource, Clone, Debug)]
pub struct SaveConfig {
    /// 存档名称
    pub world_name: String,
    /// 是否在区块卸载时自动保存
    pub save_on_unload: bool,
    /// 自动全量保存间隔（秒），0 = 禁用
    pub auto_save_interval: f64,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            world_name: DEFAULT_WORLD_NAME.to_string(),
            save_on_unload: true,
            auto_save_interval: DEFAULT_AUTO_SAVE_INTERVAL_SECONDS,
        }
    }
}

/// 自动保存计时器
#[derive(Resource, Default, Debug)]
pub struct AutoSaveTimer {
    pub timer: Option<Timer>,
    pub elapsed: f64,
}
