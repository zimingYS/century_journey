//! 汇总标签定义、加载、编译与运行时索引。

pub mod compiler;
pub mod definition;
pub mod loader;
pub mod plugin;
pub mod runtime;

pub use plugin::TagContentPlugin;
