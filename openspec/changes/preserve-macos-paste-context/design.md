## Context

必须区分活动应用、AppKit key window/first responder 与网页 DOM 焦点。不激活面板允许 Chrome 保持活动应用，同时 ClipClop 面板临时接收键盘；不保证 Chrome 网页不产生 blur，也不能同时把正常输入交给两个应用。

### 当前调用链和证据

1. `src-tauri/src/lib.rs`：ClipboardPanel 可成为 key、不可成为 main；main/quick 已设置 nonactivatingPanel。
2. `window.rs::show`：两个面板未显示时 capture_target；随后 `macos::show_as_panel`、`window.set_focus()`，并直接 mark_focused。
3. `window/macos.rs::show_as_panel`：主动调用 activateIgnoringOtherApps，再 make_key_and_order_front。
4. 本地依赖 Tauri 2.11.5 的 WebviewWindow::set_focus 转发到窗口；tao 0.35.3 的 macOS util/async.rs::set_focus 同时 makeKeyAndOrderFront 和 activateIgnoringOtherApps。因此不能保留此调用并称之为非激活显示。实施时以 Cargo.lock 实际解析版本再次核对。
5. `commands/history.rs::paste_clip` 在 spawn_blocking 中运行 workflow；workflow 写剪贴板、hide_panel、paste_to_target。必须核实原生隐藏完成时序，不能假定 IPC/隐藏调用返回意味着键盘已交还。
6. `paste/macos.rs` 只保存 PID；无条件激活目标，轮询 PID 稳定并等待 60ms 后发 Cmd+V。PID 一致不能证明面板已不再接收键盘，更不能证明网页 input 正确。
7. `window.rs::handle_focus_event` 将 application_is_active 与 Quick Look 状态结合判断。非激活面板下应用不活跃可能是正常状态，必须改用真实面板/预览窗口状态。
8. HistoryWorkspace 与 QuickPanel 已有 shown 事件、DOM focus 和 blur/focus 处理；主面板 pasteSelected 在异步粘贴返回后 enterBrowse，会安排 DOM 聚焦，需验证隐藏状态下是否产生副作用。

## Decisions

### 1. 先做有停止条件的可行性实验

在真实 macOS Chrome/WPS 环境测试：保持原应用激活，原生面板成为 key，WKWebView 可输入并正常处理中文 IME。关闭面板后不点击鼠标，在 WPS 查找框继续打字。实验同时记录应用 PID 与原生面板 key/visible 状态。

只有“不切换应用 + 面板可操作 + 关闭后原查找框继续输入”同时成立，才以此作为 WPS 修复方向。否则保留失败记录，评估窗口 key/responder 或网页 blur 行为，不直接扩张为 AX/浏览器扩展方案。

### 2. 使用已有面板，去掉所有常规显示路径的应用激活

在 macOS 原生主线程执行面板 order/key 和必要的 WKWebView first-responder 操作。移除 show_as_panel 的显式激活；替换窗口级 set_focus。仅在实测确有需要时增加一个局部原生 WebView responder 操作，不 fork Tauri、不新增通用焦点框架。

不再在未确认原生 key 状态时无条件 mark_focused。复用 PanelLifecycleState，确保首次显示、重复打开、quick→main、快捷键和托盘入口都正确；原生焦点失败须可诊断，不静默改回抢应用焦点并当作成功。

启动、设置、引导、权限跳转等有意进入应用/外部应用的路径分别审核，不机械删除仓库中所有 activate/set_focus。

### 3. 完成隐藏，再进行有界的键盘交还与粘贴

复用现有 paste_clip 单入口和串行许可。将需要 AppKit 的隐藏/状态读取调度到主线程，并向后台流程报告完成；主线程不执行轮询 sleep，也不等待自己排队的任务。

常规路径：写入剪贴板 → 完成原生隐藏/退出 key 状态 → 确认目标进程存活且仍在前台 → 短暂有界稳定检查 → 权限及目标最终检查 → 只发一次粘贴。

目标仍为活动应用时不额外 activate。若 ClipClop 因明确的应用内流程变为活动应用，可复用目标激活作为回退，但记录 fallback，且不承诺保留网页内部焦点。若出现另一个外部前台应用，或面板已重新打开/会话目标已变化，则取消旧粘贴，保留剪贴板，使用现有失败 outcome；不夺回用户新选择的应用。

利用现有生命周期 generation/目标状态校验过期流程；仅在不足时增加最小会话标识。最终发键前重查，承认系统状态检查与发键之间无法做到跨进程原子化。

