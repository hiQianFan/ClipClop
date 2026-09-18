# 实施与验证记录

代码已在当前 main 工作区实现。用户通过 macOS dev 实测确认：WPS 网页搜索框在键盘操作及点击 ClipClop 后保持输入上下文；Quick Look 曾出现同时关闭主面板和预览的回归，补齐预览非激活配置后，用户再次实测确认问题解决。完整的 IME、Space/全屏及其他应用矩阵仍未全部验证。

## 已实现

- main/quick 使用现有 NSPanel show_and_make_key，不主动激活应用；Webview::set_focus 只设置 WKWebView responder。
- 转换面板后通过私有方法 `_setPreventsActivation:` 同步 WindowServer 状态，并检查方法可用性。Quick Look 同样使用非激活样式且关闭自动 hidesOnDeactivate；需要随 macOS 升级回归。
- 主线程完成隐藏并检查 key/visible；等待有界，尚未执行的超时任务取消。
- 粘贴保存开始时目标和会话编号；重新打开面板使旧操作失效，隐藏前、注入前再次校验。目标已活动时跳过激活，其他外部应用成为前台时取消粘贴。
- quick→main 保留原目标；Quick Look 失焦判断使用实际预览 key 状态。
- 前端粘贴完成后失焦时不重新聚焦列表。

## 自动验证

- Rust 94 项通过；覆盖激活决策、会话失效、稳定等待超时。
- Vitest 26 个文件、166 项通过；覆盖粘贴完成后不抢回列表焦点。
- pnpm check 零错误/警告；pnpm build 通过。
- `cargo check --offline --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu` 已通过；这只是交叉编译检查，Windows 原生 Quality 及运行时验证仍待发布前完成。
- macOS Clippy（all-targets、warnings as errors）通过；OpenSpec strict 校验通过。
- 单测不证明原生主线程时序、键盘交还、输入法或 WPS 网页焦点保留。

## dev 验收

### 外部点击关闭回归（2026-09-16，待实机确认）

用户反馈非激活改动后，Quick Look 打开时点击其他窗口或桌面不会关闭面板与预览。
旧逻辑只监听应用失活及 Tauri 窗口失焦；非激活应用不一定产生失活通知，原生 Quick Look 也不经过 Tauri 的失焦回调。
现于面板首次显示时安装进程生命周期的 NSEvent 外部鼠标点击监听，左、右、中键外部点击统一调用现有 hide_panel，连同 Quick Look 和预览状态一起清理；保留原有失焦处理。
监听只接收其他应用事件，不拦截点击，也不监听键盘。点击主面板与 Quick Look 内部不走此关闭路径。

待验证：main/quick 无预览及打开预览两种状态下，分别点击 Chrome、其他应用及 Finder 桌面，两窗都应关闭；点击面板内部、预览内部不关闭；Space/Esc 关闭预览后主面板仍在；WPS 搜索框粘贴保持正常。当前自动检查不能证明上述原生行为。

运行 `pnpm tauri dev`，使用空白 WPS 网页表格：

1. 查找框聚焦 → 打开 ClipClop → Esc → 不点击鼠标直接输入，预期仍进入查找框。
2. 查找框聚焦 → 打开 ClipClop → 选择文本并 Enter，预期只修改查找框。以上两项各重复 10 次。
3. 检查面板搜索、中文候选、方向键、Enter/Shift+Enter、菜单与确认框。
4. 检查 main/quick、托盘/快捷键、面板互切、Quick Look 开关、外部点击、重复打开、全屏/Space、设置及权限返回。
5. 对照 Chrome 普通输入框、桌面 WPS 和 TextEdit。

失败时记录系统/Chrome 版本、入口、Esc/Enter、实际输入位置及 show_panel、panel handoff、automatic paste 日志，无需提供剪贴板内容或文档标题。

## 限制与回退

非激活面板不保证网页完全不产生 blur。若 WPS 仍重置输入，应记录失败并修订方案，不盲目重发。此变更不涉及数据库、权限、依赖或配置迁移，可单独撤回；不要重置整个工作区而丢失其他改动。
