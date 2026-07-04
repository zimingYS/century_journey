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
