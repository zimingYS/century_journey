use crate::content::item::model::{ItemModelDefinition, ItemModelRegistry};
use crate::content::item::registry::registry::ItemRegistry;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;

/// ItemId 解析后的模型定义。
pub struct ResolvedItemModel {
    /// 最终使用的模型 ID，也是 BakedModel 缓存键。
    pub model_id: Identifier,
    /// 最终用于烘焙的模型定义。
    pub definition: ItemModelDefinition,
}

/// 物品模型解析器。
///
/// 它只负责 ItemId -> ItemModelDefinition：优先显式 model 字段，其次同名模型文件，最后根据物品定义生成 fallback。
pub struct ItemModelResolver;

impl ItemModelResolver {
    /// 解析一个物品应该使用的模型定义。
    pub fn resolve(
        item: &ItemId,
        item_registry: Option<&ItemRegistry>,
        model_registry: Option<&ItemModelRegistry>,
    ) -> Option<ResolvedItemModel> {
        let item_identifier = item.identifier().clone();

        if let Some(item_definition) = item_registry.and_then(|registry| registry.get(item))
            && let Some(model_id) = &item_definition.model
            && let Some(model) = model_registry.and_then(|registry| registry.get(model_id))
        {
            return Some(ResolvedItemModel {
                model_id: model_id.clone(),
                definition: model.clone(),
            });
        }

        if let Some(model) = model_registry.and_then(|registry| registry.get(&item_identifier)) {
            return Some(ResolvedItemModel {
                model_id: item_identifier,
                definition: model.clone(),
            });
        }

        if let Some(item_definition) = item_registry.and_then(|registry| registry.get(item)) {
            return Some(ResolvedItemModel {
                model_id: item_identifier,
                definition: ItemModelDefinition::fallback_for_item_definition(item_definition),
            });
        }

        Some(ResolvedItemModel {
            model_id: item_identifier.clone(),
            definition: ItemModelDefinition::generated(item_identifier, 0.05, false),
        })
    }
}
