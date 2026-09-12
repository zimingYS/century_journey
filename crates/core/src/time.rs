//! 世界时间与模拟时钟

/// 世界时间（模拟时间，非帧时间）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorldTime {
    /// 自世界诞生以来的模拟 tick
    pub tick: u64,
}

/// 模拟时钟（驱动固定步长，见 §6.7）
#[derive(Debug, Clone, Copy)]
pub struct SimClock {
    pub tick: u64,
    pub dt: f32,
}