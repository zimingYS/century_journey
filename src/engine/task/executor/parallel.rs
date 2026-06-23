use crate::engine::task::executor::partition::{Partition, PartitionStrategy};
use std::sync::Arc;

/// 并行迭代执行器
/// 提供基于数据分片的多线程并行遍历与映射能力，自动按指定线程数拆分数据并派发执行
pub struct ParallelFor;

impl ParallelFor {
    /// 并行遍历（For Each）
    /// 将输入数据按动态策略拆分为多个批次，分发到对应数量的线程中并行执行处理逻辑，无返回值
    pub fn for_each<T, F>(data: Vec<T>, worker_count: usize, f: F)
    where
        T: Clone + Send + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        let batches = Partition::partition(&data, worker_count, PartitionStrategy::Dynamic);
        if batches.is_empty() {
            return;
        }
        let f = Arc::new(f);
        let mut handles = Vec::new();
        for batch in batches {
            let f = f.clone();
            let handle = std::thread::spawn(move || {
                for item in batch {
                    f(item);
                }
            });
            handles.push(handle);
        }
        for h in handles {
            let _ = h.join();
        }
    }

    /// 并行映射（Map）
    /// 将输入数据按动态策略拆分为多个批次，多线程并行对元素执行转换映射，最终合并所有批次的结果并返回
    /// 线程执行过程中发生 panic 的批次结果会被丢弃
    pub fn map<T, R, F>(data: Vec<T>, worker_count: usize, f: F) -> Vec<R>
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let batches = Partition::partition(&data, worker_count, PartitionStrategy::Dynamic);
        if batches.is_empty() {
            return vec![];
        }
        let f = Arc::new(f);
        let mut handles = Vec::new();
        for batch in batches {
            let f = f.clone();
            let handle = std::thread::spawn(move || {
                batch.into_iter().map(|item| f(item)).collect::<Vec<R>>()
            });
            handles.push(handle);
        }
        let mut results = Vec::new();
        for h in handles {
            if let Ok(batch_results) = h.join() {
                results.extend(batch_results);
            }
        }
        results
    }
}
