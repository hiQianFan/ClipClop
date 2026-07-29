---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - src/routes/+page.svelte
  - src/lib/clips/api.ts
  - src/lib/clips/types.ts
  - src/lib/clips/view.ts
  - src-tauri/src/clips/service.rs
  - src-tauri/src/commands/clips.rs
  - src-tauri/src/commands/preview.rs
  - src-tauri/src/storage/database.rs
workflowType: research
lastStep: 3
research_type: technical
research_topic: maintainable-open-source-clipboard-desktop-architecture
research_goals: 为 ClipClop 选择健壮、可维护且不过度设计的前后端模块边界
user_name: qianfan
date: 2026-07-28
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-07-28
**Author:** qianfan
**Research Type:** technical

---

## Research Overview

本报告对照成熟开源剪贴板工具和 Tauri、Svelte 官方设计，评估 ClipClop 的模块边界、状态所有权、IPC 与持久化结构。研究优先使用官方仓库和官方文档；项目规模不等同于架构质量，结论会区分可借鉴模式与复杂度负担。

---

## Technical Research Scope Confirmation

**Research Topic:** 可维护的开源剪贴板桌面应用架构
**Research Goals:** 为 ClipClop 选择健壮、可维护且不过度设计的前后端模块边界

**Technical Research Scope:**

- Architecture Analysis — 对比 Maccy、CopyQ、PasteBar 等项目的系统边界
- Implementation Approaches — 分析功能模块、状态与 UI 行为所有权
- Technology Stack — 分析原生 UI、Qt、Tauri/WebView 的取舍
- Integration Patterns — 分析前端、IPC、领域服务、存储之间的依赖方向
- Performance Considerations — 关注剪贴板轮询、预览资源、分页和异步竞态

**Research Methodology:**

- 使用当前公开的一手资料
- 关键结论进行多来源交叉验证
- 明确事实、推断和适用于 ClipClop 的建议

**Scope Confirmed:** 2026-07-28

## Technology Stack Analysis

### Programming Languages

对标项目代表三条成熟路线：

- **Maccy：Swift + SwiftUI/SwiftData。** 它是 macOS 专用、键盘优先的轻量工具；2.x 将 UI 从 AppKit/NSMenu 改写为 SwiftUI/NSPanel，并将存储从 Core Data 改为 SwiftData。原生栈能直接使用系统窗口、焦点和粘贴板语义，但跨平台复用有限。
- **CopyQ：C++ + Qt。** 它覆盖 macOS、Windows、Linux，并承载编辑、脚本、命令行、自定义标签等大量能力。Qt 适合大型跨平台桌面产品，但其复杂度与 ClipClop 当前目标不匹配。
- **PasteBar：Rust + Tauri + React/TypeScript。** 与 ClipClop 的 Rust + Tauri + Svelte/TypeScript 最接近，证明 WebView 前端与 Rust 系统层适合跨平台剪贴板应用；同时其 React Query、Signals、Jotai、Zustand 并存，说明大型产品可能形成多套状态机制，不能把依赖数量当作健壮性的证据。

ClipClop 当前语言选择合理：Rust 处理系统剪贴板、窗口、快捷键和持久化；TypeScript/Svelte 处理呈现与交互。没有技术证据支持为了“架构纯度”改写语言。

