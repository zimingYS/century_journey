//! 实现生存背包、装备槽和可扩展饰品槽的统一存储布局。

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::EquipmentSlot;

/// 生存模式完整背包
/// 包含 36 格主背包 + 7 格装备 + 饰品槽。
/// 实施 InventoryContainer 接口，未来与 Hotbar、Chest 等同质处理。
#[derive(Debug, Clone)]
pub struct SurvivalInventory {
    /// 主背包 36 格（4 行 × 9 列），快捷栏独立存储。
    pub backpack: [Option<ItemStack>; 36],
    /// 头盔、胸甲、护腿、靴子、披风、副手和背包。
    pub equipment: [Option<ItemStack>; EquipmentSlot::ALL.len()],
    /// 槽位数量由 AccessorySlotDefinitions 决定。
    pub accessories: Vec<Option<ItemStack>>,
}

impl Default for SurvivalInventory {
    fn default() -> Self {
        Self {
            backpack: std::array::from_fn(|_| None),
            equipment: std::array::from_fn(|_| None),
            accessories: vec![None; 6],
        }
    }
}

impl SurvivalInventory {
    /// 背包总槽位数
    pub const BACKPACK_SIZE: usize = 36;
    /// 固定装备槽总数。
    pub const EQUIPMENT_SIZE: usize = EquipmentSlot::ALL.len();

    /// 返回背包、装备和当前饰品槽的总容量。
    pub fn total_size(&self) -> usize {
        Self::BACKPACK_SIZE + Self::EQUIPMENT_SIZE + self.accessories.len()
    }

    /// 把装备槽局部索引转换为统一容器索引。
    pub const fn equipment_index(index: usize) -> usize {
        Self::BACKPACK_SIZE + index
    }

    /// 把饰品槽局部索引转换为统一容器索引。
    pub const fn accessory_index(index: usize) -> usize {
        Self::BACKPACK_SIZE + Self::EQUIPMENT_SIZE + index
    }

    /// 至少扩展到指定饰品槽数量，且不丢弃已有物品。
    pub fn ensure_accessory_slots(&mut self, count: usize) {
        if self.accessories.len() < count {
            self.accessories.resize_with(count, || None);
        }
    }

    /// 将虚拟索引映射到实际存储区域
    fn map_index(&self, index: usize) -> Option<(&'static str, usize)> {
        if index < Self::BACKPACK_SIZE {
            Some(("backpack", index))
        } else if index < Self::BACKPACK_SIZE + Self::EQUIPMENT_SIZE {
            Some(("equipment", index - Self::BACKPACK_SIZE))
        } else if index < self.total_size() {
            Some((
                "accessories",
                index - Self::BACKPACK_SIZE - Self::EQUIPMENT_SIZE,
            ))
        } else {
            None
        }
    }
}

impl InventoryContainer for SurvivalInventory {
    fn slot_count(&self) -> usize {
        self.total_size()
    }

    fn get_stack(&self, index: usize) -> Option<&ItemStack> {
        match self.map_index(index)? {
            ("backpack", i) => self.backpack[i].as_ref(),
            ("equipment", i) => self.equipment[i].as_ref(),
            ("accessories", i) => self.accessories[i].as_ref(),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        match self.map_index(index)? {
            ("backpack", i) => self.backpack[i].as_mut(),
            ("equipment", i) => self.equipment[i].as_mut(),
            ("accessories", i) => self.accessories[i].as_mut(),
            _ => None,
        }
    }

    fn set_stack(&mut self, index: usize, stack: ItemStack) {
        let slot = match self.map_index(index) {
            Some(("backpack", i)) => &mut self.backpack[i],
            Some(("equipment", i)) => &mut self.equipment[i],
            Some(("accessories", i)) => &mut self.accessories[i],
            _ => return,
        };
        if stack.is_empty() {
            *slot = None;
        } else {
            *slot = Some(stack);
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/container/survival.rs"]
mod tests;
