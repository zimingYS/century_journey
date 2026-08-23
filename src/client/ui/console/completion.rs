//! Tab 补全的循环选择状态。
//!
//! 本模块是纯数据状态机，不依赖 Bevy 类型；键盘采集系统在渲染帧驱动它，
//! 把选中的候选整行写回输入框。

/// Tab 补全的循环选择状态。
///
/// 记录上次补全写回的完整行：再次按 Tab 时若输入行未被编辑过，则循环
/// 前进到下一个候选；任何手工编辑都会让选择回到第一个候选。
#[derive(Debug, Default)]
pub struct CompletionTracker {
    /// 当前选中的候选下标。
    index: usize,
    /// 上次补全写回的完整行；与当前输入行相同表示尚未手工编辑。
    anchor: Option<String>,
}

impl CompletionTracker {
    /// 为当前输入行选出补全候选，返回要写回输入框的完整行。
    ///
    /// 候选为空时重置状态并返回 None；连续 Tab 在候选间循环。
    pub fn choose(&mut self, current: &str, candidates: &[String]) -> Option<String> {
        if candidates.is_empty() {
            self.reset();
            return None;
        }
        let advancing = self.anchor.as_deref() == Some(current);
        self.index = if advancing {
            (self.index + 1) % candidates.len()
        } else {
            0
        };
        let chosen = candidates[self.index].clone();
        self.anchor = Some(chosen.clone());
        Some(chosen)
    }

    /// 重置循环状态；关闭输入框或候选为空时调用。
    pub fn reset(&mut self) {
        self.index = 0;
        self.anchor = None;
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/console/completion.rs"]
mod tests;
