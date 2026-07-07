/// 纹理用途标签
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureUsage {
    Block,
    Item,
    #[default]
    UI,
    Font,
    Particle,
    Entity,
    Sky,
    Icon,
    Cursor,
}
