//! 容器类别与槽位语义枚举。

/// 权威玩法中可持久识别的容器分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerKind {
    /// 玩家随身的 2x2 合成区域。
    PlayerCrafting,
    /// 世界中的工作台合成区域。
    Workbench,
    /// 通用存储箱。
    Chest,
    /// 具有输入、燃料和输出槽的熔炉。
    Furnace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 描述容器槽位在规则中的用途。
pub enum ContainerSlotRole {
    /// 可自由存取的通用存储槽。
    Storage,
    /// 只用于流程输入的槽位。
    Input,
    /// 由规则生成内容的输出槽。
    Output,
    /// 为加工流程提供能量的燃料槽。
    Fuel,
}
