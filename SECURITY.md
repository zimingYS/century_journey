# 安全政策

## 报告漏洞

**请不要通过公开 Issue 报告安全漏洞。**&#8203;

请使用 GitHub 的私密漏洞报告：

<https://github.com/zimingYS/century_journey/security/advisories/new>

提交后会私下通知维护者。如果无法使用该功能，可通过 **<你的邮箱>** 联系。

## 响应时间

本项目为个人维护，我会尽力在 **7 天内** 首次回应。

## 范围

### 在范围内

- 内存安全与未定义行为（Rust 侧）
- 存档解析漏洞（恶意存档导致崩溃 / 代码执行）
- **内容加载漏洞**（恶意 mod / `.ron` 文件）
- 依赖链的已知漏洞

### 不在范围内

- 游戏性 bug（请用 [Bug 报告](.github/ISSUE_TEMPLATE/bug_report.md)）
- 单机模式下修改自己的存档
- 已公开的过时依赖（除非有实际利用路径）

## 已有防护

- `.ron` / `.toml` 内容文件在加载时经过校验
- 已启用 Secret scanning 与 Push protection，防止凭据泄漏

感谢你帮助本项目更安全。