现有 60ms 等待先作为待实测参数，不能代替隐藏完成确认。不重复发送粘贴，不把事件已投递解释成目标文本已插入。

### 4. 修正失焦和原生预览边界

保留已有失焦 debounce/token 防止旧回调隐藏新面板。以面板是否 key、受管理 Quick Look 是否正在接收交互为依据，不能单凭 NSApp.isActive=false 清除预览状态。外部点击仍应隐藏；Quick Look 交互不得误关父面板；预览关闭后恢复面板键盘上下文。测试原生预览的确切事件顺序后实现局部修正。

### 5. 前端改动是条件性的，不是重写

| 层 | 文件 | 计划 |
| --- | --- | --- |
| 原生显示/关闭 | window/macos.rs、window.rs | 必改：非激活显示、真实 key 状态、完成顺序、blur/预览 |
| 原生粘贴 | paste/macos.rs | 必改：按需激活、交还等待、最终检查 |
| 生命周期/目标 | window/lifecycle.rs、paste/mod.rs | 按需要复用/补充会话失效判断 |
| 初始化/调度 | lib.rs、workflows/paste_clip.rs、commands/history.rs | 审核主线程边界，确有需要才改 |
| Svelte | HistoryWorkspace.svelte、QuickPanel.svelte | 先复用现有事件；只修隐藏后/旧会话异步聚焦或事件适配 |
| IPC/样式/存储 | history/api.ts、CSS、数据库 | 预计无需改动 |

前端 DOM focus 只负责 ClipClop 内部输入位置，不承担 Chrome DOM 恢复。保留 Enter、Shift+Enter、搜索、菜单、确认框、Tab 和中文输入法行为；不加全局按键拦截。若需修改 Bits UI 复合控件，实施时先读取对应官方文档。

## Risks and Trade-offs

- WPS 可能对 key window 转移本身就重置输入状态：实验失败则不宣称此方案修复 WPS。
- WKWebView 在非活动应用中的 responder、IME、辅助功能可能与 SwiftUI/Maccy 不同：真实设备验证不能由 jsdom 替代。
- Quick Look、托盘、首次启动、全屏/Space 和面板互切可能造成非典型应用激活：必须单独验证。
- 托盘交互可能在捕获目标前改变系统状态：仅在可捕获有效原目标时自动粘贴，不能凭猜测选择应用。
- 失败维持 clipboard-only，不新增 AX 权限。可整体回退本 change，但需明确原 WPS 问题仍存在。

## Complexity and Validation

预估：可行性实验 0.5–1 天；原生显示/隐藏和粘贴联动 1–2 天；前端必要适配 0–0.5 天；原生回归与修正约 1–1.5 天。总体约 3–5 个工程工作日，外部环境不可用或原生兼容问题会增加时间。

自动验证：在已有 Rust 测试中覆盖按需激活、外部目标变化取消、过期会话和隐藏超时；前端仅在代码实际修改时扩充对应现有测试。运行 cargo test、cargo fmt 检查及项目现有 macOS/Windows Quality 检查；前端改动时运行 pnpm check 和相关 Vitest。此提案阶段仅运行 OpenSpec 校验。

真实设备发布门槛：WPS 查找/替换、浏览器普通输入框、WPS 桌面版、TextEdit；分别验证 Esc、Enter、Shift+Enter、鼠标选择、快捷键/托盘、main/quick、重复打开、quick→main、中文 IME、菜单/确认框、Quick Look、设置/权限返回、全屏与多 Space；WPS 核心 Esc/Enter 流程各连续 10 次无误投。记录 macOS/Chrome/WPS 环境、复现步骤和结果；用户的真实 WPS 页面不可用时必须注明尚未通过该发布门槛。

## References

- Maccy 非激活面板：https://github.com/p0deje/Maccy/blob/master/Maccy/FloatingPanel.swift
- Maccy 关闭后粘贴：https://github.com/p0deje/Maccy/blob/master/Maccy/Observables/History.swift
- Ditto 活动窗口与控件焦点分离：https://github.com/sabrogden/Ditto/blob/master/src/ExternalWindowTracker.cpp （Windows 参考，不在本次实现）
- CopyQ 焦点破坏报告：https://github.com/hluk/CopyQ/issues/3540
- CopyQ 最终合并的是文档及绕过方案：https://github.com/hluk/CopyQ/pull/3575

以上为调研时公开源码/记录，不代表所有已发布版本，也不构成已验证的 WPS 兼容性承诺。
