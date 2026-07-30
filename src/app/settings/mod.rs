//! 应用设置的数据模型与持久化入口。
//!
//! 本模块只定义设置契约和磁盘格式；设置如何作用到窗口、音量及世界流送，
//! 由 `app::flow` 的运行时系统统一协调。

mod model;
mod persistence;

pub use model::GameSettings;
pub use persistence::{
    SETTINGS_FORMAT_VERSION, load_settings, load_settings_from, restore_settings_backup,
    save_settings, save_settings_to, settings_backup_available, settings_path,
};
