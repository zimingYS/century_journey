use serde::{Deserialize, Serialize};

/// 标签文件支持的操作类型。
///
/// 成员统一使用 Identifier 字符串；井号前缀表示引用另一个标签，
/// 引用关系会在编译阶段展开。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagAction {
    /// 向现有标签追加成员。
    Append { append: Vec<String> },
    /// 从现有标签删除成员。
    Remove { remove: Vec<String> },
    /// 使用给定成员替换整个标签。
    Replace { replace: Vec<String> },
    /// 兼容带 replace 和 values 字段的 Minecraft 风格标签文件。
    Values {
        #[serde(default)]
        replace: bool,
        values: Vec<String>,
    },
}
