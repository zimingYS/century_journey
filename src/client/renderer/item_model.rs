use std::collections::HashMap;

use bevy::camera::{RenderTarget, ScalingMode, visibility::RenderLayers};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::client::renderer::held_renderer::block_renderer::HeldBlockRenderer;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::definition::RenderMode;
use crate::content::block::registry::BlockRegistry;
use crate::shared::identifier::Identifier;

const ITEM_MODEL_PREVIEW_LAYER: usize = 6;
const ITEM_MODEL_PREVIEW_SIZE: u32 = 96;
const ITEM_MODEL_PREVIEW_SPACING: f32 = 6.0;
const ITEM_MODEL_CAMERA_DISTANCE: f32 = 4.0;
const ITEM_MODEL_ORTHO_SIZE: f32 = 2.15;

#[derive(Resource, Default)]
pub struct ItemModelRenderAssets {
    block_previews: HashMap<String, ItemModelPreview>,
    prepared: bool,
}

#[derive(Clone)]
struct ItemModelPreview {
    image: Handle<Image>,
    _root: Entity,
}

/// Central entry point for inventory-style item previews.
///
/// Today this exposes block preview textures. Later, non-block item models can
/// be added here without changing inventory UI code.
pub struct ItemModelRenderer;

impl ItemModelRenderer {
    pub fn block_preview_image(
        block_identifier: &Identifier,
        previews: &ItemModelRenderAssets,
    ) -> Option<Handle<Image>> {
        previews.block_preview_image(block_identifier)
    }
}

pub fn prepare_item_model_render_assets_system(
    mut commands: Commands,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    mut previews: ResMut<ItemModelRenderAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if previews.prepared {
        return;
    }

    let Some(block_registry) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };

    let identifiers: Vec<_> = block_registry
        .identifiers()
        .filter(|identifier| identifier.to_string() != "century_journey:air")
        .cloned()
        .collect();

    for identifier in identifiers {
        previews.build_block_preview(
            &identifier,
            &mut commands,
            &mut images,
            &mut meshes,
            &mut materials,
            block_registry,
            render_assets,
        );
    }

    previews.prepared = true;
}

fn build_preview_material(
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    block_identifier: &str,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Handle<StandardMaterial>> {
    let runtime_id = block_registry.get_id_by_identifier(block_identifier)?;
    let render_mode = block_registry.get(runtime_id)?.render_mode;
    let alpha_mode = match render_mode {
        RenderMode::Opaque => AlphaMode::Opaque,
        RenderMode::Transparent => AlphaMode::Blend,
        RenderMode::Cutout | RenderMode::CustomMesh => AlphaMode::Mask(0.5),
    };

    Some(materials.add(StandardMaterial {
        base_color_texture: Some(render_assets.base_texture().clone()),
        base_color: Color::WHITE,
        alpha_mode,
        cull_mode: None,
        unlit: true,
        perceptual_roughness: 1.0,
        ..default()
    }))
}
impl ItemModelRenderAssets {
    fn block_preview_image(&self, block_identifier: &Identifier) -> Option<Handle<Image>> {
        self.block_previews
            .get(&block_identifier.to_string())
            .map(|preview| preview.image.clone())
    }

    fn build_block_preview(
        &mut self,
        block_identifier: &Identifier,
        commands: &mut Commands,
        images: &mut Assets<Image>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        block_registry: &BlockRegistry,
        render_assets: &BlockRenderAssets,
    ) -> Option<Handle<Image>> {
        let cache_key = block_identifier.to_string();
        if let Some(preview) = self.block_previews.get(&cache_key) {
            return Some(preview.image.clone());
        }

        let mesh = HeldBlockRenderer::build_mesh(block_registry, &cache_key)?;
        let material =
            build_preview_material(block_registry, render_assets, &cache_key, materials)?;

        let image = Image::new_target_texture(
            ITEM_MODEL_PREVIEW_SIZE,
            ITEM_MODEL_PREVIEW_SIZE,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        );
        let image_handle = images.add(image);
        let mesh_handle = meshes.add(mesh);

        let preview_index = self.block_previews.len() as f32;
        let target = Vec3::new(preview_index * ITEM_MODEL_PREVIEW_SPACING, -500.0, 0.0);
        let camera_pos = target
            + Vec3::new(
                ITEM_MODEL_CAMERA_DISTANCE,
                ITEM_MODEL_CAMERA_DISTANCE * 0.82,
                ITEM_MODEL_CAMERA_DISTANCE,
            );
        let preview_layer = RenderLayers::layer(ITEM_MODEL_PREVIEW_LAYER);

        let root = commands
            .spawn((
                Name::new(format!("ItemModelPreview_{cache_key}")),
                Transform::from_translation(target),
                Visibility::Inherited,
                preview_layer.clone(),
            ))
            .with_children(|root| {
                root.spawn((
                    Name::new("PreviewBlockCube"),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.08, -0.18, 0.0)),
                    Visibility::Inherited,
                    NotShadowCaster,
                    preview_layer.clone(),
                ));
            })
            .id();

        commands.spawn((
            Name::new(format!("ItemModelPreviewCamera_{cache_key}")),
            Camera3d::default(),
            Camera {
                order: -10,
                clear_color: Color::NONE.into(),
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: ITEM_MODEL_ORTHO_SIZE,
                    height: ITEM_MODEL_ORTHO_SIZE,
                },
                near: 0.0,
                far: 32.0,
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(camera_pos).looking_at(target, Vec3::Y),
            preview_layer.clone(),
        ));

        commands.spawn((
            Name::new(format!("ItemModelPreviewLight_{cache_key}")),
            PointLight {
                intensity: 18_000.0,
                range: 16.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(camera_pos + Vec3::new(0.0, 1.0, 0.4)),
            preview_layer,
        ));

        self.block_previews.insert(
            cache_key,
            ItemModelPreview {
                image: image_handle.clone(),
                _root: root,
            },
        );

        Some(image_handle)
    }
}
