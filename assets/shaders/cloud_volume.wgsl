// 体积云着色器：DDA 体素云（参考 ARTShade 的块状云层设计）。
//
// 关键约束：
//   A. 内容定义的 density 参与阈值，避免低云量时整片天空被半透明材质覆盖。
//   B. 每个 DDA cell 使用独立密度累积，而不是只累积相邻密度差，保留云团内部厚度。
//   C. 命中点沿射线单调推进，顶亮底暗不会出现放射状条纹。
//
// 渲染距离：cell_size * RENDER_DIST_FACTOR world units，超出后 dist_fade 渐变到 0。
// 太阳光晕：pow(lightDot, 8) * 0.75 简单叠加（与原版一致）。
// Alpha 输出：premultiplied → straight 除以 accum_alpha（Bevy Blend 模式）。

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
}

struct CloudVolumeExtension {
    time_seconds: f32,
    coverage: f32,
    night_factor: f32,
    twilight_glow: f32,
    cloud_min_y: f32,
    cloud_max_y: f32,
    wind_speed: f32,
    noise_scale: f32,
    cell_size: f32,
    density_threshold: f32,
    detail_strength: f32,
    visibility: f32,
    camera_position: vec4<f32>,
    sun_direction: vec4<f32>,
    wind_direction: vec4<f32>,
    tint_day: vec4<f32>,
    tint_night: vec4<f32>,
    tint_sunset: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> cloud: CloudVolumeExtension;

/// DDA 遍历的最大步数（512：配合距离提前终止足够覆盖渲染距离内的 cell）。
const MAX_STEPS: i32 = 512;

/// 云的渲染距离 = cell_size * RENDER_DIST_FACTOR；超出此距离的 cell 渐淡入天空。
const RENDER_DIST_FACTOR: f32 = 96.0;

/// 自阴影采样偏移距离（cell 单位）：沿太阳方向偏移 N 个 cell 估算穿透率。
const SHADOW_CELL_OFFSET: f32 = 1.0;

/// 自阴影消光系数（Beer-Lambert 简化的"密度→透射"斜率）。
const SHADOW_EXTINCTION: f32 = 1.5;

/// 自阴影最暗值：完全被遮蔽时，cell 仍保留该比例的亮度，避免"死黑"。
const SHADOW_MIN_TRANSMISSION: f32 = 0.45;

fn hash21(p: vec2<f32>) -> f32 {
    let n = sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453;
    return fract(n);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var sample = p;
    for (var octave = 0; octave < 4; octave = octave + 1) {
        value = value + value_noise(sample) * amplitude;
        sample = sample * 2.03 + vec2<f32>(11.7, 7.9);
        amplitude = amplitude * 0.5;
    }
    return value / 0.9375;
}

/// 云密度映射：低频团块决定覆盖，高频噪声侵蚀边缘，垂直剖面提供云的厚度。
///
/// XZ 采样由 DDA cell 中心提供，边缘保留明确的块状断面；Y 采样使用射线命中点，
/// 让同一云团拥有较亮的顶部、偏冷的底部和自然的上下收边。
fn cloud_map(world_pos: vec3<f32>) -> f32 {
    let wind = cloud.wind_direction.xy * cloud.time_seconds * cloud.wind_speed;
    let moved_xz = world_pos.xz + wind;

    let p = moved_xz * cloud.noise_scale;
    let macro = fbm(p);
    // 内容 density 是 0~1 的稀疏度，不直接作为噪声值阈值。天气 coverage
    // 提高时降低 cutoff，形成从晴天疏云到阴天连片云的连续变化。
    let threshold = clamp(
        0.61 - cloud.coverage * 0.18 + (cloud.density_threshold - 0.5) * 0.22,
        0.38,
        0.72,
    );
    let macro_shape = smoothstep(threshold - 0.04, threshold + 0.12, macro);
    let detail = fbm(p * 2.7 + vec2<f32>(19.4, -7.2));
    let edge_mask = mix(1.0, smoothstep(0.22, 0.78, detail), cloud.detail_strength);
    let shape = macro_shape * edge_mask;

    if (shape <= 0.001) {
        return 0.0;
    }

    let y_range = max(cloud.cloud_max_y - cloud.cloud_min_y, 0.001);
    let y_norm = clamp((world_pos.y - cloud.cloud_min_y) / y_range, 0.0, 1.0);
    let base_noise = value_noise(p * 0.6 + vec2<f32>(5.1, 13.7));
    let base_level = mix(0.04, 0.18, base_noise);
    let top_level = mix(0.70, 0.94, base_noise);
    let vertical_falloff = smoothstep(base_level - 0.08, base_level + 0.12, y_norm)
        * (1.0 - smoothstep(top_level - 0.16, top_level + 0.04, y_norm));
    let billow = mix(0.72, 1.0, value_noise(p * 3.8 + vec2<f32>(-4.3, 8.6)));

    return shape * vertical_falloff * billow;
}

/// 自阴影：沿太阳方向偏移 1 个 cell 采样密度，做 Beer-Lambert 简化。
///
/// 返回 light transmission（0=完全遮蔽，1=完全照亮），clamp 到 [0.45, 1.0]
/// 避免出现"死黑"的背阳侧。
fn light_transmission(hit_pos: vec3<f32>, sample_density: f32) -> f32 {
    let sun_xz = vec2<f32>(cloud.sun_direction.x, cloud.sun_direction.z);
    let sun_xz_len = length(sun_xz);
    if (sun_xz_len < 0.001) {
        return 1.0;
    }
    let sun_xz_norm = sun_xz / sun_xz_len;
    let offset = vec3<f32>(sun_xz_norm.x, 0.0, sun_xz_norm.y)
        * cloud.cell_size
        * SHADOW_CELL_OFFSET;
    let shadow_density = cloud_map(hit_pos + offset);
    let raw = exp(-(sample_density + shadow_density) * SHADOW_EXTINCTION);
    return max(raw, SHADOW_MIN_TRANSMISSION);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let ray_origin = cloud.camera_position.xyz;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);

