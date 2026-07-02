use crate::content::block::registry::BlockRegistry;
use bevy::prelude::*;

/// 定义方块对各种事件的响应。
///
/// 泛型参数 `W` 是 World 访问器类型，
/// 由 Game 层在实现时提供具体类型（如 `WorldStorage`），
/// 从而避免 Content 层直接依赖 Game 层类型。
pub trait BlockBehavior<W>: Send + Sync + 'static {
    /// 方块被破坏
    fn on_break(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _world_storage: &mut W,
        _commands: &mut Commands,
    ) {
    }

    /// 方块被放置时调用，返回 false 可取消放置
    fn on_place(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _face_normal: IVec3,
        _world_storage: &mut W,
        _commands: &mut Commands,
    ) -> bool {
        true
    }

    /// 方块被右键交互时调用
    fn on_interact(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _face_normal: IVec3,
        _interactor: Option<Entity>,
        _world_storage: &mut W,
        _commands: &mut Commands,
    ) {
    }

    /// 方块状态更新
    fn on_change(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _world_storage: &mut W,
        _commands: &mut Commands,
    ) {
    }

    /// 邻居方块变更时调用
    fn on_neighbor_update(
        &self,
        _world_pos: IVec3,
        _block_id: u16,
        _neighbor_pos: IVec3,
        _neighbor_block_id: u16,
        _world_storage: &mut W,
        _commands: &mut Commands,
    ) {
    }
}

/// 默认空行为（对所有 W 类型可用）
pub struct DefaultBlockBehavior;
impl<W> BlockBehavior<W> for DefaultBlockBehavior {}

/// 查询方块是否为固体
pub fn is_solid(block_id: u16, block_registry: &BlockRegistry) -> bool {
    block_registry
        .get(block_id)
        .map(|p| p.is_solid)
        .unwrap_or(false)
}
