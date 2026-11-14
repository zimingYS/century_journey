# cj-render — 渲染层

**职责**：把方块数据变成画面——快照输入、网格构建、bevy Mesh 生成、增量重建、光照。**读不到世界**（禁 sim/runtime），方块数据只能以纯数据快照从 app 侧注入。

**依赖纪律**：依赖 `core` + bevy；禁 sim / runtime / game（边界测试强制）。**L0 `input` / L1 `meshing` 不得 import bevy**（纯算法可无 GPU 单测，边界测试强制）。

## 模块清单与分层

| 层 | 模块 | 内容 |
|---|---|---|
| L0（无 bevy） | `input` | `BlockView` trait（方块输入唯一入口，越界→空气）、`SectionSnapshot`（18³）、`MeshData`（纯数据，可单测） |
| L1（无 bevy） | `meshing` | `build_section_mesh`：面剔除 + 顶点色 + 绕序；P0 每方块单一颜色 |
| L2 | `terrain` | `SectionMeshes` 注册表（section → 实体 + 共享材质）、`rebuild_dirty_sections` 增量重建 |
| L2 | `lighting` | 平行光 + 环境光（`spawn_light`；原 `camera.rs` 已改名） |
| L2 | `resources` | 渲染策略常量（`RENDER_CHUNK_RADIUS = 2`，5×5 chunk） |

## API 契约要点

- **快照粒度**：`SectionSnapshot` 为 **18³**（16³ + 一圈 padding，约 11.7 KB/节）——跨 section 边界的面剔除需要邻区一圈数据。
- **交接方案**：P0 用 **H2 组装根注入**（app 持有快照、建 mesh）→ P1 演进 H1（中立消息 + 通道）。H3（render 直连 world）**不采用**（违反「runtime 拥有世界数据」）。
- **增量重建**：runtime 标脏 → app 每帧 `forward_dirty_sections` 转发（render 读不到 world，这是唯一通道）→ `rebuild_dirty_sections` 重建受影响 section 网格。
- **渲染半径 ≠ 生成半径**：渲染 5×5（可见范围），生成 17×17（已生成范围）——两个独立参数。

## 状态与测试

P0-a/d ✅（朴素网格 + bevy 装配）；4b 增量重建 ✅；**15 测试**（含「16³ 实心 → 1536 面」与「叉积与法线同向」两条关键断言）。

## 易错点

- **绕序（winding order）写反 → 全部被背面剔除 → 画面全黑且不报错**。自检法：对每个面算 `cross(c1−c0, c2−c0)`，应等于该面法线。
- 重建实体必须带 `Transform`（世界坐标原点），否则网格叠在原点。
- 判空要看**网格数据**是否为空，不是 `Assets<Mesh>` 容器是否为空。
- 顶点高度位移（地形平滑方案）会破坏共面性，与贪心合并不兼容——P1 两者都要上时先定让步方。

## 已知遗留

`AmbientLight` 在相机搬去 game 时丢失（搬迁实体要逐一检查所挂组件），P1 补。
