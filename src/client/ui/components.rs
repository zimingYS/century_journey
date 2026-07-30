//! 定义多个界面屏幕共享的纯表现标记组件。

use bevy::prelude::Component;

// ===== 创造模式物品栏 =====

/// 创造模式物品栏遮罩根节点。
#[derive(Component)]
pub struct CreativeInventoryOverlay;

/// 创造模式物品栏主面板。
#[derive(Component)]
pub struct CreativeInventoryRoot;

/// 创造模式物品栏左侧分类面板。
#[derive(Component)]
pub struct CreativeCategoryPanel;

/// 创造模式物品栏中间物品网格。
#[derive(Component)]
pub struct CreativeItemGrid;

/// 创造模式物品栏右侧最近使用面板。
#[derive(Component)]
pub struct CreativeRecentPanel;

/// 创造模式物品栏搜索框容器。
#[derive(Component)]
pub struct CreativeSearchBox;

/// 创造模式物品栏搜索占位文字。
#[derive(Component)]
pub struct CreativeSearchPlaceholder;

/// 创造模式物品栏关闭按钮。
#[derive(Component)]
pub struct CreativeCloseButton;

/// 创造模式物品栏标题图标。
#[derive(Component)]
pub struct CreativeTitleIcon;

/// 创造模式物品栏内置快捷栏。
#[derive(Component)]
pub struct CreativeHotbarPanel;

// ===== 生存模式物品栏 =====

/// 生存背包根容器。
#[derive(Component)]
pub struct SurvivalInventoryRoot;

/// 生存背包遮罩层。
#[derive(Component)]
pub struct SurvivalInventoryOverlay;

/// 生存背包物品网格。
#[derive(Component)]
pub struct SurvivalItemGrid;

/// 生存背包底部快捷栏，和 CreativeHotbarPanel 分离以避免 query.single() 冲突。
#[derive(Component)]
pub struct SurvivalHotbarPanel;

/// 生存背包装备槽区域。
#[derive(Component)]
pub struct SurvivalEquipmentPanel;

/// 生存背包饰品槽区域。
#[derive(Component)]
pub struct SurvivalAccessoryPanel;

/// 生存背包玩家模型预览相机。
#[derive(Component)]
pub struct SurvivalPlayerPreviewCamera;

/// 生存背包中显示当前生命值的文本。
#[derive(Component, Default)]
pub struct SurvivalHealthText;

/// 生存背包中显示当前防御值的文本。
#[derive(Component, Default)]
pub struct SurvivalDefenseText;

/// 生存背包中显示当前饥饿值的文本。
#[derive(Component, Default)]
pub struct SurvivalHungerText;

/// 用于切换紧凑背包布局的按钮。
#[derive(Component, Default)]
pub struct CompactBackpackButton;

/// 用于请求整理背包的按钮。
#[derive(Component, Default)]
pub struct SortBackpackButton;
