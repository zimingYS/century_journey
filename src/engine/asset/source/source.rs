use crate::engine::asset::source::priority::SourcePriority;

/// 文件级资源元数据（单个文件）
#[derive(Debug, Clone)]
pub struct SourceFileMetadata {
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间
    pub modified: Option<u64>,
    /// 是否为目录
    pub is_dir: bool,
    /// 来源类型名称
    pub source_type: String,
}

/// 资源来源元数据（来源本身）
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    /// 来源名称
    pub name: String,
    /// 优先级
    pub priority: SourcePriority,
    /// 版本号
    pub version: u32,
    /// 是否启用
    pub enabled: bool,
    /// 根路径
    pub root_path: String,
    /// 描述
    pub description: String,
}

/// 统一资源来源 Trait
///
/// 所有资源来源（FileSystem / Memory / ResourcePack / Mod / Network）
/// 都必须实现此 Trait。Pipeline 只能操作此 Trait，不知道具体类型。
pub trait AssetSource: Send + Sync + 'static {
    /// 来源名称
    fn name(&self) -> &str;

    /// 来源优先级
    fn priority(&self) -> SourcePriority;

    /// 检查指定路径的资源是否存在
    fn exists(&self, path: &str) -> bool;

    /// 读取指定路径资源的全部字节
    fn read(&self, path: &str) -> Result<Vec<u8>, String>;

    /// 获取资源文件元数据
    fn metadata(&self, path: &str) -> Option<SourceFileMetadata>;

    /// 获取来源本身元数据
    fn source_metadata(&self) -> SourceMetadata;

    /// 是否启用
    fn is_enabled(&self) -> bool;

    /// 启用/禁用此来源
    fn set_enabled(&mut self, enabled: bool);
}
