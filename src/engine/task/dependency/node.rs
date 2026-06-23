/// 依赖图的节点
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// 任务 ID
    pub task_id: u64,
    /// 依赖的其他节点 ID
    pub depends_on: Vec<u64>,
    /// 是否已解决
    pub resolved: bool,
}
