# 测试指南

## 自动检查

```bash
pnpm install --frozen-lockfile
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

GitHub Actions 在 macOS 与 Windows 执行同一质量门。Vitest 覆盖不依赖 WebView 的前端逻辑；CI 仍不能替代完整桌面交互与系统剪贴板实机验证。

## 桌面冒烟测试

使用虚构内容，在每个目标平台逐项验证：

- 首次启动、单实例、托盘打开/退出和全局快捷键显示/隐藏。
- 纯文本、同时含 HTML/RTF 的格式化文本、URL、代码、色值、图片、单文件和多文件捕获。
- 格式化文本默认保留已有格式，Shift+Enter 与“仅复制纯文本”只写回纯文本；目标应用不支持格式时由系统自然降级。
- 打开面板及自动选中首条文件记录不会读取源文件；明确点击、切换子项或预览后才读取。
- 最新优先、搜索、分页、选择、预览、复制、打开、删除和清空。
- 来源应用可用与不可用两种情况；`loginwindow` 等系统会话占位进程不显示，也不回退成最近应用。
- 7/30/90 天保留期、开机启动与 Light/Dark/System 设置重启后保持。
- 键盘路径、焦点可见性、Esc 层级、减少动态效果和窄工作区尺寸。
- 文件移动/删除、数据库不可写、无预览资源等失败路径不会终止监听。

## 发布验收证据

发布 PR 应记录操作系统版本、CPU 架构、安装包文件名与 SHA-256、签名/公证结果、上述冒烟测试结果和已知问题。macOS 本机成功不能替代 Windows runner 或 Windows 实机验收。

## 待补自动化

优先补前端键盘映射、列表状态、IPC 错误恢复与设置表单测试；随后补基于真实 Tauri 的最小端到端路径。涉及系统剪贴板的测试应串行运行并在结束时恢复或清理测试内容。
