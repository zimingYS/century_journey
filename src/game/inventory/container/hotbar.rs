use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;
use std::array;
use std::sync::LazyLock;

/// 快捷栏格子数
pub const HOTBAR_SIZE: usize = 9;

/// 快捷栏数据
/// 此部分仅定义数据，不定义UI状态
#[derive(Debug)]
pub struct HotbarData {
    /// 物品堆叠，None表示空格
    pub stacks: [Option<ItemStack>; HOTBAR_SIZE],
    /// 当前选中的格子
    pub active_index: usize,
}

/// 默认生成空快捷栏
impl Default for HotbarData {
    fn default() -> Self {
        Self {
            stacks: array::from_fn(|_| None),
            active_index: 0,
        }
    }
}

impl HotbarData {
    /// 选中当前物品
    pub fn active_stack(&self) -> &ItemStack {
        static EMPTY: LazyLock<ItemStack> = LazyLock::new(|| ItemStack::empty());
        self.stacks[self.active_index].as_ref().unwrap_or(&EMPTY)
    }

    /// 获取当前选中格子的可变物品堆叠
    pub fn active_stack_mut(&mut self) -> &mut Option<ItemStack> {
        &mut self.stacks[self.active_index]
    }

    /// 设置指定格子的物品堆叠
    pub fn set_stack(&mut self, index: usize, stack: ItemStack) {
        if index < HOTBAR_SIZE {
            if stack.is_empty() {
                self.stacks[index] = None;
            } else {
                self.stacks[index] = Some(stack);
            }
        }
    }

    /// 获取指定格子的物品堆叠引用
    pub fn get_stack(&self, index: usize) -> Option<&ItemStack> {
        self.stacks.get(index).and_then(|s| s.as_ref())
    }
}

// 兼容旧的API
impl HotbarData {
    /// 选中当前物品（兼容旧 API，返回 &ItemId）
    pub fn active_item(&self) -> &ItemId {
        static AIR: LazyLock<ItemId> = LazyLock::new(|| ItemId::air());
        self.stacks[self.active_index]
            .as_ref()
            .map(|s| &s.item)
            .unwrap_or(&AIR)
    }

    /// 设定指定物品格（兼容旧 API，count=1）
    pub fn set_item(&mut self, index: usize, item_id: ItemId) {
        if index < HOTBAR_SIZE {
            if item_id.is_air() {
                self.stacks[index] = None;
            } else {
                self.stacks[index] = Some(ItemStack::single(item_id));
            }
        }
    }

    /// 清空指定格（兼容旧 API）
    pub fn clear_slot(&mut self, index: usize) {
        if index < HOTBAR_SIZE {
            self.stacks[index] = None;
        }
    }

    /// 获取所有物品 ID 数组（兼容旧 UI 代码的 `.items.to_vec()` 模式）
    pub fn items(&self) -> [ItemId; HOTBAR_SIZE] {
        array::from_fn(|i| {
            self.stacks[i]
                .as_ref()
                .map_or(ItemId::air(), |s| s.item.clone())
        })
    }

    /// 设置所有物品（兼容旧 UI 代码的 `items = [...]` 模式）
    pub fn set_items(&mut self, ids: &[ItemId]) {
        for (i, id) in ids.iter().enumerate() {
            if i >= HOTBAR_SIZE {
                break;
            }
            self.set_item(i, id.clone());
        }
    }

    /// 向右循环切换
    pub fn select_next(&mut self) {
        self.active_index = (self.active_index + 1).rem_euclid(HOTBAR_SIZE);
    }

    /// 向左循环切换
    pub fn select_prev(&mut self) {
        self.active_index = (self.active_index + HOTBAR_SIZE - 1) % HOTBAR_SIZE;
    }

    /// 选择切换
    pub fn select_by_number(&mut self, number: usize) {
        if number < HOTBAR_SIZE {
            self.active_index = number;
        }
    }
}

impl InventoryContainer for HotbarData {
    fn slot_count(&self) -> usize {
        HOTBAR_SIZE
    }

    fn get_stack(&self, index: usize) -> Option<&crate::game::inventory::item::stack::ItemStack> {
        self.stacks.get(index).and_then(|s| s.as_ref())
    }

    fn get_stack_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut crate::game::inventory::item::stack::ItemStack> {
        self.stacks.get_mut(index).and_then(|s| s.as_mut())
    }

    fn set_stack(&mut self, index: usize, stack: crate::game::inventory::item::stack::ItemStack) {
        if index < HOTBAR_SIZE {
            if stack.is_empty() {
                self.stacks[index] = None;
            } else {
                self.stacks[index] = Some(stack);
            }
        }
    }
}
