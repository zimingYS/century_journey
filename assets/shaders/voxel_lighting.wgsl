    #import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct VoxelMaterialUniform {
    block_indirect_strength: f32,
    dark_surface_strength: f32,
    padding_y: f32,
    padding_z: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> voxel: VoxelMaterialUniform;

const PERCEPTUAL_LIGHT_LEVELS = array<f32, 16>(
    0.0, 0.172005, 0.269905, 0.351293,
    0.423526, 0.489634, 0.551238, 0.609333,
    0.664583, 0.717461, 0.768317, 0.817421,
    0.864985, 0.911179, 0.956145, 1.0,
);

fn unpack_block_light(packed_value: f32) -> vec3<f32> {
    let packed = u32(clamp(round(packed_value), 0.0, 4095.0));
    let levels = vec3<f32>(
        f32((packed >> 8u) & 15u),
        f32((packed >> 4u) & 15u),
        f32(packed & 15u),
    );
    let peak = u32(max(levels.r, max(levels.g, levels.b)));
    if peak == 0u {
        return vec3<f32>(0.0);
    }
    let gain = PERCEPTUAL_LIGHT_LEVELS[peak] / (f32(peak) / 15.0);
    return levels / 15.0 * gain;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

#ifdef VERTEX_UVS_B
    var surface_albedo = pbr_input.material.base_color.rgb;
    var combined_light = vec3<f32>(1.0);
#ifdef VERTEX_COLORS
    // StandardMaterial 已把合成光色乘入底色；除回该系数才能恢复不含光的贴图颜色。
    combined_light = in.color.rgb;
    surface_albedo /= max(in.color.rgb, vec3<f32>(0.0001));
#endif
    let block_light = unpack_block_light(in.uv_b.x);
    let combined_peak = max(combined_light.r, max(combined_light.g, combined_light.b));
    // 中性暗部补光不进入权威光场，也不受相机曝光变化影响。它只在光级接近零时
    // 保留贴图轮廓，方块光增强后平滑退出，避免抬灰已照亮区域。
    let dark_surface = voxel.dark_surface_strength * (1.0 - smoothstep(0.08, 0.35, combined_peak));
    pbr_input.material.emissive = vec4<f32>(
        pbr_input.material.emissive.rgb
            + surface_albedo
                * (block_light * voxel.block_indirect_strength + vec3<f32>(dark_surface)),
        0.0,
    );
#endif

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color,
    );
#ifdef PREPASS_PIPELINE
    return deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
#endif
}
