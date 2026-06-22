use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 资源包清单
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct ResourcePackManifest {
    /// 唯一标识符
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 版本
    pub version: String,
    /// 作者
    pub author: String,
}

impl Default for ResourcePackManifest {
    fn default() -> Self {
        Self {
            id: "default".into(),
            display_name: "Default Resource Pack".into(),
            description: "CenturyJourney default resources".into(),
            version: "1.0.0".into(),
            author: "CenturyJourney".into(),
        }
    }
}
