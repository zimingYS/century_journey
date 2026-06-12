use crate::inventory::container::InventoryContainer;
use crate::inventory::item::stack::ItemStack;

/// 生存模式完整背包
/// 包含 36 格主背包 + 4 格盔甲 + 6 格饰品。
/// 实施 InventoryContainer 接口，未来与 Hotbar、Chest 等同质处理。
#[derive(Debug, Clone)]
pub struct SurvivalInventory {
    /// 主背包 36 格
    pub backpack: [Option<ItemStack>; 36],
    /// 盔甲栏 4 格（头/胸/腿/脚）
    pub armor: [Option<ItemStack>; 4],
    /// 饰品栏 6 格
    pub accessories: [Option<ItemStack>; 6],
}

impl Default for SurvivalInventory {
    fn default() -> Self {
        Self {
            backpack: std::array::from_fn(|_| None),
            armor: std::array::from_fn(|_| None),
            accessories: std::array::from_fn(|_| None),
        }
    }
}

impl SurvivalInventory {
    /// 背包总槽位数
    pub const BACKPACK_SIZE: usize = 36;
    /// 盔甲槽位数
    pub const ARMOR_SIZE: usize = 4;
    /// 饰品槽位数
    pub const ACCESSORY_SIZE: usize = 6;
    /// 总槽位数
    pub const TOTAL_SIZE: usize = Self::BACKPACK_SIZE + Self::ARMOR_SIZE + Self::ACCESSORY_SIZE;

    /// 将虚拟索引映射到实际存储区域
    fn map_index(index: usize) -> Option<(&'static str, usize)> {
        if index < Self::BACKPACK_SIZE {
            Some(("backpack", index))
        } else if index < Self::BACKPACK_SIZE + Self::ARMOR_SIZE {
            Some(("armor", index - Self::BACKPACK_SIZE))
        } else if index < Self::TOTAL_SIZE {
            Some(("accessories", index - Self::BACKPACK_SIZE - Self::ARMOR_SIZE))
        } else {
            None
        }
    }
}

impl InventoryContainer for SurvivalInventory {
    fn slot_count(&self) -> usize {
        Self::TOTAL_SIZE
    }

    fn get_stack(&self, index: usize) -> Option<&ItemStack> {
        match Self::map_index(index)? {
            ("backpack", i) => self.backpack[i].as_ref(),
            ("armor", i) => self.armor[i].as_ref(),
            ("accessories", i) => self.accessories[i].as_ref(),
            _ => None,
        }
    }

    fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        match Self::map_index(index)? {
            ("backpack", i) => self.backpack[i].as_mut(),
            ("armor", i) => self.armor[i].as_mut(),
            ("accessories", i) => self.accessories[i].as_mut(),
            _ => None,
        }
    }

    fn set_stack(&mut self, index: usize, stack: ItemStack) {
        let slot = match Self::map_index(index) {
            Some(("backpack", i)) => &mut self.backpack[i],
            Some(("armor", i)) => &mut self.armor[i],
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