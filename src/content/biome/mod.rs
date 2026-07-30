//! 汇总生物群系定义、加载器、注册表和插件。

pub mod definition;
pub mod loader;
pub mod plugin;
pub mod registry;

pub use definition::{BiomeDefinition, BiomeTerrainParams};
pub use registry::BiomeRegistry;
