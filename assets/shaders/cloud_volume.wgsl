// =============================================================================
// CenturyJourney - 3D 体素体积云着色器
// =============================================================================

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
}

// =============================================================================
// 引擎侧云层参数统一缓冲区 (Uniform Buffer)
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

// =============================================================================
// 渲染参数常量
// =============================================================================

// 最大光线步进步数 (128步既能覆盖480米渲染距离，又可保障性能)
const MAX_STEPS: i32 = 128;

// 云的最大世界空间可视距离（独立于 cell_size 固定为 480.0 米）
const CLOUD_RENDER_DISTANCE: f32 = 480.0;

// 体积光学消光强度。数值越大，光线穿透时衰减越快，云层看起来越浓密厚实。
const CLOUD_EXTINCTION: f32 = 1.15;

// 单个体素产生的最低有效不透明度，低于此值直接跳过积分。
const MIN_STEP_ALPHA: f32 = 0.0005;

// 累积不透明度接近完全不透明时的提前截止阈值，用于性能优化。
const MAX_ACCUMULATED_ALPHA: f32 = 0.985;

// 云层垂直方向的顶部和底部淡出范围（用于模拟云的平坦顶底）
const CLOUD_BOTTOM_FADE: f32 = 0.055;
const CLOUD_TOP_FADE: f32 = 0.075;

// 体素密度边缘的柔化宽度。
// 该数值作用于噪声阈值过度的区间，影响体素的“马赛克感”，而非直接模糊网格。
const VOXEL_EDGE_SOFTNESS: f32 = 0.105;

// 太阳高光散射参数。
const SUN_SCATTER_POWER: f32 = 8.0;
const SUN_SCATTER_STRENGTH: f32 = 0.65;

// =============================================================================
// 3D 伪随机哈希 (用于 3D 值噪声)
// =============================================================================
fn hash31(p: vec3<f32>) -> f32 {
    let n = sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453;
    return fract(n);
}

