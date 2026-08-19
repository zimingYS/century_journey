//! Voxel 云的几何生成：分级云团、密度场与三级角色判定。
//!
//! 这里是"云长什么样"的纯几何层，与渲染（`voxel.rs`）解耦：
//! - 分级撒点（`CloudTier`：Large/Medium/Small）形成天空纵深；
//! - 每朵云 = 宽厚核心 + 4~8 个鼓包（`CloudCluster`）；
//! - 密度场 = core_shape + lobe_shape + 低频厚度变化 + 边缘衰减；
//! - `sample_voxel` 把空间点归类为核心/鼓包/外壳，供三级 voxel 尺寸分层。
//!
//! 全部函数无副作用、同 seed 完全确定，便于白盒测试。

use bevy::math::Vec2;
use bevy::math::Vec3;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// 鼓包（lobe）voxel 边长（米），也是三级结构中的中档尺寸。
pub const CLOUD_CELL_SIZE: f32 = 6.0;
/// 核心（core）voxel 边长（米）：中央区域最大的块状结构。
pub const CORE_CELL_SIZE: f32 = 8.0;
/// 外壳（edge）voxel 边长（米）：边缘最小尺寸，形成阶梯轮廓。
pub const EDGE_CELL_SIZE: f32 = 4.0;
/// 云场水平覆盖半径（米）：云中心到边缘的最大水平距离。
pub const CLOUD_RANGE: f32 = 160.0;
/// 外壳包络膨胀系数：核心/鼓包归一化半径的判定上限。
pub const ENVELOPE_SCALE: f32 = 1.3;
/// 实心核归一化半径上限：r < 0.8 属于核心/鼓包大块，0.8~1.3 属于外壳。
pub const SOLID_CORE_R: f32 = 0.8;
/// 外壳层生成的最低密度阈值：只保留紧贴实心核外表面的单层（约 r < 0.87），
/// 避免 4m 小块铺满整个椭球外壳导致海量 draw call。
pub const EDGE_DENSITY_THRESHOLD: f32 = 0.6;
/// 底部深色材质的判定带（米）：core_bottom 以上 3m 内使用冷深灰。
pub const BOTTOM_SHADE_BAND: f32 = 3.0;
/// 低频厚度噪声的采样波长（米）：决定局部厚度起伏的空间尺度。
pub const LOW_FREQ_SCALE: f32 = 48.0;
/// 鼓包数量上限。
pub const MAX_LOBES: usize = 8;

/// 云团等级：决定核心尺寸、鼓包数量与整体比例，构成天空纵深。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudTier {
    /// 大型云（近处主力，2~4 个）。
    Large,
    /// 中型云（中距，若干）。
    Medium,
    /// 小型云（远处点缀，少量）。
    Small,
}

impl CloudTier {
    /// 核心水平半轴（米）：按等级 + 随机 [0,1) 落在区间内。
    fn core_radius(&self, h: f32) -> f32 {
        match self {
            CloudTier::Large => 40.0 + h * 20.0,
            CloudTier::Medium => 26.0 + h * 14.0,
            CloudTier::Small => 14.0 + h * 10.0,
        }
    }

    /// 核心垂直半轴占水平半轴的比例（ry/rx），整体横向:纵向 ≈ 2~3:1。
    fn vertical_ratio(&self) -> f32 {
        match self {
            CloudTier::Large => 0.38,
            CloudTier::Medium => 0.40,
            CloudTier::Small => 0.42,
        }
    }

    /// 鼓包数量：大云更多鼓包（5~8），小云较少（3~4）。
    fn lobe_count(&self, h: f32) -> usize {
        match self {
            CloudTier::Large => 5 + (h * 4.0) as usize,
            CloudTier::Medium => 4 + (h * 3.0) as usize,
            CloudTier::Small => 3 + (h * 2.0) as usize,
        }
    }

    /// 撒点距离带（米，相对云场中心）：近大远小，形成纵深。
    fn dist_range(&self) -> (f32, f32) {
        match self {
            CloudTier::Large => (0.0, 70.0),
            CloudTier::Medium => (40.0, 125.0),
            CloudTier::Small => (90.0, 155.0),
        }
    }
}

