use serde::{Deserialize, Serialize};

/// Mod 清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Mod 唯一标识符
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 命名空间（用于资源覆盖）
    pub namespace: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
}
