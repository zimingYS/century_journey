# 项目治理

本文件说明 CenturyJourney 的决策方式、权限结构与合规约定。

## 维护者

项目目前由**单一维护者**（[@zimingYS](https://github.com/zimingYS)）治理，负责：

- 设计决策拍板（含参数与未决问题）
- PR 审核与合并
- 版本发布（见 [docs/RELEASING.md](docs/RELEASING.md)）
- 社区行为准则执行

## 决策流程

核心设计（世界模拟、玩法结构、架构）**已冻结**。在此前提下：

| 变更类型 | 流程 |
|---|---|
| 机制 / 系统 / 架构级变更 | **提案 → 讨论 → 拍板 → 实施**：用 [设计提案](.github/ISSUE_TEMPLATE/design_proposal.md) Issue 模板开题，维护者拍板后才动手 |
| 小改动（bug 修复、文档、工具链、内容修正） | 直接提 PR，CI 绿 + 维护者审核合并 |
| 依赖升级 | dependabot 自动开 PR（minor/patch 合并组），major 单独审 |

**决策优先级**（冲突时从上往下裁）：

1. 核心幻想：玩家必须读懂地质与气候规律，才能长期定居
2. 系统自洽与长期可维护（架构纪律优先于局部方便）
3. 实现成本与工期

## 分支与合并

- `main` 是唯一长期分支，受 CI 门禁保护：fmt + clippy（`-D warnings`）+ 全量测试 + 架构边界测试全部通过才能合并。
- 依赖方向等架构纪律由边界测试机器执行，不依赖人工 review 记忆（见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)）。
- 审查归属由 [CODEOWNERS](.github/CODEOWNERS) 声明。

## 行为准则

参与社区（Issue / PR / 讨论）即同意遵守 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。执行责任在维护者。

## 许可与贡献授权

- 代码采用 **PolyForm Noncommercial 1.0.0**（可学习、修改、非商业使用）；提交贡献即同意以**相同许可**授权你的贡献。
- 资产（美术 / 音频 / 文案）保留所有权利，详见 LICENSE 附录。
- 因许可为非商业，本项目**不发布到 crates.io**（其要求 OSI 许可）；分发渠道是 GitHub Releases。
- 商业授权咨询：开 Issue 联系维护者。

## 治理变更

本文件的修改本身就是 PR，由维护者审核合并；重大治理变更（如引入共同维护者、变更许可）会在 Release Notes 中公告。
