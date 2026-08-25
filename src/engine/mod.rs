//! 通用基础设施层。
//!
//! Engine 只保留已经被项目使用、且不包含玩法知识的能力：
//! 资源解析与加载、持久化原语及异步任务门面。尚未形成真实实现的抽象不在此占位。

pub mod asset;
pub mod document;
pub mod localization;
pub mod persistence;
pub mod plugin_group;
pub mod task;
