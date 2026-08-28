//! 提供槽位组件、实体生成和视觉同步的统一入口。

mod durability;
mod hotbar;
mod icon;
mod spawn;

pub mod components;

pub use crate::client::ui::state::SearchInputState;
pub use crate::game::inventory::slot::SlotKind;
pub use components::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SlotCountText,
    SlotDurabilityBar, SlotDurabilityFill, SlotIcon, SlotInteractionEvent, SlotPlaceholder,
    SlotVisual,
};
pub use durability::sync_slot_durability_system;
pub use hotbar::sync_hotbar_panel_visuals;
pub use icon::{resolve_item_image_node, sync_slot_icon};
pub use spawn::{
    spawn_display_only_slot, spawn_empty_slot, spawn_empty_slot_with_placeholder,
    spawn_slot_with_item,
};
