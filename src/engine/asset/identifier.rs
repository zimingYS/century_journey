use std::fmt;

/// 统一资源标识符。
///
/// 格式：`namespace:path`
///
/// # 示例
///
/// ```text
/// century_journey:block/grass
/// century_journey:item/apple
/// century_journey:texture/ui/slot
/// ```
///
/// 业务代码禁止直接写文件路径，必须使用 `AssetId`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId {
    /// 命名空间（如 `century_journey`）
    pub namespace: String,
    /// 资源路径（如 `block/grass`）
    pub path: String,
}

impl AssetId {
    /// 从 `namespace:path` 字符串解析。
    ///
    /// # Panics
    ///
    /// 输入不包含 `:` 时 panic。生产环境应使用 `try_parse`。
    pub fn parse(raw: &str) -> Self {
        let (ns, p) = raw
            .split_once(':')
            .unwrap_or_else(|| panic!("AssetId must be 'namespace:path' format, got: {raw}"));
        Self {
            namespace: ns.to_string(),
            path: p.to_string(),
        }
    }

    /// 安全解析，失败返回 `None`。
    pub fn try_parse(raw: &str) -> Option<Self> {
        let (ns, p) = raw.split_once(':')?;
        Some(Self {
            namespace: ns.to_string(),
            path: p.to_string(),
        })
    }

    /// 使用默认命名空间 `century_journey` 构造。
    pub fn default_namespace(path: &str) -> Self {
        Self {
            namespace: "century_journey".into(),
            path: path.to_string(),
        }
    }

    /// 返回 `namespace:path` 字符串。
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}
