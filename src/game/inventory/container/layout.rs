//! 容器矩形布局。

/// 描述容器在界面与规则中共享的矩形布局。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerLayout {
    /// 每行槽位数。
    pub columns: usize,
    /// 容器行数。
    pub rows: usize,
}

impl ContainerLayout {
    /// 创建固定行列数的容器布局。
    pub const fn new(columns: usize, rows: usize) -> Self {
        Self { columns, rows }
    }

    /// 返回布局包含的槽位总数。
    pub const fn slot_count(self) -> usize {
        self.columns * self.rows
    }
}
