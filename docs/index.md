# ClipClop 文档索引

本目录以当前代码为事实基线。产品愿景描述“为什么做”，设计规范描述“应该如何表现”，实现与发布文档描述“当前已经做到什么”。发生冲突时，应先核对代码和测试，再更新文档，不能让计划性描述冒充已实现能力。

## 使用者

- [README](../README.md)：项目入口、能力、安装状态与本地开发。
- [隐私说明](privacy.md)：本地保存的数据、控制方式与限制。
- [构建与分发](distribution.md)：macOS/Windows 构建和 CI。
- [故障排查](troubleshooting.md)：常见开发和运行问题。

## 产品与设计

- [产品说明](../PRODUCT.md)：用户、目标、原则和范围边界。
- [设计系统](../DESIGN.md)：视觉 token、布局和交互规范。
- [PRD](../outputs/prds/prd-ClipClop-2026-07-10/prd.md)：当前需求范围与决策记录。
- [实施计划](implementation-plan.md)：交付范围、验收项和已知约束。

## 开发与维护

- [技术架构](architecture.md)：模块边界、存储、IPC、安全与测试策略。
- [测试指南](testing.md)：自动检查、手工冒烟和平台验收。
- [发布清单](release-checklist.md)：版本、签名、产物与发布后验证。
- [开源就绪审计](open-source-readiness.md)：当前缺口、优先级和完成定义。
- [贡献指南](../CONTRIBUTING.md)、[安全策略](../SECURITY.md)、[行为准则](../CODE_OF_CONDUCT.md)、[变更日志](../CHANGELOG.md)。

## 当前事实摘要

- 版本：`0.1.0` 开发预览。
- 平台：macOS 本地构建已验证；Windows CI 已配置，实机验收待完成。
- 内容：纯文本及其轻量识别类型、已有 HTML/RTF flavor、图片、文件引用；HTML/RTF 仅原样写回，不渲染。
- 数据：SQLite/FTS5，本地优先，无账号、云同步或遥测。
- 测试：Rust 与前端纯逻辑测试、静态检查和构建检查已存在；桌面端到端测试尚缺。
