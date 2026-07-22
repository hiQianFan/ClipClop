# ClipClop 开源就绪审计

更新日期：2026-07-22
范围：仓库文档、工程配置、CI、构建与可观察实现；未执行第三方安全审计或双平台完整实机验收。

## 结论

ClipClop 的代码质量与公共仓库文件已基本满足“公开源码”的条件。公开源码不要求 Apple Developer ID 或 Windows Authenticode，也不要求公开任何私钥。当前仍需维护者在 GitHub 上完成私密安全渠道、默认分支保护和仓库可见性确认。

项目也可以发布明确标注为开发预览的未签名安装包。未签名安装包不是稳定发行：用户会看到操作系统警告，Windows 实机证据仍缺，发布前必须检查 Draft Release 产物。签名、公证及双平台完整验收属于“签名稳定版”的更高门槛。

## 三种发布状态

| 状态 | 当前就绪度 | 必要条件 |
| --- | --- | --- |
| 公开源码 | 基本就绪 | 双语公共入口、MIT License、安全报告渠道、无 secrets、CI、分支保护及维护者确认 Public |
| 未签名 Preview 安装包 | 工作流已就绪，平台验收未完成 | updater 私钥仅存 GitHub Secret、版本与质量门通过、Draft 产物检查、Release 安装提示、各平台冒烟证据 |
| 签名稳定版 | 未就绪 | 上述全部条件，加 Apple Developer ID/公证、Windows Authenticode、完整安装/升级/卸载验收及稳定支持承诺 |

## 已完成的仓库基线

- 英文默认 README 与完整简体中文版本，统一产品名和中英文口号。
- 双语贡献指南、安全策略、行为准则与互相可达的文档索引。
- `pnpm test`、`pnpm check`、前端构建、Rust fmt/Clippy/test 均进入质量与发行工作流。
- Bug、Feature 和 Pull Request 模板要求使用虚构数据，并将漏洞引导至私密渠道。
- Dependabot 覆盖 npm、Cargo 与 GitHub Actions。
- Release workflow 同时校验 tag/手动版本、三处应用版本和全部质量门，只创建 Draft Release。
- updater 公钥可公开跟踪；私钥内容和密码不进入仓库。

## 公开源码前仍需 GitHub 侧完成

- [ ] 启用并实际测试 GitHub Private Vulnerability Reporting；若不用该功能，填写长期可用的受控安全邮箱。
- [ ] 为行为准则配置真实的私密举报联系方式。
- [ ] 确认默认分支为 `main`，启用分支保护并要求 Quality 检查通过。
- [ ] 确认 `production-release` Environment 权限、审批和 updater Secret 配置；只有私钥确实加密时才需要密码 Secret。
- [ ] 维护者检查仓库历史和待提交 diff 中不存在凭据、个人路径或真实剪贴板数据后，再批准切换为 Public。

## 未签名 Preview 发布前仍需完成

- [ ] 在 macOS 与 Windows 干净环境运行安装、启动、剪贴板、更新和卸载冒烟测试，并保存不含敏感数据的证据。
- [ ] 检查 Draft Release 中 DMG、EXE、updater 包、`.sig` 与 `latest.json` 完整且版本一致。
- [ ] 记录产物 SHA-256，并从公开下载入口重新校验。
- [ ] 按 [发布清单](release-checklist.md)填写已知问题；不要声称 Apple/Windows 发布者签名或公证已经完成。

## 后续增强

- 增加 `CODEOWNERS`、维护者/支持范围和路线图；当前尚未确认可公开的维护者信息。
- 生成第三方依赖许可证清单并确认图标、字体、原型素材的再分发权利。
- 补充桌面端到端测试、Windows 实机覆盖、性能基线与威胁模型。
- 在稳定运营阶段考虑 SBOM、可复现构建、依赖签名验证与 SLSA provenance。
