//! 定义跨层传递的标签标识值。

use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 标签标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagId(Identifier);

impl TagId {
    /// 使用给定参数创建新实例。
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self(Identifier::new(namespace, path))
    }
    /// 返回标识符的命名空间部分。
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }
    /// 返回标识符的路径部分。
    pub fn path(&self) -> &str {
        self.0.path()
    }
    /// 从完整命名空间形式解析标签标识。
    pub fn from_full(id: &str) -> Option<Self> {
        Identifier::parse(id).ok().map(Self)
    }
    /// 输出包含命名空间的完整标签标识。
    pub fn to_full(&self) -> String {
        self.0.to_string()
    }
    /// 输出数据文件使用的标签引用语法。
    pub fn to_reference(&self) -> String {
        format!("#{}", self.0)
    }
    /// 从标签引用语法解析标签标识。
    pub fn from_reference(s: &str) -> Option<Self> {
        s.strip_prefix('#').and_then(Self::from_full)
    }
}
impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
