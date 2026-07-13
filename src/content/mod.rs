//! 游戏内容定义层。
//!
//! Content 描述世界中存在的方块、物品、生物群系、配方、掉落表和标签，
//! 并通过注册表向 Game 与 Client 提供只读定义。行为规则不应放在本层。

pub mod biome;
pub mod block;
pub mod constant;
pub mod item;
pub mod loot;
pub mod recipe;
pub mod tag;
