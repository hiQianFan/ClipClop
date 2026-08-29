---
title: '跨窗口同步主题预览'
type: 'bugfix'
created: '2026-08-29'
status: 'done'
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 在主面板设置页切换主题时，主题只应用于当前 WebView；托盘 Quick 面板继续显示旧主题，通常要重启应用后才更新。

**Approach:** 复用 Tauri 的全局事件机制，在主题选择发生变化时广播预览值，让所有已加载窗口立即调用现有 `applyTheme()`。持久化仍由现有“保存”流程负责。

## Boundaries & Constraints

**Always:** Quick 与主面板在主题下拉框改变后立即一致；保存成功后的后端广播继续作为持久状态同步；保存失败回滚时所有窗口也恢复到已保存主题；保持现有 light、dark、system 三种语义。

**Ask First:** 若修复需要改成主题选择即自动保存、改变设置页保存交互，或修改 Rust 设置事务，则先询问用户。

**Never:** 不重构主题系统，不增加依赖，不修改窗口尺寸、定位、托盘点击或 macOS NSPanel 逻辑，不把 WebView 改为原生 UI。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|---------------------------|----------------|
| 实时预览 | 主面板选择 light、dark 或 system | 主面板与 Quick 同时应用所选主题 | 广播失败不阻止当前窗口预览或后续保存 |
| 保存失败 | 已预览新主题，但更新设置失败 | 两个窗口恢复已保存主题 | 保留现有错误提示 |
| Quick 未加载 | 切换主题时 Quick 尚未初始化 | 主面板正常预览；Quick 后续启动时读取当前已保存设置 | 不缓存额外预览状态 |

</frozen-after-approval>

## Code Map

- `src/lib/settings/SettingsView.svelte` -- 主题下拉框、保存与失败回滚的入口。
- `src/routes/+layout.svelte` -- 每个 WebView 的全局初始化和主题事件监听。
- `src/lib/settings/api.ts` -- 现有 `applyTheme()` 实现与主题类型。
- `src-tauri/src/workflows/settings_update.rs` -- 保存成功后广播持久化主题，保持不变。
- `src-tauri/capabilities/` -- 窗口 ACL 授权；原先只有 `default.json` 且限定 `windows: ["main"]`，quick 窗口因此拿不到 `core:event` 权限。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/capabilities/quick.json` -- 新增 quick 窗口 capability，仅允许事件监听与取消监听。这是跨窗口同步失效的首要根因。
- [x] `src/lib/settings/api.ts` -- 新增 `THEME_PREVIEW_EVENT` 与 `previewTheme()`：本地应用后广播，广播失败静默降级。
- [x] `src/lib/settings/SettingsView.svelte` -- 主题下拉切换、保存失败回滚、组件销毁恢复三处改用 `previewTheme()`。
- [x] `src/routes/+layout.svelte` -- 监听主题预览事件并复用 `applyTheme()`；新增 `cancelThemePreview` 释放监听器。
- [x] 相关前端测试 -- `src/lib/settings/api.test.ts` 覆盖显式主题、system 清除覆盖、广播失败仍保留本地预览三种情况。

**Acceptance Criteria:**
- Given 主面板与 Quick 均已加载，when 用户切换主题下拉框，then 两个窗口无需保存或重启即显示相同主题。
- Given 设置保存失败，when 主面板恢复已保存设置，then Quick 同步恢复对应主题。
- Given 用户保存主题，when 后端完成设置事务，then现有 `settings_changed` 广播继续同步最终主题与语言。

### Review Findings

- [x] [Review][Patch] 将 Quick capability 从 `core:default` 缩小为事件监听与取消监听所需权限，避免开放无关的 path/window/webview/menu/tray 等 Core IPC [src-tauri/capabilities/quick.json:7]
- [x] [Review][Patch] 防止设置保存进行中销毁 SettingsView 时，`onDestroy` 用旧 `savedSettings` 广播并覆盖已经提交的新主题 [src/lib/settings/SettingsView.svelte:145]
- [x] [Review][Patch] 捕获两个异步事件监听器的注册失败，避免 capability 或窗口销毁问题产生未处理 Promise rejection 且同步静默失效 [src/routes/+layout.svelte:17]
- [x] [Review][Patch] 发送端事件契约由单测覆盖；接收端保持简单 wiring，以类型检查和生产构建验证，避免为监听器引入额外抽象 [src/lib/settings/api.test.ts:18]

## Spec Change Log

- 2026-08-29 -- 实施期间发现 spec 未预见的首要根因：`src-tauri/capabilities/default.json` 限定 `windows: ["main"]`，quick 窗口不匹配任何 capability，因此 `core:event:allow-listen` 被 ACL 拒绝，`+layout.svelte` 的 `settings_changed` 监听在 quick 窗口根本注册不上——即保存后也不同步，与 Intent 中"要重启应用才更新"的现象一致。应用自定义命令不走 ACL，所以 quick 面板取历史正常，掩盖了该权限缺失。同一原因还导致 `QuickPanel.svelte` 的 `history_changed` 监听失效（quick 面板打开期间不随新复制内容刷新），已随本次修复一并恢复。为遵循最小权限，quick 采用独立 capability，仅授予 `core:event:allow-listen` 与 `core:event:allow-unlisten`。

## Verification

**Commands:**
- `pnpm check` -- expected: Svelte 与 TypeScript 无错误。
- `pnpm test -- --run` -- expected: 前端测试全部通过。
- `pnpm build` -- expected: 生产构建成功。

**Manual checks:**
- Windows 同时打开主面板与 Quick，依次选择 light、dark、system，确认两者即时一致；模拟或观察保存失败时的回滚一致性。