/// 一个云鼓包（lobe）：叠加在核心上的独立小型 voxel volume。
#[derive(Debug, Clone, Copy)]
pub struct CloudLobe {
    /// 相对核心中心的水平偏移 (x, z)（米）。
    pub offset: Vec2,
    /// 半轴 (rx, ry, rz)：宽、深、高各自独立。
    pub radius: Vec3,
    /// 鼓包中心相对核心中心的抬升高度（米）。
    pub lift: f32,
}

/// 一个巨型云团：宽厚核心 + 多个鼓包（Mega Cloud Cluster）。
#[derive(Debug, Clone, Copy)]
pub struct CloudCluster {
    /// 云团等级。
    pub tier: CloudTier,
    /// 核心椭球中心（世界坐标）。
    pub center: Vec3,
    /// 核心半轴 (rx, ry, rz)，横向宽、纵向厚。
    pub radius: Vec3,
    /// 包络峰值密度（0~1+），越大该团越实。
    pub density: f32,
    /// 叠加在核心上的鼓包列表（4~8 个）。
    pub lobes: [CloudLobe; MAX_LOBES],
    /// 实际鼓包数量（lobes[..count] 有效）。
    pub lobe_count: usize,
}

impl CloudCluster {
    /// 核心椭球底部高度（世界坐标 y）。
    pub fn core_bottom(&self) -> f32 {
        self.center.y - self.radius.y
    }

    /// 第 i 个鼓包的中心（世界坐标）。
    pub fn lobe_center(&self, i: usize) -> Vec3 {
        let lobe = &self.lobes[i];
        self.center + Vec3::new(lobe.offset.x, lobe.lift, lobe.offset.y)
    }
}

/// voxel 空间角色：决定 cell 尺寸与材质档位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelRole {
    /// 核心大块（8m cell，亮白）。
    Core,
    /// 鼓包中块（6m cell，主体色）。
    Lobe,
    /// 外壳小块（4m cell，灰蓝/冷深灰）。
    Edge,
}

/// 单点采样结果：角色 + 所属 cluster 的核心底部高度（底部材质判定用）。
#[derive(Debug, Clone, Copy)]
pub struct VoxelSample {
    /// 空间角色。
    pub role: VoxelRole,
    /// 所属 cluster 的核心底部高度。
    pub core_bottom: f32,
}

/// 返回 cluster 的核心与所有鼓包的 (中心, 半轴) 列表，供包络 AABB 计算。
pub fn cluster_envelopes(cluster: &CloudCluster) -> Vec<(Vec3, Vec3)> {
    let mut out = Vec::with_capacity(cluster.lobe_count + 1);
    out.push((cluster.center, cluster.radius));
    for i in 0..cluster.lobe_count {
        out.push((cluster.lobe_center(i), cluster.lobes[i].radius));
    }
    out
}

/// 分级撒出大型/中型/小型云团，形成天空纵深。
///
/// 随机性只来自确定性 `StdRng`（seed 固定则结果完全确定）；每朵云由宽厚
/// 核心 + 4~8 个鼓包构成，鼓包相互重叠融合。
pub fn generate_clusters(
    base_height: f32,
    coverage: f32,
    seed: u32,
    anchor: Vec3,
) -> Vec<CloudCluster> {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut clusters = Vec::new();

    // 数量随云量上升：Large 2~4、Medium 3~6、Small 2~4。
    let large_count = 2 + (coverage * 2.0) as usize;
    let medium_count = 3 + (coverage * 3.0) as usize;
    let small_count = 2 + (coverage * 2.0) as usize;

    spawn_tier(
        &mut clusters,
        &mut rng,
        base_height,
        anchor,
        CloudTier::Large,
        large_count,
    );
    spawn_tier(
        &mut clusters,
        &mut rng,
        base_height,
        anchor,
        CloudTier::Medium,
        medium_count,
    );
    spawn_tier(
        &mut clusters,
        &mut rng,
        base_height,
        anchor,
        CloudTier::Small,
        small_count,
    );

    clusters
}

