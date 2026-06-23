use std::sync::Arc;

/// 归约执行器
/// 提供并行 Map-Reduce 计算能力，先将数据分片后并行执行映射，再将所有分片结果归约合并为单个最终值
pub struct Reducer;

impl Reducer {
    /// 并行 Map-Reduce 计算
    pub fn parallel_reduce<T, R, F, G>(
        data: Vec<T>,
        map: F,
        reduce: G,
        identity: R,
        worker_count: usize,
    ) -> R
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> R + Send + Sync + 'static,
        G: Fn(R, R) -> R + Send + Sync + 'static,
    {
        let chunk_size = (data.len() + worker_count - 1) / worker_count;
        let chunks: Vec<_> = data.chunks(chunk_size.max(1)).map(|c| c.to_vec()).collect();

        let map = Arc::new(map);
        let mut handles = Vec::new();
        for chunk in chunks {
            let map = map.clone();
            let handle = std::thread::spawn(move || map(chunk));
            handles.push(handle);
        }

        let results: Vec<R> = handles.into_iter().filter_map(|h| h.join().ok()).collect();

        results.into_iter().fold(identity, |acc, r| reduce(acc, r))
    }
}
