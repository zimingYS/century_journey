//! 定义玩家与世界存档共同使用的规范路径。

use std::path::PathBuf;

/// 所有单机世界存档共用的根目录名。
const SAVE_DIR_NAME: &str = "saves";

/// 返回一个世界的规范存档根目录。
///
/// 玩家、区块和元数据必须从这里派生路径，确保删除世界时不会留下可被同名
/// 新世界误读的孤立数据。
pub(super) fn world_save_root(world_name: &str) -> PathBuf {
    PathBuf::from(SAVE_DIR_NAME).join(world_name)
}

#[cfg(test)]
#[path = "../../../tests/unit/game/save/path.rs"]
mod tests;
