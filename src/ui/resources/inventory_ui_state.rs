use bevy::prelude::Resource;
use crate::voxel::types::VoxelType;

#[derive(Resource, Debug)]
pub struct InventoryUiState {
    /// 背包主界面是否打开
    pub is_inventory_open: bool,
    /// 当前快捷栏选中的格子索引
    pub active_hotbar_index: usize,
    /// 快捷栏绑定的方块数据
    pub hotbar_items: [VoxelType; 9],
    /// 创造模式背包里所有可以选用的方块合集
    pub creative_palette: Vec<VoxelType>,
}

impl Default for InventoryUiState {
    fn default() -> Self {
        Self {
            is_inventory_open: false,
            active_hotbar_index: 0,
            hotbar_items: [
                VoxelType::Grass, VoxelType::Dirt, VoxelType::Stone,
                VoxelType::Air, VoxelType::Air, VoxelType::Air,
                VoxelType::Air, VoxelType::Air, VoxelType::Air,
            ],
            creative_palette: vec![
                VoxelType::Grass, VoxelType::Dirt, VoxelType::Stone,
                VoxelType::Sand, VoxelType::Water, VoxelType::Wood,
                VoxelType::Leaves,
            ],
        }
    }
}