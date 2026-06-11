use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;

/// 单个槽位对应的数据
#[derive(Debug, Clone, Default)]
pub struct SlotData {
    pub stack: Option<ItemStack>,
}

impl SlotData {
    pub fn empty() -> Self {
        Self { stack: None }
    }

    pub fn with(item_id: ItemId) -> Self {
        Self {
            stack: Some(ItemStack::single(item_id)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.as_ref().map_or(true, |s| s.is_empty())
    }

    pub fn item_id(&self) -> ItemId {
        self.stack.as_ref().map_or(ItemId::air(), |s| s.item_id.clone())
    }
}