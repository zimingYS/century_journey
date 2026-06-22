use crate::engine::asset::identifier::AssetId;
use std::collections::{HashMap, HashSet};

/// 资源依赖图
///
/// 维护资源之间的依赖关系（如 grass.json → grass.png, grass_break.ogg）。
/// 用于 Hot Reload 时自动刷新所有受影响的资源。
#[derive(Default)]
pub struct DependencyGraph {
    /// 正向依赖: parent → children
    depends_on: HashMap<String, HashSet<String>>,
    /// 反向依赖: child → parents
    depended_by: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// 添加依赖：`from` 依赖 `to`
    pub fn add_dependency(&mut self, from: &AssetId, to: &AssetId) {
        let f = from.to_string();
        let t = to.to_string();
        self.depends_on
            .entry(f.clone())
            .or_default()
            .insert(t.clone());
        self.depended_by.entry(t).or_default().insert(f);
    }

    /// 获取某个资源的直接依赖
    pub fn dependencies_of(&self, id: &AssetId) -> Vec<&String> {
        self.depends_on
            .get(&id.to_string())
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    /// 获取依赖某个资源的所有资源（用于级联重载）
    pub fn dependents_of(&self, id: &AssetId) -> Vec<&String> {
        self.depended_by
            .get(&id.to_string())
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    /// 移除资源的所有依赖关系
    pub fn remove(&mut self, id: &AssetId) {
        let key = id.to_string();
        // 移除正向依赖
        if let Some(deps) = self.depends_on.remove(&key) {
            for d in &deps {
                if let Some(parents) = self.depended_by.get_mut(d) {
                    parents.remove(&key);
                }
            }
        }
        // 移除反向依赖
        if let Some(parents) = self.depended_by.remove(&key) {
            for p in &parents {
                if let Some(deps) = self.depends_on.get_mut(p) {
                    deps.remove(&key);
                }
            }
        }
    }

    /// 依赖图是否为空
    pub fn is_empty(&self) -> bool {
        self.depends_on.is_empty()
    }

    /// 获取总依赖关系数
    pub fn len(&self) -> usize {
        self.depends_on.values().map(|s| s.len()).sum()
    }
}
