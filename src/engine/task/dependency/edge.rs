/// 依赖图的边
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// 来源节点
    pub from: u64,
    /// 目标节点
    pub to: u64,
}
