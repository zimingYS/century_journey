use bevy::prelude::*;

/// 槽位类型 — UI 层和 Game 层共享数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotKind {
    Hotbar,
    CreativeGrid,
    Recent,
    SurvivalBackpack,
    SurvivalEquipment,
    SurvivalAccessory,
    Container,
}

/// 搜索输入状态 — UI 层和 Game 层共享数据。
#[derive(Resource, Default)]
pub struct SearchInputState {
    pub active: bool,
}
