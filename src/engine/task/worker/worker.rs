use crate::engine::task::scheduler::TaskScheduler;
use std::sync::{Arc, Mutex};
use std::thread;

/// 工作线程
///
/// 无限循环：等待任务 → 执行任务 → 返回结果。
/// Worker 不知道 Scheduler/Manager 具体实现。
pub struct Worker {
    /// 线程句柄
    _thread: Option<thread::JoinHandle<()>>,
    /// 是否应该停止
    stop_signal: Arc<Mutex<bool>>,
}

impl Worker {
    /// 创建并启动 Worker
    pub fn spawn(scheduler: Arc<Mutex<TaskScheduler>>) -> Self {
        let stop = Arc::new(Mutex::new(false));
        let stop_clone = stop.clone();

        let handle = thread::spawn(move || {
            loop {
                // 检查停止信号
                if *stop_clone.lock().unwrap() {
                    break;
                }

                // 从 Scheduler 获取任务
                let job_opt = {
                    let mut sched = scheduler.lock().unwrap();
                    sched.fetch_job()
                };

                match job_opt {
                    Some(job) => {
                        // 执行任务
                        let completed = job.execute();

                        // 提交结果
                        let mut sched = scheduler.lock().unwrap();
                        sched.complete(completed);
                    }
                    None => {
                        // 队列为空，短暂休眠避免忙等
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        });

        Self {
            _thread: Some(handle),
            stop_signal: stop,
        }
    }

    /// 停止 Worker
    pub fn stop(&mut self) {
        if let Ok(mut signal) = self.stop_signal.lock() {
            *signal = true;
        }
        if let Some(handle) = self._thread.take() {
            let _ = handle.join();
        }
    }
}
