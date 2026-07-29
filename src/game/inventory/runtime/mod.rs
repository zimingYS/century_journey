mod commands;
mod hotbar;
mod slot_interaction;

pub(super) use commands::handle_inventory_command_system;
pub(super) use hotbar::handle_hotbar_command_system;
pub(super) use slot_interaction::handle_slot_interaction_system;
