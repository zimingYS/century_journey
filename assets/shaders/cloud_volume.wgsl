// =============================================================================
// CenturyJourney - 体积云着色器 (Fixed Ray March)
// =============================================================================
// 目标样式: ARTShade / Minecraft 风格的方块云
//
// 核心改进说明 (解决云边交界/切边问题)：
// -----------------------------------------------------------------------------
// 1. 抛弃了 DDA 体素步进。因为 DDA 会在 Cell 边界时导致密度突变，
//    即使加了 smoothstep 也容易留下接缝。
// 2. 改用固定步长 (Fixed Ray March) 在云层内部进行积分。
// 3. 使用带平滑阈值的 "宏观占用(Occupancy)"，并配合连续世界坐标采样，
//    让云团边缘有自然的半透明过渡。
// 4. 降低了步长比例 (MARCH_STEP_FACTOR)，极大减少了体积渲染的条带伪影。
// 5. 保留顶亮底暗、自阴影和太阳高光特性。
// =============================================================================

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
};

// =============================================================================
// 云层统一参数缓冲区 (Uniform Buffer)
// =============================================================================
struct CloudVolumeExtension {
    time_seconds: f32,
    coverage: f32,
    night_factor: f32,
    twilight_glow: f32,

    cloud_min_y: f32,
    cloud_max_y: f32,

    wind_speed: f32,
    noise_scale: f32,

    cell_size: f32,         // 体素云方块大小
    density_threshold: f32,
    detail_strength: f32,
    visibility: f32,

    camera_position: vec4<f32>,
    sun_direction: vec4<f32>,
    wind_direction: vec4<f32>,

    tint_day: vec4<f32>,
    tint_night: vec4<f32>,
    tint_sunset: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> cloud: CloudVolumeExtension;

// =============================================================================
// 常量定义
// =============================================================================

/// 光线步进最大次数，防止死循环
const MAX_STEPS: i32 = 512;

/// 渲染距离因子 (实际距离 = cell_size * RENDER_DIST_FACTOR)
const RENDER_DIST_FACTOR: f32 = 96.0;

/// 光线步进比例 (cell_size 为 10 时，步长为 2.5)。
/// 调小可增强平滑度，调大可提升性能，但易出现分层条带。
const MARCH_STEP_FACTOR: f32 = 0.40;

/// 最小光线步长，防止因单元格过小而开销爆炸
const MIN_MARCH_STEP: f32 = 1.5;

/// 自阴影消光系数 (Beer-Lambert 简化)
const SHADOW_EXTINCTION: f32 = 1.2;

/// 自阴影最低透射率，防止云体背光死黑
const SHADOW_MIN_TRANSMISSION: f32 = 0.45;

/// 自阴影沿太阳方向偏移距离
const SHADOW_CELL_OFFSET: f32 = 0.75;

// =============================================================================
// 噪声工具函数
// =============================================================================

/// 2D 伪随机哈希
fn hash21(p: vec2<f32>) -> f32 {
    let n = sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453;
    return fract(n);
}

/// 2D 值噪声 (双线性插值)
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // Hermite 平滑

    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

/// 3 层分形布朗运动 (FBM)
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var sample = p;

    for (var octave: i32 = 0; octave < 3; octave = octave + 1) {
        value = value + value_noise(sample) * amplitude;
        sample = sample * 2.03 + vec2<f32>(11.7, 7.9);
        amplitude = amplitude * 0.5;
    }

    return value / 0.9375; // 归一化
}

/// 仅用于自阴影采样的密度函数 (独立于主函数，避免重复计算开销)
fn shadow_density(world_pos: vec3<f32>) -> f32 {
    let wind = cloud.wind_direction.xy * cloud.time_seconds * cloud.wind_speed;
    let moved_xz = world_pos.xz + wind;
    let voxel = max(cloud.cell_size, 0.001);

    let cell_id = floor(moved_xz / voxel);
    let cell_center = (cell_id + vec2<f32>(0.5, 0.5)) * voxel;

    let macro_p = cell_center * cloud.noise_scale;
    let macro_noise = fbm(macro_p * 0.72);

    let coverage_bias = (cloud.coverage - 0.5) * 0.24;
    let threshold = 0.55 - coverage_bias;

    // 使用窄阈值 smoothstep 保持云块边缘的方块感
    return smoothstep(threshold - 0.06, threshold + 0.04, macro_noise);
}

