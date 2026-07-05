use bevy::prelude::Component;

// ━━ 创造模式物品栏 ━━
/// 创造模式物品栏显示根目录
#[derive(Component)]
pub struct CreativeInventoryOverlay;
/// 创造模式物品栏UI
#[derive(Component)]
pub struct CreativeInventoryRoot;
/// 创造模式物品栏分类标签面板组件
#[derive(Component)]
pub struct CreativeCategoryPanel;
/// 创造模式物品栏物品网格组件
#[derive(Component)]
pub struct CreativeItemGrid;
/// 创造模式物品栏最近使用物品面板组件
#[derive(Component)]
pub struct CreativeRecentPanel;
/// 创造模式物品栏搜索框组件
#[derive(Component)]
pub struct CreativeSearchBox;
/// 创造模式快捷栏
#[derive(Component)]
pub struct CreativeHotbarPanel;

// ━━ 生存模式物品栏 ━━

/// 生存背包根容器
#[derive(Component)]
pub struct SurvivalInventoryRoot;

/// 生存背包覆盖层（背景遮罩）
#[derive(Component)]
pub struct SurvivalInventoryOverlay;

/// 生存背包物品网格（36 格）
#[derive(Component)]
pub struct SurvivalItemGrid;

/// 生存背包底部快捷栏（与 CreativeHotbarPanel 分离，避免 query.single() 冲突）
#[derive(Component)]
pub struct SurvivalHotbarPanel;

// ━━ 未来扩展 ━━
// #[derive(Component)] pub struct HealthBar;
// #[derive(Component)] pub struct HungerBar;
// #[derive(Component)] pub struct ContainerRoot;
