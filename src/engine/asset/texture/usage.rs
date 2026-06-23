/// 纹理用途标签
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureUsage {
    Block,
    Item,
    UI,
    Font,
    Particle,
    Entity,
    Sky,
    Icon,
    Cursor,
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self::UI
    }
}
