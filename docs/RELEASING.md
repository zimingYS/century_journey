# 版本与发布流程

## 版本号策略

| 项 | 约定 |
|---|---|
| 版本源 | `Cargo.toml` 的 `[workspace.package] version`（当前 **0.1.0**）；各 crate 继承 workspace 版本 |
| 方案 | **SemVer（pre-1.0 语义）**：`0.x.y` 阶段 **minor 可以含破坏性变更**，patch 只含修复；1.0 后严格 SemVer |
| 标签 | `vX.Y.Z`（Release Drafter 的 `tag-template` 自动对应） |
| 依赖 | `Cargo.lock` 入库（应用项目惯例），CI 一律带 `--locked` |

版本号代表**发布物**，不代表设计文档版本（DESIGN.md 有自己的版本线，互不绑定）。

## 发布流程

日常变更走标准 PR 流（CI 绿 → 合并），发布是低频批操作：

```
① 冻结变更        确认 main 上待发布内容（Release Drafter 草稿即清单）
② 提升版本号      改 workspace Cargo.toml 的 version → cargo check --locked 确认 Cargo.lock 同步
③ 提交并打标签    git tag vX.Y.Z && git push origin vX.Y.Z
④ 发布            GitHub → Releases → 把 Release Drafter 的草稿补上 tag 发布
⑤ 自动发包        tag 推送触发 Release Build：三平台构建并把产物挂到该 Release（约几分钟）
```

**发布产物**（`.github/workflows/release.yml`，tag `v*` 触发）：

| 产物 | 平台 |
|---|---|
| `century_journey-windows-x86_64.zip` | Windows x86_64 |
| `century_journey-linux-x86_64.tar.gz` | Linux x86_64 |
| `century_journey-macos-aarch64.tar.gz` | macOS Apple Silicon |

每个包含：游戏二进制 + README + LICENSE。**不发布 crates.io**——PolyForm Noncommercial 非 OSI 许可，不允许；唯一分发渠道是 GitHub Releases。

> 顺序说明：发布草稿会创建 tag 并触发构建，产物会在 Release 公开后几分钟内追加上去；也可以先推 tag（产物先就位）再发布草稿，两种都成立。

**自动化现状**：

| 工具 | 作用 | 触发 |
|---|---|---|
| Release Drafter | 按 PR 标签自动归类、维护「下个版本」草稿（name/tag 模板 + semver-increment） | 每次 push main |
| Release Build | 三平台 release 构建，产物自动挂到 GitHub Release | 推送 `v*` 标签 |
| labeler | 按改动路径自动打 `area: *` 标签（草稿归类的输入） | 每个 PR |
| dependency-review | PR 依赖审查（新增依赖的漏洞/许可证检查） | 每个 PR |
| rustsec audit | `Cargo.lock` 已知漏洞审计 | lock 变更 / 每周 |

## PR 标签 → 版本增量

Release Drafter 依标签决定版本增量（配置见 `.github/release-drafter.yml`）：

| 标签 | 增量 |
|---|---|
| `feature` | minor |
| `bug` / `type: perf` / `type: refactor` / `type: tech-debt` | patch |
| `area: *` / `design` / `community` | 只归类，不决定增量 |

发版前人工确认：草稿里「新功能」多于零 ⇒ 至少 minor。

## 发布检查单

- [ ] `cargo fmt --all -- --check` / `clippy -D warnings --locked` / `test --workspace --locked` 全绿（CI 已强制）
- [ ] 架构边界测试（`layers.rs`）通过
- [ ] 版本号已提升且 `Cargo.lock` 同步
- [ ] Release 草稿的分类与增量正确（feature ⇒ minor）
- [ ] 存档兼容性：`generator_version` 是否需要 +1（世界生成逻辑变更时必须评估；破坏旧存档时在 Release Notes 顶部注明）
