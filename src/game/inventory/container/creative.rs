//! 保存创造模式分类筛选与目录选择状态，不拥有权威物品数量。

use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;

/// 创造模式物品栏的分类
#[derive(Debug, Clone)]
pub struct CreativeCategory {
    /// 分类名的本地化键（`creative.category.*`）。
    pub label_key: String,
    /// 键缺失时的兜底名；数据驱动的未知标签用标签路径派生名。
    pub label_fallback: String,
    /// 图标
    pub icon: String,
    /// 对应的标签ID
    pub tag_id: Option<TagId>,
    /// 该分类下的物品
    pub items: Vec<ItemId>,
}

impl CreativeCategory {
    /// 从标签注册表获得标签构建
    pub fn from_tag(
        tag_id: TagId,
        label_key: String,
        label_fallback: String,
        icon: String,
        items: Vec<ItemId>,
    ) -> Self {
        Self {
            label_key,
            label_fallback,
            icon,
            tag_id: Some(tag_id),
            items,
        }
    }

    /// 虚拟分类
    /// 用于类似“全部”、“收藏”等虚拟标签的分类
    pub fn virtual_category(label_key: &str, label_fallback: &str, icon: &str) -> Self {
        Self {
            label_key: label_key.to_string(),
            label_fallback: label_fallback.to_string(),
            icon: icon.to_string(),
            tag_id: None,
            items: Vec::new(),
        }
    }
}

/// 创造模式物品栏数据
#[derive(Debug, Clone, Default)]
pub struct CreativeData {
    /// 当前选中的分类索引
    pub selected_tab: usize,
    /// 搜索文本
    pub search_text: String,
    /// 动态构建的分类列表
    pub categories: Vec<CreativeCategory>,
    /// 过滤后的可见物品
    pub visible_items: Vec<ItemId>,
    /// 收藏的物品
    pub favorites: Vec<ItemId>,
}
