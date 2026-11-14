# cj-world — 存储层

**职责**：世界的持久数据结构——区块存储、玩家改动（Delta）、自然积分量、世界种子。纯数据层，无 bevy、无 ECS。

**依赖纪律**：只依赖 `core`；禁 bevy / sim / runtime / render（边界测试强制）。`delta` 与 `integral` **互不依赖**。

## 模块清单

| 模块 | 内容 |
|---|---|
| `chunk` | `Section`（16³）→ `Chunk`（16×16×48）→ `ChunkStore`（HashMap 分桶） |
| `delta` | `BlockChange` → `ChunkDelta` → `PlayerDelta`（玩家扰动的权威记录） |
| `integral` | `NaturalIntegral` → `ChunkIntegral`（自然积分量：积雪 SWE、潜水面等） |
| `seed` | `WorldSeed`（含 `generator_version`，防存档静默毁档） |

## API 契约要点

- **三级分块**：Region（落盘单元 64）/ Chunk（16×16）/ Section（16³）。**不引入 SectionDelta 内存层**——HashMap per-entry 开销已评估，压缩收益不值。
- **Delta 层次**：`PlayerDelta → HashMap<ChunkPos, ChunkDelta> → HashMap<BlockPos, BlockChange>`。
- **Integral 层次**：桶内 key 是 2D 的 `(ColumnPos, FieldRef)` 元组，**不是** `BlockPos`。
- **两态陷阱（重要）**：`None`（无记录）≠ `Some(值)`（该位置明确无此物）；`Chunk.sections[i] = None` ≠ `Some(Voxel::AIR)`（玩家挖了）。构造与匹配时永远区分这两态。
- **`WorldSeed::rng()`** 混入 `WORLD_SEED_NAMESPACE`——否则世界根种子与子系统 salt 共享位空间。
- **`generator_version` 私有 + 构造入口管控**：字段私有只防写入路径，构造入口（如 `WorldSeed::new(seed, 1)` 字面量）必须一并设计，否则私有化形同虚设。

## 状态与测试

✅ 全量实现，**76 测试**（chunk 28 / delta 19 / integral 21 / seed 8+）。

## 易错点

- `Chunk::get_block` 对世界外 y 会 panic（`section_pos().index()` 的 debug_assert），而 `section_at` 越界返回 `None`——同 crate 两种态度；读取方应先做 `WORLD_BOTTOM/TOP` 守卫（长期方案：全域化 `get_block`，见待办）。
- **读写不对称**：越界读 → 空气；越界写 → 保持断言。
- 跨 crate 索引顺序契约的「读侧」在此（`Section::set_raw`）——必须与 sim 写侧同用 `SectionLocalPos::index()`。
