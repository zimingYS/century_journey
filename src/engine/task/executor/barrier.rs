use std::sync::{Arc, Condvar, Mutex};

/// TaskBarrier
pub struct TaskBarrier {
    total: usize,
    current: Arc<(Mutex<usize>, Condvar)>,
}

impl TaskBarrier {
    pub fn new(count: usize) -> Self {
        Self {
            total: count,
            current: Arc::new((Mutex::new(0), Condvar::new())),
        }
    }

    /// Worker到达并等待其他Worker
    pub fn wait(&self) {
        let (lock, cvar) = &*self.current;
        let mut count = lock.lock().unwrap();
        *count += 1;
        if *count == self.total {
            cvar.notify_all();
        } else {
            while *count < self.total {
                count = cvar.wait(count).unwrap();
            }
        }
    }

    /// 重置 Barrier
    pub fn reset(&self) {
        let (lock, _) = &*self.current;
        *lock.lock().unwrap() = 0;
    }
}

impl Clone for TaskBarrier {
    fn clone(&self) -> Self {
        Self {
            total: self.total,
            current: self.current.clone(),
        }
    }
}
