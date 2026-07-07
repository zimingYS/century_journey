use serde::{Deserialize, Serialize};

/// 标签操作类型
///
/// 对应 V3 方案中的 append / remove / replace。
/// 所有成员统一使用 Identifier 格式（不允许 RuntimeId）。
/// `#` 前缀表示引用其他 Tag（在 Compiler 阶段展开）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagAction {
    /// 追加成员
    Append { append: Vec<String> },
    /// 删除成员
    Remove { remove: Vec<String> },
    /// 替换整个 Tag
    Replace { replace: Vec<String> },
    /// Minecraft-style tag file: `{ "replace": bool, "values": [...] }`.
    Values {
        #[serde(default)]
        replace: bool,
        values: Vec<String>,
    },
}
