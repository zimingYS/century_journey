/// 资源加载完成事件。
/// 在资源成功加载并缓存后发送。
#[derive(Debug, Clone)]
pub struct AssetLoaded {
    pub id: String,
}

/// 资源重新加载事件
/// 在文件系统变更触发重新加载后发送。
#[derive(Debug, Clone)]
pub struct AssetReloaded {
    pub id: String,
}

/// 资源加载失败事件。
#[derive(Debug, Clone)]
pub struct AssetLoadFailed {
    pub id: String,
    pub error: String,
}

/// 资源已卸载事件。
#[derive(Debug, Clone)]
pub struct AssetUnloaded {
    pub id: String,
}
