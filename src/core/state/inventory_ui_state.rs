use bevy::prelude::Resource;

#[derive(Resource, Debug)]
pub struct InventoryUiState {
    /// 背包界面是否打开
    pub is_inventory_open: bool,
    /// 当前选中快捷栏格 (0-8)
    pub active_hotbar_index: usize,
    /// 快捷栏9格方块标识符
    pub hotbar_items: [String; 9],
    /// 创造模式可用方块列表
    pub creative_palette: Vec<String>,
    /// 标签分类列表
    pub tag_categories: Vec<InventoryTagCategory>,
    /// 当前选中的标签分类索引
    pub active_category_index: usize,
}

/// 创造模式背包中的标签分类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryTagCategory {
    /// 标签的显示名称
    pub display_name: String,
    /// 标签完整标识符（如 "century_journey:natural"）
    pub tag_full: String,
    /// 该标签下的方块标识符列表
    pub items: Vec<String>,
}

impl Default for InventoryUiState {
    fn default() -> Self {
        Self {
            is_inventory_open: false,
            active_hotbar_index: 0,
            hotbar_items: [
                "century_journey:grass".to_string(),
                "century_journey:dirt".to_string(),
                "century_journey:stone".to_string(),
                "century_journey:air".to_string(),
                "century_journey:air".to_string(),
                "century_journey:air".to_string(),
                "century_journey:air".to_string(),
                "century_journey:air".to_string(),
                "century_journey:air".to_string(),
            ],
            creative_palette: Vec::new(),
            tag_categories: Vec::new(),
            active_category_index: 0,
        }
    }
}

impl InventoryUiState {
    /// 获取当前分类下应显示的方块列表
    pub fn current_category_items(&self) -> Vec<String> {
        if self.active_category_index == 0 {
            self.creative_palette.clone()
        } else {
            let idx = self.active_category_index - 1;
            self.tag_categories
                .get(idx)
                .map(|c| c.items.clone())
                .unwrap_or_default()
        }
    }
}