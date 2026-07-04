use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::tag::definition::TagAction;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::shared::identifier::Identifier;
use crate::shared::tag::identifier::TagId;
use std::collections::{HashMap, HashSet};

/// TagRegistryCompiler — 将 Definition 编译为 RuntimeTagRegistry
///
/// 编译流程:
/// 1. collect_defaults  — 从 BlockDefinition/ItemDefinition 收集 default_tags
/// 2. apply_overrides    — 读取 TagAction (append/remove/replace)
/// 3. resolve_references — 展开 #tag_ref 引用
/// 4. cycle_detect       — 检测循环引用
/// 5. build_runtime      — Identifier → RuntimeId 转换，生成最终 RuntimeTagRegistry
///
/// Compiler 实例在编译完成后立即销毁，不进入 Runtime。
pub struct TagRegistryCompiler {
    /// 编译中间状态: TagId → 成员 Identifier 集合
    pending: HashMap<TagId, HashSet<Identifier>>,
    /// 编译中间状态: TagId → 引用的 TagId 集合
    tag_refs: HashMap<TagId, HashSet<TagId>>,
}

impl TagRegistryCompiler {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            tag_refs: HashMap::new(),
        }
    }

    // ─── 第一阶段: 收集默认标签 ─────────────────────────

    /// 从 BlockProperty 收集 default_tags
    pub fn collect_from_blocks(&mut self, block_registry: &BlockRegistry) {
        for (&_block_id, prop) in block_registry.iter_properties() {
            let identifier = &prop.identifier;
            for tag_str in &prop.tags {
                let tag_id = self.parse_tag_str(tag_str);
                self.pending
                    .entry(tag_id)
                    .or_default()
                    .insert(identifier.clone());
            }
        }
    }

    /// 从 ItemDefinition 收集 default_tags
    pub fn collect_from_items(&mut self, item_registry: &ItemRegistry) {
        for def in item_registry.all_items() {
            let identifier = &def.identifier;
            for tag_str in &def.tags {
                let tag_id = self.parse_tag_str(tag_str);
                self.pending
                    .entry(tag_id)
                    .or_default()
                    .insert(identifier.clone());
            }
        }
    }

    // ─── 第二阶段: 应用 TagAction ───────────────────────

    /// 应用 TagActions (从 JSON 加载的 append/remove/replace)
    /// tag_id: 目标标签; tag_type: 标签类型 (block/item/biome); action: 操作
    pub fn apply_action(&mut self, tag_id: TagId, action: &TagAction) {
        match action {
            TagAction::Append { append } => {
                let entry = self.pending.entry(tag_id.clone()).or_default();
                for val in append {
                    if let Some(ref_tag) = val.strip_prefix('#') {
                        // Tag 引用 — 第三阶段展开
                        if let Some(ref_tag_id) = TagId::from_full(ref_tag) {
                            self.tag_refs
                                .entry(tag_id.clone())
                                .or_default()
                                .insert(ref_tag_id);
                        }
                    } else {
                        // 直接 Identifier
                        if let Ok(id) = Identifier::parse(val) {
                            entry.insert(id);
                        }
                    }
                }
            }
            TagAction::Remove { remove } => {
                if let Some(entry) = self.pending.get_mut(&tag_id) {
                    for val in remove {
                        if let Ok(id) = Identifier::parse(val) {
                            entry.remove(&id);
                        }
                    }
                }
            }
            TagAction::Replace { replace } => {
                let mut new_set = HashSet::new();
                for val in replace {
                    if let Some(ref_tag) = val.strip_prefix('#') {
                        if let Some(ref_tag_id) = TagId::from_full(ref_tag) {
                            self.tag_refs
                                .entry(tag_id.clone())
                                .or_default()
                                .insert(ref_tag_id);
                        }
                    } else if let Ok(id) = Identifier::parse(val) {
                        new_set.insert(id);
                    }
                }
                // 替换：清除旧 refs + 设置新成员
                self.tag_refs.remove(&tag_id);
                self.pending.insert(tag_id.clone(), new_set);
            }
        }
    }

    // ─── 第三阶段: 展开 Tag 引用 ───────────────────────

    /// 递归展开所有 #tag 引用，将引用 Tag 的成员合并到引用方
    pub fn resolve_references(&mut self) {
        let mut changed = true;
        let max_iterations = 100;

        for _ in 0..max_iterations {
            if !changed {
                break;
            }
            changed = false;

            // 收集所有需要展开的引用
            let refs: Vec<(TagId, TagId)> = self
                .tag_refs
                .iter()
                .flat_map(|(from, to_set)| to_set.iter().map(move |to| (from.clone(), to.clone())))
                .collect();

            for (from_tag, to_tag) in refs {
                let to_members: Vec<Identifier> = self
                    .pending
                    .get(&to_tag)
                    .map(|ids| ids.iter().cloned().collect())
                    .unwrap_or_default();

                if !to_members.is_empty() {
                    let from_entry = self.pending.entry(from_tag.clone()).or_default();
                    for id in &to_members {
                        if from_entry.insert(id.clone()) {
                            changed = true;
                        }
                    }
                    // 展开后清除该引用
                    self.tag_refs.entry(from_tag).or_default().remove(&to_tag);
                }
            }
        }
    }

    // ─── 第四阶段: 循环检测 ────────────────────────────

    /// 检测剩余的未解析引用，报告警告
    pub fn detect_unresolved(&self) -> Vec<(TagId, TagId)> {
        let mut unresolved = Vec::new();
        for (from, to_set) in &self.tag_refs {
            for to in to_set {
                // 检查被引用 Tag 是否没有任何成员
                if !self.pending.contains_key(to) {
                    unresolved.push((from.clone(), to.clone()));
                }
            }
        }
        unresolved
    }

    // ─── 第五阶段: 构建 Runtime ─────────────────────────

    /// 构建最终 RuntimeTagRegistry
    /// 将 Identifier 转换为 BlockRegistry 中的运行时 u16 ID
    pub fn build_runtime(
        self,
        block_registry: &BlockRegistry,
        _item_registry: &ItemRegistry,
    ) -> RuntimeTagRegistry {
        let mut runtime = RuntimeTagRegistry::default();

        for (tag_id, identifiers) in self.pending {
            let runtime_ids: HashSet<u16> = identifiers
                .iter()
                .filter_map(|id| block_registry.get_id_by_identifier(&id.to_string()))
                .collect();

            if !runtime_ids.is_empty() {
                runtime.insert(tag_id, runtime_ids);
            }
        }

        runtime
    }

    // ─── 辅助 ───────────────────────────────────────────

    fn parse_tag_str(&self, tag_str: &str) -> TagId {
        if let Some((ns, path)) = tag_str.split_once('/') {
            TagId::new(ns, path)
        } else {
            TagId::new("century_journey", tag_str)
        }
    }
}

impl Default for TagRegistryCompiler {
    fn default() -> Self {
        Self::new()
    }
}
