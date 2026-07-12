use bevy::prelude::*;

/// 生存模式中固定存在的装备槽。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
    Cape,
    Offhand,
    Backpack,
}

impl EquipmentSlot {
    pub const ALL: [Self; 7] = [
        Self::Helmet,
        Self::Chestplate,
        Self::Leggings,
        Self::Boots,
        Self::Offhand,
        Self::Cape,
        Self::Backpack,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Helmet => "头盔",
            Self::Chestplate => "胸甲",
            Self::Leggings => "护腿",
            Self::Boots => "靴子",
            Self::Cape => "披风",
            Self::Offhand => "副手",
            Self::Backpack => "背包",
        }
    }

    pub const fn placeholder(self) -> &'static str {
        match self {
            Self::Helmet => "头",
            Self::Chestplate => "胸",
            Self::Leggings => "腿",
            Self::Boots => "靴",
            Self::Cape => "披",
            Self::Offhand => "副",
            Self::Backpack => "包",
        }
    }
}

/// 一个由内容定义的饰品槽。模组可以在 UI 创建前替换此资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessorySlotDefinition {
    pub id: String,
    pub display_name: String,
    pub placeholder: String,
}

impl AccessorySlotDefinition {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            placeholder: placeholder.into(),
        }
    }
}

/// 生存物品栏右侧饰品栏的内容定义。
#[derive(Resource, Debug, Clone)]
pub struct AccessorySlotDefinitions {
    pub slots: Vec<AccessorySlotDefinition>,
}

impl Default for AccessorySlotDefinitions {
    fn default() -> Self {
        Self {
            slots: vec![
                AccessorySlotDefinition::new("ring_1", "戒指 I", "I"),
                AccessorySlotDefinition::new("ring_2", "戒指 II", "II"),
                AccessorySlotDefinition::new("necklace", "项链", "项"),
                AccessorySlotDefinition::new("charm", "护符", "符"),
                AccessorySlotDefinition::new("belt", "腰带", "带"),
                AccessorySlotDefinition::new("wings", "翅膀", "翼"),
            ],
        }
    }
}
