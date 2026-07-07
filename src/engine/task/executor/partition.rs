/// 分区策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionStrategy {
    /// 固定大小
    Fixed(usize),
    /// 动态（worker_count * 4）
    Dynamic,
    /// 自动计算最优 Batch Size
    Auto,
}

/// 将数据分区为 Batch
pub struct Partition;

impl Partition {
    /// 按策略分区
    pub fn partition<T: Clone>(
        data: &[T],
        worker_count: usize,
        strategy: PartitionStrategy,
    ) -> Vec<Vec<T>> {
        let total = data.len();
        let batch_count = match strategy {
            PartitionStrategy::Fixed(size) => total.div_ceil(size),
            PartitionStrategy::Dynamic => worker_count * 4,
            PartitionStrategy::Auto => {
                let base = (total / worker_count).max(1);
                base.min(64)
            }
        };

        if batch_count == 0 {
            return vec![];
        }

        let batch_size = total.div_ceil(batch_count);
        data.chunks(batch_size.max(1)).map(|c| c.to_vec()).collect()
    }
}
