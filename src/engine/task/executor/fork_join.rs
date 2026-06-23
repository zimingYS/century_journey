use std::sync::{Arc, Mutex};

/// 分支-合并 并行执行工具
/// 提供基础的多线程任务派发与同步等待能力，支持无共享上下文、带共享上下文两种任务执行模式
pub struct ForkJoin;

impl ForkJoin {
    /// 执行无共享上下文的并行任务
    /// 将所有任务分别派发至独立线程中执行，等待全部线程结束后返回执行成功的结果集合
    /// 执行过程中发生 panic 的线程会被过滤，不会体现在最终返回结果中
    pub fn fork_join<F>(tasks: Vec<F>) -> Vec<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut handles = Vec::with_capacity(tasks.len());
        for task in tasks {
            handles.push(std::thread::spawn(task));
        }
        let results: Vec<_> = handles.into_iter().map(|h| h.join()).collect();
        results.into_iter().filter_map(|r| r.ok()).collect()
    }

    /// 携带共享上下文的并行任务执行
    /// 在共享互斥上下文的前提下派发多线程任务，每个任务均可通过互斥锁访问共享上下文
    /// 执行过程中发生 panic 的线程会被过滤，不会体现在最终返回结果中
    pub fn fork_join_with<C, F>(ctx: Arc<Mutex<C>>, tasks: Vec<F>) -> Vec<()>
    where
        C: Send + 'static,
        F: FnOnce(&Mutex<C>) + Send + 'static,
    {
        let mut handles = Vec::with_capacity(tasks.len());
        for task in tasks {
            let ctx = ctx.clone();
            handles.push(std::thread::spawn(move || {
                task(&ctx);
            }));
        }
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    }
}
