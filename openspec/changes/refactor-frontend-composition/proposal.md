## Why

ClipClop 前端已经按 `history`、`settings`、`updater`、`onboarding` 形成稳定的功能边界，纯逻辑和状态模块也有良好测试；当前维护风险来自视图装配层继续增长：`HistoryWorkspace.svelte` 同时承载应用菜单、历史动作栏、删除确认和窗口编排，`SettingsView.svelte` 同时承载多类设置流程与不可审查的超长 CSS。最近一次非法 scoped CSS 令整个设置页失去样式，证明该结构已经影响可靠性。

本变更先把既有前端边界写入架构文档，再按真实职责拆分高风险组件。采用轻量功能切片，不把 Rust 领域模型在前端复制成完整 DDD/Clean Architecture。

## What Changes

- 先更新中英文架构文档，规定前端功能切片、装配组件、展示组件、逻辑模块、IPC adapter 和样式所有权的依赖方向。
- 将 Settings 中独立且高变化的 General、快捷键、更新状态和发布记录拆出；SettingsView 继续拥有加载、保存、分类导航和页面生命周期。
- 将 History 的标题栏/应用菜单与动作栏/删除确认拆出；HistoryWorkspace 继续拥有会话装配、DOM focus、键盘上下文和视图切换。
- 让被抽取视图的 CSS 随其组件移动，清理 Settings 死选择器和整块 CSS 忽略；保留 Bits UI 或动态 HTML 边界所需的局部 `:global()`。
- 在移动行为前补最小 characterization tests，并在每个阶段运行生产构建以拦截最终 CSS 生成错误。
- 明确桌面应用的文本选择策略：静态界面使用默认鼠标指针且不可选择，仅文本输入/编辑控件和右侧正文预览恢复文本光标与选择复制。
- 仅在拆分后出现至少三个语义与结构均一致的消费者时，另案评估共享 UI primitive；本变更不预建组件库。

## Capabilities

### New Capabilities

- `frontend-composition`: 定义 ClipClop 前端功能切片、组件职责、样式所有权和行为冻结式迁移要求。
- `desktop-text-selection`: 统一桌面 UI 的静态文本指针/选择规则，并保留编辑与正文预览能力。

### Modified Capabilities

- 无。除明确新增的静态文本选择策略外，布局、键盘、焦点、更新、预览和设置行为保持不变。

## Non-Goals

- 不引入完整 DDD/Clean Architecture 目录、Repository、UseCase class、DI、事件总线或新状态库。
- 不建立通用 Button、Row、Keycap 或完整 `ui/` 组件库。
- 不按行数、文件数、`:global()` 数量或 token 引用次数驱动拆分。
- 不全仓机械替换 px 或强制接通全部 spacing token。
- 不重写现有 `HistorySession`、`PreviewSession`、presentation、keyboard、pager 或 updater store。
- 不改变视觉、文案、Tauri IPC、Rust 后端、数据库或平台行为。
- 不禁止输入框内部选择，也不禁用右侧文本/链接正文预览的选择和复制。

## Success Signal

- 架构文档能明确回答一个前端行为、状态、IPC 调用和 CSS 分别应由哪一层拥有；Settings 和 History 的高变化视图可以独立修改和测试，而无需重写其编排状态。
- 全部自动化门禁和生产构建通过，macOS/Windows 的设置、历史、键盘、焦点、更新和预览行为与迁移前一致。

## Impact

- 文档：`docs/architecture.md`、`docs/architecture.zh-CN.md`、`docs/interaction-contract.md`。
- 前端：`src/lib/settings/*`、`src/lib/history/HistoryWorkspace.svelte` 及新增的同功能目录组件和测试。
- 不新增依赖，不修改 `src-tauri/`。
