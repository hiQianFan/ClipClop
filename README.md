# ClipClop

轻量、离线优先的跨平台剪贴板历史工具。技术栈为 Tauri 2、Rust、Svelte 5、TypeScript 和 Vite。

> 当前状态：`0.1.0` 开发预览版。macOS 已完成本地构建验证；Windows 构建工作流已配置，但公开安装包、代码签名与双平台实机验收尚未完成。

## 功能

- 捕获纯文本、图片和文件引用，并在本地搜索与预览。
- 在列表中按 Enter 复制，`⌘/Ctrl+K` 打开操作菜单。
- 全局快捷键呼出：macOS 为 `⌃⌘C`，Windows 为 `Ctrl+Alt+C`。托盘常驻、Light/Dark、保留期限与忽略来源应用；退出应用即停止记录。
- macOS 与 Windows 双端构建；无账号、云同步、遥测或链接联网增强。
- 通过 GitHub Releases 检查经过 Tauri updater 签名验证的新版本；自动检查最多每天一次，下载和安装由用户确认。

## 安装

项目暂未发布经过签名、公证的公开安装包。开发者可从源码运行；维护者构建测试安装包的方法见 [构建与分发](docs/distribution.md)。不要把本地未签名构建当作正式发行版分发。

公开预览版发布后，设置页可以检查、下载并安装更新。macOS 首次安装使用一个兼容 Intel 与 Apple Silicon 的 Universal DMG，Windows 使用 x64 setup EXE；自动更新不会上传剪贴板内容或设备资料。

## 隐私边界

ClipClop 会在本机保存剪贴板内容、来源应用信息和文件路径引用。数据不会上传，复制的 URL 也不会触发联网抓取。应用默认不判断或过滤“敏感内容”；请使用忽略来源、删除、清空历史、保留期限或退出应用来控制记录。完整说明见 [隐私说明](docs/privacy.md)。

## 命名约定

- 用户可见的产品名统一使用 `ClipClop`，包括窗口标题、macOS 应用名和 Windows 开始菜单名称。
- 包、仓库、数据库和普通代码标识使用小写 `clipclop`。
- Rust 应用库目标使用 `clipclop_lib`；它是 Tauri 应用入口库，不等同于独立业务内核。
- Bundle Identifier 使用 `com.clipclop.desktop`。

## 开发环境

- Node.js：通过 nvm 使用 `.nvmrc` 指定版本。
- 包管理器：pnpm。
- Rust：通过 rustup 使用 stable toolchain。
- macOS：需要 Xcode Command Line Tools。

```bash
nvm use
pnpm install
pnpm tauri dev
```

Windows 开发还需要 Microsoft C++ Build Tools 与 WebView2。项目使用 pnpm 锁文件和 Cargo 锁文件；提交前请使用 `pnpm install --frozen-lockfile` 验证依赖可复现。

## 基础检查

```bash
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

当前还没有前端自动化测试脚本；`pnpm check` 与 `pnpm build` 不能替代行为测试。这是公开发布前的已知缺口。

## 参与项目

- 文档导航与项目状态：[docs/index.md](docs/index.md)
- 开发与提交约定：[CONTRIBUTING.md](CONTRIBUTING.md)
- 安全问题报告：[SECURITY.md](SECURITY.md)
- 行为准则：[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- 版本变化：[CHANGELOG.md](CHANGELOG.md)

项目采用 [MIT License](LICENSE)。提交安全漏洞时请不要创建公开 issue；当前仓库尚未填写私密报告渠道，维护者应在公开发布前完成 `SECURITY.md` 中的占位项。
