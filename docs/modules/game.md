# cj-game — 玩法层

**职责**：玩家与世界的一切玩法交互——输入采集、目标拾取、玩家移动、玩法域（物品/工艺/农业/观测）、UI。不重新实现底层模拟（模拟算法在 sim/存储在 world，此处只消费）。

**依赖纪律**：依赖 `core` + `runtime` + `render` + bevy；玩法域之间**不互相调用**，只通过 Message / Resource 交换（否则 game 会最先烂掉——它是增长最快的 crate）。

## 五层结构

| 层 | 模块 | 状态 | 内容 |
|---|---|---|---|
| L0 | `input/` | ✅ | `intent.rs`（`PlayerIntent` 意图、`ToggleCursorCapture`、`ToggleMovementMode`）、`cursor.rs`（光标捕获） |
| L1 | `interaction/` | ✅ | `target.rs`（`PlayerTarget` 资源、DDA 射线拾取、`update_player_target` 系统） |
| L2 | `player/` | ✅ | `camera.rs`（第一人称视角）、`motion.rs`（步行/飞行、重力、跳跃）、`collision.rs`（AABB 碰撞） |
| L3 | 玩法域 | ⏳ 未建 | `item` / `crafting` / `farming` / `observation`（P2 起逐个建） |
| L4 | `ui` | ⏳ 未建 | HUD、提示框（P2） |

> **层级规则**：`interaction` 必须在 `player` **之下**——`player/action` 要消费「指着什么」，反向依赖会成环。

## API 契约要点

- **I1 输入 → 意图**：只有 `input` 碰 bevy 输入 API（`ButtonInput` / `AccumulatedMouseMotion`），上层只见 `PlayerIntent`。键位可重绑、将来 AI 走同一入口。
- **I2 玩家 → 世界**：方块改动**必须**写 `WorldData::place_block / remove_block`（落 `PlayerDelta`），不得直接改存储。
- **I3 界面 → 逻辑**：UI 只读状态、只发意图。
- **玩家结构**：Body 实体（AABB 中心）+ 相机子实体（眼睛偏移）；相机旋转 `EulerRot::YXZ`，前向 = `rotation * Vec3::NEG_Z`。
- **移动**：`MovementMode::{Walk, Fly}`。Walk = 重力（-30）+ 碰撞 + 跳跃（10，先于重力累积）+ 平移（5）；Fly = 无视重力（20）。碰撞轴序 **X → Z → Y**（Y 最后防卡墙），按最小半边长分子步防穿墙（上限 32 子步）。
- **拾取**：DDA（Amanatides-Woo）体素遍历，射程 8；**起点格不检测**（相机在方块内不自我选中）。
- **顺序**：`GameSet` 链 `Input → Interaction → Player → Domain → Ui`，`configure_sets` 显式声明（无 configure_sets 的 SystemSet 只是标签）。

## 状态与测试

L0-L2 ✅（飞行相机 / 重力碰撞跳跃 / 射线拾取）；**11 测试**（碰撞 7 + 运动 4）。

## 已知问题（待修）

1. `interaction/target.rs` 反向依赖 `player::collision::world_is_solid`（L1→L2）——`world_is_solid` 应下沉到 interaction 层。
2. `target.rs` 的 `raycast` 是纯函数但零测试（应补 ~5 条：直行命中/距离截断/起点格不自选/最近优先/零方向）。

## 易错点

- 跳跃速度必须在重力累积**之前**应用，且删掉旧的 `GRAVITY * dt` 行——否则重力被加两次（P0 实测踩坑）。
- `desired_velocity` 必须保留垂直分量——删掉它测试会变同义反复（tautological）。
- `GlobalTransform` 在 bevy 0.19+ **没有** `forward()`，前向用 `rotation * Vec3::NEG_Z`。
