# ClipClop 文档索引

简体中文 | [English](index.md)

本目录以当前代码和测试为实现状态的事实基线。产品与设计文档描述目标；实现与发布文档描述已经验证的内容。

## 公开项目入口

- [README](../README.zh-CN.md)：产品概览、平台状态、隐私边界、安装与开发。
- [贡献指南](../CONTRIBUTING.zh-CN.md)：贡献流程和质量门。
- [安全策略](../SECURITY.zh-CN.md)：私密漏洞报告渠道和支持范围。
- [行为准则](../CODE_OF_CONDUCT.zh-CN.md)：社区协作与执行方式。
- [变更日志](../CHANGELOG.md)：用户可见的重要变化。

## 使用者与维护者参考

- [隐私说明](privacy.md)：本地保存的数据、控制方式与限制。
- [构建与分发](distribution.md)：macOS/Windows 构建和 CI。
- [故障排查](troubleshooting.md)：常见开发和运行问题。
- [测试指南](testing.md)：自动检查、手工冒烟和平台验收。
- [发布清单](release-checklist.md)：源码公开、未签名预览和签名稳定版的发布门槛。
- [开源就绪审计](open-source-readiness.md)：当前缺口、优先级和完成定义。
- [技术架构](architecture.md)：模块边界、存储、IPC、安全与测试策略。

## 当前事实

- 版本：`0.1.0` 开发预览。
- 平台：macOS 本地构建已验证；Windows CI 已配置，实机验收待完成。
- 数据：SQLite/FTS5 本地存储，无账号、云同步或遥测。
- 测试：Rust 与前端纯逻辑测试、静态检查和生产构建已进入 CI；完整桌面端到端测试尚缺。
