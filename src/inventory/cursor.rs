use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;

/// 鼠标悬浮物品
#[derive(Debug, Clone, Default)]
pub struct CursorData {
    /// 当前悬浮的物品堆叠，None 表示空手
    stack: Option<ItemStack>,
}

impl CursorData {
    /// 取出光标中的物品堆叠，光标变为空
    pub fn take_stack(&mut self) -> Option<ItemStack> {
        self.stack.take()
    }

    /// 设置光标中的物品堆叠。空堆叠等价于 clear
    pub fn set_stack(&mut self, stack: ItemStack) {
        if stack.is_empty() {
            self.stack = None;
        } else {
            self.stack = Some(stack);
        }
    }

    /// 获取光标中的物品堆叠只读引用
    pub fn stack(&self) -> Option<&ItemStack> {
        self.stack.as_ref()
    }

    /// 获取光标中的物品堆叠可变引用
    pub fn stack_mut(&mut self) -> Option<&mut ItemStack> {
        self.stack.as_mut()
    }

    /// 是否有物品
    pub fn has_item(&self) -> bool {
        self.stack.as_ref().map_or(false, |s| !s.is_empty())
    }

    /// 清除光标
    pub fn clear(&mut self) {
        self.stack = None;
    }
}

/// 此部分用于兼容旧的API
impl CursorData {
    /// 拾取物品（兼容旧 API，count=1）
    pub fn pick_up(&mut self, item_id: ItemId) {
        if item_id.is_air() {
            return;
        }
        self.stack = Some(ItemStack::single(item_id));
    }

    /// 放置物品，返回被放置的物品 ID（兼容旧 API）
    pub fn place(&mut self) -> Option<ItemId> {
        self.stack.take().map(|s| s.item)
    }

    /// 交换物品（兼容旧 API）
    pub fn swap(&mut self, new_item: ItemId) -> Option<ItemId> {
        let old = self.stack.take().map(|s| s.item);
        if !new_item.is_air() {
            self.stack = Some(ItemStack::single(new_item));
        }
        old
    }

    /// 获取物品引用（兼容旧 API）
    pub fn item(&self) -> Option<&ItemId> {
        self.stack.as_ref().map(|s| &s.item)
    }
}