    // 几乎水平的射线（朝下或水平）看不到云。
    if (ray_dir.y <= 0.0005) {
        discard;
    }

    // 视线与云层 Y 范围的交点。
    let t0 = (cloud.cloud_min_y - ray_origin.y) / ray_dir.y;
    let t1 = (cloud.cloud_max_y - ray_origin.y) / ray_dir.y;
    let t_near = min(t0, t1);
    let t_far = max(t0, t1);
    if (t_far < 0.0) {
        discard;
    }
    let t_start = max(t_near, 0.0);
    if (t_start >= t_far) {
        discard;
    }

    // 云颜色：顶亮底暗由垂直插值决定。
    var cloud_tint = mix(cloud.tint_day.rgb, cloud.tint_night.rgb, cloud.night_factor);
    cloud_tint = mix(cloud_tint, cloud.tint_sunset.rgb, cloud.twilight_glow * 0.5);
    let top_color = cloud_tint;
    let bottom_color = cloud_tint * (1.0 - 0.4);

    let cell_size = cloud.cell_size;
    let wind = cloud.wind_direction.xy * cloud.time_seconds * cloud.wind_speed;

    // DDA 入口：射线进入云层的水平位置与 cell 索引。
    let entry_pos = ray_origin + ray_dir * t_start;
    let start_xz = entry_pos.xz + wind;
    var cell_idx = floor(start_xz / cell_size);
    let dir_xz = ray_dir.xz;

    // DDA 步进方向与参数。
    var step_dir = vec2<f32>(0.0, 0.0);
    if (dir_xz.x > 0.0) {
        step_dir.x = 1.0;
    } else if (dir_xz.x < 0.0) {
        step_dir.x = -1.0;
    }
    if (dir_xz.y > 0.0) {
        step_dir.y = 1.0;
    } else if (dir_xz.y < 0.0) {
        step_dir.y = -1.0;
    }

    var inv_dir = vec2<f32>(1e20, 1e20);
    var t_max = vec2<f32>(1e20, 1e20);
    var t_delta = vec2<f32>(1e20, 1e20);
    if (abs(dir_xz.x) > 0.0001) {
        inv_dir.x = 1.0 / dir_xz.x;
        t_delta.x = abs(cell_size * inv_dir.x);
        let offset = select(0.0, 1.0, step_dir.x > 0.0);
        t_max.x = ((cell_idx.x + offset) * cell_size - start_xz.x) * inv_dir.x;
    }
    if (abs(dir_xz.y) > 0.0001) {
        inv_dir.y = 1.0 / dir_xz.y;
        t_delta.y = abs(cell_size * inv_dir.y);
        let offset = select(0.0, 1.0, step_dir.y > 0.0);
        t_max.y = ((cell_idx.y + offset) * cell_size - start_xz.y) * inv_dir.y;
    }

    let t_limit = t_far;
    let render_dist = cell_size * RENDER_DIST_FACTOR;

    var accum_color = vec3<f32>(0.0, 0.0, 0.0);
    var accum_alpha = 0.0;
    var transmission = 1.0;

