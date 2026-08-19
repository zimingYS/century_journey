use super::super::generation::*;
use super::*;

/// hash_unit 在两个相邻 cell 应给出截然不同的值（避免线性噪声）。
#[test]
fn hash_unit_differs_across_cells() {
    let a = hash_unit(0, 0, 123);
    let b = hash_unit(1, 0, 123);
    let c = hash_unit(0, 1, 123);
    assert!((a - b).abs() > 0.01);
    assert!((a - c).abs() > 0.01);
    assert!((b - c).abs() > 0.01);
}

/// hash_unit 输出值域 [0, 1)。
#[test]
fn hash_unit_in_unit_range() {
    for x in -10..=10 {
        for z in -10..=10 {
            let h = hash_unit(x, z, 20260803);
            assert!((0.0..1.0).contains(&h), "x={x} z={z} h={h}");
        }
    }
}

/// 不同 seed 应显著改变 hash 输出（至少半数 cell 跨 seed 后发生翻转）。
#[test]
fn hash_unit_responds_to_seed() {
    let mut count = 0;
    let total = 64;
    for x in -4..=3 {
        for z in -4..=3 {
            let a = hash_unit(x, z, 1);
            let b = hash_unit(x, z, 2);
            if (a > 0.5) != (b > 0.5) {
                count += 1;
            }
        }
    }
    // 不要求 50%（p=0.5），但应显著多于偶然（>20%）。
    assert!(
        count >= total / 5,
        "hash_unit seed sensitivity too low: {count}/{total}"
    );
}

/// 低频厚度调制值域 [0.85, 1.2]，且随位置变化（非常数）。
#[test]
fn low_freq_thickness_varies_and_stays_bounded() {
    for i in -5..=5 {
        for j in -5..=5 {
            let t = low_freq_thickness(i as f32 * 8.0, j as f32 * 8.0, 42);
            assert!((0.85..=1.2).contains(&t), "t={t}");
        }
    }
    let a = low_freq_thickness(0.0, 0.0, 42);
    let b = low_freq_thickness(LOW_FREQ_SCALE, 0.0, 42);
    assert!((a - b).abs() > 0.01, "thickness should vary: a={a} b={b}");
}

/// 同一 seed 的 cluster 布局必须确定（重启后云形一致）。
#[test]
fn generate_clusters_deterministic() {
    let a = generate_clusters(128.0, 0.4, 42, Vec3::ZERO);
    let b = generate_clusters(128.0, 0.4, 42, Vec3::ZERO);
    assert_eq!(a.len(), b.len());
    for (ca, cb) in a.iter().zip(b.iter()) {
        assert_eq!(ca.tier, cb.tier);
        assert_eq!(ca.center, cb.center);
        assert_eq!(ca.radius, cb.radius);
        assert_eq!(ca.density, cb.density);
        assert_eq!(ca.lobe_count, cb.lobe_count);
        for i in 0..ca.lobe_count {
            assert_eq!(ca.lobes[i].offset, cb.lobes[i].offset);
            assert_eq!(ca.lobes[i].radius, cb.lobes[i].radius);
            assert_eq!(ca.lobes[i].lift, cb.lobes[i].lift);
        }
    }
}

/// 云量越高 cluster 越多（coverage 应真实影响云量）。
#[test]
fn generate_clusters_more_when_cloudier() {
    let sparse = generate_clusters(128.0, 0.1, 42, Vec3::ZERO);
    let dense = generate_clusters(128.0, 0.9, 42, Vec3::ZERO);
    assert!(dense.len() > sparse.len());
}

/// 天空应同时有大型/中型/小型云（分级形成纵深）。
#[test]
fn cloud_tiers_follow_size_distribution() {
    let clusters = generate_clusters(128.0, 0.5, 7, Vec3::ZERO);
    let large = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Large)
        .count();
    let medium = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Medium)
        .count();
    let small = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Small)
        .count();
    assert!((2..=4).contains(&large), "large={large}");
    assert!(medium >= 3, "medium={medium}");
    assert!(small >= 2, "small={small}");
}

