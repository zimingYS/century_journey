use crate::engine::task::scheduler::priority::TaskPriority;

/// 帧预算管理器
/// 用于限制每一帧内各优先级任务的最大派发数量，避免单帧任务调度过载
#[derive(Debug, Clone)]
pub struct FrameBudget {
    /// 每帧关键级任务的最大派发数量上限
    critical_per_frame: usize,
    /// 每帧高优先级任务的最大派发数量上限
    high_per_frame: usize,
    /// 每帧普通优先级任务的最大派发数量上限
    normal_per_frame: usize,
    /// 每帧低优先级任务的最大派发数量上限
    low_per_frame: usize,
    /// 每帧空闲级任务的最大派发数量上限
    idle_per_frame: usize,
    /// 本帧已派发的关键级任务计数
    dispatched_critical: usize,
    /// 本帧已派发的高优先级任务计数
    dispatched_high: usize,
    /// 本帧已派发的普通优先级任务计数
    dispatched_normal: usize,
    /// 本帧已派发的低优先级任务计数
    dispatched_low: usize,
    /// 本帧已派发的空闲级任务计数
    dispatched_idle: usize,
}

impl FrameBudget {
    /// 创建帧预算管理器实例
    pub fn new(critical: usize, high: usize, normal: usize, low: usize, idle: usize) -> Self {
        Self {
            critical_per_frame: critical,
            high_per_frame: high,
            normal_per_frame: normal,
            low_per_frame: low,
            idle_per_frame: idle,
            dispatched_critical: 0,
            dispatched_high: 0,
            dispatched_normal: 0,
            dispatched_low: 0,
            dispatched_idle: 0,
        }
    }

    /// 判断指定优先级的任务是否允许派发
    /// 若该优先级本帧已派发数量未达到上限则返回 true
    pub fn can_dispatch(&self, priority: TaskPriority) -> bool {
        match priority {
            TaskPriority::Critical => self.dispatched_critical < self.critical_per_frame,
            TaskPriority::High => self.dispatched_high < self.high_per_frame,
            TaskPriority::Normal => self.dispatched_normal < self.normal_per_frame,
            TaskPriority::Low => self.dispatched_low < self.low_per_frame,
            TaskPriority::Idle => self.dispatched_idle < self.idle_per_frame,
        }
    }

    /// 记录一次任务派发
    /// 将对应优先级的本帧已派发计数加 1
    pub fn record_dispatch(&mut self, priority: TaskPriority) {
        match priority {
            TaskPriority::Critical => self.dispatched_critical += 1,
            TaskPriority::High => self.dispatched_high += 1,
            TaskPriority::Normal => self.dispatched_normal += 1,
            TaskPriority::Low => self.dispatched_low += 1,
            TaskPriority::Idle => self.dispatched_idle += 1,
        }
    }

    /// 重置本帧计数
    /// 通常在每帧结束时调用，将所有优先级的已派发计数清零，开启新一帧的预算周期
    pub fn reset_frame(&mut self) {
        self.dispatched_critical = 0;
        self.dispatched_high = 0;
        self.dispatched_normal = 0;
        self.dispatched_low = 0;
        self.dispatched_idle = 0;
    }

    /// 获取本帧已派发的任务总数
    /// 统计所有优先级已派发的任务数量之和
    pub fn dispatched_this_frame(&self) -> usize {
        self.dispatched_critical
            + self.dispatched_high
            + self.dispatched_normal
            + self.dispatched_low
            + self.dispatched_idle
    }
}

impl Default for FrameBudget {
    /// 关键级任务无数量上限，高优每帧 16 个、普通 8 个、低优 4 个、空闲级 1 个
    fn default() -> Self {
        Self::new(usize::MAX, 16, 8, 4, 1)
    }
}
