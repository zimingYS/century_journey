//! 组织云层定义的数据格式与注册表。
//!
//! 云层参数（高度、密度、速度、色调）属于纯表现配置，由 Content 层统一
//! 编译与校验，Client 层读取注册表生成渲染实体。不包含任何玩法规则。

pub mod definition;
pub mod plugin;
pub mod registry;

pub use definition::CloudDefinition;
pub use plugin::CloudContentPlugin;
pub use registry::CloudRegistry;
