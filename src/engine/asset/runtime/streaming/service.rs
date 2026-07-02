use crate::engine::asset::runtime::context::RuntimeContext;
use crate::engine::asset::runtime::job::AssetJob;
use crate::engine::asset::runtime::streaming::priority::StreamPriority;
use std::collections::HashSet;

/// 流加载服务
///
/// 支持依赖顺序 Streaming。
/// 资源依赖图由 RuntimeContext 维护，StreamingService 按依赖拓扑顺序加载。
pub struct StreamingService {
    /// 每帧最大处理数
    max_per_frame: usize,
    /// 已调度但未完成的 key 集合（去重）
    pending: HashSet<String>,
}

impl StreamingService {
    pub fn new(max_per_frame: usize) -> Self {
        Self {
            max_per_frame,
            pending: HashSet::new(),
        }
    }

    /// 提交流加载请求
    pub fn stream(&mut self, key: &str, _priority: StreamPriority, ctx: &mut RuntimeContext) {
        if self.pending.contains(key) {
            return;
        }
        self.pending.insert(key.to_string());
        // 通过 Context 标记（实际 Job 由 Scheduler 管理）
        ctx.retain(key);
        // 记录到 context 的 assets
        ctx.assets.entry(key.to_string()).or_insert_with(|| {
            crate::engine::asset::runtime::context::AssetInfo {
                id: crate::engine::asset::identifier::AssetId::parse(&format!(
                    "century_journey:{key}"
                )),
                asset_type: String::new(),
                size_bytes: 0,
                state: String::new(),
                source: String::new(),
                dependencies: Vec::new(),
                version: 1,
                hash: 0,
            }
        });
    }

    /// 处理一批（由 RuntimeScheduler 调用）
    pub fn process_batch(&mut self, _ctx: &mut RuntimeContext, batch: &[AssetJob]) -> Vec<String> {
        let count = self.max_per_frame.min(batch.len());
        let mut loaded = Vec::new();
        for job in batch.iter().take(count) {
            let key = job.key().to_string();
            self.pending.remove(&key);
            loaded.push(key);
        }
        loaded
    }

    /// 取消指定 key
    pub fn cancel(&mut self, key: &str) {
        self.pending.remove(key);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl crate::engine::asset::runtime::service::RuntimeService for StreamingService {
    fn name(&self) -> &str {
        "StreamingService"
    }

    fn update(&mut self, _ctx: &mut RuntimeContext) {
        // 实际处理由 Scheduler 统一调度
    }
}
