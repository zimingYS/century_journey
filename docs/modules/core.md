# cj-core — 基础类型层

**职责**：零依赖的基础类型与常量——体素、坐标、世界常量、游戏时间、确定性 RNG、插值、方块 id、观测采样引用。整个 workspace 的公共词汇表。

**依赖纪律**：不依赖任何内部 crate 与 bevy（边界测试 `core_stays_engine_free` 强制）。

## 模块清单

| 模块 | 内容 |
|---|---|
| `voxel` | `Voxel` 三段结构（block u16 + shape + state，共 4 字节）；`Voxel::AIR` |
| `pos` | `BlockPos` / `ChunkPos` / `SectionPos` / `SectionLocalPos` / `ColumnPos`（字段全部公开） |
| `spec` | 世界常量：`SECTION_SIZE=16`、`SECTION_VOLUME=4096`、`SECTION_COUNT_USIZE=48`、`WORLD_BOTTOM=-256` / `WORLD_TOP=512`、`SEA_LEVEL=128` |
| `time` | 游戏历法：节气、季节、游戏日/年换算 |
| `rng` | `WorldRng`（确定性哈希 RNG；`unit2` 等采样；子系统 salt 用 `WORLD_SEED_NAMESPACE` 隔离） |
| `math` | `lerp` / `smoothstep` 等插值原语（value noise / fBm 的积木） |
| `block` | `BlockId`：`AIR=0` + `STONE/DIRT/GRASS/SAND/WATER` 常量；`is_air()` / `occludes()` |
| `field_ref` | `FieldRef` / `SampleValue`（观测采样的统一引用，供 sim 产出与工具消费） |

## API 契约要点

- **`BlockId`**：u16 运行时索引，**不进存档**（存档存 id 字符串，由内容注册表映射）；`const fn` 实现走 `raw() == 0` 比较（`PartialEq::eq` 非 const）。
- **`occludes` vs 碰撞判定**：`occludes` 回答「是否遮挡相邻方块的面」（面剔除用），不是「是否挡人」。P0 水是不透明占位 ⇒ `occludes = true` 但应可穿过——两者绝不能复用。
- **`SectionLocalPos::index()`**：局部格子 ↔ 线性索引的唯一换算入口。sim 写缓冲与 `world::Section::set_raw` 必须**都**用它（跨 crate 硬契约，见 ARCHITECTURE §5）。

## 状态与测试

✅ 全量实现，**52 测试**。`BlockId::AIR.raw() == 0` 有编译期断言锁「零值即空气」。

## 易错点

- `SECTION_VOLUME`（4096，一个 section 的格子数）与 `SECTION_COUNT_USIZE`（48，一个 chunk 的 section 数）名字相近、用途完全不同，极易误用。
- `const fn` 里不能写 `a == b`；比较用 `raw()` / `match`。
- 坐标类型字段保持公开（项目约定：不加重构访问器）；只有「误改静默毁档」的字段才私有（如 `WorldSeed::generator_version`）。
