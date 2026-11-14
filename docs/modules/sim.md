# cj-sim — 纯模拟层

**职责**：世界生成与各领域模拟算子。**纯数据进出**：不接触 bevy、不接触存储，输入种子/RNG，输出缓冲区数据。所有算法在此层可无 GPU 单测。

**依赖纪律**：只依赖 `core`；禁 bevy / world / runtime / render（边界测试强制）。生成器**不能返回 `cj-world` 类型、不能收 `WorldSeed`**——只收 `u64` / `WorldRng`，输出 `&mut [BlockId]` 等纯数据，由 runtime 落盘。

## 领域划分（7 域 + worldgen）

| 域 | 状态 | 计划阶段 | 内容 |
|---|---|---|---|
| `worldgen` | ✅ 已实现 | P0 | 噪声 → 高度场 → 五方块地形（石/土/草/沙/水） |
| `field` | 占位 | P4 | 场容器与采样 |
| `op` | 占位 | P4/P6 | 纯算子（扩散、平流、侵蚀） |
| `geo` | 占位 | P3/P6 | 运行时地质 |
| `hydro` | 占位 | P4 | 水文 |
| `climate` | 占位 | P3/P4 | 气候 |
| `eco` | 占位 | P5 | 生态演替 |
| `event` | 占位 | P4/P6 | 灾害事件 |

**域间纪律**：领域之间**不互相调用**，跨域只通过「场」交换数据（边界测试 `sim_domains_do_not_reference_each_other` 强制）。空文件是占位声明，**不提前填**——提前填会引入无调用方的死代码。

## worldgen 契约（P0 已定）

- **噪声选型**：不引外部 noise crate——`WorldRng::unit2` 网格哈希 + `core::math::{lerp, smoothstep}` 插值得 value noise / fBm。
- **输出**：按 section 填充纯数据缓冲（`&mut [BlockId]`）；**格子写入顺序必须用 `core::SectionLocalPos::index()`**（跨 crate 硬契约）。
- **确定性**：同种子同参数 ⇒ 同地形（有测试锁）。
- **沙滩带**：`BEACH_RISE = 3`，水面附近（`surface_y ≤ SEA_LEVEL + BEACH_RISE`）出沙。
- **早退**：48 个 section 中约 44 个走平凡路径（全空气/全石头），只对边界 section 逐格计算；`fill_section_per_cell` 作为对照实现（oracle）保留。

## 状态与测试

worldgen ✅（**5 测试**：沙滩带 4 条 + 水体填充 1 条）；其余 7 域占位。运行时性能基线：289 chunk 生成 **364-430 ms**（实测，2026-09-15）。

## 易错点

- **oracle 与实现同源 ⇒ 等价性测试失效**：对照实现本身带 bug 时两边同错同对，必须补独立断言（读回校验）。
- 插值组合要枚举四个角点；`smoothstep` 只作用于小数部分。
- 禁止「跳写空气格」——要与 oracle 的写入集合完全一致。
- 循环边界用 `SECTION_SIZE`（i32）时变量也是 i32；出现 `as i32` 就是在手推换算的信号。
