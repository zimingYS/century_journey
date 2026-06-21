use crate::engine::asset::identifier::AssetId;

/// 资源路径解析器。
pub struct AssetPathResolver {
    /// 资源根目录
    root: String,
}

impl AssetPathResolver {
    /// 创建以`assets/`为根目录的默认解析器。
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into()
        }
    }

    /// 将 AssetId 解析为文件路径。
    /// 示例: `century_journey:block/grass` → `assets/block/grass.png`
    pub fn resolve(&self, id: &AssetId) -> String {
        format!("{}/{}.png", self.root, id.path)
    }

    /// 将 AssetId 解析为不带扩展名的路径。
    pub fn resolve_raw(&self, id: &AssetId) -> String {
        format!("{}/{}", self.root, id.path)
    }
}

impl Default for AssetPathResolver {
    fn default() -> Self {
        Self::new("assets")
    }
}
