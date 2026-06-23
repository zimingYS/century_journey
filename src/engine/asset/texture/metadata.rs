use crate::engine::asset::texture::usage::TextureUsage;

/// 纹理元数据
#[derive(Debug, Clone)]
pub struct TextureMetadata {
    /// 像素宽度
    pub width: u32,
    /// 像素高度
    pub height: u32,
    /// 宽高比 (width / height)
    pub aspect_ratio: f32,
    /// 是否为像素风格（默认 true）
    pub pixel_art: bool,
    /// 纹理用途
    pub usage: TextureUsage,
    /// 是否有 mipmap
    pub mip_level: u32,
}

impl TextureMetadata {
    /// 从图像尺寸创建元数据
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            aspect_ratio: width as f32 / height.max(1) as f32,
            pixel_art: true,
            usage: TextureUsage::default(),
            mip_level: 0,
        }
    }

    /// 指定用途
    pub fn with_usage(mut self, usage: TextureUsage) -> Self {
        self.usage = usage;
        self
    }

    /// 设置为非像素艺术（线性采样）
    pub fn with_smooth(mut self) -> Self {
        self.pixel_art = false;
        self
    }
}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self::from_size(1, 1)
    }
}
