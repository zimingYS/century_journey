use cj_core::math::{lerp, smoothstep};
use cj_core::pos::{SectionLocalPos, SectionPos};
use cj_core::rng::WorldRng;
use cj_core::spec::{CHUNK_SIZE, SEA_LEVEL, SECTION_SIZE, SECTION_VOLUME};
use cj_core::voxel::BlockId;

/// 基础噪声的空间缩放系数。
/// 数值越小，地形起伏越平缓、尺度越大
const NOISE_SCALE: f32 = 1.0 / 64.0;

/// 分形噪声的叠加层数
const OCTAVES: u32 = 4;

/// 每层噪声频率的增长倍率
const LACUNARITY: f32 = 2.0;

/// 每层噪声振幅的衰减倍率
const GAIN: f32 = 0.5;

/// 地形高度变化的最大振幅
const AMPLITUDE: f32 = 24.0;

/// 地形高度相对于基准高度允许偏移的整数范围
const HEIGHT_CLAMP_SPAN: i32 = 40;

/// 地表土壤层的默认厚度（方块数）
const SOIL_DEPTH: i32 = 3;

/// 地形高度噪声使用的随机种子盐值。
/// 用于将地形高度生成与其他随机过程的种子空间隔离
const TERRAIN_HEIGHT_SALT: u64 = 0x1001;

// 实现 2D Value Noise
#[inline]
fn value_noise_2d(rng: &WorldRng, salt: u64, x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;

    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let tx = x - x0 as f32;
    let tz = z - z0 as f32;

    let sx = smoothstep(0.0, 1.0, tx);
    let sz = smoothstep(0.0, 1.0, tz);

    let v00 = rng.unit2(salt, x0, z0);
    let v10 = rng.unit2(salt, x1, z0);
    let v01 = rng.unit2(salt, x0, z1);
    let v11 = rng.unit2(salt, x1, z1);

    let x0_value = lerp(v00, v10, sx);
    let x1_value = lerp(v01, v11, sx);

    lerp(x0_value, x1_value, sz)
}

/// 实现 2D fBm 噪声
#[inline]
fn fbm_2d(rng: &WorldRng, salt: u64, x: f32, z: f32, octaves: u32) -> f32 {
    let mut frequency = 1.0;
    let mut amplitude = 1.0;

    let mut value = 0.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        let octave_salt = salt.wrapping_add(octave as u64);

        value += value_noise_2d(rng, octave_salt, x * frequency, z * frequency) * amplitude;

        amplitude_sum += amplitude;

        frequency *= LACUNARITY;
        amplitude *= GAIN;
    }

    if amplitude_sum == 0.0 {
        0.0
    } else {
        value / amplitude_sum
    }
}

// ---- 高度 ----
/// 根据世界坐标生成地形表面高度。
#[inline]
fn terrain_height(rng: &WorldRng, x: i32, z: i32) -> i32 {
    let noise = fbm_2d(
        rng,
        TERRAIN_HEIGHT_SALT,
        x as f32 * NOISE_SCALE,
        z as f32 * NOISE_SCALE,
        OCTAVES,
    );

    let offset = ((noise - 0.5) * 2.0 * AMPLITUDE).round() as i32;

    (SEA_LEVEL + offset).clamp(SEA_LEVEL - HEIGHT_CLAMP_SPAN, SEA_LEVEL + HEIGHT_CLAMP_SPAN)
}

// ---- 判定 ----
/// 根据地表高度和世界 Y 坐标确定方块类型。
#[inline]
fn block_at_column(surface_y: i32, y: i32) -> BlockId {
    if y > surface_y && y <= SEA_LEVEL {
        BlockId::WATER
    } else if y > surface_y {
        BlockId::AIR
    } else if y == surface_y {
        if surface_y < SEA_LEVEL {
            BlockId::SAND
        } else {
            BlockId::GRASS
        }
    } else if y >= surface_y - SOIL_DEPTH {
        BlockId::DIRT
    } else {
        BlockId::STONE
    }
}

// ---- 填充 ----
/// 判断当前 Section 是否可以使用单一方块填充
fn section_uniform_block(rng: &WorldRng, section: SectionPos) -> Option<BlockId> {
    let y_min = section.block_y_start();
    let y_max = y_min + SECTION_SIZE - 1;

    // 计算当前section的16x16个柱子的地表高度范围
    let mut min_surface = i32::MAX;
    let mut max_surface = i32::MIN;

    for z in 0..SECTION_SIZE {
        for x in 0..SECTION_SIZE {
            let world_x = section.chunk.x * CHUNK_SIZE + x;
            let world_z = section.chunk.z * CHUNK_SIZE + z;
            let surface_y = terrain_height(rng, world_x, world_z);

            min_surface = min_surface.min(surface_y);
            max_surface = max_surface.max(surface_y);
        }
    }

    // 整层高于所有地表，同时高于海平面 → AIR
    if y_min > max_surface && y_min > SEA_LEVEL {
        return Some(BlockId::AIR);
    }

    // 整层高于所有地表，同时完全位于海平面以下 → WATER
    if y_min > max_surface && y_max <= SEA_LEVEL {
        return Some(BlockId::WATER);
    }

    // 整层低于所有地表的土壤底面 → STONE
    if y_max < min_surface - SOIL_DEPTH {
        return Some(BlockId::STONE);
    }

    None
}

/// 逐格填充对应方块
fn fill_section_per_cell(rng: &WorldRng, section: SectionPos, out: &mut [BlockId; SECTION_VOLUME]) {
    for z in 0..SECTION_SIZE {
        for x in 0..SECTION_SIZE {
            let world_x = section.chunk.x * CHUNK_SIZE + x;
            let world_z = section.chunk.z * CHUNK_SIZE + z;

            // 高度只依赖 (x, z)，一列只计算一次。
            let surface_y = terrain_height(rng, world_x, world_z);

            for y in 0..SECTION_SIZE {
                let world_y = section.block_y_start() + y;

                let local = SectionLocalPos {
                    x: x as u8,
                    y: y as u8,
                    z: z as u8,
                };
                let index = local.index();

                out[index] = block_at_column(surface_y, world_y);
            }
        }
    }
}
pub fn fill_section(rng: &WorldRng, section: SectionPos, out: &mut [BlockId; SECTION_VOLUME]) {
    if let Some(block) = section_uniform_block(rng, section) {
        out.fill(block);
        return;
    }

    fill_section_per_cell(rng, section, out);
}
