# ClipClop 架构说明

简体中文 | [English](architecture.md)

本文记录 ClipClop 演进过程中必须保持稳定的边界，重点是运行时职责、隐私敏感流程和面板生命周期，而不是逐一罗列源文件。

## 运行时结构

ClipClop 包含两层：

- `src/` 是 Svelte 界面。功能代码通过各自的 `api.ts` 调用 Tauri command，负责 DOM 焦点、键盘交互、渲染和无障碍。
- `src-tauri/src/` 是 Rust 宿主，负责系统剪贴板、持久化、全局快捷键、原生窗口、预览、自动粘贴、日志和更新。

Tauri command 只是适配层。业务规则放在对应功能模块，原生窗口行为统一封装在 `window` 模块之后。

历史界面按状态所有权拆分：`HistorySession` 只拥有列表、分页和选中项；`PreviewSession` 只拥有资源 URL、缩略图、缓存、防抖与请求作废版本。`HistoryWorkspace` 负责两者的调用顺序，并继续拥有 DOM 焦点、键盘路由和多文件 `fileIndex` 游标。不得让 Session 反向操作组件状态，也不得复制同一份运行时状态。

### 前端组合

前端继续按 feature 组织，不在每个 feature 内照搬后端式 `domain`、`application`、`infrastructure` 或 `presentation` 分层。Route 负责窗口装配；feature orchestrator 负责 Session 装配、DOM 焦点、跨子视图命令和生命周期；feature component 负责自身模板与局部复合交互；Session、store 与纯逻辑模块负责长期状态和可测试决策；各 feature 的 `api.ts` 保持为 Tauri transport 边界。依赖按此方向单向流动，展示组件不得直接调用原始 Tauri IPC。

每份运行时状态只有一个 owner。子组件通过窄值和 callback 协作，不镜像 Session 或 store 状态。只有职责能独立变化、能隔离局部交互或能独立测试时才拆组件；单纯重复 markup 或 CSS 不足以建立抽象。共享 primitive 至少需要三个等价消费者，并且必须净减维护成本。

### 样式所有权

根目录 `DESIGN.md` 是产品级设计约束的权威文件；带日期的 `_bmad-output/**/DESIGN.md` 只是具体工作流产物，不能覆盖根文档。

`src/app.css` 只承载设计 token、reset 和真正全局的基础规则。自有 markup 的 CSS 留在渲染它的组件中。`:global()` 只用于 Bits UI 渲染后代、明确的动态富文本等真实作用域边界，不能作为跨 feature 共享样式的机制。现有 token 能准确表达意图时应复用；一次性几何保留在局部，不为它预建 token 或共享 UI 层。

Rust 宿主保持具体服务，不为单一 SQLite 或单一平台实现增加 Repository、Factory 或 DI 接口。`AssetService` 负责 WebView 内的资源读取和缩略图，`ExternalPreviewService` 负责 Quick Look 等外部预览及其临时文件生命周期；两者只共享可克隆的 `HistoryService` 句柄。平台粘贴实现位于 `paste` 的平台子模块，设置模型、快捷键规则和持久化服务分别放在 `settings` 子模块。数据库迁移和设置查询位于 `storage` 的独立实现文件，但继续由同一个具体 `Database` 类型提供。

## 面板生命周期

主窗口是临时命令面板，不是普通应用窗口。原生窗口焦点和 DOM 焦点是两个不同层次：

1. Rust 开始一个新的面板 generation。
2. Rust 调整并显示原生窗口。
3. 平台适配器请求前台激活。
4. Rust 发送 `panel_shown`。
5. Svelte 处理 `panel_shown` 并设置 DOM 焦点。

不得在原生显示和激活尝试之前发送 `panel_shown`，否则 WebView 中的元素可能已经获得 DOM 焦点，而原生窗口仍处于后台。

收到 `panel_shown` 时前端不会无条件重置。Settings 与引导页是刻意进入的模式，呼出时予以保留（底层历史会话通过 `history_changed` 保持实时，退出时即为最新）。在历史视图内，`restore_browse_position` 独立保留上次的页码和选中项，`preserve_search_conditions` 独立保留搜索词和筛选条件；两项默认均关闭。

### 状态模型

`PanelLifecycleState` 是原生生命周期的唯一事实来源：

```text
Hidden → Showing → Focused → BlurPending → Hidden
                    ↑             |
                    └─────────────┘
```

每次显示都会开启新的 generation。待处理失焦令牌包含 generation 和 revision；重新显示、重新聚焦或隐藏都会让旧令牌失效。窗口至少真正获得一次焦点前，启动阶段的失焦事件会被忽略。

`PreviewState` 与生命周期分离，只表示原生预览是否活跃。原生预览可以临时取得焦点，而不触发主面板隐藏。

### 键盘命令优先级

DOM 焦点用于选择输入上下文，不能让窗口级命令失效。Svelte Workspace 只维护一个窄窗口路由器，优先级为：已处理事件停止；面板关闭（`Command/Ctrl+W`）不依赖焦点；Escape 只退出一层；随后由焦点控件处理原生按键；只有 Browse 上下文拥有列表方向键和列表动作。不得把列表导航提升到窗口路由，也不得在各控件中重复实现面板命令。

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

## 持久化与历史生命周期

数据库结构变化必须递增 `SCHEMA_VERSION`，并明确迁移所有仍受支持的已发布结构。v5 将不可变的创建时间与最后使用时间分开保存；启用“最近使用移到顶部”时，按时间保留会依据最后使用时间。迁移后的数据库不支持降级到 `0.1.x`。

历史限制在捕获和设置更新边界执行。时间限制与条目数量限制相互独立，同时启用时两者都生效。首次启动快速入门使用固定的内置示例和本地资源，绝不读取真实剪贴板历史。

删除历史时先清理外部预览缓存，再提交数据库删除。缓存清理失败必须保留数据库记录，使操作可以重试，避免产生失去持久化标识的敏感孤儿文件；这不是可交换的实现细节。

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
