//! 输入框历史回溯状态：记录最近提交的行，供 ↑/↓ 键翻阅。
//!
//! 本模块是纯数据状态机，不依赖 Bevy 类型；键盘采集系统在渲染帧驱动它，
//! 并把浏览结果整行写回 `EditableText`。

/// 输入历史最多保留的行数，超过后丢弃最旧的行，避免会话内无限增长。
const MAX_ENTRIES: usize = 128;

/// 输入框的历史回溯状态。
///
/// `entries` 在会话内持久（含指令行）；`position` 与 `draft` 只在输入框
/// 打开期间有效，关闭或提交后调用 [`InputHistory::reset_browsing`] 回到
/// 编辑新行的初始状态。
#[derive(Debug, Default)]
pub struct InputHistory {
    /// 最近提交的原始输入行（已去除首尾空白），最新的在末尾。
    entries: Vec<String>,
    /// 当前浏览到的历史下标；None 表示不在浏览状态（正在编辑新行）。
    position: Option<usize>,
    /// 首次进入浏览时保存的草稿，按 ↓ 越过最新一条时恢复。
    draft: String,
}

impl InputHistory {
    /// 记录一次提交的原始行：空白行与上一条重复时跳过。
    pub fn record(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || self.entries.last().is_some_and(|last| last == trimmed) {
            return;
        }
        self.entries.push(trimmed.to_owned());
        if self.entries.len() > MAX_ENTRIES {
            self.entries.remove(0);
        }
    }

    /// 浏览上一条（更旧）输入；首次进入浏览时保存当前草稿。
    ///
    /// 返回 None 表示没有可浏览的历史；已到最旧一条时停留并继续返回该条。
    pub fn browse_older(&mut self, current_line: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        let next = match self.position {
            Some(index) => index.saturating_sub(1),
            None => {
                self.draft = current_line.to_owned();
                self.entries.len() - 1
            }
        };
        self.position = Some(next);
        Some(self.entries[next].as_str())
    }

    /// 浏览下一条（更新）输入；越过最新一条时恢复草稿并回到编辑新行状态。
    ///
    /// 返回 None 表示当前不在浏览状态，输入框应保持不变。
    pub fn browse_newer(&mut self) -> Option<&str> {
        let index = self.position?;
        if index + 1 >= self.entries.len() {
            self.position = None;
            Some(self.draft.as_str())
        } else {
            self.position = Some(index + 1);
            Some(self.entries[index + 1].as_str())
        }
    }

    /// 重置浏览状态回到编辑新行；关闭输入框或提交后调用。
    pub fn reset_browsing(&mut self) {
        self.position = None;
        self.draft.clear();
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/console/input_history.rs"]
mod tests;
