use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::*;
use std::collections::HashMap;

/// 配方注册表
#[derive(Resource, Default)]
pub struct RecipeRegistry {
    entries: HashMap<Identifier, RecipeDefinition>,
}

impl RecipeRegistry {
    pub fn register(&mut self, id: Identifier, def: RecipeDefinition) {
        if self.entries.insert(id.clone(), def).is_some() {
            log::warn!("[Recipe] 重复注册配方：{}", id);
        }
    }

    /// 根据 Identifier 获取配方
    pub fn get(&self, id: &Identifier) -> Option<&RecipeDefinition> {
        self.entries.get(id)
    }

    /// 遍历所有配方
    pub fn all_recipes(&self) -> impl Iterator<Item = (&Identifier, &RecipeDefinition)> {
        self.entries.iter()
    }

    // 先不加粗筛索引（比如按首个非Tag材料分桶）。
    // 等 game/crafting 的匹配系统跑起来、你能实测配方数量级之后，
    // 如果线性扫描确实是瓶颈，再回来加，不要现在预先优化。
}
