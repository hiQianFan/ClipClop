## Why

macOS + Chrome 的 WPS 网页版中，用户聚焦查找输入框后，打开并退出 ClipClop，直接打字会进入单元格；无需执行粘贴即可复现。用户确认切换其他桌面应用也会触发同样现象，而 WPS 桌面版正常。因此这是网页对应用/窗口焦点变化的兼容问题，尚未证明是 ClipClop 独有缺陷，也尚未证明不激活面板能解决。

当前 macOS 面板虽然设置了 nonactivatingPanel，但显示时显式激活 ClipClop，随后窗口 set_focus() 也会激活应用；自动粘贴又无条件激活目标进程。需要避免不必要的应用切换，同时保持面板键盘输入和正确的关闭时序。

## What Changes

- 先验证原生不激活面板 + WKWebView 键盘输入能否保留 Chrome/WPS 查找输入上下文；通过后完成实现与回归。
- main/quick 显示流程使用原生面板键盘焦点，避免激活整个 ClipClop 应用。
- 校正实际焦点事件、面板切换、失焦关闭和 Quick Look 的生命周期处理。
- 粘贴等待面板实际退出键盘接收状态；目标仍为活动应用时跳过多余激活，目标异常时采用受控恢复或现有 clipboard-only 结果。
- 保留现有前端交互和 IPC；只在原生事件行为变化或隐藏后异步 DOM 聚焦会干扰上下文时做小范围适配。
- 增加不包含剪贴板正文、窗口标题、网页 URL 的焦点诊断和必要回归验证。

## Capabilities

### New Capabilities

- `macos-paste-context`: macOS 非激活面板、键盘交还、粘贴时序及目标变化保护。

### Modified Capabilities

- None. 现有 `stabilize-keyboard-focus` 的前端 Browse/Search/Menu 等键盘契约继续保留。

## Impact

主要改动在 Rust/macOS 原生窗口层；预计 3–5 个后端文件，必要时扩展到 lib.rs、粘贴 workflow/command。前端预计 0–2 个组件及对应现有测试，不改变页面设计，不新增 IPC、依赖、设置项、权限或数据库结构。Windows 行为保留并做编译回归。

复杂度为中等：代码量较小，主要成本是 AppKit、WKWebView、Tauri 焦点事件和 Quick Look 的集成验证。初步估计 3–5 个工程工作日（含约半天到一天可行性实验），前提是能使用可复现的 WPS 页面；这是工作量估计，不是兼容性保证。

## Non-Goals

- 不修改 WPS 网页、不注入脚本、不开发 Chrome 扩展、不模拟鼠标点击。
- 不增加通用 AX 网页控件恢复、Windows Ditto 式 SetFocus、全局键盘钩子。
- 不承诺任意网页在临时转移键盘输入后仍保留内部状态。
- 不把增加固定延迟作为修复，不在不确定是否已粘贴时重发按键。

## Approval Boundary

状态：待用户确认。本 change 仅提交提案，不修改产品代码。用户确认后按 tasks.md 执行；若可行性实验失败，记录证据并修订方案，不把未验证实现宣称为 WPS 修复。
