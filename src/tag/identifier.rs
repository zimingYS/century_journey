use std::fmt;
use serde::{Deserialize, Serialize};

/// 标签标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagId{
    /// 标签ID
    namespace: String,
    /// 标签ID路径
    path: String,
}

impl TagId{
    // 支持自动接收&str等可转为String的类型
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// 从完整标识符解析成TAG
    pub fn from_full(id: &str) -> Option<Self> {
        let (namespace , path) = id.split_once(':')?;
        if namespace.is_empty() || path.is_empty() {
            return None;
        }
        Some(Self::new(namespace, path))
    }

    /// 转为完整标识符字符串
    pub fn to_full(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }

    /// 标签引用格式
    pub fn to_reference(&self) -> String {
        format!("#{}", self.to_full())
    }

    /// 从标签引用字符串解析
    pub fn from_reference(ref_str: &str) -> Option<Self> {
        let trimmed = ref_str.strip_prefix('#')?;
        Self::from_full(trimmed)
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
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
    pub fn from_dir_name(dir: &str) -> Self{
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
