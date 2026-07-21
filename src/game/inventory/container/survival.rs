use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::equipment::EquipmentSlot;
use crate::game::inventory::item::stack::ItemStack;

/// 生存模式完整背包
/// 包含 36 格主背包 + 4 格盔甲 + 6 格饰品。
/// 实施 InventoryContainer 接口，未来与 Hotbar、Chest 等同质处理。
#[derive(Debug, Clone)]
pub struct SurvivalInventory {
    /// 主背包 27 格，快捷栏独立存储。
    pub backpack: [Option<ItemStack>; 27],
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
    pub const BACKPACK_SIZE: usize = 27;
    pub const EQUIPMENT_SIZE: usize = EquipmentSlot::ALL.len();

    pub fn total_size(&self) -> usize {
        Self::BACKPACK_SIZE + Self::EQUIPMENT_SIZE + self.accessories.len()
    }

    pub const fn equipment_index(index: usize) -> usize {
        Self::BACKPACK_SIZE + index
    }

    pub const fn accessory_index(index: usize) -> usize {
        Self::BACKPACK_SIZE + Self::EQUIPMENT_SIZE + index
    }

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
