---
stepsCompleted: [1, 2]
inputDocuments: []
workflowType: 'research'
lastStep: 4
research_type: 'technical'
research_topic: 'Windows/macOS 开源剪贴板工具自动粘贴实现'
research_goals: '验证 ClipClop 的自动粘贴设计，并形成可落地的跨平台实现与权限方案'
user_name: 'qianfan'
date: '2026-07-14'
web_research_enabled: true
source_verification: true
---

# 自动粘贴技术调研

## Technical Research Scope Confirmation

**Research Topic:** Windows/macOS 开源剪贴板工具自动粘贴实现

**Research Goals:** 验证 ClipClop 的自动粘贴设计，并形成可落地的跨平台实现与权限方案。

研究范围包括目标窗口记录与恢复、剪贴板格式、按键注入、macOS 权限、Windows UIPI、时序竞态与失败降级。关键结论同时核对开源项目源码与平台官方文档。

**Scope Confirmed:** 2026-07-14

## Technology Stack Analysis

### ClipClop 当前栈与适配结论

ClipClop 使用 Tauri 2、Rust、Svelte 5、`clipboard-rs` 和 `windows-sys`。现有剪贴板写入层已能保留多种 clipboard flavor，因此自动粘贴不应改成控件级文本注入；应继续写系统剪贴板，再向原目标窗口发送平台粘贴快捷键。

这与成熟项目一致：

