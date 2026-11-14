# cj-runtime — 模拟侧运行时

**职责**：模拟侧**唯一接触 ECS** 的 crate。提供世界数据门面 `WorldData`（读写入口）、启动生成装配、玩家改动与脏区跟踪。上层（game / app）通过它访问世界，不直接拿 `&mut` 存储。

**依赖纪律**：依赖 `core` + `sim` + `world` + bevy；禁 render / game（边界测试强制）。

## 模块清单

| 模块 | 内容 |
|---|---|
| `world_data` | `WorldData`：种子、区块存储、`PlayerDelta` 归属、脏 section 集合、生成入口 |
| `plugin` | `RuntimePlugin`（§6.7 频率分层的接缝，当前聚合用） |

## API 契约要点

**读路径**（宽容）：

| 方法 | 语义 |
|---|---|
| `block_at(pos)` | **Delta 优先**读取：先查 `PlayerDelta`，无改动再读 `ChunkStore`；越界返回空气 |
| `chunks()` | 只读访问存储（内部用途/渲染快照抽取） |

**写路径**（严格、受控单入口）：

| 方法 | 语义 |
|---|---|
| `place_block(pos, id)` | 写 `PlayerDelta` + 标记脏 section（**自身 + 6 邻居**，共 7 个——面剔除要读邻区一圈 padding） |
| `remove_block(pos)` | 同上（记录为空气的改动） |
| `take_dirty_sections()` | 取走并清空脏 section 快照（app 每帧转发给 render） |

> **设计红线**：玩家的方块改动**必须**走 `place_block` / `remove_block`，不得直接改 `ChunkStore`——Delta 是权威记录，直接改存储会让「种子 + Delta」模型失效。

**生成装配**（P0-c）：

- 初始加载半径 8 chunk（17×17 = 289）；生成流程：`WorldSeed.rng()` → sim 纯数据 → 按位置写入 `ChunkStore`。
- **整段全空气的 section 不分配**（保持 `None ≠ Some(AIR)` 语义）。
- 实测基线：289 chunk / 28,776,128 非空气格 / **364-430 ms**（列高度缓存的 48 倍冗余已评估为不值得优化；重新评估触发条件：半径 16 或流式加载）。

## 状态与测试

写入路径 ✅（P0-4a）；脏 section 跟踪 ✅（P0-4a）；**17 测试**。

## 易错点

- 标脏要含 6 邻居——只标自身会导致邻居边缘面剔除错误（黑边/漏面）。
- scratch 缓冲复用时必须**写满全部格子**——「上次剩下的值」是典型静默缺陷。
- `take_dirty_sections` 语义是「取走即清空」，消费方不得重复消费。