来源：[Maccy 官方仓库](https://github.com/p0deje/Maccy)、[Maccy 2.0 架构迁移说明](https://github.com/p0deje/Maccy/discussions/818)、[CopyQ 官方仓库](https://github.com/hluk/copyq)、[PasteBar 官方仓库](https://github.com/PasteBar/PasteBarApp)

### Development Frameworks and Libraries

Tauri 官方架构把应用定义为 Rust 主进程与 WebView 前端的组合，前端通过消息传递调用系统能力。Tauri 不规定前端框架，也不替应用建立领域架构；因此 Svelte 组件不应直接承担数据库、系统窗口或粘贴板规则。

Svelte 5 的 runes 适合建立与组件生命周期一致的局部响应式状态。对 ClipClop 而言，历史页面需要一个局部 session/model 管理分页、选择和预览异步状态，而不是新增全局状态库。只有跨视图持续存在的能力（例如下载中的更新器）才适合模块级 store。

框架边界建议保持为：

`Svelte UI → TypeScript feature API → Tauri command → Rust domain/service → storage/platform adapter`

来源：[Tauri 架构](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md)、[Tauri 前端调用 Rust](https://v2.tauri.app/develop/calling-rust/)、[Svelte 官方文档](https://svelte.dev/docs/svelte/what-are-runes)

### Database and Storage Technologies

成熟项目采用的数据库不同，但共同点是持久化属于应用核心而不是 UI：

- Maccy 2.x 使用 SwiftData，并特别保留从旧存储升级和降级的兼容性。
- PasteBar 使用 Diesel ORM 和 SQLite。
- ClipClop 使用 `rusqlite` + bundled SQLite，依赖更少且足以支持当前单机历史数据。

ClipClop 不需要为了 DDD 引入 ORM 或 repository interface。当前真正的健壮性缺口是 **schema migration**：发布后的本地历史不能依靠“删库重建”升级。最小正确设计是在现有 database 模块内增加版本化迁移，而不是更换存储技术。

来源：[Maccy 2.0 存储迁移说明](https://github.com/p0deje/Maccy/discussions/818)、[PasteBar 技术栈](https://github.com/PasteBar/PasteBarApp)

### Development Tools and Platforms

ClipClop 已使用与此类项目相符的轻量工具链：

- Vite、SvelteKit static adapter、TypeScript 与 `svelte-check`
- Vitest 进行前端纯逻辑测试
- Cargo test、Clippy 与 rustfmt 进行 Rust 验证
- Tauri CLI 负责开发、打包和平台配置

可维护性主要来自边界和可运行检查，不来自新增工具。当前无需引入 Storybook、状态管理库、DI 框架或前端 repository 层。应优先给 HistorySession 的竞态/选择规则、数据库迁移和键盘交互留下最小测试。

来源：[Tauri 开发文档](https://v2.tauri.app/develop/)、[Tauri 配置文件](https://v2.tauri.app/develop/configuration-files/)

### Cloud Infrastructure and Deployment

本地优先剪贴板工具的核心路径不需要云基础设施。Maccy 强调本地、安全和轻量；CopyQ 也将历史持久化在本机。ClipClop 当前采用本地 SQLite、平台剪贴板和桌面更新器，符合产品边界。

云同步、容器、服务端 API、消息队列均不应进入当前架构。只有产品明确增加多设备同步后，才需要把同步作为独立 bounded context，并处理加密、冲突解决和身份边界。

来源：[Maccy 官方仓库](https://github.com/p0deje/Maccy)、[CopyQ 官方仓库](https://github.com/hluk/copyq)

### Technology Adoption Trends

对标项目呈现的趋势不是统一到某个框架，而是：

1. 系统能力继续留在原生层；
2. UI 技术可演进，但历史、粘贴与存储语义保持稳定；
3. 键盘优先是剪贴板工具的核心交互能力；
4. 复杂产品会自然出现插件、脚本和多状态系统，但轻量产品不应提前承担这些成本。

对 ClipClop 的技术栈结论是：**保留现有 Rust/Tauri/Svelte/SQLite；重构边界，不更换框架，不新增状态依赖。**

置信度：高。技术栈事实来自官方仓库和官方框架文档；“不新增状态库”是结合 ClipClop 当前规模作出的架构判断。

## Integration Patterns Analysis

### API Design Patterns

ClipClop 是单进程桌面应用，不需要 REST、GraphQL、gRPC 或 API Gateway。它真正的 API 是 Tauri command：

- 前端使用 `invoke` 发出有返回值的请求；
- Rust command 负责反序列化、调用应用能力并返回可序列化结果；
- `src/lib/clips/api.ts` 集中保存 command 名称和 TypeScript 返回类型，避免组件散落字符串调用。

这一点已经设计正确。需要继续收紧的是 Rust command 的职责：command 应是薄 IPC adapter；预览文件解析、缓存清理等规则应落到对应应用服务或平台模块。`commands/preview.rs` 当前比 `commands/clips.rs` 更接近“功能实现层”，边界不一致。

推荐调用链：

`component → HistorySession → clips/api.ts → Tauri command → ClipService/PreviewService → Database/platform`

不建议引入通用 `CommandBus`、前端 repository interface 或自动生成 RPC client；当前十余个稳定命令不足以抵消这些抽象成本。

来源：[Tauri Commands 官方文档](https://v2.tauri.app/develop/calling-rust/)、[Tauri IPC 概念](https://v2.tauri.app/concept/inter-process-communication/)

### Communication Protocols

Tauri 提供两种适合 ClipClop 的 IPC 语义：

- **Commands**：请求—响应，适用于查询分页、读取详情、复制、粘贴、删除、打开预览。
- **Events**：单向 fire-and-forget，适用于 `clips_changed`、`panel_shown` 这类由原生生命周期触发的失效通知。

ClipClop 当前选择基本正确：业务操作走 command，系统变化走 event。事件载荷应保持为“发生了什么”，而不是复制一份前端状态；收到 `clips_changed` 后重新查询权威数据，比通过事件增量拼接 UI 列表更稳健。

不需要 WebSocket、消息队列或进程内 event bus。Tauri 官方还提醒异步事件可能乱序，因此事件不应承担选择状态或分页事务。若未来出现大文件流式传输，再针对该路径使用 Tauri Channel；目前 data URL 预览已经足够，是否优化应由测量决定。

来源：[Tauri IPC：Events 与 Commands](https://v2.tauri.app/concept/inter-process-communication/)、[Tauri 前端调用 Rust](https://v2.tauri.app/develop/calling-rust/)

### Data Formats and Standards

Command 参数与返回值通过 serde/JSON 边界传输，当前 `ClipPage`、`ClipDetail` 和 `PasteOutcome` 都是适合 IPC 的显式 DTO。该边界应保持：

- Rust 内部错误转换为稳定、可序列化的应用错误；
- 前端 API 文件把 command DTO 映射为 TypeScript 类型；
- 二进制预览不混入列表 DTO，按需单独读取；
- 列表返回摘要，详情返回完整类型信息。

无需引入 Protobuf、MessagePack 或共享 schema 生成器。真正需要防止的是 Rust 与 TypeScript 类型手工漂移；在当前规模下，用 command 集成检查和少量契约测试即可。

来源：[Tauri command 参数、返回值与错误处理](https://v2.tauri.app/develop/calling-rust/)

### System Interoperability Approaches

三个对标项目都把系统互操作视为独立职责：

- Maccy 直接使用 macOS Pasteboard、Panel 与 Accessibility；
- CopyQ 通过 Qt 加平台实现覆盖 X11、Wayland、Windows、macOS；
- PasteBar 和 ClipClop 通过 Rust/Tauri 层隔离平台能力。

ClipClop 的 `clipboard/`、`paste.rs`、`window/` 平台模块方向正确。领域层不应知道 Svelte 焦点，也不应把平台窗口对象传入数据库；UI 同样不应自行操作系统剪贴板。复制与自动粘贴必须保留为不同用例，因为后者还涉及隐藏面板、恢复目标窗口及权限降级。

CopyQ 的公开问题也表明“粘贴到先前聚焦窗口”在不同桌面环境下不是普通 clipboard write，而是独立、可能降级的系统工作流。ClipClop 的 `PasteOutcome` 显式表达降级结果，是比简单 `Result<()>` 更健壮的设计。

来源：[CopyQ 功能与键盘操作](https://github.com/hluk/copyq)、[CopyQ 跨桌面焦点问题](https://github.com/hluk/copyq/issues/1601)

### Event-Driven Integration

事件适合触发“重新同步”，不适合拥有交互状态：

1. 原生 clipboard monitor 捕获内容；
2. ClipService 去重并持久化；
3. Rust 发出 `clips_changed`；
4. HistorySession 使当前查询失效并刷新；
5. Session 根据刷新结果保留、替换或清除 selection；
6. UI 根据 session 状态重新渲染。

`panel_shown` 同理：它表示一次应用生命周期变化，HistorySession 可以重置到最新历史；但 DOM focus 必须由已挂载的 HistoryView 在下一次渲染后恢复。Tauri 官方特别提示组件 setup/mount 期间事件可能早于 DOM 可用，并要求卸载时清理监听器；这支持把订阅生命周期放在 HistoryView/Session 的明确创建与销毁位置。

不建议使用 event sourcing、CQRS 或内部 publish-subscribe 层。SQLite 才是历史权威，事件只是失效信号。

来源：[Tauri 事件系统与 Svelte 清理示例](https://v2.tauri.app/develop/calling-rust/#event-system)、[Svelte 生命周期](https://svelte.dev/docs/svelte/lifecycle-hooks)

### Frontend State and Focus Integration

选择状态与 DOM 焦点是相关但不同的状态：

- `selectedId` 是 HistorySession 的业务交互上下文；
- `document.activeElement` 是浏览器当前输入目标；
- 点击预览或翻页按钮可以改变 DOM 焦点，但不应清空 `selectedId`；
- HistoryView 统一判断当前区域是否是临时输入模式，再把历史快捷键路由给选中项；
- Search、菜单、确认框和文件 tablist 拥有各自短暂的键盘模式，应阻止历史快捷键穿透。

因此不应让每个按钮在点击后各自 `listbox.focus()`。那是分散修补，并且会破坏按钮、菜单和辅助技术语义。正确集成点是 HistoryView 的键盘路由与焦点协调；HistoryList 只拥有 listbox 内导航并暴露一个窄 `focus()`。

这是 ClipClop 当前焦点缺陷的根因：快捷键与 listbox DOM focus 绑定，而不是与仍然存在的 history interaction context 绑定。

### Integration Security Patterns

本地桌面应用不需要 OAuth、JWT 或 mTLS，但 IPC 仍是安全边界：

- 所有 command 输入继续在 Rust 信任边界验证；
- 文件索引、分页大小、查询长度和路径访问不能只依赖前端；
- Tauri capability 应按窗口和插件授予最小权限；
- 不向远程页面开放 command；
- 预览和 opener 只能处理数据库中已捕获、经过验证的资源。

Tauri 官方说明 capability 可以限制各窗口/WebView 可用权限，但自定义 command 默认对注册它们的应用窗口可用。ClipClop 若未来增加独立设置窗口或远程内容 WebView，应再拆 capability；当前单一可信本地 WebView 不需要创建复杂权限矩阵。

来源：[Tauri Capabilities](https://v2.tauri.app/security/capabilities/)、[Tauri Permissions](https://v2.tauri.app/security/permissions/)

### Integration Conclusions

ClipClop 不需要分布式架构。最稳健、最小的集成模型是：

```text
platform clipboard/window
        ↓ event
Rust application/domain service
        ↕ command DTO
feature API
        ↕
HistorySession
        ↓
HistoryView / HistoryList / PreviewPane
```

依赖只能向下；事件只能用于失效和生命周期通知；selection 与异步请求一致性由 HistorySession 管理；DOM focus 与键盘模式由 HistoryView 管理。

置信度：高。IPC、安全和生命周期事实来自 Tauri/Svelte 官方文档；模块落点是结合 ClipClop 当前调用链作出的设计判断。
