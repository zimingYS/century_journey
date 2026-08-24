//! 根据输入框当前内容生成指令补全候选与用法提示。
//!
//! 本模块是纯函数：根名候选由注册表生成，参数候选委托各指令家族的
//! 候选函数，保持指令知识的单一事实来源。候选所需但纯函数自身无法
//! 访问的注册表数据（如物品清单）由调用方通过 [`SuggestContext`] 提供。

use crate::content::item::registry::ItemRegistry;

use super::parse::{COMMANDS, CommandSpec};

/// 指令候选计算的上下文：承载候选需要、但纯函数自身无法访问的注册表数据。
#[derive(Debug, Default, Clone)]
pub struct SuggestContext {
    /// 物品注册表中全部物品的短名（已排序、去重，不含命名空间前缀）。
    pub(crate) item_names: Vec<String>,
}

impl SuggestContext {
    /// 从物品注册表构建候选上下文；物品短名排序以保证候选顺序稳定。
    pub fn from_registry(registry: &ItemRegistry) -> Self {
        let mut item_names: Vec<String> = registry
            .all_items()
            .map(|definition| definition.identifier.path().to_owned())
            .collect();
        item_names.sort();
        item_names.dedup();
        Self { item_names }
    }
}

/// 当前输入行的指令补全结果。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandCompletions {
    /// 可整行替换输入框的候选行（以空格结尾，便于继续输入参数）。
    pub lines: Vec<String>,
    /// 唯一匹配到的指令家族用法提示；根级歧义或未匹配时为 None。
    pub usage: Option<&'static str>,
}

/// 为完整输入行（含前导 '/'）计算补全候选与用法提示。
///
/// 只处理指令行；普通文本行返回空结果。正在补全的词取行尾最后一个词，
/// 行尾以空白结尾时视为尚未输入的下一个新词；匹配均忽略大小写。
pub fn completions(line: &str, context: &SuggestContext) -> CommandCompletions {
    let Some(stripped) = line.trim_start().strip_prefix('/') else {
        return CommandCompletions::default();
    };
    let ends_with_space = stripped.ends_with(char::is_whitespace);
    let tokens: Vec<&str> = stripped.split_whitespace().collect();
    let (completed, prefix) = match tokens.split_last() {
        Some((last, rest)) if !ends_with_space => (rest, *last),
        _ => (&tokens[..], ""),
    };
    if completed.is_empty() {
        complete_root(prefix)
    } else {
        complete_arguments(completed, prefix, context)
    }
}

/// 补全指令根名：返回以 prefix 开头的指令完整行与唯一匹配的用法。
fn complete_root(prefix: &str) -> CommandCompletions {
    let lowered = prefix.to_ascii_lowercase();
    let matches: Vec<&CommandSpec> = COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(&lowered))
        .collect();
    CommandCompletions {
        lines: matches
            .iter()
            .map(|spec| format!("/{} ", spec.name))
            .collect(),
        usage: if matches.len() == 1 {
            Some(matches[0].usage)
        } else {
            None
        },
    }
}

/// 补全指令参数：委托匹配根名的家族候选函数生成候选词。
///
/// 候选行的根名使用注册表的规范小写形式，已输入的子指令原样保留。
fn complete_arguments(
    completed: &[&str],
    prefix: &str,
    context: &SuggestContext,
) -> CommandCompletions {
    let Some(spec) = COMMANDS
        .iter()
        .find(|spec| spec.name.eq_ignore_ascii_case(completed[0]))
    else {
        return CommandCompletions::default();
    };
    let candidates = (spec.suggest)(&completed[1..], prefix, context);
    let head = std::iter::once(spec.name)
        .chain(completed[1..].iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    CommandCompletions {
        lines: candidates
            .iter()
            .map(|candidate| format!("/{head} {candidate} "))
            .collect(),
        usage: Some(spec.usage),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/game/command/suggest.rs"]
mod tests;
