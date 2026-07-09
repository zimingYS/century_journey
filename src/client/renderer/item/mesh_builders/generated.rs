use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::TextureFormat;

const ITEM_WORLD_WIDTH: f32 = 0.78;
const ALPHA_CUTOFF: u8 = 30;

pub struct GeneratedItemMeshBuilder;

impl GeneratedItemMeshBuilder {
    pub fn build_mesh(image: &Image, thickness: f32) -> Mesh {
        let size = image.size();
        let width = size.x as i32;
        let height = size.y as i32;

        let Some(pixels) = image.data.as_deref() else {
            return empty_mesh();
        };

        if pixels.is_empty() || width <= 0 || height <= 0 {
            return empty_mesh();
        }

        let tex_w = width as f32;
        let tex_h = height as f32;
        let world_w = ITEM_WORLD_WIDTH;
        let world_h = world_w * tex_h / tex_w;
        let pixel_w = world_w / tex_w;
        let pixel_h = world_h / tex_h;
        let front_z = thickness * 0.5;
        let back_z = -front_z;

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let Some(color) = pixel_color(image, pixels, width, height, x, y) else {
                    continue;
                };

                let pixel = PixelCell::new(x, y, tex_w, tex_h, world_w, world_h, pixel_w, pixel_h);
                let uv = pixel.center_uvs();

                push_quad(
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                    [0.0, 0.0, 1.0],
                    [
                        [pixel.x0, pixel.y0, front_z],
                        [pixel.x1, pixel.y0, front_z],
                        [pixel.x1, pixel.y1, front_z],
                        [pixel.x0, pixel.y1, front_z],
                    ],
                    uv,
                    color,
                );
                push_quad(
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                    [0.0, 0.0, -1.0],
                    [
                        [pixel.x1, pixel.y0, back_z],
                        [pixel.x0, pixel.y0, back_z],
                        [pixel.x0, pixel.y1, back_z],
                        [pixel.x1, pixel.y1, back_z],
                    ],
                    uv,
                    shade_color(color, 0.82),
                );

                let side_color = shade_color(color, 0.68);
                if pixel_color(image, pixels, width, height, x + 1, y).is_none() {
                    push_quad(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [1.0, 0.0, 0.0],
                        [
                            [pixel.x1, pixel.y1, back_z],
                            [pixel.x1, pixel.y1, front_z],
                            [pixel.x1, pixel.y0, front_z],
                            [pixel.x1, pixel.y0, back_z],
                        ],
                        uv,
                        side_color,
                    );
                }
                if pixel_color(image, pixels, width, height, x - 1, y).is_none() {
                    push_quad(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [-1.0, 0.0, 0.0],
                        [
                            [pixel.x0, pixel.y0, back_z],
                            [pixel.x0, pixel.y0, front_z],
                            [pixel.x0, pixel.y1, front_z],
                            [pixel.x0, pixel.y1, back_z],
                        ],
                        uv,
                        side_color,
                    );
                }
                if pixel_color(image, pixels, width, height, x, y - 1).is_none() {
                    push_quad(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [0.0, 1.0, 0.0],
                        [
                            [pixel.x0, pixel.y0, back_z],
                            [pixel.x1, pixel.y0, back_z],
                            [pixel.x1, pixel.y0, front_z],
                            [pixel.x0, pixel.y0, front_z],
                        ],
                        uv,
                        side_color,
                    );
                }
                if pixel_color(image, pixels, width, height, x, y + 1).is_none() {
                    push_quad(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [0.0, -1.0, 0.0],
                        [
                            [pixel.x1, pixel.y1, back_z],
                            [pixel.x0, pixel.y1, back_z],
                            [pixel.x0, pixel.y1, front_z],
                            [pixel.x1, pixel.y1, front_z],
                        ],
                        uv,
                        side_color,
                    );
                }
            }
        }

        let mut mesh = empty_mesh();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}

struct PixelCell {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    u_center: f32,
    v_center: f32,
}

impl PixelCell {
    fn new(
        x: i32,
        y: i32,
        tex_w: f32,
        tex_h: f32,
        world_w: f32,
        world_h: f32,
        pixel_w: f32,
        pixel_h: f32,
    ) -> Self {
        let x0 = -world_w * 0.5 + x as f32 * pixel_w;
        let x1 = x0 + pixel_w;
        let y0 = world_h * 0.5 - y as f32 * pixel_h;
        let y1 = y0 - pixel_h;
        let u_center = (x as f32 + 0.5) / tex_w;
        let v_center = 1.0 - (y as f32 + 0.5) / tex_h;

        Self {
            x0,
            x1,
            y0,
            y1,
            u_center,
            v_center,
        }
    }

    fn center_uvs(&self) -> [[f32; 2]; 4] {
        [[self.u_center, self.v_center]; 4]
    }
}

fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

fn pixel_color(
    image: &Image,
    pixels: &[u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
) -> Option<[f32; 4]> {
    if x < 0 || x >= width || y < 0 || y >= height {
        return None;
    }

    let pixel_count = width as usize * height as usize;
    if pixel_count == 0 {
        return None;
    }

    let bytes_per_pixel = pixels.len() / pixel_count;
    if bytes_per_pixel < 4 {
        return None;
    }

    let base = ((y * width + x) as usize) * bytes_per_pixel;
    if base + 3 >= pixels.len() {
        return None;
    }

    let (r, g, b, a) = match image.texture_descriptor.format {
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => (
            pixels[base + 2],
            pixels[base + 1],
            pixels[base],
            pixels[base + 3],
        ),
        _ => (
            pixels[base],
            pixels[base + 1],
            pixels[base + 2],
            pixels[base + 3],
        ),
    };

    if a <= ALPHA_CUTOFF {
        return None;
    }

    Some([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ])
}

fn shade_color(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        color[0] * factor,
        color[1] * factor,
        color[2] * factor,
        color[3],
    ]
}

fn push_quad(
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    normal: [f32; 3],
    points: [[f32; 3]; 4],
    quad_uvs: [[f32; 2]; 4],
    color: [f32; 4],
) {
    let base = vertices.len() as u32;
    vertices.extend_from_slice(&points);
    normals.extend_from_slice(&[normal; 4]);
    uvs.extend_from_slice(&quad_uvs);
    colors.extend_from_slice(&[color; 4]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}