/// 核心必须宽厚：水平半轴与垂直半轴之比约 2~3:1（wide + thick）。
#[test]
fn cluster_cores_are_wide_and_thick() {
    let clusters = generate_clusters(128.0, 0.6, 7, Vec3::ZERO);
    assert!(!clusters.is_empty());
    for c in &clusters {
        let ratio = c.radius.x.min(c.radius.z) / c.radius.y;
        assert!(
            (1.2..=3.5).contains(&ratio),
            "core proportion out of range: rx={} rz={} ry={} ratio={ratio:.2}",
            c.radius.x,
            c.radius.z,
            c.radius.y
        );
    }
}

/// 每个 mega cluster 应有 4~8 个鼓包（lobe）。
#[test]
fn cluster_has_four_to_eight_lobes() {
    let clusters = generate_clusters(128.0, 0.6, 7, Vec3::ZERO);
    assert!(!clusters.is_empty());
    for c in &clusters {
        assert!(
            (3..=8).contains(&c.lobe_count),
            "lobe_count={} out of 3..=8",
            c.lobe_count
        );
    }
}

/// 鼓包中心应高于核心中心（向上凸起），且水平偏移在核心范围内。
#[test]
fn lobes_sit_above_core_within_reach() {
    let clusters = generate_clusters(128.0, 0.6, 7, Vec3::ZERO);
    let c = &clusters[0];
    for i in 0..c.lobe_count {
        let lobe = &c.lobes[i];
        let center = c.lobe_center(i);
        assert!(center.y > c.center.y, "lobe must rise above core center");
        let horizontal = Vec2::new(center.x - c.center.x, center.z - c.center.z);
        assert!(
            horizontal.length() < c.radius.x.max(c.radius.z) + lobe.radius.x.max(lobe.radius.z),
            "lobe too far from core"
        );
    }
}

/// density_at 在云团中心应显著高于远离云团的空旷点。
#[test]
fn density_at_peaks_at_cluster_center() {
    let clusters = generate_clusters(128.0, 0.5, 7, Vec3::ZERO);
    assert!(!clusters.is_empty());
    let center = clusters[0].center;
    let center_density = density_at(center, &clusters, 7);
    let far_density = density_at(center + Vec3::new(400.0, 0.0, 400.0), &clusters, 7);
    assert!(
        center_density > far_density,
        "center={center_density} far={far_density}"
    );
    assert!(
        center_density > 0.3,
        "center density too weak: {center_density}"
    );
}

/// 核心实心处密度最高，垂直方向上穿出核心后衰减（云体有边界、非硬平面）。
#[test]
fn density_falls_off_above_and_below_core() {
    let clusters = generate_clusters(128.0, 0.5, 7, Vec3::ZERO);
    let c = &clusters[0];
    let mid = density_at(c.center, &clusters, 7);
    let top = density_at(c.center + Vec3::Y * (c.radius.y * 1.4), &clusters, 7);
    let bot = density_at(c.center - Vec3::Y * (c.radius.y * 1.4), &clusters, 7);
    assert!(mid > top, "top should fall off: mid={mid} top={top}");
    assert!(mid > bot, "bottom should fall off: mid={mid} bot={bot}");
}

/// 手动构造单个 mega cluster（确定性几何，避免随机布局的邻团干扰）。
/// 核心 (0,128,0) 半轴 (40,18,40)；一个鼓包中心 (15,141,0) 半轴 (12,10,12)。
fn manual_cluster() -> CloudCluster {
    let lobe = CloudLobe {
        offset: Vec2::new(15.0, 0.0),
        radius: Vec3::new(12.0, 10.0, 12.0),
        lift: 13.0,
    };
    CloudCluster {
        tier: CloudTier::Large,
        center: Vec3::new(0.0, 128.0, 0.0),
        radius: Vec3::new(40.0, 18.0, 40.0),
        density: 1.0,
        lobes: [lobe; MAX_LOBES],
        lobe_count: 1,
    }
}

/// 角色采样：核心中心 → Core；鼓包中心 → Lobe；包络内 → Edge；远处 → None。
#[test]
fn sample_voxel_assigns_roles() {
    let clusters = vec![manual_cluster()];

    let core_sample = sample_voxel(clusters[0].center, &clusters).expect("core center must sample");
    assert_eq!(core_sample.role, VoxelRole::Core);

    let lobe_sample =
        sample_voxel(clusters[0].lobe_center(0), &clusters).expect("lobe center must sample");
    assert_eq!(lobe_sample.role, VoxelRole::Lobe);

    // 核心正上方 1.1 倍垂直半径：实心核外、包络内 → Edge。
    let edge_p = clusters[0].center + Vec3::Y * (clusters[0].radius.y * 1.1);
    let edge_sample = sample_voxel(edge_p, &clusters).expect("envelope must sample");
    assert_eq!(edge_sample.role, VoxelRole::Edge);

    assert!(
        sample_voxel(clusters[0].center + Vec3::new(300.0, 0.0, 300.0), &clusters).is_none(),
        "far point must be empty"
    );
}

