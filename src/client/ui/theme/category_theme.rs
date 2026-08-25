//! 定义创造物品分类标签的图标主题。

use bevy::prelude::*;
use std::collections::HashMap;

/// 分类图标主题：数据驱动的未知标签按图标表取图标，缺省用默认图标。
///
/// 分类名称不在此配置：静态分类使用 `creative.category.*` 本地化键，
/// 未知标签的兜底名由标签路径派生并在界面层通过 `get_or` 查询。
#[derive(Resource, Debug, Clone)]
pub struct CategoryTheme {
    icons: HashMap<String, String>,
}

impl Default for CategoryTheme {
    fn default() -> Self {
        let mut icons = HashMap::new();
        icons.insert("century_journey:solid".to_string(), "🪨".to_string());
        icons.insert("century_journey:natural".to_string(), "🌍".to_string());
        icons.insert(
            "century_journey:tree_plantable".to_string(),
            "🌱".to_string(),
        );
        Self { icons }
    }
}

impl CategoryTheme {
    /// 获取标签的图标，未配置则使用默认。
    pub fn icon(&self, tag_full: &str) -> String {
        self.icons
            .get(tag_full)
            .cloned()
            .unwrap_or_else(|| "📦".to_string())
    }
}
