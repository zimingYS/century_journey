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
#import bevy_pbr::prepass_utils
#import bevy_pbr::view_transformations::depth_ndc_to_view_z

struct WaterMaterialExtension {
    time_seconds: f32,
    wave_scale: f32,
    foam_strength: f32,
    depth_fade: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> water: WaterMaterialExtension;

fn smooth_band(value: f32, low: f32, high: f32) -> f32 {
    return smoothstep(low, high, clamp(value, low, high));
}

fn wave_height(world_position: vec3<f32>, time_seconds: f32) -> f32 {
    let p = world_position.xz * water.wave_scale;
    let first = sin(dot(p, vec2<f32>(1.0, 0.32)) + time_seconds * 1.45);
    let second = sin(dot(p, vec2<f32>(-0.42, 1.0)) - time_seconds * 1.05 + 1.7);
    let third = sin(dot(p, vec2<f32>(0.72, 0.66)) + time_seconds * 0.72 + 3.1);
    return first * 0.52 + second * 0.31 + third * 0.17;
}

fn water_wave_normal(world_position: vec3<f32>, time_seconds: f32) -> vec3<f32> {
    let delta = 0.08;
    let center = wave_height(world_position, time_seconds);
    let x_offset = wave_height(world_position + vec3<f32>(delta, 0.0, 0.0), time_seconds);
    let z_offset = wave_height(world_position + vec3<f32>(0.0, 0.0, delta), time_seconds);
    return normalize(vec3<f32>(-(x_offset - center), 0.22, -(z_offset - center)));
}

fn surface_depth(in: VertexOutput) -> f32 {
    let surface_view_z = abs(depth_ndc_to_view_z(in.position.z));
    var scene_view_z = surface_view_z + water.depth_fade;
#ifdef DEPTH_PREPASS
    let scene_depth = prepass_utils::prepass_depth(in.position, 0u);
    // 反向深度的清屏值为 0；清屏时保留最大水深，避免天空把泡沫误判成岸线。
    if (scene_depth > 0.0001) {
        scene_view_z = abs(depth_ndc_to_view_z(scene_depth));
    }
#endif
    return max(scene_view_z - surface_view_z, 0.0);
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let world_position = in.world_position.xyz;
    let top_surface = smooth_band(abs(in.world_normal.y), 0.45, 0.92);
    let depth = surface_depth(in);
    let depth_factor = smooth_band(depth, 0.0, water.depth_fade);

    // 两层正弦波叠加到顶面法线，保持侧面法线以免水墙出现错误高光。
    let animated_normal = water_wave_normal(world_position, water.time_seconds);
    pbr_input.N = normalize(mix(pbr_input.N, animated_normal, top_surface * 0.88));

    let shallow_color = vec3<f32>(0.045, 0.62, 0.78);
    let deep_color = vec3<f32>(0.018, 0.22, 0.46);
    let depth_color = mix(shallow_color, deep_color, depth_factor);
    pbr_input.material.base_color.rgb = mix(
        pbr_input.material.base_color.rgb,
        depth_color,
        0.76,
    );
    pbr_input.material.base_color.a *= mix(0.52, 0.94, depth_factor);

    // 真实厚度趋近零的区域生成不规则泡沫带，避免整圈静态白边。
    let crest = 0.5 + 0.5 * wave_height(world_position * 1.7, water.time_seconds * 0.83);
    let shoreline = 1.0 - smooth_band(depth, 0.06, 0.72);
    let foam = shoreline * smooth_band(crest, 0.48, 0.78) * top_surface * water.foam_strength;
    pbr_input.material.base_color.rgb = mix(
        pbr_input.material.base_color.rgb,
        vec3<f32>(0.72, 0.96, 1.0),
        clamp(foam, 0.0, 0.78),
    );
    let ripple = 0.5 + 0.5 * sin(dot(world_position.xz, vec2<f32>(1.7, -1.15)) + water.time_seconds * 1.25);
    let surface_glow = top_surface * (0.025 + ripple * 0.055);
    pbr_input.material.emissive = vec4<f32>(
        depth_color * surface_glow + vec3<f32>(0.18, 0.55, 0.64) * foam * 0.32,
        1.0,
    );

    // Fresnel 反射让掠射角高光更集中，同时保留水体本身的透色。
    let fresnel = pow(1.0 - clamp(dot(pbr_input.N, pbr_input.V), 0.0, 1.0), 5.0);
    pbr_input.material.reflectance *= 1.0 + fresnel * 1.8;
    pbr_input.material.perceptual_roughness = clamp(
        pbr_input.material.perceptual_roughness - fresnel * 0.09,
        0.089,
        1.0,
    );

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef PREPASS_PIPELINE
    return deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // 额外的冷色太阳闪光填补没有环境立方体时的反射可读性。
    let highlight_direction = normalize(vec3<f32>(-0.38, 0.84, -0.26));
    let highlight = pow(max(dot(reflect(-pbr_input.V, pbr_input.N), highlight_direction), 0.0), 18.0);
    out.color.rgb += vec3<f32>(0.40, 0.86, 1.0) * highlight * (0.16 + fresnel * 0.90) * top_surface;
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
#endif
}
