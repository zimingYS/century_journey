//! 局部光照任务的调优常量。

/// 普通流送任务一次最多处理的核心水平列数。
pub(super) const LOCAL_TARGET_COLUMN_BATCH_SIZE: usize = 24;
/// 玩家交互一次最多合并的水平列数，覆盖常用一至两圈传播半径。
pub(super) const LOCAL_INTERACTION_COLUMN_BATCH_LIMIT: usize = 25;
/// 可见区块发现阶段保留的最大候选数，避免每个固定步扫描完整窗口。
pub(super) const LOCAL_DISCOVERY_QUEUE_LIMIT: usize = 512;
/// 普通光照任务允许进入的任务池积压倍数；通道有限并发保证不会无界增长。
pub(super) const LOCAL_TASK_BACKLOG_FACTOR: usize = 2;
/// 局部光照任务允许的最小基础并发数；列集由 pop_columns 从队列移除，天然不相交。
pub(super) const LOCAL_LIGHTING_MIN_IN_FLIGHT: usize = 2;
/// 局部光照任务允许的最大基础并发数；上限保证与网格、全局重建共享线程池时不过度超发。
pub(super) const LOCAL_LIGHTING_MAX_IN_FLIGHT: usize = 3;
