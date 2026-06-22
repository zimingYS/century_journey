use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};

/// 判断当前是否可以放置方块
///
/// 预留未来逻辑：
/// - 生存模式：检查背包中是否有该方块
/// - 创造模式：始终允许
pub fn can_place_block(_block_id: u16, gamemode: &PlayerGameMode) -> bool {
    match gamemode.mode {
        GameMode::Creative => true,
        GameMode::Survival => {
            // TODO: 检查背包中是否有对应方块
            true // 暂时允许（开发阶段）
        }
    }
}

/// 判断当前是否可以破坏方块
///
/// 预留未来逻辑：
/// - 创造模式：可以破坏任何方块（含基岩）
/// - 生存模式：需要合适工具 + 不能破坏基岩
pub fn can_break_block(_block_id: u16, gamemode: &PlayerGameMode) -> bool {
    match gamemode.mode {
        GameMode::Creative => true,
        GameMode::Survival => {
            // TODO: 检查工具、硬度、基岩保护
            true // 暂时允许（开发阶段）
        }
    }
}
