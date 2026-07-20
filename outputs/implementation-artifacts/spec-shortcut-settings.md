---
title: '设置模块无障碍与快捷键管理'
type: 'feature'
created: '2026-07-20'
status: 'done'
baseline_commit: '68891312602b309afd3d768c2771cd87b886abfa'
context: ['docs/architecture.md', 'DESIGN.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 重构后的设置视图可完成基本表单操作，但分类当前态、内容区名称、焦点迁移、状态播报和 11px 辅助文字仍有无障碍与可读性缺口；多组快捷键也缺少集中展示，模型中的全局快捷键仍被运行时固定值覆盖。

**Approach:** 保留轻量侧栏，新增独立“快捷键”分类，完整展示已实现键位，仅允许修改全局呼出快捷键；同步补齐设置页语义、焦点和状态反馈，并让 Rust 安全注册和持久化全局快捷键。

## Boundaries & Constraints

**Always:** 遵循现有模块化单体和 feature-first 边界；按当前平台显示键帽；面板、列表、文件导航和设置快捷键只读；可编辑全局组合必须包含修饰键和主键；注册、持久化、注销保持一致，失败时旧快捷键继续生效；全部操作支持标准键盘和可见焦点。

**Ask First:** 增加第二个可编辑快捷键；改变现有快捷键语义或设置窗口整体方向。

**Never:** 建立任意命令映射系统；覆盖系统复制、粘贴或常见窗口组合；增加底层键盘监听权限；仅靠颜色表达状态。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 展示 | 打开快捷键分类 | 按作用域显示当前平台键位 | 读取失败就地播报 |
| 保存 | 合法新组合 | 注册并持久化，旧组合失效 | 失败则恢复旧组合 |
| 非法/占用 | 无主键、保留或无法注册 | 当前设置不变 | 明确错误并保留焦点 |
| 恢复默认 | 选择默认值并保存 | 默认组合重新生效 | 失败保留原组合 |

</frozen-after-approval>

## Code Map

- `src/lib/settings/SettingsView.svelte` -- 设置导航、表单、焦点、播报和快捷键界面。
- `src/lib/settings/shortcuts.ts` -- 录制、平台显示与前端预校验纯函数。
- `src/lib/settings/api.ts` -- 设置 IPC 契约。
- `src-tauri/src/settings.rs` -- 设置默认值与快捷键验证。
- `src-tauri/src/commands/settings.rs` -- 保存时注册、持久化与回滚。
- `src-tauri/src/lib.rs` -- 启动时注册持久化快捷键。

## Tasks & Acceptance

**Execution:**
- [x] `src/lib/settings/SettingsView.svelte` -- 新增快捷键分类和唯一可编辑的全局呼出项；补齐导航当前态、区域标题、确认焦点、实时播报及辅助文字层级。
- [x] `src/lib/settings/shortcuts.ts`、`src/lib/settings/shortcuts.test.ts` -- 实现并测试跨平台录制、显示、非法及保留组合判断。
- [x] `src-tauri/src/settings.rs`、`src-tauri/src/commands/settings.rs`、`src-tauri/src/lib.rs` -- 验证并动态注册持久化快捷键，保证失败回滚，覆盖 Rust 测试。
- [x] `DESIGN.md`、`docs/testing.md` -- 同步快捷键分类和无障碍契约。
- [x] `src/lib/settings/SettingsView.svelte`、`src/routes/+page.svelte` -- 按用户实机复核改为纵向 tablist 双区键盘模型，调整侧栏比例、字号、内嵌焦点和 Esc 事件所有权。

**Acceptance Criteria:**
- Given 只使用键盘，when 打开设置、切换分类、录制或恢复快捷键、保存、取消清空并返回，then 全流程无需鼠标且焦点可预测。
- Given 使用屏幕阅读器，when 分类或异步状态变化，then 当前分类、内容区和结果可识别或播报。
- Given macOS 或 Windows 用户查看快捷键，when 页面完成加载，then 只显示对应平台键帽，且只有“呼出/隐藏 ClipClop”可编辑。

## Spec Change Log

- 2026-07-20：用户根据实机截图要求设置入口先聚焦侧栏，↑/↓自动切换分类，→进入详情，←返回侧栏；同时要求加宽侧栏并修复 Tab 焦点框。新增纵向 tablist、双区焦点与布局修正任务，避免原先“切换后强制聚焦标题”造成导航中断，以及主页面 Esc 捕获抢占设置内部状态。
- 2026-07-20：用户实测发现从侧栏按 Tab 后焦点会漂出可视区域。侧栏现显式接管正向 Tab，与 `→` 一样聚焦并滚动到当前详情首个控件；详情未加载时保留侧栏焦点，避免依赖 WebView 默认 Tab 顺序。

## Design Notes

侧栏使用纵向 tablist 与 roving tabindex，当前项以 `aria-selected` 标记；内容区是对应 tabpanel 并保留可见标题。录制按钮自身捕获按键，不使用全局键盘监听。辅助文字至少 12px 并有稳定行高。

## Verification

**Commands:**
- `pnpm test && pnpm check && pnpm build` -- 前端测试、类型检查和构建通过。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- Rust 格式正确。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` -- 无 lint 错误。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 测试通过。

**Manual checks (if no CLI):**
- macOS/Windows 实机确认快捷键切换、冲突回滚和完整 Tab 顺序。

## Suggested Review Order

1. [设置页键盘模型与快捷键界面](../../src/lib/settings/SettingsView.svelte#L82) — 先确认纵向导航、Tab/方向键迁移、布局比例与录制交互。
2. [快捷键转换和前端校验](../../src/lib/settings/shortcuts.ts#L1) 与 [对应单元测试](../../src/lib/settings/shortcuts.test.ts#L1) — 检查跨平台键位、保留组合和显示格式。
3. [设置保存事务](../../src-tauri/src/commands/settings.rs#L22) 与 [启动注册回退](../../src-tauri/src/lib.rs#L63) — 检查新旧全局快捷键的注册、持久化和失败安全性。
4. [Rust 快捷键验证](../../src-tauri/src/settings.rs#L1) — 确认后端不会信任前端输入。
5. [主面板 Esc 事件边界](../../src/routes/+page.svelte#L1) — 确认设置页内部状态优先处理 Esc。
6. [设计契约](../../DESIGN.md) 与 [测试清单](../../docs/testing.md) — 对照最终交互和实机验证范围。
