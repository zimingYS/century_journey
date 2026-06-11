use std::array;
use crate::inventory::item::id::ItemId;

/// 快捷栏格子数
pub const HOTBAR_SIZE: usize = 9;

/// 快捷栏数据
/// 此部分仅定义数据，不定义UI状态
#[derive(Debug)]
pub struct HotbarData{
    /// 物品格
    pub items: [ItemId; HOTBAR_SIZE],
    /// 当前选中的格子
    pub active_index: usize,
}

/// 默认生成空快捷栏
impl Default for HotbarData {
    fn default() -> Self {
        Self {
            items: array::from_fn(|_| ItemId::air()),
            active_index: 0,
        }
    }
}

impl HotbarData{
    /// 选中当前物品
    pub fn active_item(&self) -> &ItemId{
        &self.items[self.active_index]
    }

    /// 设定指定物品格
    pub fn set_item(&mut self, index: usize, item_id: ItemId){
        if index < HOTBAR_SIZE {
            self.items[index] = item_id;
        }
    }

    /// 清空制定格
    pub fn clear_slot(&mut self, index: usize){
        if index < HOTBAR_SIZE {
            self.items[index] = ItemId::air();
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
    pub fn select_by_number(&mut self, number: usize){
        if number < HOTBAR_SIZE {
            self.active_index = number;
        }
    }
}