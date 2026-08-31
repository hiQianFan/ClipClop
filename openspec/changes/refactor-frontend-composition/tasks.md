## 1. 文档和行为基线

- [x] 1.1 在 `docs/architecture.md` 增加 Frontend composition 与 Style ownership，记录 feature-first、单向依赖、状态单 owner、组件拆分条件和 `:global()` 边界
- [x] 1.2 同步 `docs/architecture.zh-CN.md`，明确根 `DESIGN.md` 是产品设计约束的权威文件，`_bmad-output/**/DESIGN.md` 是具体工作流产物
- [x] 1.3 在 `docs/interaction-contract.md` 增加桌面文本选择契约：静态 UI 禁止选择，文本输入/编辑控件和右侧正文预览允许选择
- [x] 1.4 记录迁移前 `pnpm check`、`pnpm test`、`pnpm build` 结果；生产构建不得包含 CSS syntax/minify warning

## 2. 桌面文本选择策略

- [x] 2.1 在 `app.css` 设置应用表面的默认箭头、`user-select:none` 和 WebKit 对应声明，不在各组件重复规则
- [x] 2.2 为文本型 input、textarea 和明确 contenteditable 恢复文本光标与选择，不误将 checkbox/switch 当作文本输入
- [x] 2.3 仅为右侧 `.preview-body.text-preview` 恢复正文选择复制；preview meta、文件路径、颜色值、loading、发布记录和其他静态 UI 保持不可选择
- [ ] 2.4 macOS 与 Windows 人工验证静态文本无法拖选、搜索/练习输入可编辑选择、右侧文本与链接正文可选择复制

## 3. Settings 护栏

- [x] 3.1 补充 Settings 加载、保存、取消还原和分类切换 ownership 测试
- [x] 3.2 固化快捷键录制的开始、取消、无效组合、恢复默认和保存行为
- [x] 3.3 固化更新状态动作、发布记录方向键选择和 release detail 行为
- [x] 3.4 固化 Windows Preview capability 与 macOS 系统设置入口的条件呈现

## 4. Settings 拆分

- [x] 4.1 抽出 `GeneralSettings.svelte`，父组件继续拥有可保存 Settings，子组件拥有 capability 查询和平台入口的局部状态
- [x] 4.2 抽出 `ShortcutSettings.svelte`，移动录制状态、校验、显示和对应测试/CSS
- [x] 4.3 抽出 `UpdateSettings.svelte`，复用现有 updater store，不复制异步生命周期状态
- [x] 4.4 抽出 `ReleaseNotes.svelte`，封装发布列表选择、键盘导航、loading/error/detail
- [x] 4.5 SettingsView 仅保留页面生命周期、保存/回滚、Tabs 和尚无独立行为的简单内容
- [x] 4.6 将 CSS 随 DOM owner 移动并正常格式化；清理死选择器，移除整块 `css_unused_selector` 忽略
- [x] 4.7 仅在三个以上 Settings 子组件仍有完全相同 two-zone 实现且能够净减码时抽取 `SettingRow`，否则记录无需抽取

## 5. History 护栏

- [x] 5.1 固化应用菜单互斥、动作触发、Escape 和关闭焦点行为
- [x] 5.2 固化操作菜单 preview 门控、复制/粘贴/删除动作和菜单关闭焦点
- [x] 5.3 固化删除确认的 invoker、取消/失败回退、成功后 successor selection 与 list focus
- [x] 5.4 固化 window command priority，确保拆分后不重复处理快捷键

## 6. History 拆分

- [x] 6.1 抽出 `AppTitleBar.svelte`，移动品牌、应用菜单及其局部 CSS；Workspace 通过窄 callbacks 提供设置、更新、关于和退出动作
- [x] 6.2 抽出 `HistoryActionBar.svelte`，移动操作菜单、删除确认及其局部 CSS；不移动 HistorySession/PreviewSession ownership
- [x] 6.3 将 Bits UI open 状态与 keyboard mode 的同步集中到命名函数，删除模板内重复赋值，不新增状态库
- [x] 6.4 可脱离 DOM 的新增键盘决策合并进现有 `history/keyboard.ts`；保留 panel summon/focus 编排在 Workspace
- [x] 6.5 确认没有新增无行为的 `HistoryView` wrapper、通用 Row/Button/Keycap 或跨 feature 全局样式

## 7. 验收

- [x] 7.1 审计展示组件不直接 `invoke()`，Session/store 不接触 DOM，CSS 随 DOM owner，依赖保持单向
- [x] 7.2 执行 `pnpm check`、`pnpm test`、`pnpm build`、`git diff --check`
- [x] 7.3 执行 `openspec validate refactor-frontend-composition --strict`
- [ ] 7.4 macOS 与 Windows 人工验证主面板/Quick/Settings、菜单/确认、浅深主题、键盘焦点、更新与 Preview capability
- [x] 7.5 按文档、文本选择、Settings、AppTitleBar、HistoryActionBar 分批提交，每批可独立回滚

## 验收记录

- 2026-08-31：`pnpm check` 通过（0 errors / 0 warnings）；`pnpm test` 通过（20 files / 98 tests）；`pnpm build` 通过且无 CSS syntax/minify warning；`git diff --check` 通过；OpenSpec strict validate 通过。
- 2026-08-31：macOS 实机检查发现正文预览空白区错误显示文本指针，已在 `3509205` 将白名单收窄到实际正文 `pre`，用户复核通过；Windows 因当前无可用设备仍待验证，故 2.4 与 7.4 保持未完成。
- 提交边界：文档 `af3a049`；文本选择 `e0cc705`、`3509205`；Settings `74333aa`；History chrome `611b164`；行为修复 `f339b00`；测试 `b374ff0`；CSS ownership `1f785f6`。
