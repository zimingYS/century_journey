pub mod click;
pub mod routing;
pub mod transfer;

pub use click::{left_click_slot, right_click_slot, shift_click};
pub use routing::{handle_slot_interaction, survival_index};
pub use transfer::{insert_into_container, insert_into_player, insert_into_range};
