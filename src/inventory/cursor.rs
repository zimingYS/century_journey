use crate::inventory::item::id::ItemId;

/// 鼠标悬浮物品
#[derive(Debug, Clone, Default)]
pub struct CursorData {
    /// 当前悬浮的物品，None 表示空手
    pub item: Option<ItemId>,
}

impl CursorData {
    /// 拾取物品
    pub fn pick_up(&mut self, item_id: ItemId) {
        self.item = Some(item_id);
    }

    /// 放置物品，返回被放置的物品
    pub fn place(&mut self) -> Option<ItemId> {
        self.item.take()
    }

    /// 交换
    pub fn swap(&mut self, new_item: ItemId) -> Option<ItemId> {
        let old = self.item.take();
        self.item = Some(new_item);
        old
    }

    /// 清除
    pub fn clear(&mut self) {
        self.item = None;
    }

    /// 是否有物品
    pub fn has_item(&self) -> bool {
        self.item.as_ref().map_or(false, |id| !id.is_air())
    }

    /// 获取物品引用
    pub fn item(&self) -> Option<&ItemId> {
        self.item.as_ref()
    }
}