/// 生成某一等级的若干云团。
fn spawn_tier(
    clusters: &mut Vec<CloudCluster>,
    rng: &mut StdRng,
    base_height: f32,
    anchor: Vec3,
    tier: CloudTier,
    count: usize,
) {
    for _ in 0..count {
        // 位置：按等级的距离带 + 随机方向；高度在云层基线附近错落。
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let (dist_min, dist_max) = tier.dist_range();
        let dist = rng.random_range(dist_min..dist_max);
        let cx = anchor.x + angle.cos() * dist;
        let cz = anchor.z + angle.sin() * dist;
        let cy = base_height + rng.random_range(-6.0..8.0);

        // 核心：横向宽（椭圆）、纵向厚（ry/rx ≈ 0.42~0.48）。
        let rx = tier.core_radius(rng.random_range(0.0..1.0));
        let rz = rx * rng.random_range(0.75..1.3);
        let ry = rx * tier.vertical_ratio() * rng.random_range(0.85..1.2);
        let density = rng.random_range(0.95..1.2);

        // 4~8 个鼓包：位置/大小/高度各自独立，与核心部分重叠融合成云顶 hill。
        let avg_r = (rx + rz) * 0.5;
        let lobe_count = tier.lobe_count(rng.random_range(0.0..1.0));
        let mut lobes = [CloudLobe {
            offset: Vec2::ZERO,
            radius: Vec3::ZERO,
            lift: 0.0,
        }; MAX_LOBES];
        for lobe_slot in lobes.iter_mut().take(lobe_count) {
            let lobe_angle = rng.random_range(0.0..std::f32::consts::TAU);
            let lobe_dist = avg_r * rng.random_range(0.1..0.9);
            let lrx = avg_r * rng.random_range(0.25..0.5);
            let lrz = avg_r * rng.random_range(0.25..0.5);
            // 鼓包扁而低：凸出核心顶最多约 0.5ry，避免把云拉成窄高柱。
            let lry = ry * rng.random_range(0.5..0.9);
            let lift = ry * rng.random_range(0.15..0.6);
            *lobe_slot = CloudLobe {
                offset: Vec2::new(lobe_angle.cos() * lobe_dist, lobe_angle.sin() * lobe_dist),
                radius: Vec3::new(lrx, lry, lrz),
                lift,
            };
        }

        clusters.push(CloudCluster {
            tier,
            center: Vec3::new(cx, cy, cz),
            radius: Vec3::new(rx, ry, rz),
            density,
            lobes,
            lobe_count,
        });
    }
}

/// 3D 密度场：core_shape + lobe_shape + 低频厚度变化 + 边缘衰减。
///
/// - 核心与鼓包椭球密度相加（`+`）：重叠区密度更高 → 更实、更连续。
/// - 核心垂直厚度受 2D 低频噪声调制（±15%），让局部厚度起伏、边缘阶梯
///   长短不一，而非规则椭球。
/// - 边缘衰减已由 `ellipsoid_density` 的平滑段承担。
pub fn density_at(p: Vec3, clusters: &[CloudCluster], seed: u32) -> f32 {
    let mut d = 0.0;
    for cluster in clusters {
        let core_d = core_density(p, cluster, seed);
        let lobe_d = lobe_density(p, cluster);
        d += cluster.density * (core_d + lobe_d);
    }
    d
}

/// 核心密度：椭球 + 低频厚度调制。
fn core_density(p: Vec3, cluster: &CloudCluster, seed: u32) -> f32 {
    let thick = low_freq_thickness(p.x, p.z, seed);
    let radius = Vec3::new(cluster.radius.x, cluster.radius.y * thick, cluster.radius.z);
    ellipsoid_density(p, cluster.center, radius)
}

/// 鼓包密度：各 lobe 椭球叠加（重叠融合）。
fn lobe_density(p: Vec3, cluster: &CloudCluster) -> f32 {
    let mut d: f32 = 0.0;
    for i in 0..cluster.lobe_count {
        let lobe = &cluster.lobes[i];
        d += ellipsoid_density(p, cluster.lobe_center(i), lobe.radius);
    }
    d
}

