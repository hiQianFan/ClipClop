# 贡献指南

感谢你帮助改进 ClipClop。项目仍处于 `0.1.0` 开发阶段，提交前请先确认需求没有超出 [PRODUCT.md](PRODUCT.md) 与 [DESIGN.md](DESIGN.md) 的 v1 边界。

## 开始之前

1. 对 bug、文档修正和小型可逆改动，可以直接提交 PR。
2. 对新功能、数据迁移、依赖升级、权限或隐私边界变化，请先创建 issue 说明动机、用户影响和替代方案。
3. 安全问题不要公开披露，按 [SECURITY.md](SECURITY.md) 处理。

## 本地开发

需要 Node.js（见 `.nvmrc`）、pnpm 9.15.3、Rust stable，以及对应平台的 Tauri 系统依赖。

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

提交前运行：

```bash
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

当前没有前端自动化测试命令。涉及键盘操作、焦点、剪贴板或窗口生命周期的改动，必须在 PR 中记录执行过的手工验证；测试矩阵见 [docs/testing.md](docs/testing.md)。

## 变更约定

- Rust 业务规则放在对应模块，Tauri command 保持薄；Svelte 通过 feature `api.ts` 调用 IPC。
- 不在日志、测试快照或 issue 中提交真实剪贴板正文、完整 URL、文件路径或其他私密数据。
- 已发布 migration 不修改；数据库变化新增有序 migration，并写升级/回退验证。
- 用户可见行为、权限、数据处理或发布流程改变时，同一 PR 必须更新相应文档。
- 提交信息建议采用 `type(scope): summary`，常用类型为 `feat`、`fix`、`docs`、`test`、`refactor`、`build`、`ci`。

## Pull Request

PR 描述应包括：问题与方案、影响平台、验证证据、隐私/权限影响、UI 变化截图（如适用）以及未解决风险。请保持变更聚焦，不把无关格式化或重构混入功能提交。
