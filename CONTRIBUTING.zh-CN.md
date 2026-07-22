# 为 ClipClop 贡献

简体中文 | [English](CONTRIBUTING.md)

感谢你帮助改进 ClipClop。这里是配置开发环境、提出改动和提交 Pull Request 的唯一入口。

## 开始之前

1. Bug 修复、文档修正和小型可逆改动可以直接提交 Pull Request。
2. 新功能、数据迁移、依赖升级、权限或隐私边界变化请先创建 issue，说明动机、用户影响和替代方案。
3. 不要在 issue、测试数据、截图、日志或 Pull Request 中包含真实剪贴板内容、令牌、完整私密 URL 或个人文件路径。

## 本地开发

需要 `.nvmrc` 指定的 Node.js、pnpm `9.15.3`、Rust stable，以及对应平台的 [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)：macOS 使用 Xcode Command Line Tools，Windows 使用 Microsoft C++ Build Tools 与 WebView2。

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

Svelte 界面位于 `src/`，Rust/Tauri 应用位于 `src-tauri/`。

提交 Pull Request 前运行：

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

涉及键盘、焦点、系统剪贴板、权限或窗口生命周期的改动还必须手工验证，并在 Pull Request 中记录平台和不含敏感数据的结果。

## 变更约定

- Rust 业务规则放在对应模块，Tauri command 保持薄；Svelte 通过 feature `api.ts` 调用 IPC。
- 在迁移系统建立前，数据库结构变化必须递增 `SCHEMA_VERSION`；旧版本的开发数据库需要重置。
- 用户行为、权限、数据处理或发布流程改变时，同一 Pull Request 必须更新相应文档。
- 保持变更聚焦。提交信息建议使用 `type(scope): summary`，常用类型为 `feat`、`fix`、`docs`、`test`、`refactor`、`build`、`ci`。

## 分支与版本

- `main` 是唯一长期分支，并应始终保持可发布状态。
- 改动使用短期功能或修复分支，通过 Pull Request 合并，并在合并后删除；项目不使用 release 分支。
- 版本遵循语义化版本。只有发布 `vX.Y.Z` 标签与 GitHub Release 后，该版本才正式存在；分支不代表版本。
- 依赖升级由维护者按需审查。GitHub 漏洞提醒保持开启，但不使用定时依赖升级 Pull Request。

## Pull Request

请说明问题与方案、影响平台、验证证据、隐私/权限影响、适用时的 UI 截图，以及未解决风险。所有证据都应使用虚构数据。

## 安全问题

请勿在公开 Issue 或 Pull Request 中披露尚未修复的漏洞。请使用 [GitHub 私密漏洞报告](https://github.com/hiQianFan/ClipClop/security/advisories/new)，并仅使用虚构数据说明受影响版本、平台、复现步骤和影响。

## 维护者发布流程

1. 同步更新 `package.json`、`src-tauri/Cargo.toml` 和 `src-tauri/tauri.conf.json` 中的版本号。
2. 从 `main` 手动运行 **Release** workflow 并填写该版本；它只创建 Draft Release，不会立即公开。
3. 检查 macOS DMG、Windows 安装包、更新签名和 `latest.json`；条件允许时在两个平台实际安装。
4. 检查自动生成的版本说明后，将 Draft 发布为普通 Release。任何产物或冒烟测试未完成时都继续保留 Draft；发布后的 `vX.Y.Z` 标签与 Release 是版本的正式记录。
