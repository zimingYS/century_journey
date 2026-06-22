/// 资源包容器的运行时元数据
#[derive(Debug, Clone)]
pub struct ResourcePackMetadata {
    /// 资源包 ID
    pub id: String,
    /// 根目录
    pub root_path: String,
    /// 是否启用
    pub enabled: bool,
    /// 是否加载完成
    pub loaded: bool,
    /// 包含的资源文件数
    pub asset_count: u32,
}

impl ResourcePackMetadata {
    pub fn new(id: &str, root_path: &str) -> Self {
        Self {
            id: id.to_string(),
            root_path: root_path.to_string(),
            enabled: false,
            loaded: false,
            asset_count: 0,
        }
    }
}