// =============================================================================
// 云密度计算函数 (核心)
// =============================================================================
// 结构拆解：
//  1. cell_center 决定宏观占用 (Occupancy)
//  2. world_pos 决定连续内部密度
// 结果：
//  云的外部呈方块状 (Minecraft风格)，云的内部呈连续体积感。
// =============================================================================
fn cloud_map(world_pos: vec3<f32>) -> f32 {
    // -------------------------------------------------------------------------
    // 1. 风场与坐标偏移
    // -------------------------------------------------------------------------
    let wind = cloud.wind_direction.xy * cloud.time_seconds * cloud.wind_speed;
    let moved_xz = world_pos.xz + wind;

    // -------------------------------------------------------------------------
    // 2. 体素 (Cell) 定位
    // -------------------------------------------------------------------------
    let voxel = max(cloud.cell_size, 0.001);
    let cell_id = floor(moved_xz / voxel);
    let cell_center = (cell_id + vec2<f32>(0.5, 0.5)) * voxel;

    // -------------------------------------------------------------------------
    // 3. 宏观云形 (Macro Occupancy)
    //    仅根据 Cell 中心判断该块是否属于云，避免让云变成连续的一团。
    // -------------------------------------------------------------------------
    let macro_p = cell_center * cloud.noise_scale;
    let macro_noise = fbm(macro_p * 0.72);

    let coverage_bias = (cloud.coverage - 0.5) * 0.24;
    let occupancy_threshold = 0.55 - coverage_bias;

    // 使用极窄的 smoothstep 产生略微柔和的接缝，防止硬切边
    let occupied = smoothstep(
        occupancy_threshold - 0.045,
        occupancy_threshold + 0.035,
        macro_noise
    );

    if (occupied <= 0.001) {
        return 0.0;
    }

    // -------------------------------------------------------------------------
    // 4. 连续内部密度 (世界空间坐标)
    //    使用连续的世界坐标采样高频噪声，打破单元格内的均匀感。
    // -------------------------------------------------------------------------
    let local_p = moved_xz * cloud.noise_scale;

    let detail_noise = value_noise(local_p * 1.30 + vec2<f32>(31.7, -17.3));
    let detail = mix(0.80, 1.0, smoothstep(0.25, 0.75, detail_noise));

    // -------------------------------------------------------------------------
    // 5. 边缘细节微调
    // -------------------------------------------------------------------------
    let edge_noise = value_noise(local_p * 2.0 + vec2<f32>(19.4, -7.2));
    let edge_signal = smoothstep(0.20, 0.80, edge_noise);
    let edge_strength = clamp(cloud.detail_strength * 0.18, 0.0, 0.18);
    let edge = mix(1.0, edge_signal, edge_strength);

    // -------------------------------------------------------------------------
    // 6. 垂直剖面 (顶亮底暗)
    // -------------------------------------------------------------------------
    let y_range = max(cloud.cloud_max_y - cloud.cloud_min_y, 0.001);
    let y_norm = clamp((world_pos.y - cloud.cloud_min_y) / y_range, 0.0, 1.0);

    let vertical_noise = value_noise(macro_p * 0.30 + vec2<f32>(5.1, 13.7));
    let base_level = mix(0.03, 0.10, vertical_noise);
    let top_level = mix(0.82, 0.95, vertical_noise);

    let bottom_falloff = smoothstep(base_level, base_level + 0.10, y_norm);
    let top_falloff = 1.0 - smoothstep(top_level - 0.12, top_level + 0.03, y_norm);
    let vertical = bottom_falloff * top_falloff;

    // -------------------------------------------------------------------------
    // 7. 内部起伏 (Billow)
    // -------------------------------------------------------------------------
    let billow_noise = value_noise(local_p * 1.7 + vec2<f32>(-4.3, 8.6));
    let billow = mix(0.90, 1.0, billow_noise);

    // -------------------------------------------------------------------------
    // 8. 最终密度合成
    //    宏观占用和连续细节相乘，形成方块化轮廓+平滑内部体积感。
    // -------------------------------------------------------------------------
    let density = occupied * detail * edge * vertical * billow;
    return clamp(density, 0.0, 1.0);
}

// =============================================================================
// 自阴影计算
// =============================================================================
// 使用极轻量的采样 (仅偏移一次)，避免因阴影进一步强调 Cell 接缝。
// =============================================================================
fn light_transmission(hit_pos: vec3<f32>, sample_density: f32) -> f32 {
    let sun_xz = vec2<f32>(cloud.sun_direction.x, cloud.sun_direction.z);
    let sun_xz_len = length(sun_xz);

    if (sun_xz_len < 0.001) {
        return 1.0;
    }

    let sun_xz_norm = sun_xz / sun_xz_len;
    let offset = vec3<f32>(sun_xz_norm.x, 0.0, sun_xz_norm.y) * cloud.cell_size * SHADOW_CELL_OFFSET;

    let shadow_density = shadow_density(hit_pos + offset);

    // 主样本占 70%，阴影样本占 30%，使阴影过渡柔和平滑
    let shadow_amount = sample_density * 0.70 + shadow_density * 0.30;
    let raw = exp(-shadow_amount * SHADOW_EXTINCTION);

    return max(raw, SHADOW_MIN_TRANSMISSION);
}