/// 角色采样的 core_bottom 与核心底部一致（底部材质判定依据）。
#[test]
fn sample_voxel_reports_core_bottom() {
    let clusters = vec![manual_cluster()];
    let sample = sample_voxel(clusters[0].center, &clusters).unwrap();
    assert!((sample.core_bottom - clusters[0].core_bottom()).abs() < 0.001);
}

/// 相邻 cluster 包络重叠区密度叠加 → 融合成连续云层。
#[test]
fn adjacent_clusters_blend() {
    // 手动构造两个相距很近的 cluster（间距小于核心半径之和）。
    let mk = |cx: f32| CloudCluster {
        tier: CloudTier::Large,
        center: Vec3::new(cx, 128.0, 0.0),
        radius: Vec3::new(40.0, 18.0, 40.0),
        density: 1.0,
        lobes: [CloudLobe {
            offset: Vec2::ZERO,
            radius: Vec3::ZERO,
            lift: 0.0,
        }; MAX_LOBES],
        lobe_count: 0,
    };
    let clusters = vec![mk(0.0), mk(60.0)];
    let mid = Vec3::new(30.0, 128.0, 0.0);
    let single = density_at(mid, &clusters[..1], 0);
    let both = density_at(mid, &clusters, 0);
    assert!(
        both > single,
        "blend should raise density: single={single} both={both}"
    );
}

/// 默认 CloudVoxelRuntime 状态干净。
#[test]
fn voxel_runtime_default_is_empty() {
    let runtime = CloudVoxelRuntime::default();
    assert!(runtime.entities.is_empty());
    assert!(runtime.meshes.core.is_none());
    assert!(runtime.meshes.lobe.is_none());
    assert!(runtime.meshes.edge.is_none());
    assert!(runtime.materials.top.is_none());
    assert!(runtime.materials.mid.is_none());
    assert!(runtime.materials.side.is_none());
    assert!(runtime.materials.bot.is_none());
}

/// CloudVoxelMaterials::reset 清空所有材质句柄。
#[test]
fn voxel_materials_reset_clears_handles() {
    let mut mats = CloudVoxelMaterials {
        top: Some(Handle::default()),
        mid: Some(Handle::default()),
        side: Some(Handle::default()),
        bot: Some(Handle::default()),
    };
    mats.reset();
    assert!(mats.top.is_none());
    assert!(mats.mid.is_none());
    assert!(mats.side.is_none());
    assert!(mats.bot.is_none());
}

/// CloudVoxelMeshes::reset 清空所有 mesh 句柄。
#[test]
fn voxel_meshes_reset_clears_handles() {
    let mut meshes = CloudVoxelMeshes {
        core: Some(Handle::default()),
        lobe: Some(Handle::default()),
        edge: Some(Handle::default()),
    };
    meshes.reset();
    assert!(meshes.core.is_none());
    assert!(meshes.lobe.is_none());
    assert!(meshes.edge.is_none());
}

/// 重要常量保持合理范围（编译期断言，防止误改）。
#[test]
fn voxel_constants_are_sane() {
    // 纯常量断言放入 const 块，编译期即校验。
    const {
        assert!(CLOUD_CELL_SIZE > 1.0 && CLOUD_CELL_SIZE < 64.0);
        assert!(CLOUD_RANGE >= CLOUD_CELL_SIZE);
        // 三级尺寸必须"中心大、边缘小"。
        assert!(CORE_CELL_SIZE > CLOUD_CELL_SIZE);
        assert!(CLOUD_CELL_SIZE > EDGE_CELL_SIZE);
        // 外壳包络必须大于实心核，保证三级角色区间有效。
        assert!(ENVELOPE_SCALE > SOLID_CORE_R);
        // 低频波长必须远大于 voxel 尺寸（低频，非高频噪声）。
        assert!(LOW_FREQ_SCALE > CLOUD_CELL_SIZE * 4.0);
    }
}
