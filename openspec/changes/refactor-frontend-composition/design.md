## Context

前端当前已经是 feature-first 结构。`api.ts` 负责 Tauri transport，`HistorySession`、`PreviewSession` 和 updater store 负责长期状态，纯 presentation/keyboard/pager 模块负责可脱离 DOM 测试的决策。缺失的不是领域层，而是视图层的职责止损线和 CSS 所有权规则。

除明确的文本选择策略外，本变更采用行为冻结式迁移：先记录现状和测试，再移动模板与其样式，最后删除旧代码。拆分以“独立变化原因”判断，不以目标行数判断。

## Target frontend model

```text
routes                 应用入口和窗口级装配
  ↓
feature orchestrator   会话装配、DOM focus、跨子视图命令和生命周期
  ↓
feature components     展示与局部复合交互；通过 props/callbacks 通信
  ↓
feature state/logic    session/store 与可测试决策模块
  ↓
feature api.ts         唯一 Tauri invoke adapter
```

依赖保持单向。展示组件不得直接 `invoke()`，但可以调用同 feature 的明确 action callback。装配组件允许拥有 DOM focus，因为它正是 focus 副作用的边界；不为追求“纯”而把 DOM 操作包装成 service。

## Decisions

### 1. 文档先定义边界，不预先定义组件目录

在代码移动前，向两份 `docs/architecture*` 增加 Frontend composition 与 Style ownership。文档记录稳定判断规则：

- 继续按 feature 组织，而非横向复制 domain/application/infrastructure/presentation 目录。
- 状态必须有单一 owner；展示组件不镜像 session/store 状态。
- 组件仅在能独立变化、独立测试或隔离局部复杂交互时拆分。
- 简单 markup/CSS 重复不是自动建组件的理由；共享抽象遵循 rule of three。
- `app.css` 只承载 token、reset 和真正全局基础；自有 CSS 随拥有其 DOM 的组件。
- `:global()` 只用于 Bits UI 跨 Svelte 边界的 DOM、动态富文本或其他可说明的作用域边界，不作为跨业务组件共享机制。

具体组件名和迁移阶段留在本 OpenSpec，不写入长期架构文档。

### 2. Settings 按独立工作流拆分

目标结构不是强制一 tab 一文件：

```text
settings/
  SettingsView.svelte          加载/保存、Tabs、关闭和状态消息
  GeneralSettings.svelte       平台设置入口与 Preview capability 状态
  ShortcutSettings.svelte      录制状态、校验和快捷键展示
  UpdateSettings.svelte        更新状态 rail 与动作
  ReleaseNotes.svelte          发布列表、选择和详情
```

History、Appearance、About 等低复杂度模板可以继续留在 SettingsView，直到它们形成独立行为。`UpdateSettings` 直接读取既有 updater store，避免把完整 store 状态复制成大量 props；保存型 Settings 数据仍由父组件拥有并通过窄 props/callbacks 传递。

CSS 与对应模板一起移动。Settings 原有超长 CSS 正常格式化，删除已不存在的 update-card 规则，逐项处理 Svelte unused-selector 诊断后移除整块 `css_unused_selector` 忽略。不得把 Settings CSS 搬到全局文件来规避 scoped CSS。

### 3. History 只拆稳定的展示边界

```text
history/
  HistoryWorkspace.svelte      HistorySession/PreviewSession 装配、view/mode、focus、窗口命令
  AppTitleBar.svelte           品牌、应用菜单和菜单本地 focus 行为
  HistoryActionBar.svelte      操作菜单、删除确认和动作呈现
```

既有 HistoryList 与 ClipPreview 不改变边界。不新增只包裹 List/Preview/ActionBar 的 `HistoryView`。`onPanelShown`、`resumeBrowse`、`resetToLatest` 保留在 Workspace，因为它们协调 session、view 和 DOM focus，不是独立纯 lifecycle。

Bits UI 的受控 `menuOpen`、`appMenuOpen`、`deletePending` 继续作为控件真实状态；`mode` 继续表示键盘上下文。打开/关闭同步集中到命名函数，避免模板内散落赋值，但不引入第二套状态机或状态库。