// =============================================================================
// 片元着色器主入口
// =============================================================================
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // -------------------------------------------------------------------------
    // 1. 射线初始化
    // -------------------------------------------------------------------------
    let ray_origin = cloud.camera_position.xyz;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);

    // 视线朝下或完全水平时，跳过云层绘制
    if (ray_dir.y <= 0.0005) {
        discard;
    }

    // -------------------------------------------------------------------------
    // 2. 射线与云层高度盒求交
    // -------------------------------------------------------------------------
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

    // -------------------------------------------------------------------------
    // 3. 云层基础颜色与渲染距离
    // -------------------------------------------------------------------------
    var cloud_tint = mix(cloud.tint_day.rgb, cloud.tint_night.rgb, cloud.night_factor);
    cloud_tint = mix(cloud_tint, cloud.tint_sunset.rgb, cloud.twilight_glow * 0.5);

    let top_color = cloud_tint;
    let bottom_color = cloud_tint * 0.60;

    let cell_size = max(cloud.cell_size, 0.001);
    let render_dist = cell_size * RENDER_DIST_FACTOR;

    // -------------------------------------------------------------------------
    // 4. 固定步长光线步进 (Fixed Ray March)
    //    消除 DDA 导致的 Cell 接缝条带伪影。
    // -------------------------------------------------------------------------
    let march_step = max(cell_size * MARCH_STEP_FACTOR, MIN_MARCH_STEP);

    var t = t_start;
    var accum_color = vec3<f32>(0.0, 0.0, 0.0);
    var accum_alpha = 0.0;
    var transmission = 1.0;

    for (var i: i32 = 0; i < MAX_STEPS; i = i + 1) {
        // 超出云层范围
        if (t >= t_far) {
            break;
        }
        // 不透明度饱和提前截止
        if (accum_alpha >= 0.985) {
            break;
        }

        let sample_pos = ray_origin + ray_dir * t;
        let horizontal_dist = length(sample_pos.xz - ray_origin.xz);

        // 超出水平渲染距离
        if (horizontal_dist >= render_dist) {
            break;
        }

        let density = cloud_map(sample_pos);

        if (density > 0.001) {
            // -------------------------------------------------------------
            // 距离淡出 (Distance Fade)
            // -------------------------------------------------------------
            let dist_fade = 1.0 - smoothstep(render_dist * 0.55, render_dist, horizontal_dist);

            // -------------------------------------------------------------
            // 自阴影采样优化 (每 8 步更新一次)
            //    降低采样频率，避免因高频阴影计算导致性能损耗与闪烁。
            // -------------------------------------------------------------
            if ((i % 8) == 0) {
                transmission = light_transmission(sample_pos, density);
            }

            // -------------------------------------------------------------
            // 顶亮底暗颜色混合
            // -------------------------------------------------------------
            let local_y = clamp((sample_pos.y - cloud.cloud_min_y) /
                                max(cloud.cloud_max_y - cloud.cloud_min_y, 0.001), 0.0, 1.0);
            let height_factor = smoothstep(0.0, 1.0, local_y);
            let base_col = mix(bottom_color, top_color, height_factor);
            let col = base_col * mix(SHADOW_MIN_TRANSMISSION, 1.0, transmission);

            // -------------------------------------------------------------
            // 体积积分 (固定步长)
            //    固定步长值乘以密度，得到当前采样点贡献的不透明度增量。
            // -------------------------------------------------------------
            let step_alpha = clamp(density * march_step * 0.026, 0.0, 0.18) * dist_fade;

            // -------------------------------------------------------------
            // 从前往后混合 (Alpha Blending)
            // -------------------------------------------------------------
            accum_color = accum_color + col * step_alpha * (1.0 - accum_alpha);
            accum_alpha = accum_alpha + step_alpha * (1.0 - accum_alpha);
        }

        // 向前迈进一步
        t = t + march_step;
    }

    // -------------------------------------------------------------------------
    // 5. 太阳光晕 (高光散射)
    // -------------------------------------------------------------------------
    let sun_dir = normalize(cloud.sun_direction.xyz);
    let light_dot = max(dot(ray_dir, sun_dir), 0.0);
    let scatter = pow(light_dot, 8.0) * 0.75;
    accum_color = accum_color + cloud_tint * scatter * accum_alpha;

    if (accum_alpha <= 0.0001) {
        discard;
    }

    // -------------------------------------------------------------------------
    // 6. 输出处理 (预乘 Alpha 转 直通 Alpha)
    // -------------------------------------------------------------------------
    let straight_rgb = accum_color / max(accum_alpha, 0.001);

    let coverage_alpha = smoothstep(0.0, 0.12, cloud.coverage);
    let final_alpha = accum_alpha * cloud.tint_day.a * cloud.visibility * coverage_alpha;

    pbr_input.material.base_color = vec4<f32>(straight_rgb, final_alpha);
    pbr_input.material.emissive = vec4<f32>(vec3<f32>(0.0), 1.0);

    // -------------------------------------------------------------------------
    // 7. Bevy PBR 管线输出
    // -------------------------------------------------------------------------
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}