# 变更日志

本文件记录用户可见的重要变化，格式遵循 Keep a Changelog 的分类思路，版本遵循语义化版本。

## [Unreleased]

## [0.1.0] - 2026-07-22

### Added

- 本地剪贴板历史捕获、SQLite/FTS5 搜索与分页。
- 文本、链接、代码、色值、图片和文件引用展示与复制。
- 全局快捷键、托盘、主题、保留期限、开机启动和忽略来源应用设置。
- macOS Universal DMG 与 Windows x64 安装包。
- 英文与简体中文产品介绍、贡献指南和隐私说明。
- Issue 与 Pull Request 模板。

### Changed

- README 改为以产品介绍、下载和隐私价值为核心。
- 精简公开文档，贡献指南作为唯一开发启动入口。
- Release workflow 在打包前统一校验版本并运行完整质量门，生成安装包、更新签名及更新清单。

> 预览版安装包尚未使用 Apple Developer ID 或 Windows Authenticode 签名，系统可能显示安全确认；更新文件另有完整性签名。
