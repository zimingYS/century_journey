use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;

/// 单个槽位对应的数据 （纯数据，非UI组件）
#[derive(Debug, Clone)]
pub struct SlotData {
    /// 槽位中的物品堆叠，None表示空
    pub stack: Option<ItemStack>,
}

/// 槽位交互动作类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotAction {
    /// 左键点击
    LeftClick,
    /// 右键点击
    RightClick,
    /// Shift + 左键点击
    ShiftClick,
    /// Q 丢弃 1 个
    DropOne,
    /// Shift+Q 丢弃整组
    DropAll,
}

impl Default for SlotData {
    fn default() -> Self {
        Self::empty()
    }
}

impl SlotData {
    /// 创建空槽位
    pub fn empty() -> Self {
        Self { stack: None }
    }

    /// 创建含指定物品的槽位
    pub fn with(item_id: ItemId) -> Self {
        Self {
            stack: Some(ItemStack::single(item_id)),
        }
    }

    /// 创建含指定堆叠的槽位
    pub fn with_stack(stack: ItemStack) -> Self {
        if stack.is_empty() {
            Self { stack: None }
        } else {
            Self { stack: Some(stack) }
        }
    }

    /// 槽位是否为空
    pub fn is_empty(&self) -> bool {
        self.stack.as_ref().is_none_or(|s| s.is_empty())
    }

    /// 槽位是否为满
    pub fn is_full(&self) -> bool {
        self.stack.as_ref().is_some_and(|s| s.is_full())
    }

    /// 获取到物品ID
    pub fn item_id(&self) -> ItemId {
        self.stack
            .as_ref()
            .map_or(ItemId::air(), |s| s.item.clone())
    }

    /// 获取堆叠只读引用
    pub fn get(&self) -> Option<&ItemStack> {
        self.stack.as_ref()
    }

    /// 获取堆叠可变引用
    pub fn get_mut(&mut self) -> Option<&mut ItemStack> {
        self.stack.as_mut()
    }

    /// 设置槽位中的物品堆叠
    pub fn set(&mut self, stack: ItemStack) {
        if stack.is_empty() {
            self.stack = None;
        } else {
            self.stack = Some(stack);
        }
    }

    /// 取出槽位中的所有物品，槽位变为空
    pub fn take(&mut self) -> Option<ItemStack> {
        self.stack.take()
    }

    /// 清空槽位
    pub fn clear(&mut self) {
        self.stack = None;
    }

    /// 将给定堆叠插入槽位
    pub fn insert(&mut self, incoming: ItemStack) -> Option<ItemStack> {
        if incoming.is_empty() {
            return None;
        }
        match &mut self.stack {
            None => {
                self.stack = Some(incoming);
                None
            }
            Some(existing) if existing.is_same_item(&incoming) => {
                let available = ItemStack::MAX_STACK_SIZE - existing.count;
                let to_move = available.min(incoming.count);
                existing.count += to_move;
                let remainder = incoming.count - to_move;
                if remainder > 0 {
                    Some(ItemStack::new(incoming.item.clone(), remainder))
                } else {
                    None
                }
            }
            Some(_) => {
                // 不同物品 → 交换
                let old = self.stack.take();
                self.stack = Some(incoming);
                old
            }
        }
    }

    /// 尝试从other合并尽可能多的物品到自身
    pub fn merge_from(&mut self, other: &mut ItemStack) -> u32 {
        match &mut self.stack {
            Some(existing) => existing.merge_from(other),
            None => {
                // 槽位空 → 整个移入
                let moved = other.count;
                self.stack = Some(other.clone());
                *other = ItemStack::empty();
                moved
            }
        }
    }

    /// 从槽位抽取指定数量的物品
    pub fn extract(&mut self, amount: u32) -> Option<ItemStack> {
        match &mut self.stack {
            Some(stack) => {
                let result = stack.take(amount);
                if stack.is_empty() {
                    self.stack = None;
                }
                result
            }
            None => None,
        }
    }

    /// 从槽位拿走一半
    pub fn split_half(&mut self) -> Option<ItemStack> {
        match &mut self.stack {
            Some(stack) => {
                let result = stack.split_half();
                if stack.is_empty() {
                    self.stack = None;
                }
                result
            }
            None => None,
        }
    }

    /// 从槽位拿走一个
    pub fn take_one(&mut self) -> Option<ItemStack> {
        self.extract(1)
    }

    /// 尝试往槽位放入一个同种物品
    pub fn add_one(&mut self, item: &ItemId) -> bool {
        match &mut self.stack {
            Some(existing)
                if &existing.item == item && existing.count < ItemStack::MAX_STACK_SIZE =>
            {
                existing.count += 1;
                true
            }
            None => {
                self.stack = Some(ItemStack::single(item.clone()));
                true
            }
            _ => false,
        }
    }
}
