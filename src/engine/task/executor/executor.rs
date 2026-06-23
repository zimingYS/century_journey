use crate::engine::task::executor::batch::{BatchDispatcher, BatchResult};
use crate::engine::task::executor::fork_join::ForkJoin;
use crate::engine::task::executor::parallel::ParallelFor;
use crate::engine::task::executor::partition::PartitionStrategy;
use crate::engine::task::executor::reducer::Reducer;

/// 并行执行器
/// 封装多种并行计算原语，提供并行循环、映射、归约、任务分发等能力
pub struct ParallelExecutor {
    /// 工作线程数量
    worker_count: usize,
}

impl ParallelExecutor {
    /// 创建并行执行器实例
    pub fn new(worker_count: usize) -> Self {
        Self { worker_count }
    }

    /// 获取当前配置的工作线程数量
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// 并行循环
    /// 对输入数据集合中的每个元素，并行执行指定的处理闭包
    pub fn parallel_for<T, F>(&self, data: Vec<T>, f: F)
    where
        T: Clone + Send + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        ParallelFor::for_each(data, self.worker_count, f);
    }

    /// 并行映射
    /// 对输入数据集合中的每个元素并行执行映射转换，返回映射后的结果集合
    pub fn parallel_map<T, R, F>(&self, data: Vec<T>, f: F) -> Vec<R>
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        ParallelFor::map(data, self.worker_count, f)
    }

    /// 并行归约
    /// 先对分片后的数据并行执行映射计算，再通过归约函数逐层合并所有分片结果，最终返回单个聚合值
    pub fn parallel_reduce<T, R, F, G>(&self, data: Vec<T>, map: F, reduce: G, identity: R) -> R
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> R + Send + Sync + 'static,
        G: Fn(R, R) -> R + Send + Sync + 'static,
    {
        Reducer::parallel_reduce(data, map, reduce, identity, self.worker_count)
    }

    /// 分支-合并
    /// 提交一组一次性任务并行执行，所有任务执行完成后统一返回
    pub fn fork_join<F>(&self, tasks: Vec<F>) -> Vec<()>
    where
        F: FnOnce() + Send + 'static,
    {
        ForkJoin::fork_join(tasks)
    }

    /// 批量分发
    /// 按照指定的分片策略拆分数据，分发到各工作线程批量处理，返回所有批次的执行结果
    pub fn dispatch_batch<T, F>(
        &self,
        data: Vec<T>,
        strategy: PartitionStrategy,
        f: F,
    ) -> Vec<BatchResult>
    where
        T: Clone + Send + 'static,
        F: Fn(usize, Vec<T>) -> BatchResult + Send + Sync + 'static,
    {
        BatchDispatcher::dispatch(data, self.worker_count, strategy, f)
    }
}

impl Default for ParallelExecutor {
    /// 自动读取系统可用并行度作为工作线程数，读取失败时回退为4个线程
    fn default() -> Self {
        let count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self::new(count)
    }
}
