use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};

const FRAME_TEXTURE_SIZE: u32 = 24;
const FRAME_SLICE: f32 = 8.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFrameKind {
    Creative,
    Survival,
    Generic,
    Modal,
}

#[derive(Resource, Debug, Clone)]
pub struct UiFrameAssets {
    creative: Handle<Image>,
    survival: Handle<Image>,
    generic: Handle<Image>,
    modal: Handle<Image>,
}

impl UiFrameAssets {
    fn get(&self, kind: UiFrameKind) -> Handle<Image> {
        match kind {
            UiFrameKind::Creative => self.creative.clone(),
            UiFrameKind::Survival => self.survival.clone(),
            UiFrameKind::Generic => self.generic.clone(),
            UiFrameKind::Modal => self.modal.clone(),
        }
    }
}

pub fn create_ui_frame_assets_system(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let creative = images.add(frame_image(
        [30, 32, 35, 255],
        [112, 87, 54, 255],
        [54, 46, 36, 255],
    ));
    let survival = images.add(frame_image(
        [29, 31, 36, 255],
        [86, 91, 102, 255],
        [43, 46, 54, 255],
    ));
    let generic = images.add(frame_image(
        [27, 29, 34, 255],
        [67, 73, 86, 255],
        [39, 43, 51, 255],
    ));
    let modal = images.add(frame_image(
        [23, 25, 29, 255],
        [105, 112, 127, 255],
        [45, 49, 57, 255],
    ));
    commands.insert_resource(UiFrameAssets {
        creative,
        survival,
        generic,
        modal,
    });
}

pub fn apply_ui_frame_system(
    assets: Res<UiFrameAssets>,
    mut commands: Commands,
    query: Query<(Entity, &UiFrameKind), Added<UiFrameKind>>,
) {
    for (entity, kind) in &query {
        commands.entity(entity).insert(ImageNode {
            image: assets.get(*kind),
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(FRAME_SLICE),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
            ..default()
        });
    }
}

fn frame_image(outer: [u8; 4], border: [u8; 4], center: [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((FRAME_TEXTURE_SIZE * FRAME_TEXTURE_SIZE * 4) as usize);
    for y in 0..FRAME_TEXTURE_SIZE {
        for x in 0..FRAME_TEXTURE_SIZE {
            let edge = x
                .min(y)
                .min(FRAME_TEXTURE_SIZE - 1 - x)
                .min(FRAME_TEXTURE_SIZE - 1 - y);
            let color = if edge < 2 {
                outer
            } else if edge < FRAME_SLICE as u32 {
                border
            } else {
                center
            };
            data.extend_from_slice(&color);
        }
    }
    Image::new(
        Extent3d {
            width: FRAME_TEXTURE_SIZE,
            height: FRAME_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_frame_is_large_enough_for_nine_slice() {
        assert!(FRAME_TEXTURE_SIZE as f32 > FRAME_SLICE * 2.0);
        let image = frame_image([1; 4], [2; 4], [3; 4]);
        assert_eq!(image.texture_descriptor.size.width, FRAME_TEXTURE_SIZE);
        assert_eq!(image.texture_descriptor.size.height, FRAME_TEXTURE_SIZE);
    }
}
