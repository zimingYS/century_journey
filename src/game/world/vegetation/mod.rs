//! 实现已加载世界中的稀疏植被索引和权威树苗生长规则。

mod growth;
mod plugin;
mod runtime;

pub(in crate::game::world) use plugin::VegetationPlugin;
