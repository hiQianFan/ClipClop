---
stepsCompleted: [1, 2, 3]
inputDocuments: []
workflowType: 'research'
lastStep: 3
research_type: 'technical'
research_topic: 'clipboard history deduplication and recency promotion'
research_goals: '评估 ClipClop 跨时间重复内容的识别、成本、竞品行为与最小正确实现'
user_name: 'qianfan'
date: '2026-08-22'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-08-22
**Author:** qianfan
**Research Type:** technical

---

## Research Overview

评估 ClipClop 如何识别完全重复的剪贴板内容，并在再次复制时复用旧记录、更新时序并置顶；同时核验主流剪贴板工具的行为，并评估 SHA-256 与 SQLite 索引的成本。

---

## Technical Research Scope Confirmation

**Research Topic:** clipboard history deduplication and recency promotion
**Research Goals:** 评估 ClipClop 跨时间重复内容的识别、成本、竞品行为与最小正确实现

**Technical Research Scope:**

- Architecture Analysis - 捕获、哈希、持久化与历史排序
- Implementation Approaches - 全历史去重、原记录提升与事务边界
- Technology Stack - Rust、RustCrypto SHA-2、SQLite/rusqlite
- Integration Patterns - clipboard watcher、HistoryService 与数据库
- Performance Considerations - 大内容哈希、索引查询与写放大

**Research Methodology:**

- 当前公开资料与开源源码交叉核验
- 以本地代码路径确认 ClipClop 的现状
- 对不公开实现的商业软件明确标注证据限制

**Scope Confirmed:** 2026-08-22

## Technology Stack Analysis

### Programming Languages

ClipClop 的相关路径已经由 Rust 实现：剪贴板读取后使用 `sha2::Sha256` 对各 flavor 的格式名与 payload 流式更新哈希，再把十六进制摘要写入 SQLite。无需引入新语言或把去重移到 Svelte 前端。RustCrypto 官方文档确认 `sha2` 是纯 Rust 的 SHA-2 实现，并提供增量 `Digest` API。

_Source: https://docs.rs/sha2/latest/sha2/_

### Development Frameworks and Libraries

现有 `sha2`、`hex`、`chrono` 与 `rusqlite` 已覆盖内容指纹、时间戳和事务数据库操作。Maccy 的公开 Swift 源码也表明成熟的桌面剪贴板工具通常在原生捕获/持久化层处理历史项，而不是在列表 UI 中事后清理。

_Sources: https://github.com/RustCrypto/hashes · https://github.com/p0deje/Maccy/blob/master/Maccy/Clipboard.swift_

### Database and Storage Technologies

SQLite 是正确边界。当前表已经保存 `content_hash`，但仅有历史排序索引；全历史哈希查找若没有 `content_hash` 索引会退化为扫描。SQLite 官方说明查询规划器依赖应用提供合适索引，因此最小数据库变化应围绕现有列增加索引，并把“查找重复项 + 更新时间/元数据”放在同一数据库操作中。

_Sources: https://www.sqlite.org/queryplanner.html · https://www.sqlite.org/lang_createindex.html_

### Development Tools and Platforms

本议题不需要新的开发平台。现有 Cargo 测试和临时 SQLite 数据库足以验证：相隔较久的相同内容只保留一条、记录移动到顶部、总数不增加、不同内容不合并。`rusqlite` 提供事务抽象，可用于保证复用旧记录时的原子性。

_Source: https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html_

### Cloud Infrastructure and Deployment

不适用。ClipClop 的历史和去重均为本机能力，不应增加云服务、网络请求或远端内容分析。

### Technology Adoption Trends

公开证据显示去重是成熟剪贴板管理器中的常见预期：Maccy 维护者明确说明产品会去重，并指出富文本的动态附加数据可能让视觉上相同的内容逃过去重；其 2.7.0 发布说明也继续修复由 transient pasteboard types 引起的重复项。这说明难点主要是“内容等价性的定义”，而非 SHA-256 本身。

_Sources: https://github.com/p0deje/Maccy/discussions/818 · https://github.com/p0deje/Maccy/blob/master/appcast.xml_

## Integration Patterns Analysis

### API Design Patterns

这里不需要 REST、GraphQL、gRPC 或 webhook。正确集成点是现有同步领域 API：将 `HistoryService::capture` 背后的“近期存在检查 + 插入”替换为单个数据库操作 `capture_or_promote`。它返回实际生效的记录 ID：新内容返回新 ID，重复内容返回被提升的旧 ID，使现有 `history_changed.latest_id` 事件自然刷新并选中第一项。

### Communication Protocols

系统剪贴板 watcher 仍是唯一输入，Tauri `history_changed` 仍是唯一 UI 失效通知。重复捕获不能继续返回 `None`，否则后端虽提升记录，前端却不会刷新。无需增加消息队列、WebSocket 或新事件类型。

### Data Formats and Standards

现有哈希表达的是“完整 clipboard flavor bundle 完全相同”：依次包含每个 flavor 的格式名和原始 payload；来源应用、时间、预览和 metadata 不参与。这比“可见文本相同”严格：相同文字若 HTML/RTF 或 transient flavor 不同，仍可能形成两项。Maccy 的源码和维护者说明也显示，富文本动态数据与 transient pasteboard types 正是视觉重复仍出现的主要原因。

_Sources: https://github.com/p0deje/Maccy/blob/master/Maccy/Models/HistoryItem.swift · https://github.com/p0deje/Maccy/discussions/818 · https://github.com/p0deje/Maccy/blob/master/appcast.xml_

### System Interoperability Approaches

采用现有点对点链路即可：`clipboard watcher → HistoryService → SQLite → history_changed → Svelte history session`。不删除旧项，而是保留旧 ID，仅更新 `last_used_at`。这样无需重建 flavors、FTS 数据或以 ID 为键的外部预览缓存，也不会制造删除成功但重插失败的窗口。

### Microservices Integration Patterns

不适用。ClipClop 是单机桌面应用；拆分服务、网关、服务发现与 Saga 都会增加无收益的故障面。

### Event-Driven Integration

继续复用现有单一变更事件。捕获重复项时发出 `{ latest_id: existing_id }`，与真正新增项保持相同的前端契约。CopyQ 的公开行为说明完全相同的 clip 会移动到顶部；Alfred 官方材料也说明非 transient 的重用会把条目带回顶部，支持 MRU 语义。

_Sources: https://github.com/hluk/CopyQ/issues/576 · https://www.alfredapp.com/help/features/clipboard/accessing-clipboard-history/_

### Integration Security and Consistency

“查 hash → 更新旧项”或“找不到 → 插入新项”必须在一个 SQLite 事务和现有数据库互斥锁内完成。SQLite 官方说明事务具备原子性，且同一时间只允许一个写事务；`rusqlite::Transaction` 默认在未提交时回滚。当前两秒查询与插入分别取得数据库锁，理论上存在检查后插入竞态，合并操作也顺便消除该问题。

_Sources: https://sqlite.org/transactional.html · https://sqlite.org/lang_transaction.html · https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html_

### Integration Decision

- 重复捕获始终提升旧项，与“手动使用历史后是否置顶”的设置分开；这是一次新的外部复制事件。
- 保留旧项的 `id`、`created_at`、source、flavors、metadata、FTS 与预览缓存，只更新 `last_used_at`。
- 保留期清理先于捕获：已经过期并被删除的内容重新复制时作为新项插入，语义合理。
- 不在本次改动中批量清理既有重复项；它需要同步删除按旧 ID 保存的预览缓存，应作为独立维护操作。

