/// 资源已注册事件。
///
/// 在 `AssetRegistry::register()` 时发送。
#[derive(Debug, Clone)]
pub struct AssetRegistered {
    pub id: String,
}

/// 资源加载完成事件。
///
/// 在资源成功加载并缓存后发送。
#[derive(Debug, Clone)]
pub struct AssetLoaded {
    pub id: String,
}

/// 资源重新加载事件。
///
/// 在热重载触发后重新加载完成时发送。
#[derive(Debug, Clone)]
pub struct AssetReloaded {
    pub id: String,
}

/// 资源变更事件。
///
/// 在文件系统检测到资源变化时发送。
#[derive(Debug, Clone)]
pub struct AssetChanged {
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
