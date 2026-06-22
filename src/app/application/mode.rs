//! # Mode
//!
//! 应用运行模式。
//!
//! 定义程序支持的运行模式及其行为。

/// 应用运行模式
#[derive(Clone, Debug)]
pub enum AppMode {
    /// 客户端
    Client,
    /// 服务端
    Server,
    /// 编辑器
    Editor,
}
