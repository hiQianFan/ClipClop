# 为 ClipClop 贡献

简体中文 | [English](CONTRIBUTING.md)

感谢你帮助改进 ClipClop。项目仍处于 `0.1.0` 开发预览阶段。提出较大改动前，请先确认没有超出 [PRODUCT.md](PRODUCT.md) 与 [DESIGN.md](DESIGN.md) 的 v1 边界。

## 开始之前

1. Bug 修复、文档修正和小型可逆改动可以直接提交 Pull Request。
2. 新功能、数据迁移、依赖升级、权限或隐私边界变化请先创建 issue，说明动机、用户影响和替代方案。
3. 安全漏洞不要公开披露，请按[安全策略](SECURITY.zh-CN.md)处理。
4. 不要在 issue、测试数据、截图、日志或 Pull Request 中包含真实剪贴板内容、令牌、完整私密 URL 或个人文件路径。

## 本地开发

需要 `.nvmrc` 指定的 Node.js、pnpm `9.15.3`、Rust stable，以及对应平台的 Tauri 系统依赖。

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

提交 Pull Request 前运行：

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

涉及键盘、焦点、系统剪贴板、权限或窗口生命周期的改动还必须手工验证，并在 Pull Request 中记录平台和不含敏感数据的结果；测试矩阵见 [docs/testing.md](docs/testing.md)。

## 变更约定

- Rust 业务规则放在对应模块，Tauri command 保持薄；Svelte 通过 feature `api.ts` 调用 IPC。
- 不修改已发布 migration；数据库变化新增有序 migration，并记录升级和回退验证。
- 用户行为、权限、数据处理或发布流程改变时，同一 Pull Request 必须更新相应文档。
- 保持变更聚焦。提交信息建议使用 `type(scope): summary`，常用类型为 `feat`、`fix`、`docs`、`test`、`refactor`、`build`、`ci`。

## Pull Request

请说明问题与方案、影响平台、验证证据、隐私/权限影响、适用时的 UI 截图，以及未解决风险。所有证据都应使用虚构数据。
