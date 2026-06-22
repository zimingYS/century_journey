/// 资源来源优先级
///
/// 数值越小优先级越高。SourceManager 按优先级从高到低查找。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourcePriority {
    /// 最高优先级 — 内存（Editor / Runtime Generated）
    Memory = 0,
    /// Mod 覆盖
    Mod = 1,
    /// 用户资源包
    UserResourcePack = 2,
    /// 默认资源包
    DefaultResourcePack = 3,
    /// 文件系统（最后兜底）
    Filesystem = 4,
    /// 网络来源（延迟加载）
    Network = 5,
}

impl SourcePriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Mod => "mod",
            Self::UserResourcePack => "user_resource_pack",
            Self::DefaultResourcePack => "default_resource_pack",
            Self::Filesystem => "filesystem",
            Self::Network => "network",
        }
    }
}
