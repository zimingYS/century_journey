# cj-app — 启动与组装

**职责**：唯一入口。启动装配（生成、快照、光照）、插件按层组装、启动配置。**不放业务逻辑**——`main.rs` 出现业务系统 = 组装逻辑泄漏到入口。

**依赖纪律**：依赖全部 crate + bevy。只引用别人的 **Plugin 类型**，不直接调别人的内部系统。

## 模块清单

| 模块 | 内容 |
|---|---|
| `main.rs` | 一行启动：`App::new().add_plugins(plugin::AppPlugin).run()` |
| `plugin.rs` | `AppPlugin` 按层装配（顺序见下） |
| `bootstrap.rs` | 启动装配：生成地形、构建快照、`fill_snapshot` 抽取、`forward_dirty_sections` 每帧中转 |
| `settings.rs` | 启动配置：`DEFAULT_WORLD_SEED` / `INITIAL_LOAD_RADIUS_CHUNKS` 等（启动参数集中于此，领域默认值留各 crate） |
| `tests/layers.rs` | **架构边界测试（7 条）**，读源码做文本断言 |

## 插件装配顺序（「谁能用谁」的事实来源）

```
DefaultPlugins → RuntimePlugin → RenderPlugin → GamePlugin → BootstrapPlugin
```

各顶层 plugin 只做聚合；子 plugin 按层挂载（详见 ARCHITECTURE §3）。

## 启动装配契约

- **生成 → 快照链**：`generate_initial_area`（289 chunk，半径 8）→ `build_section_snapshots`（18³ 快照）→ 首次全量建网格。两步在同一 crate 内 `.chain()` 保证顺序。
- **`fill_snapshot`**：从 `WorldData` 抽取 18³ 快照的唯一实现——**必须走 `world.block_at`（Delta 优先）**，不得绕过 Delta 直读存储（曾有本地 `block_at` 助手绕过 Delta，已删除；这是「派生缓存必须服从权威模型」的实例）。
- **`forward_dirty_sections`**：每帧把 `WorldData::take_dirty_sections()` 的结果转成快照发给 render（render 读不到 world，app 是唯一中转）。

## 架构边界测试（7 条）

| 断言 | 防的是什么 |
|---|---|
| core 无 `bevy::` | 纯数据层被污染 |
| sim 无 `bevy::` / `cj_world` / `cj_runtime` | sim 的硬边界 |
| world 无 `bevy::` / `cj_sim` / `cj_runtime` | 纯数据层被污染 |
| render 无 `cj_sim` / `cj_runtime` | render 的硬边界 |
| runtime 无 `cj_render` / `cj_game` | 下层不知上层 |
| `render/src/{input,meshing}.rs` 无 `bevy::` | L0/L1 失去可单测性 |
| sim 七域互不 `crate::<域>` 引用 | 领域间只能通过「场」交换 |

> 这类「读源码做文本断言」的测试很土但非常有效——拦的是「某天顺手 import 一下」这种慢慢腐蚀架构的行为。

## 状态与测试

✅ 启动链路全通（生成 347ms / 快照 625 节 101ms / 网格 50 节 102ms，实测）；边界测试 7 + bin 测试 1。

## 易错点

- `main.rs` 不出现业务系统；启动常量归 `settings.rs` 不归领域 crate。
- 快照构建对世界外 y 要做 `WORLD_BOTTOM/TOP` 守卫（`Chunk::get_block` 越界会 panic，全局化前先守卫）。
