use bevy::prelude::*;

/// 稳定的运行时容器分类。它只用于路由和布局，不构成稳定 Mod API。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerKind {
    PlayerCrafting,
    Workbench,
    Chest,
    Furnace,
}

/// 槽位类型 — UI 层和 Game 层共享数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotKind {
    Hotbar,
    CreativeGrid,
    Recent,
    SurvivalBackpack,
    SurvivalEquipment,
    SurvivalAccessory,
    Container(ContainerKind),
}

/// 搜索输入状态 — UI 层和 Game 层共享数据。
#[derive(Resource, Default)]
pub struct SearchInputState {
    pub active: bool,
}