- [Maccy `Clipboard.swift`](https://github.com/p0deje/Maccy/blob/3fe63ec3a0eabf6605d40c48b3c85b7bf555c86a/Maccy/Clipboard.swift) 使用 NSPasteboard 保存多格式内容，再通过 CGEvent 发送带 Command 标志的粘贴键；它还处理特殊键盘布局、事件源和本地事件抑制。
- [CopyQ Windows platform implementation](https://github.com/hluk/CopyQ/blob/e84cc055a507378cd3146d6c0a23a96f8d543342/src/platform/win/winplatformwindow.cpp) 保存目标窗口，恢复前台窗口，等待修饰键释放，然后用 `SendInput` 发送 Ctrl+V；必要时降级到 Shift+Insert。
- [Ditto `ExternalWindowTracker.cpp`](https://github.com/sabrogden/Ditto/blob/d36f864f9e6bc3558e11e3f1c9f5f522b8079702/src/ExternalWindowTracker.cpp) 同时跟踪 active window 和 focused child window，恢复最小化窗口，并在发送粘贴前等待目标成为 foreground window。
- [PasteBar Tauri implementation](https://github.com/PasteBar/PasteBarApp/blob/89ac53e3caba181bfa84628c2d0eed3c80eaa016/src-tauri/src/main.rs) 同样采用剪贴板写入加模拟粘贴；macOS 先检查 Accessibility，Windows 直接发送，但其固定 3 秒等待明显偏保守，不适合照搬。

### 平台库选择

macOS 适合直接通过 CoreGraphics FFI 创建 CGEvent，并用 ApplicationServices 的 Accessibility trust API检查权限。ClipClop 已使用 AppKit/Objective-C 依赖，不需要引入重量级自动化框架。

Windows 适合继续使用 `windows-sys`，增加 `Win32_UI_Input_KeyboardAndMouse` feature，调用 `GetForegroundWindow`、`IsWindow`、`ShowWindow`、`SetForegroundWindow` 和 `SendInput`。不需要引入跨平台键鼠模拟库，因为窗口恢复和错误分类仍然必须写平台代码。

### 时序与可靠性结论

固定 sleep 只能作为短暂让步，不能作为成功条件。更稳健的顺序是：写剪贴板成功、隐藏面板、恢复目标窗口、轮询确认前台窗口、确认用户已释放打开面板所用修饰键、再注入粘贴键。CopyQ 暴露了 raise 前后等待和修饰键释放等待配置；Ditto 也实现了 `WaitForActiveWnd`。因此 ClipClop 应采用有上限的条件等待，而非单一固定延迟。

### 权限与发行

macOS 的自动粘贴依赖 Accessibility trust；未授权时应保留“已复制”结果并给出可操作提示。Windows 普通窗口不需要权限弹窗，但 `SendInput` 受 UIPI 限制，普通权限 ClipClop 不能可靠注入高完整性管理员进程。默认要求管理员权限会扩大安全面，不建议采用。

### 初步优化决策

1. 目标窗口在 ClipClop `set_focus()` 之前捕获，并由 Rust 后端持有。
2. Enter 调用新的 `paste_clip`；现有 `copy_clip` 保留为纯复制能力。
3. 使用“条件等待 + 小上限”，不使用 PasteBar 式 3 秒固定延迟。
4. 注入前等待全局快捷键修饰键释放，避免 Ctrl/Command 尚未抬起造成错误组合键。
5. 失败统一降级为 copied-only，并返回结构化原因供 UI 提示。
6. 暂不恢复旧剪贴板，避免目标应用异步读取时发生竞态。

## Integration Patterns Analysis

### 前端与原生层 API 契约

前端只负责表达用户意图，不持有 HWND、PID 或 macOS 应用对象。建议提供两个明确命令：

- `copy_clip(id)`：只写剪贴板，不关闭面板。
- `paste_clip(id)`：执行写入、隐藏、恢复目标和按键注入，返回结构化 `PasteOutcome`。

`PasteOutcome` 至少区分 `pasted`、`copied_permission_required`、`copied_target_lost`、`copied_focus_failed` 和 `copied_injection_failed`。所有失败状态都意味着剪贴板写入已经成功；若写入失败则直接返回命令错误，不进入窗口切换阶段。

### 目标捕获协议

目标必须在 `show_panel()` 调用 `show/set_focus` 前捕获。Apple 将 `NSWorkspace.frontmostApplication` 定义为接收键盘事件的最前台应用，因此保存其 PID 是合适的应用级目标；Windows 保存 `GetForegroundWindow()` 的 HWND 与 PID，并在使用前重新验证 HWND 仍属于原 PID，避免句柄复用。

来源：[Apple `frontmostApplication`](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication)、[Microsoft `SetForegroundWindow`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow)。

### 激活与确认协议

macOS 新版 AppKit 建议 cooperative activation：ClipClop 将激活权让给之前的 `NSRunningApplication`，请求目标激活，再轮询 frontmost PID。调用 `activate()` 本身不保证激活成功，因此不能以调用返回作为唯一成功条件。

Windows 的 `SetForegroundWindow` 受前台锁规则限制，即使调用条件看似成立也可能失败。实现必须同时检查返回值和 `GetForegroundWindow() == target`。若目标最小化，先 `ShowWindow(SW_RESTORE)`；若最终确认失败，禁止发送 Ctrl+V。

来源：[Apple cooperative activation / `activate()`](https://developer.apple.com/documentation/appkit/nsapplication/activate%28%29)、[Microsoft `SetForegroundWindow`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow)。

### 输入注入协议

macOS 使用同一 CGEventSource 创建 V key down/up，并添加 Command flag；事件发送前检查 Post Event/Accessibility trust。Maccy 的实现还覆盖特殊键盘布局和事件抑制，说明直接依赖当前字符布局生成 `v` 不够可靠，应该使用物理键位或明确虚拟键码。

Windows 用单次 `SendInput` 提交 Ctrl down、V down、V up、Ctrl up 四个事件，并校验返回数量。CopyQ 证明不同目标程序可能需要 Shift+Insert，但 ClipClop MVP 先用 Ctrl+V；后续可按进程规则增加降级，而不是失败后立即再发一次、造成重复粘贴。

来源：[Maccy `Clipboard.swift`](https://github.com/p0deje/Maccy/blob/3fe63ec3a0eabf6605d40c48b3c85b7bf555c86a/Maccy/Clipboard.swift)、[CopyQ `winplatformwindow.cpp`](https://github.com/hluk/CopyQ/blob/e84cc055a507378cd3146d6c0a23a96f8d543342/src/platform/win/winplatformwindow.cpp)、[Microsoft `SendInput`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)。

### 状态机与并发

建议使用后端单实例状态机：`Idle -> ClipboardWritten -> PanelHidden -> TargetActivated -> Injected -> Idle`。原子 in-flight 标记拒绝 Enter 键盘重复触发。捕获的新目标覆盖旧目标；目标超过合理有效期或进程已退出时降级只复制。

窗口恢复采用短间隔、有界轮询，而非固定长 sleep。CopyQ 提供 raise 前、raise 后、修饰键释放等独立等待项；Ditto 使用 `WaitForActiveWnd`。ClipClop 初版可以采用内部常量并通过测试验证，不必过早暴露高级设置。

### 剪贴板与 watcher 协作

平台剪贴板序列号或 change count 是识别自身写入的最佳信号；现有内容哈希仍可作为跨平台兜底。自动粘贴结束后不恢复旧剪贴板，因为目标应用可能延迟读取。若未来支持恢复，需要等待目标确认消费或提供显式延迟配置，不能在 SendInput 返回后立即恢复。

### 安全和失败边界

- macOS 仅申请 Accessibility/Post Event 权限，不申请 Input Monitoring。
- Windows 不默认提权；UIPI 导致无法粘贴管理员应用时返回 copied-only。
- 目标焦点未确认时绝不注入，优先避免内容粘贴到 ClipClop、其他应用或密码输入框。
- 不记录待粘贴内容或目标窗口标题到生产日志；仅记录错误类别和平台代码。

## Implementation Assessment

### 已落地

- 新增 Rust `PasteController`，显示 ClipClop 前捕获 macOS PID 或 Windows HWND+PID。
- 新增 `paste_clip` 命令：先完整写剪贴板，再隐藏面板、恢复目标、确认前台状态并注入粘贴键。
- macOS 使用 Post Event 权限预检/请求、CGEventSource 和成对 Command+V 事件；权限不足时保留复制结果并重新显示面板提示。
- Windows 验证 HWND 仍属于原 PID，恢复最小化窗口，确认 foreground，等待修饰键释放，并校验 `SendInput` 四个事件全部插入。
- Enter、双击和主按钮执行自动粘贴；操作菜单保留“仅复制到剪贴板”。
- 所有平台失败都返回 copied-only 结构化结果，不在目标未确认时盲发快捷键。

### 验证结果

- macOS `cargo test`：10 个测试通过。
- `cargo clippy --all-targets -- -D warnings`：通过。
- `svelte-check`：0 errors / 0 warnings。
- Windows Rust target 已安装，但本机缺少 `x86_64-w64-mingw32-gcc`，交叉检查在 `libsqlite3-sys` 编译原生 SQLite 时停止；需要 Windows CI 或真机完成平台编译和行为验证。

### 后续可选优化

Maccy 通过 `NSPanel.nonactivatingPanel` 让原应用始终保持 active，理论上比重新激活 PID 更快、更稳。Tauri 默认 WebViewWindow 并非该面板模型，强行修改私有窗口类风险较高，因此本次采用已验证的 PID 激活和前台确认路径。若真机数据表明 macOS 焦点恢复仍是主要失败源，可单独评估 NSPanel 插件，并保留当前实现作为 fallback。
