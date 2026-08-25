//! 定义装备槽类别、占位文本和装备容器状态。

use bevy::prelude::*;

/// 生存模式中固定存在的装备槽。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    /// 头盔槽。
    Helmet,
    /// 胸甲槽。
    Chestplate,
    /// 护腿槽。
    Leggings,
    /// 靴子槽。
    Boots,
    /// 披风槽。
    Cape,
    /// 副手槽。
    Offhand,
    /// 背包扩展槽。
    Backpack,
}

impl EquipmentSlot {
    /// 固定装备槽的稳定展示顺序。
    pub const ALL: [Self; 7] = [
        Self::Helmet,
        Self::Chestplate,
        Self::Leggings,
        Self::Boots,
        Self::Offhand,
        Self::Cape,
        Self::Backpack,
    ];

    /// 返回空槽位占位文字的本地化键（`equipment.placeholder.*`）。
    pub const fn placeholder_key(self) -> &'static str {
        match self {
            Self::Helmet => "equipment.placeholder.helmet",
            Self::Chestplate => "equipment.placeholder.chestplate",
            Self::Leggings => "equipment.placeholder.leggings",
            Self::Boots => "equipment.placeholder.boots",
            Self::Cape => "equipment.placeholder.cape",
            Self::Offhand => "equipment.placeholder.offhand",
            Self::Backpack => "equipment.placeholder.backpack",
        }
    }
}

/// 一个由内容定义的饰品槽。模组可以在 UI 创建前替换此资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessorySlotDefinition {
    /// 存档和内容引用使用的稳定槽位 ID。
    pub id: String,
    /// 空槽位占位文字的本地化键（`accessory.placeholder.*`）。
    pub placeholder_key: String,
}

impl AccessorySlotDefinition {
    /// 创建一项饰品槽定义；占位文字以本地化键表示。
    pub fn new(id: impl Into<String>, placeholder_key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            placeholder_key: placeholder_key.into(),
        }
    }
}

/// 生存物品栏右侧饰品栏的内容定义。
#[derive(Resource, Debug, Clone)]
pub struct AccessorySlotDefinitions {
    /// 按界面和存档顺序排列的饰品槽定义。
    pub slots: Vec<AccessorySlotDefinition>,
}

impl Default for AccessorySlotDefinitions {
    fn default() -> Self {
        Self {
            slots: vec![
                AccessorySlotDefinition::new("ring_1", "accessory.placeholder.ring-1"),
                AccessorySlotDefinition::new("ring_2", "accessory.placeholder.ring-2"),
                AccessorySlotDefinition::new("necklace", "accessory.placeholder.necklace"),
                AccessorySlotDefinition::new("charm", "accessory.placeholder.charm"),
                AccessorySlotDefinition::new("belt", "accessory.placeholder.belt"),
                AccessorySlotDefinition::new("wings", "accessory.placeholder.wings"),
            ],
        }
    }
}
