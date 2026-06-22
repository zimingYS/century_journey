use crate::engine::asset::runtime::context::RuntimeContext;
use std::collections::HashMap;

/// 内存逐出策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// 最近最少使用（默认）
    LRU,
    /// 最不频繁使用（Server 场景）
    LFU,
    /// 先进先出
    FIFO,
    /// 手动控制（Editor 场景）
    Manual,
}

/// 内存服务
///
/// 按 EvictionPolicy 管理和释放资源内存。
/// 支持按类型预算限制。
pub struct MemoryService {
    /// 每个资源的占用大小
    sizes: HashMap<String, u64>,
    /// 当前总占用
    current_usage: u64,
    /// 总预算
    budget: u64,
    /// 逐出策略
    policy: EvictionPolicy,
    /// 类型预算
    type_budgets: HashMap<String, u64>,
    /// 自动卸载计数
    unload_count: u32,
    /// 上次 cleanup 时间
    last_cleanup: f64,
    /// cleanup 间隔（秒）
    cleanup_interval: f64,
}

impl MemoryService {
    pub fn new(budget: u64, policy: EvictionPolicy) -> Self {
        let mut type_budgets = HashMap::new();
        type_budgets.insert("texture".into(), 512 * 1024 * 1024);
        type_budgets.insert("audio".into(), 256 * 1024 * 1024);
        type_budgets.insert("shader".into(), 128 * 1024 * 1024);
        Self {
            sizes: HashMap::new(),
            current_usage: 0,
            budget,
            policy,
            type_budgets,
            unload_count: 0,
            last_cleanup: 0.0,
            cleanup_interval: 5.0,
        }
    }

    /// 注册资源占用
    pub fn register(&mut self, key: &str, size: u64, _asset_type: &str) {
        self.sizes.insert(key.to_string(), size);
        self.current_usage += size;
    }

    /// 注销
    pub fn unregister(&mut self, key: &str) {
        if let Some(size) = self.sizes.remove(key) {
            self.current_usage = self.current_usage.saturating_sub(size);
        }
    }

    /// 是否超预算
    pub fn is_over_budget(&self) -> bool {
        self.current_usage > self.budget
    }

    /// 按策略获取逐出候选
    pub fn eviction_candidates(&self, ctx: &RuntimeContext, target_free: u64) -> Vec<String> {
        let mut candidates: Vec<(&String, &u64)> = self
            .sizes
            .iter()
            .filter(|(k, _)| ctx.is_unused(k))
            .collect();

        match self.policy {
            EvictionPolicy::LRU => {
                candidates.sort_by(|a, b| {
                    let ta = ctx.references.get(a.0).map_or(0.0, |e| e.last_access);
                    let tb = ctx.references.get(b.0).map_or(0.0, |e| e.last_access);
                    ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            EvictionPolicy::FIFO => {
                candidates.sort_by(|a, b| {
                    let ta = ctx.references.get(a.0).map_or(0.0, |e| e.first_loaded);
                    let tb = ctx.references.get(b.0).map_or(0.0, |e| e.first_loaded);
                    ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            EvictionPolicy::LFU => {
                // 按 strong_count 排序（最少使用的在前）
                candidates.sort_by(|a, b| {
                    let ca = ctx.references.get(a.0).map_or(0, |e| e.strong_count);
                    let cb = ctx.references.get(b.0).map_or(0, |e| e.strong_count);
                    ca.cmp(&cb)
                });
            }
            EvictionPolicy::Manual => return vec![],
        }

        let mut result = Vec::new();
        let mut freed = 0u64;
        for (key, size) in &candidates {
            result.push((*key).clone());
            freed += **size;
            if freed >= target_free {
                break;
            }
        }
        result
    }

    /// 定期清理（由 RuntimeScheduler 调用）
    pub fn cleanup(&mut self, ctx: &mut RuntimeContext) -> Vec<String> {
        let now = now_secs();
        if now - self.last_cleanup < self.cleanup_interval {
            return vec![];
        }
        self.last_cleanup = now;

        let candidates = self.eviction_candidates(ctx, 64 * 1024 * 1024);
        for key in &candidates {
            self.unregister(key);
            self.unload_count += 1;
        }
        candidates
    }

    pub fn current_usage(&self) -> u64 {
        self.current_usage
    }
    pub fn budget(&self) -> u64 {
        self.budget
    }
    pub fn unload_count(&self) -> u32 {
        self.unload_count
    }
}

impl crate::engine::asset::runtime::service::RuntimeService for MemoryService {
    fn name(&self) -> &str {
        "MemoryService"
    }

    fn update(&mut self, ctx: &mut RuntimeContext) {
        let _ = self.cleanup(ctx);
        ctx.diagnostics.memory_usage_bytes = self.current_usage;
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
