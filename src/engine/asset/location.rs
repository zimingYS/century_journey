use std::path::PathBuf;

/// 已解析的资源物理位置——[`AssetResolver`](super::resolver::AssetResolver) 的唯一输出，
/// [`AssetPipeline`](super::pipeline::AssetPipeline) 和 [`AssetFiles`](super::files::AssetFiles) 的唯一输入。
///
/// 存在的意义：让"AssetId 怎么变成路径"只有一份实现，其它代码只消费这个结构体，
/// 不再各自拼 `format!("assets/{}...", ...)`。
#[derive(Debug, Clone)]
pub struct AssetLocation {
    /// 相对于 assets 根目录的路径，可直接传给 Bevy `AssetServer::load`
    pub relative_path: String,
    /// 完整文件系统路径，用于同步读取（`AssetFiles`、贴图图集烘焙等场景）
    pub full_path: PathBuf,
}

impl AssetLocation {
    pub fn new(relative_path: impl Into<String>, full_path: impl Into<PathBuf>) -> Self {
        Self {
            relative_path: relative_path.into(),
            full_path: full_path.into(),
        }
    }
}
