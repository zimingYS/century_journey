/// 旧手持方块渲染器名称的兼容别名。
///
/// 实际实现已经迁移到统一物品渲染系统的方块 mesh 构建器中。
pub type HeldBlockRenderer =
    crate::client::renderer::item::mesh_builders::block_cube::BlockCubeMeshBuilder;
