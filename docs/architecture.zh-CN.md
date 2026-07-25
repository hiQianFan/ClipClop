# ClipClop 架构说明

简体中文 | [English](architecture.md)

本文记录 ClipClop 演进过程中必须保持稳定的边界，重点是运行时职责、隐私敏感流程和面板生命周期，而不是逐一罗列源文件。

## 运行时结构

ClipClop 包含两层：

- `src/` 是 Svelte 界面。功能代码通过各自的 `api.ts` 调用 Tauri command，负责 DOM 焦点、键盘交互、渲染和无障碍。
- `src-tauri/src/` 是 Rust 宿主，负责系统剪贴板、持久化、全局快捷键、原生窗口、预览、自动粘贴、日志和更新。

Tauri command 只是适配层。业务规则放在对应功能模块，原生窗口行为统一封装在 `window` 模块之后。

## 面板生命周期

主窗口是临时命令面板，不是普通应用窗口。原生窗口焦点和 DOM 焦点是两个不同层次：

1. Rust 开始一个新的面板 generation。
2. Rust 调整并显示原生窗口。
3. 平台适配器请求前台激活。
4. Rust 发送 `panel_shown`。
5. Svelte 重置浏览会话并设置 DOM 焦点。

不得在原生显示和激活尝试之前发送 `panel_shown`，否则 WebView 中的元素可能已经获得 DOM 焦点，而原生窗口仍处于后台。

### 状态模型

`PanelLifecycleState` 是原生生命周期的唯一事实来源：

```text
Hidden → Showing → Focused → BlurPending → Hidden
                    ↑             |
                    └─────────────┘
```

每次显示都会开启新的 generation。待处理失焦令牌包含 generation 和 revision；重新显示、重新聚焦或隐藏都会让旧令牌失效。窗口至少真正获得一次焦点前，启动阶段的失焦事件会被忽略。

`PreviewState` 与生命周期分离，只表示原生预览是否活跃。原生预览可以临时取得焦点，而不触发主面板隐藏。

### 必须保持的不变量

- 不得在 `window::hide_panel` 之外调用 `WebviewWindow::hide()`。
- 每次隐藏都必须说明 `HideReason`：失焦、Escape、粘贴或快捷键。
- 不得把预览状态和面板生命周期合并。
- 不得用多个独立 Atomic 表示一个需要一致快照的生命周期。
- 原生焦点事件负责更新 `PanelLifecycleState`；前端 DOM 焦点不能替代原生焦点验证。
- 延迟任务必须携带 generation/revision 令牌，并在主线程再次确认状态后才能修改窗口。

## 平台适配

平台行为保持在小型适配模块中：

- `window/windows.rs` 负责 Win32 前台激活。
- `window/macos.rs` 负责 NSPanel 激活与 Quick Look。
- `window.rs` 负责平台无关的编排和尺寸。
- `window/lifecycle.rs` 是纯状态机，必须能够在没有真实窗口时进行单元测试。

### Windows 焦点策略

Windows 适配器优先使用常规前台 API，并通过 `GetForegroundWindow` 验证窗口归属。不得使用 Tauri/tao 的模拟 Alt 回退或 `AttachThreadInput`；它们都曾导致事件循环重入或死锁。

调整 foreground-lock timeout 只作为兼容回退，并由 RAII guard 保证恢复。回退必须保持隔离，只记录不包含剪贴内容的结果；只有真实设备证据证明不再需要时才能移除。

### macOS 预览策略

Quick Look 可以临时取得焦点而不隐藏主面板。显式隐藏面板时也必须关闭 Quick Look 并清除预览状态。

## 日志

诊断日志只包含运行事件和错误文本，不得包含剪贴板载荷或预览内容。

Windows GUI 进程只写应用日志文件。不要重新加入 Windows stderr 目标：`tauri dev` 子进程可能比终端存活更久，而 fern 向已关闭管道写入时会触发 panic。其他平台开发环境可以同时写 stderr。

## 验证门槛

所有改动都必须通过 `CONTRIBUTING.zh-CN.md` 中的自动检查。涉及焦点、键盘、粘贴、预览或窗口生命周期的修改还必须在真实设备验证：

1. 冷启动后立即输入。
2. 启动后立即使用方向键。
3. 连续用全局快捷键呼出和隐藏。
4. 再次启动程序，确认激活已有进程。
5. 切换到其他应用后再返回。
6. 完成一次粘贴后再次呼出。
7. 让 `tauri dev` 的父终端结束，再继续使用应用。
8. 确认系统前台窗口属于 ClipClop，日志没有 panic。
9. macOS 打开和关闭 Quick Look 后，确认选择状态被保留。
10. 条件允许时，在多显示器和不同缩放比例重复验证。

验证必须使用虚构剪贴内容，不得在证据中包含私人路径或真实剪贴板数据。

## 有意保留的后续事项

以下是演进方向，不构成绕过当前边界的理由：

- 统计 Windows foreground-lock 回退的真实触发率，再决定是否移除。
- 只有焦点抖动产生可测运行成本时，才替换当前短时阻塞防抖任务。
- 出现更多窗口类型或尺寸策略时，再将尺寸逻辑拆为独立模块。
- 只有真实设备证明现有时序仍不足时，才增加单独的前端 `panel_activated` 事件。
- 新增原生窗口前应设计独立生命周期实例，不得复用主面板的单例状态。