// =============================================================================
// 3D 值噪声 (基于三线性插值)
// =============================================================================
fn value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // 平滑 Hermite 曲线

    // 立方体的 8 个顶点哈希值
    let n000 = hash31(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash31(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash31(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash31(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash31(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash31(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash31(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash31(i + vec3<f32>(1.0, 1.0, 1.0));

    // 三线性插值 (X/Y/Z 依次插值)
    let low = mix(mix(n000, n100, u.x), mix(n010, n110, u.x), u.y);
    let high = mix(mix(n001, n101, u.x), mix(n011, n111, u.x), u.y);
    return mix(low, high, u.z);
}

// =============================================================================
// 3层 FBM (分形布朗运动)
// =============================================================================
fn fbm3(p: vec3<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var sample = p;

    for (var octave: i32 = 0; octave < 3; octave = octave + 1) {
        value += value_noise_3d(sample) * amplitude;
        // 频率倍率，并增加偏移以避免网格对齐
        sample = sample * 2.03 + vec3<f32>(11.7, 7.9, 23.5);
        amplitude *= 0.5; // 振幅递减
    }

    return value / 0.9375; // 归一化返回值域到 [0, 1] 附近
}

// =============================================================================
// 云层密度计算函数
//
// 核心架构：真正的 3D 体素密度。
// X、Y、Z 使用统一的空间噪声尺度 (`noise_scale`)。
// 体素(`cell_size`) 仅用于决定采样的离散网格位置，而噪声尺度决定云团的宏观形状。
// =============================================================================
fn get_layer_density(
    world_xz: vec2<f32>,
    world_y: f32,
    time: f32,
    coverage: f32,
    narrowness: f32,
    layer_idx: i32,
    cell_size: f32,
    layer_bottom: f32
) -> f32 {
    //  风场驱动偏移，产生云的飘移动画
    let wind = cloud.wind_direction.xy * time * cloud.wind_speed;
    var moved_xz = world_xz + wind;

    // 不同云层叠加不同的空间偏移，避免三层云完全重叠对齐
    if (layer_idx == 2) { moved_xz += vec2<f32>(5333.0, 2187.0); }
    if (layer_idx == 3) { moved_xz += vec2<f32>(-2814.0, 6942.0); }

    // XZ 平面体素化坐标 (量化网格)
    let voxel = max(cell_size, 0.001);
    let cell_id = floor(moved_xz / voxel);
    let cell_center = (cell_id + vec2<f32>(0.5, 0.5)) * voxel;

    // Y 轴体素化 (为了保持完全各向同性的 3D 体素感)
    let y_cell = floor((world_y - layer_bottom) / voxel);
    let y_center = layer_bottom + (y_cell + 0.5) * voxel;

    // 确定噪声缩放比例
    let fallback_scale = max(narrowness * 0.15, 0.0001);
    let noise_scale = select(fallback_scale, max(cloud.noise_scale, 0.0001), cloud.noise_scale > 0.0001);

    // 不同层应用不同的宏观缩放以产生细节差异
    var layer_scale = 1.0;
    if (layer_idx == 2) { layer_scale = 1.15; }
    if (layer_idx == 3) { layer_scale = 1.30; }

    // 基于体素中心计算三维空间中的 FBM 采样坐标
    let relative_y = y_center - layer_bottom;
    let base_position = vec3<f32>(cell_center.x, relative_y, cell_center.y) * noise_scale * layer_scale;

    let p3_a = base_position + vec3<f32>(0.0, time * 0.012, 0.0);
    let p3_b = base_position * 1.07 + vec3<f32>(17.3, 31.7, 9.4) + vec3<f32>(0.0, time * 0.008, 0.0);

    var density = mix(fbm3(p3_a), fbm3(p3_b), 0.5);

    // 细节噪声层级 (Detail)
    let detail_strength = clamp(cloud.detail_strength, 0.0, 1.0);
    let detail_position = base_position * 2.02 + vec3<f32>(45.6, 12.8, 78.9) + vec3<f32>(0.0, time * 0.018, 0.0);
    let detail = fbm3(detail_position);
    density = mix(density, density * 0.72 + detail * 0.28, detail_strength);

    // 最精细层级噪声 (Fine detail)
    let fine_position = base_position * 4.03 + vec3<f32>(91.7, 17.2, 43.5) + vec3<f32>(0.0, time * 0.028, 0.0);
    let fine = fbm3(fine_position);
    density += (fine - 0.5) * 0.10 * detail_strength;

    // 平滑与归一化
    density = clamp(density, 0.0, 1.0);
    density = smoothstep(0.10, 0.90, density);

    // 覆盖度 (Coverage) 映射
    let actual_coverage = clamp(coverage, 0.0, 1.0); // 修复原版 coverage=0 时强制 0.85 的硬编码
    let coverage_threshold = mix(0.72, 0.40, actual_coverage); // 覆盖度越高，阈值越低，云越多

    // 外部密度阈值调节
    let external_threshold = clamp(cloud.density_threshold, 0.0, 1.0);
    let threshold = mix(coverage_threshold, external_threshold, select(0.0, 1.0, cloud.density_threshold > 0.001));

    // 软体素边缘过渡 (VOXEL_EDGE_SOFTNESS)
    let edge = max(VOXEL_EDGE_SOFTNESS, 0.001);
    return smoothstep(threshold - edge, threshold + edge, density);
}

// =============================================================================
// 射线与垂直云层区块相交判定
// =============================================================================
fn ray_slab_intersect(origin: vec3<f32>, dir: vec3<f32>, y_bottom: f32, y_top: f32) -> vec2<f32> {
    let safe_dir_y = select(0.000001, dir.y, abs(dir.y) > 0.000001); // 防止视线绝对水平时除零错误
    let t0 = (y_bottom - origin.y) / safe_dir_y;
    let t1 = (y_top - origin.y) / safe_dir_y;
    var t_near = min(t0, t1);
    var t_far = max(t0, t1);
    if (t_far < 0.0) { return vec2<f32>(-1.0, -1.0); }
    t_near = max(t_near, 0.0);
    return vec2<f32>(t_near, t_far);
}

// =============================================================================
// 单层云的光线追踪 (核心 DDA 算法)
// =============================================================================
fn trace_layer(
    cam_pos: vec3<f32>,
    dir: vec3<f32>,
    t_near: f32,
    t_far: f32,
    layer_bottom: f32,
    layer_top: f32,
    coverage: f32,
    cell_size: f32,
    narrowness: f32,
    time: f32,
    render_dist: f32,
    layer_idx: i32,
    top_col: vec3<f32>,
    bot_col: vec3<f32>
) -> vec4<f32> {
    if (t_near >= t_far) { return vec4<f32>(0.0); }

    let voxel = max(cell_size, 0.001);
    let t_limit = min(t_far, t_near + render_dist);
    if (t_near >= t_limit) { return vec4<f32>(0.0); }

    // 获取风场数据
    let wind = cloud.wind_direction.xy * time * cloud.wind_speed;
    let entry_pos = cam_pos + dir * t_near;
    let start_xz = entry_pos.xz + wind;
    let dir_xz = dir.xz;

    // 累积变量 (返回的是非预乘的直通 Alpha)
    var acc_color = vec3<f32>(0.0);
    var acc_alpha = 0.0;

    // DDA 初始化：计算入口所在的体素网格
    var cell_idx = floor(start_xz / voxel);

    // 计算 XZ 平面的行进方向
    var step_dir = vec2<f32>(0.0, 0.0);
    if (dir_xz.x > 0.0) { step_dir.x = 1.0; }
    else if (dir_xz.x < 0.0) { step_dir.x = -1.0; }
    if (dir_xz.y > 0.0) { step_dir.y = 1.0; }
    else if (dir_xz.y < 0.0) { step_dir.y = -1.0; }

    // 计算方向倒数、跨过一个体素所需的时间
    let inv_dir_x = select(1e20, 1.0 / dir_xz.x, abs(dir_xz.x) > 0.0001);
    let inv_dir_y = select(1e20, 1.0 / dir_xz.y, abs(dir_xz.y) > 0.0001);
    let t_delta_x = abs(voxel * inv_dir_x);
    let t_delta_y = abs(voxel * inv_dir_y);

    // 计算到达下一个网格边界的时间 tMax
    var t_max_x = 1e20;
    var t_max_y = 1e20;
    if (step_dir.x > 0.0) { t_max_x = ((cell_idx.x + 1.0) * voxel - start_xz.x) * inv_dir_x; }
    else if (step_dir.x < 0.0) { t_max_x = (cell_idx.x * voxel - start_xz.x) * inv_dir_x; }
    if (step_dir.y > 0.0) { t_max_y = ((cell_idx.y + 1.0) * voxel - start_xz.y) * inv_dir_y; }
    else if (step_dir.y < 0.0) { t_max_y = (cell_idx.y * voxel - start_xz.y) * inv_dir_y; }

    var cur_t = t_near;

    // DDA 光线步进主循环
    for (var i: i32 = 0; i < MAX_STEPS; i = i + 1) {
        if (acc_alpha >= MAX_ACCUMULATED_ALPHA) { break; }
        if (cur_t >= t_limit) { break; }

        // 获得离开当前体素最近的边界时间
        let boundary_t = min(t_max_x, t_max_y);
        let segment_start = cur_t;
        let segment_end = min(t_near + boundary_t, t_limit);

        // 防止因浮点误差导致 0 长度区间
        if (segment_end > segment_start + 0.00001) {

            // ================================================================
            // 在体素区间的正中心进行采样
            // ================================================================
            let sample_t = 0.5 * (segment_start + segment_end);
            let sample_pos = cam_pos + dir * sample_t;

            // 当前体素的中心坐标 (由风场偏移修正回世界空间)
            let voxel_center_winded = (vec2<f32>(cell_idx) + vec2<f32>(0.5, 0.5)) * voxel;
            let world_cell = voxel_center_winded - wind;

            let density = get_layer_density(
                world_cell, sample_pos.y, time, coverage, narrowness, layer_idx, cell_size, layer_bottom
            );

            // ================================================================
            // 基于真实路径长度的 Beer-Lambert 体积积分。
            // ================================================================
            let segment_length = segment_end - segment_start;

            // 垂直方向的高度渐变 (顶亮底暗)
            let local_y = clamp((sample_pos.y - layer_bottom) / max(layer_top - layer_bottom, 0.001), 0.0, 1.0);
            let bottom_fade = smoothstep(0.0, CLOUD_BOTTOM_FADE, local_y);
            let top_fade = 1.0 - smoothstep(1.0 - CLOUD_TOP_FADE, 1.0, local_y);
            let vertical_fade = bottom_fade * top_fade;

            // 基于光线真实长度的距离淡出 (修复了基于 XZ 平面的淡出判定)
            let hit_dist = sample_t;
            let distance_fade = 1.0 - smoothstep(render_dist * 0.70, render_dist, hit_dist);

            let effective_density = density * vertical_fade * distance_fade;

            if (effective_density > 0.0001) {
                // Beer-Lambert 消光公式求不透明度
                let optical_depth = effective_density * segment_length * CLOUD_EXTINCTION;
                let step_alpha = 1.0 - exp(-optical_depth);

                if (step_alpha > MIN_STEP_ALPHA) {
                    let vertical_color = mix(bot_col, top_col, smoothstep(0.0, 1.0, local_y));

                    // 从前往后 Alpha 混合 (直通 Alpha)
                    let remaining = 1.0 - acc_alpha;
                    acc_color += vertical_color * step_alpha * remaining;
                    acc_alpha += step_alpha * remaining;
                }
            }
        }

        // 沿着网格向前推进
        cur_t = segment_end;
        if (cur_t >= t_limit) { break; }

        if (t_max_x < t_max_y) {
            cell_idx.x += step_dir.x;
            t_max_x += t_delta_x;
        } else {
            cell_idx.y += step_dir.y;
            t_max_y += t_delta_y;
        }
    }

    return vec4(acc_color, acc_alpha);
}

// =============================================================================
// 片元着色器主入口
// =============================================================================
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // 相机视线与位置
    let ray_origin = cloud.camera_position.xyz;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);

    // 云层尺寸与布局定义
    let base_altitude = cloud.cloud_min_y;
    let layer_thick = max(cloud.cloud_max_y - cloud.cloud_min_y, 0.001);

    // 降低层间距，使三层云在视觉上融合为一个整体，而非彼此脱离
    let layer_gap = layer_thick * 0.45;
    let cell_size = max(cloud.cell_size, 0.001);
    let render_dist = CLOUD_RENDER_DISTANCE; // 改用固定距离，不再被体素大小绑架

    // 云体颜色处理 (昼夜与黄昏)
    var cloud_tint = mix(cloud.tint_day.rgb, cloud.tint_night.rgb, clamp(cloud.night_factor, 0.0, 1.0));
    cloud_tint = mix(cloud_tint, cloud.tint_sunset.rgb, clamp(cloud.twilight_glow * 0.5, 0.0, 1.0));
    let top_color = cloud_tint;
    let bottom_color = cloud_tint * 0.40; // 底部较暗，呈现立体阴影

    let effective_coverage = clamp(cloud.coverage, 0.0, 1.0);

    // 光线追踪三层云
    // Layer 1
    let l1_bot = base_altitude;
    let l1_top = base_altitude + layer_thick;
    let dist1 = ray_slab_intersect(ray_origin, ray_dir, l1_bot, l1_top);
    var col1 = vec4<f32>(0.0);
    var depth1 = 1e30;
    if (dist1.y > 0.0 && dist1.x < dist1.y) {
        col1 = trace_layer(ray_origin, ray_dir, dist1.x, dist1.y, l1_bot, l1_top, effective_coverage, cell_size, 0.07, cloud.time_seconds, render_dist, 1, top_color, bottom_color);
        depth1 = dist1.x;
    }

    // Layer 2
    let l2_bot = l1_top + layer_gap;
    let l2_top = l2_bot + layer_thick;
    let dist2 = ray_slab_intersect(ray_origin, ray_dir, l2_bot, l2_top);
    var col2 = vec4<f32>(0.0);
    var depth2 = 1e30;
    if (dist2.y > 0.0 && dist2.x < dist2.y) {
        col2 = trace_layer(ray_origin, ray_dir, dist2.x, dist2.y, l2_bot, l2_top, effective_coverage * 0.78, cell_size, 0.07, cloud.time_seconds, render_dist, 2, top_color, bottom_color);
        depth2 = dist2.x;
    }

    // Layer 3
    let l3_bot = l2_top + layer_gap;
    let l3_top = l3_bot + layer_thick;
    let dist3 = ray_slab_intersect(ray_origin, ray_dir, l3_bot, l3_top);
    var col3 = vec4<f32>(0.0);
    var depth3 = 1e30;
    if (dist3.y > 0.0 && dist3.x < dist3.y) {
        col3 = trace_layer(ray_origin, ray_dir, dist3.x, dist3.y, l3_bot, l3_top, effective_coverage * 0.58, cell_size, 0.07, cloud.time_seconds, render_dist, 3, top_color, bottom_color);
        depth3 = dist3.x;
    }

    // 对三层云从远到近进行排序 (冒泡排序)
    var arr_col = array<vec4<f32>, 3>(col1, col2, col3);
    var arr_dep = array<f32, 3>(depth1, depth2, depth3);
    for (var i = 0; i < 2; i++) {
        for (var j = 0; j < 2 - i; j++) {
            if (arr_dep[j] < arr_dep[j + 1]) {
                let tmp_d = arr_dep[j];
                arr_dep[j] = arr_dep[j + 1];
                arr_dep[j + 1] = tmp_d;
                let tmp_c = arr_col[j];
                arr_col[j] = arr_col[j + 1];
                arr_col[j + 1] = tmp_c;
            }
        }
    }

    // 混合三层云 (直通 Alpha 混合)
    // trace_layer 返回的是非预乘 RGB 和直通 Alpha。
    var final_rgb = vec3<f32>(0.0);
    var final_alpha = 0.0;
    for (var k = 0; k < 3; k++) {
        let layer = arr_col[k];
        if (layer.a > 0.0) {
            let remaining = 1.0 - final_alpha;
            final_rgb += layer.rgb * layer.a * remaining;
            final_alpha += layer.a * remaining;
        }
    }

    // 太阳光晕散射效果
    let sun_dir = normalize(cloud.sun_direction.xyz);
    let light_dot = max(dot(ray_dir, sun_dir), 0.0);
    let scatter = pow(light_dot, SUN_SCATTER_POWER) * SUN_SCATTER_STRENGTH;
    final_rgb += cloud_tint * scatter * final_alpha;

    if (final_alpha <= 0.001) { discard; }

    // 最终透明度控制与输出
    let tint_alpha = max(cloud.tint_day.a, 0.0);
    let visibility = clamp(cloud.visibility, 0.0, 1.0);
    let coverage_alpha = smoothstep(0.0, 0.12, effective_coverage);
    let final_alpha_output = final_alpha * tint_alpha * visibility * coverage_alpha;

    pbr_input.material.base_color = vec4<f32>(final_rgb, final_alpha_output);
    pbr_input.material.emissive = vec4<f32>(vec3<f32>(0.0), 1.0);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}