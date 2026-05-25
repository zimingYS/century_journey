use bevy::prelude::Component;
use crate::voxel::types::VoxelType;

/// 十字准心组件
#[derive(Component)]
pub struct Crosshair;

/// 创造模式背包物品栏组件
#[derive(Component)]
pub struct CreativeInventoryMenu;

/// 物品栏内的选择格子
#[derive(Component)]
pub struct PaletteSlot{
    pub voxel_type: VoxelType,
}