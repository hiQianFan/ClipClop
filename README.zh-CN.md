# ClipClop

简体中文 | [English](README.md)

**轻量、离线优先的跨平台剪贴板历史工具。**

ClipClop 使用 Tauri 2、Rust、Svelte 5、TypeScript 和 Vite 构建。

> 当前状态：`0.1.0` 开发预览版。macOS 已完成本地构建验证；Windows 构建工作流已配置，Windows 实机验收仍待完成。

## 功能

- 捕获纯文本及其已有 HTML/RTF flavor、图片和文件引用；界面只安全展示纯文本。
- 按 Enter 保留可用格式粘贴，按 Shift+Enter 粘贴纯文本。直接粘贴失败时，内容仍留在系统剪贴板供手动粘贴。
- 全局快捷键呼出：macOS 默认为 `⌃⌘C`，Windows 默认为 `Ctrl+Alt+C`，可在“设置 → 快捷键”中修改。
- 托盘常驻，支持 Light/Dark 主题、保留期限和开机启动；退出 ClipClop 即停止捕获。
- 通过 GitHub Releases 检查经过 Tauri updater 签名验证的新版本；自动检查最多每天一次，下载和安装由用户确认。
- 无账号、云同步、遥测、广告或复制链接的联网增强。

macOS 首次直接粘贴会请求辅助功能/Post Event 权限；拒绝后仍可复制并手动粘贴。Windows 普通权限进程不能向管理员窗口注入输入，此时使用同样的回退方式。

## 安装

预览安装包发布后可从 [GitHub Releases](https://github.com/hiQianFan/ClipClop/releases) 下载：macOS 使用 Universal DMG，Windows 使用 x64 setup EXE。

当前预览安装包尚未使用 Apple Developer ID 或 Windows Authenticode 签名。macOS 可能要求在“系统设置 → 隐私与安全性”中允许应用，Windows 可能显示“未知发布者”或 SmartScreen 警告。Tauri updater 签名用于验证更新完整性，不能替代操作系统的发布者签名。如果不希望绕过系统警告，可以从源码构建或等待签名版本。

## 隐私

ClipClop 会在当前设备保存剪贴板内容、来源应用信息、文件路径引用和设置，不会上传这些数据，复制的 URL 也不会触发联网请求。ClipClop 不推测或过滤敏感内容；请使用删除单条、清空历史、保留期限或退出应用来控制捕获。

本地数据库没有由 ClipClop 进行应用层加密，依赖系统账户权限和 FileVault/BitLocker 等磁盘保护。完整说明见[隐私说明](docs/privacy.md)。

## 平台状态

| 平台 | 安装包 | 状态 |
| --- | --- | --- |
| macOS（Apple Silicon 与 Intel） | Universal DMG | 本地构建已验证；实机冒烟待完成 |
| Windows x64 | NSIS setup EXE | CI 已配置；实机冒烟待完成 |

## 从源码开发

环境要求：

- `.nvmrc` 指定的 Node.js 版本
- pnpm `9.15.3`
- 通过 rustup 安装的 Rust stable
- Tauri 平台依赖：macOS 需要 Xcode Command Line Tools；Windows 需要 Microsoft C++ Build Tools 和 WebView2

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

提交改动前运行全部本地质量检查：

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Vitest 覆盖更新节流、列表状态、快捷键格式化和粘贴回退等前端逻辑。完整桌面交互仍需 macOS 与 Windows 实机冒烟测试。

## 参与项目与支持

- [文档索引](docs/index.zh-CN.md)
- [贡献指南](CONTRIBUTING.zh-CN.md)
- [安全策略](SECURITY.zh-CN.md)——不要在公开 issue 中披露漏洞
- [行为准则](CODE_OF_CONDUCT.zh-CN.md)
- [变更日志](CHANGELOG.md)

ClipClop 采用 [MIT License](LICENSE)。
