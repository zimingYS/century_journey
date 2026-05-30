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
    pub identifier: String,
}

/// HUD快捷栏(物品栏)
#[derive(Component)]
pub struct HudHotbarContainer;

/// HUD快捷栏(物品栏)中的单个物品槽位
#[derive(Component)]
pub struct HudHotbarSlot {
    pub index: usize,
}

/// 专门用来标记背包物品栏
#[derive(Component)]
pub struct PacksHotbarSlot {
    pub hotbar_index: usize,
}

/// HUD快捷栏(物品栏)外的高亮选择框
#[derive(Component)]
pub struct HudHotbarSelector;