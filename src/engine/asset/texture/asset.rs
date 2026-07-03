use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::texture::metadata::TextureMetadata;
use crate::engine::asset::texture::usage::TextureUsage;
use bevy::prelude::*;

/// 纹理资源对象
#[derive(Debug, Clone)]
pub struct TextureAsset {
    /// 纹理对应的 GPU 图像句柄
    pub handle: Handle<Image>,
    /// 纹理元数据，包含尺寸、用途等属性信息
    pub metadata: TextureMetadata,
    /// 资源唯一标识 ID
    pub asset_id: AssetId,
}

impl TextureAsset {
    /// 创建纹理资源实例
    pub fn new(handle: Handle<Image>, metadata: TextureMetadata, asset_id: AssetId) -> Self {
        Self {
            handle,
            metadata,
            asset_id,
        }
    }

    /// 获取纹理的宽度（像素）
    pub fn width(&self) -> u32 {
        self.metadata.width
    }

    /// 获取纹理的高度（像素）
    pub fn height(&self) -> u32 {
        self.metadata.height
    }

    /// 获取纹理的宽高比
    pub fn aspect_ratio(&self) -> f32 {
        self.metadata.aspect_ratio
    }

    /// 获取纹理的用途标识
    pub fn usage(&self) -> TextureUsage {
        self.metadata.usage
    }

    /// 获取纹理元数据的不可变引用
    pub fn metadata_ref(&self) -> &TextureMetadata {
        &self.metadata
    }

    /// 获取图像句柄的不可变引用
    pub fn handle_ref(&self) -> &Handle<Image> {
        &self.handle
    }
}

impl Default for TextureAsset {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            metadata: TextureMetadata::default(),
            asset_id: crate::engine::asset::identifier::asset_id(""),
        }
    }
}
