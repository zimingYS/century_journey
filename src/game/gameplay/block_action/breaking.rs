//! 计算工具适配、可采集性和方块破坏耗时等规则。

use crate::content::block::definition::BlockProperty;
use crate::content::item::{ItemRegistry, ToolData};
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::tag::identifier::TagId;

const BLOCK_TAG_NAMESPACE: &str = "century_journey";
const UNBREAKABLE_TAG: &str = "unbreakable";
const REPLACEABLE_TAG: &str = "overworld_replaceable";
const BASE_BREAK_SECONDS_PER_HARDNESS: f32 = 1.0;
const MIN_SURVIVAL_BREAK_SECONDS: f32 = 0.08;
const MIN_TOOL_EFFICIENCY: f32 = 0.1;
const INCORRECT_TOOL_BREAK_TIME_MULTIPLIER: f32 = 5.0;

/// 判断指定方块在当前模式和标签规则下是否允许被破坏。
pub fn can_break_block(
    block_id: u16,
    gamemode: &PlayerGameMode,
    tags: Option<&RuntimeTagRegistry>,
) -> bool {
    if block_id == 0 {
        return false;
    }
    if gamemode.is_creative() {
        return true;
    }
    !is_unbreakable_block(block_id, tags)
}

/// 从当前手持物品解析工具属性；空堆或未知物品返回 `None`。
pub fn active_tool_data<'a>(
    active_stack: &ItemStack,
    item_registry: Option<&'a ItemRegistry>,
) -> Option<&'a ToolData> {
    if active_stack.is_empty() {
        return None;
    }

    item_registry
        .and_then(|registry| registry.get(&active_stack.item))
        .and_then(|definition| definition.tool_data())
}

/// 判断工具类型和采集等级是否满足方块掉落要求。
pub fn can_harvest_block(block: &BlockProperty, active_tool: Option<&ToolData>) -> bool {
    let Some(required_tool) = block.required_tool else {
        return true;
    };

    let Some(tool) = active_tool else {
        return false;
    };

    tool.tool_type == required_tool && tool.tier.harvest_level() >= block.harvest_level
}

/// 计算破坏方块所需秒数；返回 `None` 表示没有有效破坏规则。
pub fn block_break_seconds(
    block: &BlockProperty,
    gamemode: &PlayerGameMode,
    active_tool: Option<&ToolData>,
) -> Option<f32> {
    if gamemode.is_creative() {
        return Some(0.0);
    }
    if block.hardness <= 0.0 {
        return Some(0.0);
    }
    if !can_harvest_block(block, active_tool) {
        return Some(
            (block.hardness
                * BASE_BREAK_SECONDS_PER_HARDNESS
                * INCORRECT_TOOL_BREAK_TIME_MULTIPLIER)
                .max(MIN_SURVIVAL_BREAK_SECONDS),
        );
    }

    let effective_tool = block.effective_tool.or(block.required_tool);
    let efficiency = match (effective_tool, active_tool) {
        (Some(effective_tool), Some(tool)) if tool.tool_type == effective_tool => tool.efficiency,
        _ => 1.0,
    }
    .max(MIN_TOOL_EFFICIENCY);

    Some(
        (block.hardness * BASE_BREAK_SECONDS_PER_HARDNESS / efficiency)
            .max(MIN_SURVIVAL_BREAK_SECONDS),
    )
}

/// 判断运行时方块是否属于不可破坏标签。
pub fn is_unbreakable_block(block_id: u16, tags: Option<&RuntimeTagRegistry>) -> bool {
    tags.is_some_and(|tags| tags.contains(&block_tag(UNBREAKABLE_TAG), block_id))
}

/// 判断目标体素是否可被新方块直接替换。
pub fn is_replaceable_block(block_id: u16, tags: Option<&RuntimeTagRegistry>) -> bool {
    block_id == 0 || tags.is_some_and(|tags| tags.contains(&block_tag(REPLACEABLE_TAG), block_id))
}

fn block_tag(path: &str) -> TagId {
    TagId::new(BLOCK_TAG_NAMESPACE, path)
}
