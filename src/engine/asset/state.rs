use crate::engine::asset::identifier::AssetId;

/// 资源生命周期状态（完整12态）
///
/// ```text
/// Registered → Resolving → Loading → LoadedBytes → Deserializing
///                                                  ↓
///                                              Processing → Validated → Caching → Ready
///                                                                                  ↓
///                                                                              Reloading → Loading → ...
///                                                              Processing → Failed
///                                                                           Disposed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    /// 已注册，等待 Pipeline 开始
    Registered,
    /// 正在 Resolver Stage
    Resolving,
    /// 正在从 Source 读取原始字节
    Loading,
    /// 原始字节已加载，等待反序列化
    LoadedBytes,
    /// 正在 Deserialize / Loader Stage
    Deserializing,
    /// 正在 Processor Chain
    Processing,
    /// 已通过 Validator 验证
    Validated,
    /// 正在写入 Cache
    Caching,
    /// 加载完成，Handle 可用
    Ready,
    /// 正在热重载
    Reloading,
    /// 加载失败
    Failed,
    /// 已卸载
    Disposed,
}

/// 增强版资源元数据
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub id: AssetId,
    pub asset_type: String,
    pub state: AssetState,
    pub source: String,
    pub load_time: Option<f64>,
    pub ref_count: u32,
    pub file_size: Option<u64>,
    pub dependency_count: u32,
    pub version: u32,
    pub last_error: Option<String>,
}

impl AssetMetadata {
    pub fn new(id: AssetId, source: impl Into<String>) -> Self {
        Self {
            id,
            asset_type: String::new(),
            state: AssetState::Registered,
            source: source.into(),
            load_time: None,
            ref_count: 0,
            file_size: None,
            dependency_count: 0,
            version: 1,
            last_error: None,
        }
    }
}
