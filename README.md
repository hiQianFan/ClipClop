# ClipClop

轻量、离线优先的跨平台剪贴板历史工具。技术栈为 Tauri 2、Rust、Svelte 5、TypeScript 和 Vite。

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

## 基础检查

```bash
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```