纯键盘决策若能脱离 DOM，则扩展既有 `history/keyboard.ts`；不新增平行 `window-router.ts`。只有在拆分后出现真正独立且可用窄输入输出表达的召唤策略时，才另案抽取 panel lifecycle。

### 4. 本轮不建立共享 UI 层

报告提出的通用 Button、Switch、Keycap、Row 当前没有同时满足结构、语义和交互一致性。History、Quick 与 Onboarding 的 row 使用不同 ARIA、尺寸和内容，不合并。

`SettingRow` 也不作为前置条件。迁移完成后若至少三个设置子组件仍重复完全相同的 two-zone markup 和行为，可以在同一阶段末尾用净减码方式抽取；否则保留语义 HTML 与局部 CSS。

Spacing token 仅在触碰规则且值语义完全相同时采用。一次性几何和视觉校准值可以保留 px；不以 token 引用数作为验收标准。

### 5. 测试先于移动，验证行为而非库内部

每个阶段先补会因该次移动而失效的最小测试：

- Settings：加载/保存 ownership、快捷键录制、更新动作和发布记录键盘选择。
- History：应用菜单动作、操作菜单门控、删除确认 focus 回退，以及现有 window keyboard priority。
- CSS：`pnpm build` 必须无 CSS syntax/minify warning；`pnpm check` 不能替代生产构建。

测试调用既有 API/store mock，不为大组件建立新的测试 framework，也不复测 Bits UI 内部行为。

### 6. 文本选择采用全局默认与窄白名单

ClipClop 是命令面板而非网页文档。应用表面的静态标题、列表摘要、设置说明、菜单、状态、发布记录、文件路径和元信息统一使用默认箭头并禁止拖选，避免误现 I-beam 和蓝色选区。

该策略由 `app.css` 的应用级基础规则承载，而不是在每个子组件重复：

- 默认：`cursor: default`、`user-select: none` 和 WebKit 对应声明。
- 文本输入白名单：无 type/text/search/url/email/password 类型的文本 input、textarea 和明确的 contenteditable 恢复 `cursor: text` 与 `user-select: text`。
- 内容白名单：右侧 `.preview-body.text-preview` 恢复文本选择；它覆盖文本与链接正文，但不扩展到 preview meta、文件路径、颜色值或 loading placeholder。
- 按钮、菜单项和链接保留各自可感知的 hover/focus/点击行为，但其标签文字不可拖选；不统一强制网页式 pointer 手型。

这一条是本变更唯一有意调整的用户交互，不属于行为冻结。需要同步写入 `docs/interaction-contract.md`，并在 macOS/Windows WebView 中人工验证鼠标指针、拖选和复制。

## Migration plan

```text
架构/交互文档与 baseline
  → 文本选择策略
  → Settings characterization tests
  → Settings 子视图 + CSS 所有权迁移
  → History characterization tests
  → AppTitleBar
  → HistoryActionBar
  → 删除旧规则、依赖审计与文档核对
```

Settings 与 History 分开提交；History 的两个子组件也可独立提交。每批迁移都必须保持可运行，不允许先创建空组件库或兼容样式层等待后续消费者。

## Risks and mitigations

| 风险 | 缓解 |
|---|---|
| scoped CSS 在组件边界后不再命中 Bits UI DOM | 模板与 CSS 同批迁移，仅对真实跨边界后代使用局部 `:global()` |
| props/callbacks 数量反而增加耦合 | 超过窄输入输出时停止拆分，让 orchestrator 保留该职责 |
| menu/dialog 拆分改变焦点回退 | 移动前固化 invoker、Escape 和 close autofocus 测试 |
| 格式化 CSS 产生视觉变化 | 只格式化和移动声明，不改值；每批做浅/深主题人工核对 |
| 与既有前端架构 OpenSpec 重叠 | 本变更继承现有 HistorySession/API 边界，不重新设计已完成的逻辑层 |

## Verification

每阶段执行：

```sh
pnpm check
pnpm test
pnpm build
git diff --check
openspec validate refactor-frontend-composition --strict
```

最终在 macOS 与 Windows 验证主面板、Quick、Settings 各分类、菜单与删除确认、浅/深主题、键盘焦点、更新状态和文件预览入口。
