use crate::engine::task::dependency::node::DependencyNode;
use std::collections::{HashMap, HashSet};

/// Task依赖图
/// 管理所有任务的依赖关系。禁止循环依赖。
pub struct DependencyGraph {
    nodes: HashMap<u64, DependencyNode>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// 注册节点
    pub fn register(&mut self, task_id: u64) {
        self.nodes.entry(task_id).or_insert(DependencyNode {
            task_id,
            depends_on: Vec::new(),
            resolved: false,
        });
    }

    /// 添加依赖：from 依赖 to
    pub fn add_dependency(&mut self, from: u64, to: u64) {
        self.register(from);
        self.register(to);
        if let Some(node) = self.nodes.get_mut(&from)
            && !node.depends_on.contains(&to)
        {
            node.depends_on.push(to);
        }
    }

    /// 检查某个任务的所有依赖是否已解决
    pub fn all_resolved(&self, task_id: u64) -> bool {
        self.nodes.get(&task_id).is_none_or(|node| {
            node.depends_on
                .iter()
                .all(|dep| self.nodes.get(dep).is_none_or(|n| n.resolved))
        })
    }

    /// 标记节点为已解决
    pub fn mark_resolved(&mut self, task_id: u64) {
        if let Some(node) = self.nodes.get_mut(&task_id) {
            node.resolved = true;
        }
    }

    /// 获取依赖该任务的所有未解决任务
    pub fn dependents_of(&self, task_id: u64) -> Vec<u64> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.depends_on.contains(&task_id) && !node.resolved)
            .map(|(k, _)| *k)
            .collect()
    }

    /// 检查是否存在循环依赖
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        for &id in self.nodes.keys() {
            if !visited.contains(&id) && self.dfs_cycle(id, &mut visited, &mut in_stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(&self, id: u64, visited: &mut HashSet<u64>, in_stack: &mut HashSet<u64>) -> bool {
        visited.insert(id);
        in_stack.insert(id);
        if let Some(node) = self.nodes.get(&id) {
            for &dep in &node.depends_on {
                if in_stack.contains(&dep) {
                    return true;
                }
                if !visited.contains(&dep) && self.dfs_cycle(dep, visited, in_stack) {
                    return true;
                }
            }
        }
        in_stack.remove(&id);
        false
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