    for (var i = 0; i < MAX_STEPS; i = i + 1) {
        if (accum_alpha >= 0.99) {
            break;
        }

        // 提前终止：当前 cell 的水平距离已超出渲染距离，后续全部被距离 fade 归零，
        // 继续步进纯属浪费；直接结束本次射线。
        let cell_world_xz = (vec2<f32>(cell_idx) + 0.5) * cell_size - wind;
        if (length(cell_world_xz - ray_origin.xz) > render_dist) {
            break;
        }

        // ★ 用 cell 入口 t 计算命中点，保证斜射线的高度和颜色沿射线单调变化。
        let cell_min_xz = cell_idx * cell_size;
        let cell_max_xz = cell_min_xz + cell_size;
        var t_entry_x = -1.0e20;
        var t_entry_z = -1.0e20;
        if (abs(dir_xz.x) > 0.0001) {
            let entry_x = select(cell_max_xz.x, cell_min_xz.x, step_dir.x > 0.0);
            t_entry_x = (entry_x - start_xz.x) * inv_dir.x;
        }
        if (abs(dir_xz.y) > 0.0001) {
            let entry_z = select(cell_max_xz.y, cell_min_xz.y, step_dir.y > 0.0);
            t_entry_z = (entry_z - start_xz.y) * inv_dir.y;
        }
        let hit_t = t_start + max(max(t_entry_x, t_entry_z), 0.0);

        if (hit_t <= t_limit) {
            let hit_pos = ray_origin + ray_dir * hit_t;
            let clamped_y = clamp(hit_pos.y, cloud.cloud_min_y, cloud.cloud_max_y);
            let y_range = max(cloud.cloud_max_y - cloud.cloud_min_y, 0.001);
            let local_y = (clamped_y - cloud.cloud_min_y) / y_range;
            // 垂直视线只有一个 XZ cell，取云体内部样本才能避免从底沿
            // 的零密度直接漏掉整朵云；斜射线仍由命中点决定明暗高度。
            let sample_y = clamp(clamped_y + y_range * 0.35, cloud.cloud_min_y, cloud.cloud_max_y);
            let world_cell = vec3<f32>(cell_world_xz.x, sample_y, cell_world_xz.y);
            let density = cloud_map(world_cell);

            if (density > 0.001) {
                let hit_dist = length(hit_pos.xz - ray_origin.xz);
                let dist_fade = 1.0 - smoothstep(render_dist * 0.55, render_dist, hit_dist);

                if (i % 2 == 0) {
                    transmission = light_transmission(hit_pos, density);
                }
                // 与 ARTShade 的入口 cell 相同，单个竖直体素也按完整密度贡献 alpha；
                // 自阴影只影响亮度，不能把背光云错误地变成透明云。
                let base_col = mix(bottom_color, top_color, smoothstep(0.0, 1.0, local_y));
                let col = base_col * mix(SHADOW_MIN_TRANSMISSION, 1.0, transmission);
                let step_alpha = clamp(density * 0.92, 0.0, 0.96) * dist_fade;
                accum_color = accum_color + col * step_alpha * (1.0 - accum_alpha);
                accum_alpha = accum_alpha + step_alpha * (1.0 - accum_alpha);
            }
        }

        // DDA 步进：跨越最近的 cell 边界。
        if (t_max.x < t_max.y) {
            cell_idx.x = cell_idx.x + step_dir.x;
            if (t_start + t_max.x > t_limit) {
                break;
            }
            t_max.x = t_max.x + t_delta.x;
        } else {
            cell_idx.y = cell_idx.y + step_dir.y;
            if (t_start + t_max.y > t_limit) {
                break;
            }
            t_max.y = t_max.y + t_delta.y;
        }
    }

    // 太阳光晕：太阳方向散射叠加（参考 ARTShade sun glow）。
    let sun_dir = normalize(cloud.sun_direction.xyz);
    let light_dot = max(dot(ray_dir, sun_dir), 0.0);
    let scatter = pow(light_dot, 8.0) * 0.75;
    accum_color = accum_color + cloud_tint * scatter * accum_alpha;

    // 正常输出：accum_color 是预乘 alpha 的（累积时乘了 (1-accum_alpha)），
    // 但 Bevy AlphaMode::Blend 用 straight alpha（非预乘），必须除以 accum_alpha 转回。
    let straight_rgb = accum_color / max(accum_alpha, 0.001);
    // coverage=0 必须完全隐藏；0.12 以上只改变形状覆盖，不重复削弱不透明度。
    let coverage_alpha = smoothstep(0.0, 0.12, cloud.coverage);
    let final_alpha = accum_alpha * cloud.tint_day.a * cloud.visibility * coverage_alpha;
    pbr_input.material.base_color = vec4<f32>(straight_rgb, final_alpha);
    pbr_input.material.emissive = vec4<f32>(vec3<f32>(0.0), 1.0);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
