use crate::game::inventory::slot::SlotData;

/// 通用容器 (箱子/熔炉等)
/// TODO - 未来扩展
#[derive(Debug, Clone)]
pub struct ContainerData {
    pub slots: Vec<SlotData>,
    pub container_type: ContainerType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    Chest,
    Furnace,
    CraftingTable,
    Custom(u8),
}
