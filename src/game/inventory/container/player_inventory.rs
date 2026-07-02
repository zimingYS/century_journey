use crate::shared::item_id::ItemId;

/// 生存模式玩家背包
/// TODO（未来扩展）
pub struct PlayerInventoryData {
    pub main_slots: [ItemId; 27],
    pub hotbar_slots: [ItemId; 9],
    pub selected_hotbar: usize,
}
