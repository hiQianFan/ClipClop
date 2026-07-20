# 发布清单

## 版本与范围

- [ ] 冻结范围，清空阻塞发布的 issue；更新 `CHANGELOG.md`。
- [ ] `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 版本一致。
- [ ] 目标 `vX.Y.Z` tag 与上述三处版本一致；发布 workflow 的版本校验步骤通过。
- [ ] README 的状态、能力和已知限制与代码一致。
- [ ] 新权限、联网行为和数据处理已完成安全/隐私审查。

## 质量与平台

- [ ] `docs/testing.md` 的自动检查全部通过。
- [ ] macOS 与 Windows 实机完成桌面冒烟测试并保存证据。
- [ ] 数据库从上一公开版本升级成功；新安装与清理卸载路径验证完成。
- [ ] 无障碍、键盘、Light/Dark、不同 DPI/工作区完成验收。

## 产物

- [ ] macOS Universal `.dmg`、`.app.tar.gz` 与 updater `.sig` 完整；若未使用 Developer ID/公证，Release 明确标注首次打开方式。
- [ ] Windows x64 NSIS `.exe` 与 updater `.sig` 完整；若未使用 Authenticode，Release 明确标注未知发布者/SmartScreen 风险。
- [ ] Draft 阶段检查 macOS、Windows 安装包、updater `.sig` 和 `latest.json`，随后发布为普通 Release（不能标记为 prerelease）。
- [ ] Release 发布后立即从上一公开版本分别完成 macOS 和 Windows 自动更新，验证 updater 签名、安装、重启以及历史/设置保留；失败时撤回 Release。
- [ ] 记录产物文件名、架构、大小和 SHA-256；验证 CI 产物来自目标 tag。
- [ ] 确认 bundle 标识、图标、许可证和第三方许可证材料完整。

## 开源治理

- [ ] `SECURITY.md` 已配置真实私密报告渠道。
- [ ] 仓库 issue/PR 模板、维护者/所有者和支持范围已明确。
- [ ] 分支保护、必需检查、最小权限和依赖更新策略已启用。
- [ ] 发布说明区分已验证能力、实验能力与已知问题。

## 发布后

- [ ] 从公开下载入口重新下载并校验哈希、签名、安装和启动。
- [ ] 创建对应 GitHub Release，链接变更日志与隐私说明。
- [ ] 监控安装/启动问题和安全报告渠道；必要时准备撤回或补丁发布。
