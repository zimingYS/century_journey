use crate::engine::asset::runtime::streaming::priority::StreamPriority;

/// 统一的 Runtime Job
///
/// Scheduler 统一管理所有 Job 类型，替代各 Manager 自有的 Queue。
#[derive(Debug, Clone)]
pub enum AssetJob {
    /// 流加载
    Streaming {
        key: String,
        priority: StreamPriority,
        timestamp: f64,
    },
    /// 热重载
    Reload { key: String, timestamp: f64 },
    /// 卸载（引用计数为 0 且超保留期）
    Unload { key: String, timestamp: f64 },
    /// 校验
    Validate { key: String, timestamp: f64 },
}

impl AssetJob {
    pub fn key(&self) -> &str {
        match self {
            Self::Streaming { key, .. }
            | Self::Reload { key, .. }
            | Self::Unload { key, .. }
            | Self::Validate { key, .. } => key,
        }
    }

    pub fn timestamp(&self) -> f64 {
        match self {
            Self::Streaming { timestamp, .. }
            | Self::Reload { timestamp, .. }
            | Self::Unload { timestamp, .. }
            | Self::Validate { timestamp, .. } => *timestamp,
        }
    }

    /// 优先级排序值（越小越优先）
    pub fn order(&self) -> u8 {
        match self {
            Self::Reload { .. } => 0,
            Self::Streaming { priority, .. } => *priority as u8,
            Self::Validate { .. } => 10,
            Self::Unload { .. } => 20,
        }
    }
}
