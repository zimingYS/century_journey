use crate::engine::task::executor::partition::{Partition, PartitionStrategy};
use std::sync::{Arc, Mutex};

/// Batch执行结果
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_index: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// Batch分发器
pub struct BatchDispatcher;

impl BatchDispatcher {
    pub fn dispatch<T, F>(
        data: Vec<T>,
        worker_count: usize,
        strategy: PartitionStrategy,
        f: F,
    ) -> Vec<BatchResult>
    where
        T: Clone + Send + 'static,
        F: Fn(usize, Vec<T>) -> BatchResult + Send + Sync + 'static,
    {
        let batches = Partition::partition(&data, worker_count, strategy);
        let results = Arc::new(Mutex::new(Vec::new()));
        let f = Arc::new(f);
        let mut handles = Vec::new();
        for (i, batch) in batches.into_iter().enumerate() {
            let results = results.clone();
            let f = f.clone();
            handles.push(std::thread::spawn(move || {
                let r = f(i, batch);
                results.lock().unwrap().push(r);
            }));
        }
        for h in handles {
            let _ = h.join();
        }
        let guard = results.lock().unwrap();
        guard.clone()
    }
}
