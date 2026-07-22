# 发布清单

本清单区分公开源码、未签名 Preview 与签名稳定版。公开源码不要求平台代码签名；Tauri updater 私钥、平台签名私钥及密码在任何情况下都不得公开。

## A. 公开源码

- [ ] 英文默认 README 和简体中文 README 内容一致，产品名为 `ClipClop`，中英文口号正确。
- [ ] MIT License、贡献指南、安全策略、行为准则、Issue/PR 模板与 Dependabot 已检查。
- [ ] GitHub Private Vulnerability Reporting 已启用并测试，行为准则私密联系方式可用。
- [ ] 仓库历史和待提交 diff 不含凭据、个人路径、真实剪贴板内容或 updater 私钥。
- [ ] `main` 分支保护、必需 Quality 检查和最小 Actions 权限已配置。
- [ ] 维护者明确批准仓库从 Private 切换为 Public。

## B. 每个版本的版本与质量

- [ ] 冻结范围并更新 `CHANGELOG.md`。
- [ ] `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 版本一致。
- [ ] `vX.Y.Z` tag 或手动 workflow 输入与应用版本一致。
- [ ] `pnpm test && pnpm check && pnpm build` 全部通过。
- [ ] `cargo fmt --check`、Clippy `-D warnings` 与 `cargo test` 全部通过。
- [ ] 新权限、联网行为和数据处理已经完成安全与隐私审查。
- [ ] macOS 与 Windows 实机冒烟结果使用虚构数据记录；未执行的项目明确写入已知限制。

## C. 未签名 Preview 安装包

- [ ] `production-release` Environment 中存在 `TAURI_SIGNING_PRIVATE_KEY`；私钥有密码时才配置 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- [ ] macOS Universal `.dmg`、`.app.tar.gz` 与 updater `.sig` 完整。
- [ ] Windows x64 NSIS `.exe`、updater 包与 `.sig` 完整。
- [ ] `latest.json` 同时包含正确的平台、版本、下载地址和 updater 签名。
- [ ] Workflow 只创建 Draft Release；维护者检查所有产物和自动生成的 Release notes 后再决定是否 Publish。
- [ ] 面向用户的安装说明只在 README 安装段和 Release notes 中说明：macOS 可能需要在“隐私与安全性”中允许，Windows 可能显示“未知发布者”或 SmartScreen。
- [ ] Release 不声称具有 Apple Developer ID、公证或 Windows Authenticode；updater 签名不描述为平台发布者签名。
- [ ] 记录产物文件名、架构、大小和 SHA-256，并确认来自目标 tag/commit。
- [ ] 若存在上一公开版本，Release 发布后分别验证 macOS 与 Windows 自动更新、重启以及历史/设置保留；首个公开版本改为验证干净安装、`latest.json` 与更新包签名。失败时撤回 Release。

## D. 签名稳定版附加条件

- [ ] macOS Developer ID Application 签名、公证与 stapling 全部通过，并从公开下载入口验证 Gatekeeper。
- [ ] Windows Authenticode 发布者签名有效，并在干净 Windows 环境验证安装和卸载。
- [ ] macOS/Windows 新安装、升级、卸载、剪贴板核心路径、无障碍、Light/Dark、不同 DPI/工作区完整验收。
- [ ] 数据库从上一公开版本升级成功，清理卸载路径已验证。
- [ ] 支持版本、维护响应、兼容性和迁移承诺已公开。
- [ ] 第三方许可证材料、图标与其他分发资产权利已经确认。

## 发布后

- [ ] 从公开 Release 重新下载，校验哈希、签名、安装和启动。
- [ ] 检查 `/releases/latest` 与应用 updater 能发现正确版本。
- [ ] 监控安装、启动、更新和安全报告渠道；必要时准备撤回或补丁发布。
