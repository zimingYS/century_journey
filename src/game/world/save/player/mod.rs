pub mod player_io;
pub mod player_manager;
pub mod player_model;

// Re-export commonly used types for backward compatibility
pub use player_manager::{
    PlayerSaveManager, auto_save_player_system, gamemode_dirty_tracking_system,
    inventory_dirty_tracking_system, load_player_on_enter_system, player_position_dirty_system,
    save_on_exit_system,
};
pub use player_model::{PlayerSaveData, SAVE_VERSION, SaveItemStack};
