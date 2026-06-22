use crate::engine::asset::identifier::AssetId;

/// 资源加载请求
///
/// 由业务代码通过 `AssetManager` 提交，进入 Pipeline 处理。
#[derive(Debug, Clone)]
pub struct AssetRequest {
    /// 资源标识符
    pub id: AssetId,
    /// 资源类型标签（用于选择正确的 Loader）
    pub asset_type: String,
    /// 资源来源类型（filesystem / memory / network）
    pub source: String,
    /// 是否强制重新加载（跳过缓存）
    pub force_reload: bool,
}

impl AssetRequest {
    /// 创建纹理加载请求
    pub fn texture(id: AssetId) -> Self {
        Self {
            id,
            asset_type: "texture".into(),
            source: "filesystem".into(),
            force_reload: false,
        }
    }

    /// 创建 JSON 加载请求
    pub fn json(id: AssetId) -> Self {
        Self {
            id,
            asset_type: "json".into(),
            source: "filesystem".into(),
            force_reload: false,
        }
    }

    /// 创建自定义类型请求
    pub fn custom(id: AssetId, asset_type: &str) -> Self {
        Self {
            id,
            asset_type: asset_type.to_string(),
            source: "filesystem".into(),
            force_reload: false,
        }
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub fn with_force_reload(mut self) -> Self {
        self.force_reload = true;
        self
    }
}
