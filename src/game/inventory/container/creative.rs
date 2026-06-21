use crate::game::inventory::item::id::ItemId;
use crate::shared::tag::identifier::TagId;

/// 创造模式物品栏的分类
#[derive(Debug, Clone)]
pub struct CreativeCategory {
    /// 显示名称
    pub display_name: String,
    /// 图标
    pub icon: String,
    /// 对应的标签ID
    pub tag_id: Option<TagId>,
    /// 该分类下的物品
    pub items: Vec<ItemId>,
}

impl CreativeCategory {
    /// 从标签注册表获得标签构建
    pub fn from_tag(tag_id: TagId, display_name: String, icon: String, items: Vec<ItemId>) -> Self {
        Self {
            display_name,
            icon,
            tag_id: Some(tag_id),
            items,
        }
    }

    /// 虚拟分类
    /// 用于类似“全部”、“收藏”等虚拟标签的分类
    pub fn virtual_category(display_name: &str, icon: &str) -> Self {
        Self {
            display_name: display_name.to_string(),
            icon: icon.to_string(),
            tag_id: None,
            items: Vec::new(),
        }
    }
}

/// 创造模式物品栏数据
#[derive(Debug, Clone)]
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

impl Default for CreativeData {
    fn default() -> Self {
        Self {
            selected_tab: 0,
            search_text: String::new(),
            categories: Vec::new(),
            visible_items: Vec::new(),
            favorites: Vec::new(),
        }
    }
}