use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::runtime::context::RuntimeContext;
use crate::engine::asset::runtime::job::AssetJob;
use crate::engine::asset::runtime::service::RuntimeService;
use crate::engine::asset::state::AssetState;
use bevy::prelude::*;
use std::collections::VecDeque;

/// 统一的 Runtime Scheduler
///
/// 替代各 Manager 自有的 Queue，统一调度所有 AssetJob。
/// 每帧按优先级处理一批 Job，分发给对应的 Service。
///
/// 新增任何 Runtime 功能只需注册到 Scheduler，无需修改 Runtime。
#[derive(Resource)]
pub struct RuntimeScheduler {
    /// 全局 Job 队列
    queue: VecDeque<AssetJob>,
    /// 注册的 Service
    services: Vec<Box<dyn RuntimeService>>,
    /// 每帧最大 Job 处理数
    max_jobs_per_frame: usize,
}

impl RuntimeScheduler {
    pub fn new(max_jobs_per_frame: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            services: Vec::new(),
            max_jobs_per_frame,
        }
    }

    /// 注册一个 Runtime Service
    pub fn register_service(&mut self, service: impl RuntimeService) {
        self.services.push(Box::new(service));
    }

    /// 提交 Job 到全局队列
    pub fn submit(&mut self, job: AssetJob) {
        self.queue.push_back(job);
        self.sort();
    }

    /// 批量提交
    pub fn submit_batch(&mut self, jobs: Vec<AssetJob>) {
        for job in jobs {
            self.queue.push_back(job);
        }
        self.sort();
    }

    /// 取消指定 key 的所有 Job
    pub fn cancel(&mut self, key: &str) {
        self.queue.retain(|j| j.key() != key);
    }

    /// 每帧更新：处理一批 Job → 更新所有 Service → 同步状态
    pub fn update(
        &mut self,
        ctx: &mut RuntimeContext,
        cache: &mut AssetCache,
        registry: &mut AssetRegistry,
    ) {
        // 1. 处理一批 Job
        let count = self.max_jobs_per_frame.min(self.queue.len());
        let batch: Vec<AssetJob> = self.queue.drain(..count).collect();

        for job in &batch {
            let key = job.key();
            match job {
                AssetJob::Streaming { .. } => {
                    // 触发 Pipeline 重新加载
                    ctx.retain(key);
                }
                AssetJob::Reload { .. } => {
                    cache.remove(key);
                    registry.set_state(key, AssetState::Reloading);
                }
                AssetJob::Unload { .. } => {
                    cache.remove(key);
                    registry.set_state(key, AssetState::Disposed);
                    ctx.release(key);
                }
                AssetJob::Validate { .. } => {
                    // 后续：运行 Validator
                }
            }
        }

        // 2. 更新所有 Service
        for service in &mut self.services {
            service.update(ctx);
        }

        // 3. 将未处理的 Job 放回（如果有上限限制）
    }

    /// 启动所有 Service
    pub fn startup(&mut self, ctx: &mut RuntimeContext) {
        for service in &mut self.services {
            service.startup(ctx);
        }
    }

    /// 关闭所有 Service
    pub fn shutdown(&mut self, ctx: &mut RuntimeContext) {
        for service in &mut self.services {
            service.shutdown(ctx);
        }
    }

    /// 待处理 Job 数量
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    fn sort(&mut self) {
        self.queue.make_contiguous().sort_by_key(|j| j.order());
    }
}

impl Default for RuntimeScheduler {
    fn default() -> Self {
        Self::new(50)
    }
}
