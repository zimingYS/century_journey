//! 游戏规则与世界模拟层。
//!
//! Game 是玩法行为的唯一权威来源，负责方块行为、合成、物品栏、玩家状态和
//! 世界推进。Client 只展示结果，Content 只提供定义。

pub mod block;
pub mod command;
pub mod crafting;
pub mod gameplay;
pub mod inventory;
pub mod player;
pub mod plugin_group;
pub mod save;
pub mod simulation;
pub mod world;

pub use world::state::HeadlessWorldPlugin;
