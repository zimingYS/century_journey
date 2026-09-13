//! 世界时间与模拟时钟

/// 模拟时钟的固定更新频率。
pub const SIM_TPS: u32 = 20;

/// 模拟时钟的固定步长，单位为秒。
pub const SIM_DT: f32 = 1.0 / SIM_TPS as f32;

/// 每 5 Hz 执行一次所需的 Tick 数。
pub const TICKS_PER_5HZ: u64 = 4;

/// 每 1 Hz 执行一次所需的 Tick 数。
pub const TICKS_PER_1HZ: u64 = 20;

/// 世界时间（模拟时间，非帧时间）
///
/// 后续确定世界历法参数后，这个类型将扩展为更高层的世界时间表示，
/// 例如游戏日、季节、年份等。
///
/// 注意：这里的 `tick` 表示的是**世界模拟时间**，而 [`SimClock::tick`]
/// 表示的是**模拟调度步计数**。当前二者数值同步，但职责不同。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorldTime {
    /// 自世界诞生以来的模拟 tick
    pub tick: u64,
}

/// 模拟时钟（驱动固定步长，见 §6.7）
#[derive(Debug, Clone, Copy)]
pub struct SimClock {
    /// 当前模拟 Tick。
    pub tick: u64,
    /// 固定模拟步长，单位为秒。
    pub dt: f32,
}

impl SimClock {
    /// 使用指定固定步长创建模拟时钟。
    pub const fn new(dt: f32) -> Self {
        SimClock { tick: 0, dt }
    }

    /// 推进一个模拟 Tick
    #[inline]
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// 判断当前 Tick 是否应该执行一个指定频率的任务
    #[inline]
    pub fn should_run(&self, every: u64) -> bool {
        debug_assert!(every > 0, "simulation frequency interval must be > 0");
        self.tick.is_multiple_of(every)
    }

    /// 获取自模拟开始以来经过的模拟时间
    #[inline]
    pub fn elapsed_seconds(&self) -> f32 {
        self.tick as f32 * self.dt
    }
}

impl Default for SimClock {
    /// 使用项目规定的固定模拟步长创建模拟时钟。
    fn default() -> Self {
        Self::new(SIM_DT)
    }
}
