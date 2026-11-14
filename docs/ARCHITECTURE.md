# 系统架构

> CenturyJourney 的 crate 分层、依赖纪律与数据流。模块级契约见 [modules/](modules/)。
> 对应内部设计文档的 §6（架构）与 LAYERS 细则；本文是面向协作者的公开版。

## 一、七个 crate

workspace 由 7 个 crate 组成，依赖方向**自底向上单向**：

```
                    ┌────────────┐
                    │    app     │  启动与组装（唯一 main）
                    └─────┬──────┘
              ┌───────────┼───────────┐
              ▼           ▼           ▼
         ┌────────┐  ┌─────────┐  ┌───────┐
         │  game  │  │ render  │  │runtime│
         └───┬────┘  └────┬────┘  └───┬───┘
             │            │       ┌───┴────┐
             │            │       ▼        ▼
             │            │   ┌──────┐ ┌──────┐
             │            │   │ sim  │ │ world│
             │            │   └───┬──┘ └───┬──┘
             ▼            ▼       ▼        ▼
         ┌────────────────────────────────────┐
         │               core                 │  零依赖
         └────────────────────────────────────┘
```

| crate | 职责 | 允许依赖 | 禁止 |
|---|---|---|---|
| `cj-core` | 基础类型：体素、坐标、常量、RNG、插值、方块 id、采样引用 | 无 | 一切 |
| `cj-sim` | 纯模拟：世界生成与各领域算子 | `core` | bevy / world / runtime / render |
| `cj-world` | 存储层：区块、玩家改动 Delta、自然积分量、世界种子 | `core` | bevy / sim / runtime / render |
| `cj-runtime` | 模拟侧唯一接触 ECS：世界数据门面、生成装配、脏区跟踪 | `core` `sim` `world` bevy | render / game |
| `cj-render` | 渲染：快照 → 网格数据 → bevy Mesh / 光照 | `core` bevy | sim / runtime / game |
| `cj-game` | 玩法：输入 → 交互 → 玩家 → 玩法域 → UI | `core` `runtime` `render` bevy | 重实现底层模拟 |
| `cj-app` | 启动与组装 | 全部 + bevy | 不放业务逻辑 |

**验证手段**：`cargo tree -p cj-sim` 的树里不出现 bevy；`crates/app/tests/layers.rs` 的 7 条架构边界测试读源码做文本断言（每次提交前必跑，CI 强制）。

## 二、crate 内层次（要点）

- **层间单向依赖**：上层可用下层，下层不知道上层。
- **`lib.rs` 只做声明**（模块声明 + re-export），Plugin 与逻辑各归其位。
- **只有依赖 bevy 的 crate 才有插件**（runtime / render / game / app）；core / sim / world 是纯数据层。
- `render` 的 L0 `input` / L1 `meshing` **不得 import bevy**——纯算法不初始化 GPU 也能单测（现有网格测试全靠这条）。
- `game` 五层：L0 `input` → L1 `interaction` → L2 `player` → L3 玩法域 → L4 `ui`；**`interaction` 必须在 `player` 之下**（`player` 要消费「指着什么」）。
- `sim` 领域之间（worldgen / geo / hydro / climate / eco / structure / event）**不互相调用**，只通过「场」交换数据。

## 三、插件组装

`app/src/plugin.rs` 的 `AppPlugin` 按层装配，顺序即「谁能用谁」的事实来源：

```
DefaultPlugins → RuntimePlugin → RenderPlugin → GamePlugin → BootstrapPlugin
```

每个顶层 Plugin **只做聚合**（几行），功能挂在各层的子 plugin 上。`SystemSet` 显式声明执行顺序（bevy 系统默认并行无序，`configure_sets` 才是真顺序）。

## 四、关键数据流

**启动生成链**（P0）：

```
WorldSeed.rng() → cj-sim 世界生成（纯数据：&mut [BlockId]）
  → cj-runtime 写入 ChunkStore → cj-app 生成 SectionSnapshot（18³，含一圈 padding）
  → cj-render 构建 MeshData（纯数据）→ bevy Mesh 显示
```

**玩家改动链**（方块放置/破坏）：

```
玩家输入 → PlayerIntent → 玩家动作
  → WorldData::place_block / remove_block（受控写入：落 PlayerDelta + 标记脏 section）
  → 每帧 forward_dirty_sections 中转快照
  → cj-render::rebuild_dirty_sections 增量重建受影响网格
```

**世界状态权威模型**：权威 = **世界种子 + PlayerDelta**；内存中的方块数据只是二者的展开结果（派生缓存）。自然积分量（积雪、潜水面等）随 Delta 落盘；连续场（温度、水位等）运行时派生、不持久化。存档因此只需「种子 + Delta」。

## 五、跨 crate 硬契约

| 契约 | 内容 | 违反后果 |
|---|---|---|
| **索引顺序** | sim 写缓冲的格子顺序必须与 `world::Section::set_raw` 读取顺序一致，**两侧都用 `core::SectionLocalPos::index()`** | 静默错位（不报错、数据错） |
| **快照粒度** | `SectionSnapshot` 为 18³（16³ + 一圈 padding），供跨 section 边界面剔除 | 边界面丢失/多余 |
| **读写不对称** | 越界**读**返回空气；越界**写**保持断言（是 bug 就要响） | 查询天然越界，写越界必须炸 |
| **零值即空气** | `BlockId::AIR.raw() == 0` 由编译期断言锁死 | 存档/内存默认值语义 |

## 六、强制手段（防腐蚀）

| 级别 | 手段 |
|---|---|
| 编译期 | `Cargo.toml` 依赖表决定谁能依赖谁（`cargo tree` 可验证） |
| 测试期 | `crates/app/tests/layers.rs` 架构边界测试（7 条文本断言） |
| 提交门禁 | CI：`cargo fmt --check` + `clippy -D warnings --locked` + `test --workspace --locked` |
| 依赖安全 | PR 依赖审查（dependency review）+ rustsec 安全审计（见 `.github/workflows/`） |

## 七、当前实现状态（摘要）

| crate | 状态 | 测试 |
|---|---|---|
| core | ✅ 全量 | 52 |
| world | ✅ 全量 | 76 |
| sim | 世界生成已实现；7 个领域域占位（P3-P6） | 5 |
| runtime | 世界数据门面、写入路径、脏区跟踪 | 17 |
| render | 朴素网格渲染 + 增量重建 + 光照 | 15 |
| game | L0-L2（输入/拾取/玩家）；L3 玩法域未建 | 11 |
| app | 启动组装 + 架构边界测试 | 7 + 1 |

模块级细节与 API 契约见 [modules/](modules/) 各篇。
