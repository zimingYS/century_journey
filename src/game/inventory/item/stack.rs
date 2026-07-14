use crate::content::item::ItemRegistry;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;

/// 单个物品实例携带的可变数据。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemInstanceData {
    /// 工具剩余可用次数。None 表示尚未初始化或该物品没有耐久。
    pub durability: Option<u32>,
}

/// 工具承受一次耐久消耗后的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDamageResult {
    NotDamageable,
    Damaged { remaining: u32 },
    Broken,
}

// 这边先临时使用全部物品最大堆叠64个
// 以后会根据物品种类进行最大堆叠分类
// 故现在先写到这边
/// 物品最大堆叠数
pub const MAX_STACK_SIZE: u32 = 64;

/// 物品堆叠
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// 物品类型
    pub item: ItemId,
    /// 堆叠数量
    pub count: u32,
    /// 该堆物品共享的实例数据；具有不同实例数据的物品不能合并。
    pub instance: ItemInstanceData,
}

impl ItemStack {
    // 由数量创建物品
    pub fn new(item: ItemId, count: u32) -> Self {
        Self {
            item,
            count,
            instance: ItemInstanceData::default(),
        }
    }

    /// 使用指定实例数据创建物品堆。
    pub fn with_instance(item: ItemId, count: u32, instance: ItemInstanceData) -> Self {
        Self {
            item,
            count,
            instance,
        }
    }

    // 创建一个物品
    pub fn single(item_id: ItemId) -> Self {
        Self::new(item_id, 1)
    }

    /// 创建空堆叠
    pub fn empty() -> Self {
        Self {
            item: ItemId::air(),
            count: 0,
            instance: ItemInstanceData::default(),
        }
    }

    // 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item.is_air()
    }

    /// 判断是否与另一个堆叠同种物品
    pub fn is_same_item(&self, other: &ItemStack) -> bool {
        self.item == other.item && self.instance == other.instance
    }

    /// 判断是否可以与另一个堆叠合并
    pub fn can_merge(&self, other: &ItemStack) -> bool {
        // 必须是相同物品且小于最大可堆叠物品上限
        if !self.is_same_item(other) {
            return false;
        }
        self.count.saturating_add(other.count) <= MAX_STACK_SIZE
    }

    /// 从另一个堆叠合并尽可能多的物品到自身
    pub fn merge_from(&mut self, other: &mut ItemStack) -> u32 {
        if !self.is_same_item(other) || other.is_empty() {
            return 0;
        }
        let available = MAX_STACK_SIZE.saturating_sub(self.count);
        let to_move = available.min(other.count);
        self.count += to_move;
        other.count -= to_move;
        if other.count == 0 {
            *other = ItemStack::empty();
        }
        to_move
    }

    /// 将自身对半拆分，返回拆分出的新堆叠
    pub fn split_half(&mut self) -> Option<ItemStack> {
        if self.count <= 1 {
            return None;
        }
        let half = self.count / 2;
        self.count -= half;
        Some(ItemStack::with_instance(
            self.item.clone(),
            half,
            self.instance.clone(),
        ))
    }

    /// 从自身取走指定数量，返回取走的堆叠
    pub fn take(&mut self, amount: u32) -> Option<ItemStack> {
        if amount == 0 || self.is_empty() {
            return None;
        }
        let taken = amount.min(self.count);
        self.count -= taken;
        let result = ItemStack::with_instance(self.item.clone(), taken, self.instance.clone());
        if self.count == 0 {
            *self = ItemStack::empty();
        }
        Some(result)
    }

    /// 获取物品类型引用（兼容旧代码中的 `.item_id` 字段访问）
    pub fn item_id(&self) -> &ItemId {
        &self.item
    }

    /// 获取方块标识符引用
    pub fn block_identifier<'a>(&self, item_registry: &'a ItemRegistry) -> Option<&'a Identifier> {
        item_registry.block_identifier(&self.item)
    }

    /// 检查剩余空间（还能合并多少个同种物品）
    pub fn remaining_space(&self) -> u32 {
        MAX_STACK_SIZE.saturating_sub(self.count)
    }

    /// 是否已达最大堆叠数
    pub fn is_full(&self) -> bool {
        self.count >= MAX_STACK_SIZE
    }

    /// 返回当前剩余耐久；尚未使用过的工具会返回 None。
    pub fn durability(&self) -> Option<u32> {
        self.instance.durability
    }

    /// 消耗一次工具耐久。首次消耗时从物品定义中的最大耐久开始计算。
    pub fn damage_tool(&mut self, amount: u32, max_durability: u32) -> ToolDamageResult {
        if amount == 0 || max_durability == 0 || self.is_empty() {
            return ToolDamageResult::NotDamageable;
        }

        let remaining = self
            .instance
            .durability
            .unwrap_or(max_durability)
            .min(max_durability)
            .saturating_sub(amount);
        if remaining == 0 {
            *self = Self::empty();
            ToolDamageResult::Broken
        } else {
            self.instance.durability = Some(remaining);
            ToolDamageResult::Damaged { remaining }
        }
    }

    /// 返回显示用的数量文本（空/1 时不显示）
    pub fn count_text(&self) -> Option<String> {
        if self.is_empty() || self.count <= 1 {
            None
        } else {
            Some(self.count.to_string())
        }
    }

    /// 获取最大堆叠数
    pub const MAX_STACK_SIZE: u32 = 64;
}

// 初始化物品堆叠为空气
impl Default for ItemStack {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod stage_seven_tests {
    use super::*;

    #[test]
    fn stage_seven_tool_instance_loses_durability_and_breaks() {
        let mut tool = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));

        assert_eq!(
            tool.damage_tool(1, 3),
            ToolDamageResult::Damaged { remaining: 2 }
        );
        assert_eq!(tool.durability(), Some(2));
        assert_eq!(
            tool.damage_tool(1, 3),
            ToolDamageResult::Damaged { remaining: 1 }
        );
        assert_eq!(tool.damage_tool(1, 3), ToolDamageResult::Broken);
        assert!(tool.is_empty());
    }

    #[test]
    fn stage_seven_different_instance_data_never_merges() {
        let mut used = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));
        used.instance.durability = Some(4);
        let unused = ItemStack::single(ItemId::item("century_journey:test_pickaxe"));

        assert!(!used.is_same_item(&unused));
        assert!(!used.can_merge(&unused));
    }
}
