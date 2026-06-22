use crate::engine::asset::identifier::AssetId;
use std::collections::HashMap;

/// 资源数据库条目
#[derive(Debug, Clone)]
pub struct DatabaseEntry {
    /// 全局唯一标识符（跨会话稳定）
    pub guid: String,
    /// 内容哈希（用于检测变更）
    pub hash: u64,
    /// 版本号
    pub version: u32,
    /// 直接依赖
    pub dependencies: Vec<AssetId>,
    /// 来源
    pub source: String,
    /// 资源类型
    pub asset_type: String,
    /// 文件大小
    pub file_size: u64,
}

/// Asset Database — 资源信息数据库
///
/// 与 Registry（运行时状态）分离。
/// Database 保存资源的静态元信息（GUID/Hash/Version/Dependencies），
/// 供 Editor / Mod / Resource Pack 查询。
#[derive(Default)]
pub struct AssetDatabase {
    entries: HashMap<String, DatabaseEntry>,
}

impl AssetDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册资源信息
    pub fn register(&mut self, id: &AssetId, entry: DatabaseEntry) {
        self.entries.insert(id.to_string(), entry);
    }

    /// 查询资源信息
    pub fn get(&self, id: &AssetId) -> Option<&DatabaseEntry> {
        self.entries.get(&id.to_string())
    }

    /// 按 GUID 查找
    pub fn find_by_guid(&self, guid: &str) -> Option<&DatabaseEntry> {
        self.entries.values().find(|e| e.guid == guid)
    }

    /// 获取资源的直接依赖
    pub fn dependencies_of(&self, id: &AssetId) -> Vec<&AssetId> {
        self.entries
            .get(&id.to_string())
            .map(|e| e.dependencies.iter().collect())
            .unwrap_or_default()
    }

    /// 获取依赖指定资源的所有资源（反向依赖）
    pub fn dependents_of(&self, id: &AssetId) -> Vec<&String> {
        let target = id.to_string();
        self.entries
            .iter()
            .filter(|(_, e)| e.dependencies.iter().any(|d| d.to_string() == target))
            .map(|(k, _)| k)
            .collect()
    }

    /// 验证资源哈希是否变更
    pub fn has_changed(&self, id: &AssetId, new_hash: u64) -> bool {
        self.entries
            .get(&id.to_string())
            .map(|e| e.hash != new_hash)
            .unwrap_or(true)
    }

    /// 条目总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
