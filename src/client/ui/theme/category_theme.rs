use bevy::prelude::*;
use std::collections::HashMap;

/// 分类显示配置
#[derive(Resource, Debug, Clone)]
pub struct CategoryTheme {
    pub entries: HashMap<String, CategoryDisplay>,
}

/// 单个分类的显示信息
#[derive(Debug, Clone)]
pub struct CategoryDisplay {
    pub display_name: String,
    pub icon: String,
}

impl Default for CategoryTheme {
    fn default() -> Self {
        let mut entries = HashMap::new();
        entries.insert(
            "century_journey:solid".to_string(),
            CategoryDisplay {
                display_name: "固体".into(),
                icon: "🪨".into(),
            },
        );
        entries.insert(
            "century_journey:natural".to_string(),
            CategoryDisplay {
                display_name: "自然".into(),
                icon: "🌍".into(),
            },
        );
        entries.insert(
            "century_journey:tree_plantable".to_string(),
            CategoryDisplay {
                display_name: "作物".into(),
                icon: "🌱".into(),
            },
        );
        Self { entries }
    }
}

impl CategoryTheme {
    /// 获取标签的显示名，未配置则使用TagPath
    pub fn display_name(&self, tag_full: &str) -> String {
        self.entries
            .get(tag_full)
            .map(|e| e.display_name.clone())
            .unwrap_or_else(|| {
                // 例如：从 "century_journey:solid" 提取 "solid" 并将首字母大写。
                tag_full
                    .split(':')
                    .next_back()
                    .unwrap_or(tag_full)
                    .to_string()
            })
    }

    /// 获取标签的图标，未配置则使用默认
    pub fn icon(&self, tag_full: &str) -> String {
        self.entries
            .get(tag_full)
            .map(|e| e.icon.clone())
            .unwrap_or_else(|| "📦".to_string())
    }
}
