# Content Format

Century Journey content definitions use JSON files below assets/definitions.
Every definition file must declare the current top-level version:

    {
      "format_version": 1
    }

The current version is 1. Missing, older, and newer versions are rejected.
Changing this number requires an explicit loader or migration change.

## Directory Layout and Namespaces

定义文件的目录层级必须与标识符一一对应，统一使用
`assets/definitions/<类别>/<namespace>/<name>.json`：

    definitions/blocks/century_journey/stone.json
    definitions/items/century_journey/stone_axe.json
    definitions/loot/blocks/century_journey/grass.json
    definitions/biomes/century_journey/forest.json
    definitions/recipes/century_journey/stone_axe.json
    definitions/tags/biome/century_journey/cold.json
    definitions/tree_species/century_journey/oak.json

规则：

- 每个定义必须位于自己的命名空间子目录（如 `century_journey/`），
  禁止把文件直接放在类别根目录（旧式扁平布局已被移除）。
- 路径中的 namespace 必须与文件内容中的 `identifier` 前缀一致；
  从路径推导标识符的类别（配方、掉落表、标签）不再提供扁平兜底。
- 纹理、模型等引用路径不参与命名空间目录约定，仍按资源类别扁平存放
  （如 `textures/blocks/stone.png`），通过 JSON 字段显式引用。
- 未来 Mod 通过新增内容根 + 独立命名空间目录隔离定义，避免同路径冲突。

## Validation

Run the content validator without launching the client:

    cargo run --locked -- --check-content assets

The command checks JSON parsing, format_version, duplicate identifiers,
textures, recipe inputs and outputs, biome block references, loot entries, and
tag members. It exits with a non-zero status when any error is found and runs as
an independent CI job.

## Override Order

The built-in asset directory has the lowest content priority. Additional
definition roots can be supplied through the platform path-list environment
variable CJ_CONTENT_OVERRIDES.

    $env:CJ_CONTENT_OVERRIDES = "packs\base;packs\local"

Sources are evaluated in this order:

1. CJ_ASSET_ROOT, or assets when it is unset.
2. Each CJ_CONTENT_OVERRIDES entry from left to right.
3. A later source replaces an earlier JSON file only when both use the exact
   same relative path below definitions.

Files with different paths accumulate. Defining the same identifier from two
different paths is an error, because that would make ownership ambiguous.

This contract currently covers JSON content definitions only. Texture, model,
script, binary compatibility, load hooks, and runtime code extension are not a
stable Mod API. The environment variable names and Rust traits may change
before that API is explicitly versioned.

## Tree Species

树种定义放在 `assets/definitions/tree_species/`，由 Content 层统一编译和校验。每份树种文件必须声明：

- `identifier`、`display_name`；
- `sapling_block`、`trunk_block`、`leaves_block`；
- `growth` 对象；
- 成熟阶段的 `blueprint.trunk_height` 与 `blueprint.crown_radius`，每个范围都包含 `min`、`max`。

`growth` 使用游戏分钟描述生命周期，不依赖真实时间或渲染帧：

- `sapling_duration_game_minutes`：树苗成为幼树前至少经过的时间，缺失时默认为 `1440`；
- `young_duration_game_minutes`：幼树成为成熟树前至少经过的时间，缺失时默认为 `4320`；
- `retry_interval_game_minutes`：空间受阻或相关区块尚未加载时的再次检查间隔，缺失时默认为 `5`。

三个时长显式声明时必须大于零。旧字段 `attempt_interval_game_minutes` 仅作为
`retry_interval_game_minutes` 的读取别名，便于旧内容继续加载；`chance_per_attempt` 已删除，即使旧文件仍包含该未知字段，
它也不会参与生长判定，新内容应移除它。

`young_blueprint` 是可选的幼树树形，格式与成熟 `blueprint` 相同。缺失时 Game 层会根据成熟树形确定性派生较小树形。
树干高度必须满足 `1 <= min <= max <= 64`，树冠半径必须满足 `1 <= min <= max <= 16`；幼树和成熟树形都会执行相同校验。

树苗可放置和继续生长的地面约束来自树苗方块的 `placement.required_support_tag`，树种文件不重复声明支撑标签。
所有方块引用必须存在，树苗不能同时映射到多个树种，空气不能作为树苗、树干或树叶。

## Dynamic Clouds

动态云定义放在 `assets/definitions/clouds/<namespace>/<name>.json`。每份定义必须声明：

- `identifier`：稳定云场标识，用于以后由世界或天气配置选择云场；
- `density` 与 `seed`：程序纹理的覆盖阈值和确定性种子；
- `layers`：按高度从低到高排列的 1 至 8 个表现层；
- 可选 `patches`：相机附近的 billboard 云片，默认关闭。

每层的 `size` 同时是云平面边长和纹理世界重复周期，范围为 32 至 4096。`speed`
使用世界单位/秒，`wind_direction` 是水平 X/Z 向量，运行时只负责归一化，不允许零向量。
云层高度、尺寸、速度、色调、不透明度和云片数量都会经过内容校验；运行时不再用常量覆盖
这些定义值。

云层属于 Client 表现。`CloudWeatherState` 仅提供云量、风速倍率和能见度的有限范围输入；
当前没有权威天气模拟。以后 Game 层接入粗粒度天气单元时，应通过跨层适配器更新该表现状态，
不能让云实体、材质或渲染帧时间反向决定降雨、温度、存档或其他世界规则。
