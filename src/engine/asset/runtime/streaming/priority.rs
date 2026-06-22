/// 流加载优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamPriority {
    /// 最高（当前帧必须加载）
    Critical = 0,
    /// 高（玩家视野内）
    High = 1,
    /// 普通
    Normal = 2,
    /// 低（后台预加载）
    Low = 3,
    /// 最低（空闲时加载）
    Idle = 4,
}
