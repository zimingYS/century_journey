pub mod data;
mod io;
mod plugin;
pub mod runtime;

pub use data::model::{PlayerSaveData, SAVE_VERSION, SaveItemStack};
pub use io::{
    player_backup_available, player_save_path, read_player_backup, read_player_data,
    restore_player_backup,
};
pub(super) use plugin::PlayerSavePlugin;
pub use runtime::manager::PlayerSaveManager;
pub use runtime::write::save_player_now;
