//! 应用设置的数据模型与持久化入口。
//!
//! 本模块只定义设置契约和磁盘格式；设置如何作用到窗口、音量及世界流送，
//! 由 `app::flow` 的运行时系统统一协调。

mod keybinds;
mod keybinds_toml;
mod model;
mod persistence;

pub use keybinds::{
    BindingKey, KEY_ACTIONS, KeyAction, KeyActionSpec, Keybinds, action_label_localized, spec_of,
};
pub use keybinds_toml::{
    binding_display_localized, binding_key_name, keybinds_path, load_keybinds, load_keybinds_from,
    parse_binding_key, parse_keybinds_toml, save_keybinds, save_keybinds_to,
};
pub use model::GameSettings;
pub use persistence::{
    SETTINGS_DOCUMENT_FORMAT, load_settings, load_settings_from, restore_settings_backup,
    save_settings, save_settings_to, settings_backup_available, settings_file_exists,
    settings_path,
};