/// 椭球密度：实心核（r < 0.55）为 1，0.55~1.3 平滑衰减到 0。
fn ellipsoid_density(p: Vec3, center: Vec3, radius: Vec3) -> f32 {
    let n = (p - center) / radius;
    let r = n.length();
    if r >= ENVELOPE_SCALE {
        return 0.0;
    }
    if r <= 0.55 {
        return 1.0;
    }
    let t = ((r - 0.55) / (ENVELOPE_SCALE - 0.55)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// 低频厚度调制：2D 低频 value noise（波长 `LOW_FREQ_SCALE`），输出约 0.85~1.2。
///
/// 低频（相对 6m voxel 是超大尺度）保证只改变局部厚度与边缘轮廓，
/// 不产生高频碎片噪声。
pub fn low_freq_thickness(x: f32, z: f32, seed: u32) -> f32 {
    let n1 = value_noise_2d(x / LOW_FREQ_SCALE, z / LOW_FREQ_SCALE, seed);
    let n2 = value_noise_2d(
        x / (LOW_FREQ_SCALE * 0.5),
        z / (LOW_FREQ_SCALE * 0.5),
        seed.wrapping_add(0x9E3779B9),
    );
    0.85 + (n1 * 0.7 + n2 * 0.3) * 0.35
}

/// 2D value noise：网格 hash + smoothstep 双线性插值，返回 [0, 1]。
fn value_noise_2d(x: f32, z: f32, seed: u32) -> f32 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = x - xi;
    let zf = z - zi;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = zf * zf * (3.0 - 2.0 * zf);

    let xi = xi as i32;
    let zi = zi as i32;
    let c00 = hash_unit(xi, zi, seed);
    let c10 = hash_unit(xi + 1, zi, seed);
    let c01 = hash_unit(xi, zi + 1, seed);
    let c11 = hash_unit(xi + 1, zi + 1, seed);

    let x0 = c00 + (c10 - c00) * u;
    let x1 = c01 + (c11 - c01) * u;
    x0 + (x1 - x0) * v
}

/// 单点角色采样：先查核心/鼓包实心核（r < 0.8，取最近者），
/// 再查包络（r < 1.3），返回角色与所属 cluster 的核心底部高度。
pub fn sample_voxel(p: Vec3, clusters: &[CloudCluster]) -> Option<VoxelSample> {
    let mut best_solid = f32::MAX;
    let mut best_role = None;
    let mut best_env = f32::MAX;
    let mut best_env_bottom = 0.0;

    for cluster in clusters {
        let bottom = cluster.core_bottom();

        let core_r = normalized_radius(p, cluster.center, cluster.radius);
        if core_r < SOLID_CORE_R && core_r < best_solid {
            best_solid = core_r;
            best_role = Some(VoxelSample {
                role: VoxelRole::Core,
                core_bottom: bottom,
            });
        }
        if core_r < best_env {
            best_env = core_r;
            best_env_bottom = bottom;
        }

        for i in 0..cluster.lobe_count {
            let c = cluster.lobe_center(i);
            let r = normalized_radius(p, c, cluster.lobes[i].radius);
            if r < SOLID_CORE_R && r < best_solid {
                best_solid = r;
                best_role = Some(VoxelSample {
                    role: VoxelRole::Lobe,
                    core_bottom: bottom,
                });
            }
            if r < best_env {
                best_env = r;
                best_env_bottom = bottom;
            }
        }
    }

    if let Some(sample) = best_role {
        return Some(sample);
    }
    if best_env < ENVELOPE_SCALE {
        return Some(VoxelSample {
            role: VoxelRole::Edge,
            core_bottom: best_env_bottom,
        });
    }
    None
}

/// 到椭球中心的归一化距离（椭球坐标系下的"半径"，<1 在椭球内）。
fn normalized_radius(p: Vec3, center: Vec3, radius: Vec3) -> f32 {
    let n = (p - center) / radius;
    n.length()
}

/// 将 (x, z, seed) 映射到 [0, 1) 的确定性 2D hash，用于噪声网格插值。
pub fn hash_unit(x: i32, z: i32, seed: u32) -> f32 {
    let mut h: u32 = seed.wrapping_add(0x9E3779B9);
    h = h.wrapping_mul(2654435761);
    h ^= (x as u32).wrapping_mul(0x85EBCA77);
    h = h.rotate_left(13);
    h ^= (z as u32).wrapping_mul(0xC2B2AE3D);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EBCA6B);
    h ^= h >> 13;
    (h & 0x00FFFFFF) as f32 / 0x01000000 as f32
}
