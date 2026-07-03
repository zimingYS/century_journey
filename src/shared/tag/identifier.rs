use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 标签标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagId(Identifier);

impl TagId {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self(Identifier::new(namespace, path))
    }
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }
    pub fn path(&self) -> &str {
        self.0.path()
    }
    pub fn from_full(id: &str) -> Option<Self> {
        Identifier::parse(id).ok().map(Self)
    }
    pub fn to_full(&self) -> String {
        self.0.to_string()
    }
    pub fn to_reference(&self) -> String {
        format!("#{}", self.0)
    }
    pub fn from_reference(s: &str) -> Option<Self> {
        s.strip_prefix('#').and_then(Self::from_full)
    }
}
impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 标签注册表类型
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TagRegistryType {
    Block,
    Item,
    Biome,
    Fluid,
    // 其他
    Custom(String),
}

impl TagRegistryType {
    /// 根据目录名进行标签分类
    pub fn from_dir_name(dir: &str) -> Self {
        match dir {
            "block" => Self::Block,
            "item" => Self::Item,
            "biome" => Self::Biome,
            "fluid" => Self::Fluid,
            _ => Self::Custom(dir.to_owned()),
        }
    }

    /// 根据标签转换为目录名进行分类
    pub fn to_dir_name(&self) -> &str {
        match self {
            Self::Block => "block",
            Self::Item => "item",
            Self::Biome => "biome",
            Self::Fluid => "fluid",
            Self::Custom(dir) => dir.as_str(),
        }
    }
}
