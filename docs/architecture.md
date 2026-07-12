# ClipClop 技术架构

状态：v1 架构基线  
适用范围：macOS、Windows；Tauri 2 + Rust + Svelte 5；本地单机应用

## 0. 命名规范

| 场景 | 规范名称 | 示例 |
|---|---|---|
| 产品品牌、窗口标题、商店名称 | `ClipClop` | `ClipClop.app`、Windows 开始菜单中的 `ClipClop` |
| npm、Cargo package、仓库、数据库文件 | `clipclop` | `clipclop`、`clipclop.db` |
| Rust 应用库目标 | `clipclop_lib` | `clipclop_lib::run()` |
| 未来可复用业务内核 crate | `clipclop_core` | 仅在 CLI、daemon 或第二个客户端真实出现后创建 |
| Bundle Identifier | `com.clipclop.desktop` | macOS/Windows 打包标识 |
| TypeScript 变量与函数 | `camelCase` | `copyClip`、`selectedClip` |
| TypeScript 类型与 Svelte 组件 | `PascalCase` | `ClipDetail`、`ClipRow.svelte` |
| Rust 模块、变量与函数 | `snake_case` | `copy_clip`、`clipboard_monitor` |
| Rust 类型与枚举 | `PascalCase` | `ClipService`、`CopyMode` |
| 常量与环境变量 | `SCREAMING_SNAKE_CASE` | `CLIPCLOP_LOG` |

禁止在正式产品和项目文档中混用 `Clip-Clop`、`Clip Clop` 或 `clip-clop`。URL、第三方平台 slug 因占用限制无法使用 `clipclop` 时，可以使用 `clipclop-app`，但不改变产品品牌。

`clipclop_lib` 是当前 Tauri package 的 library target。Tauri 的 `main.rs` 调用它以兼容桌面和可能的移动入口；它承担应用组合根、commands 和当前业务模块的编译载体，但不是单独发布或可跨应用复用的“功能内核”。当前业务核心是 `clips` 等领域模块。只有出现第二个真实入口并需要脱离 Tauri 复用业务能力时，才将这些模块迁移到独立的 `clipclop_core` crate。

## 1. 结论

ClipClop 采用 **模块化单体 + 轻量领域模型 + 平台适配层**，不采用完整 DDD 分层。

完整 DDD 通常适合业务规则复杂、存在多个聚合、多个持久化实现、多个交付端或多人团队并行维护的系统。ClipClop 的核心复杂度主要来自跨平台剪贴板 API、格式转换、文件缓存和窗口生命周期，而不是复杂业务规则。照搬 `domain/application/infrastructure/interface` 四层、Repository trait、CQRS、领域事件总线和依赖注入容器，会增加文件数量和调用跳转，却不能降低当前风险。

本项目只采用 DDD 中真正有价值的部分：

- 使用统一语言：`Clip`、`Flavor`、`CopyMode`、`SourceApp`、`RetentionPolicy`。
- 领域规则集中在 Rust 模块，不放进 Tauri command、SQL 或 Svelte 组件。
- 按业务能力划分模块，模块对外暴露小而明确的 API。
- 平台实现和存储实现位于边界，不能反向污染领域规则。
- 只有出现第二个真实实现时才引入 trait；不为假想替换预建抽象。

## 2. 当前工程与目标架构

当前代码已从 `create-tauri-app` 模板演进为可运行的模块化单体：Rust 侧包含剪贴板监听、SQLite/FTS5、类型化 commands、设置、托盘、全局快捷键与窗口生命周期；Svelte 侧包含 Quick Panel、预览和独立设置窗口。SQLite 是历史记录的唯一真相来源，前端只通过 IPC DTO 访问业务能力。后续仍按本章边界拆分增长，不为尚未出现的模块预建空层级。

Tauri 官方把项目视为 JavaScript 前端与 `src-tauri` Rust 工程两部分，并建议桌面入口保持在 `main.rs`，主要初始化写在 `lib.rs`。前端通过 message passing 调用 Rust；commands 支持参数、返回值、错误和异步，events 更动态、无返回值且只传 JSON。因此本项目使用 command 处理请求/响应，event 只通知状态失效，不用 event 承载业务事务或二进制内容。[Tauri 项目结构](https://v2.tauri.app/start/project-structure/) · [Tauri 架构](https://v2.tauri.app/concept/architecture/) · [调用 Rust](https://v2.tauri.app/develop/calling-rust/)

```text
Svelte 视图
  ↓ 调用 feature api
TypeScript IPC 封装
  ↓ invoke(command)
Rust commands（传输边界）
  ↓ 参数校验与调用
Rust 业务模块
  ↓
SQLite / 文件缓存 / macOS 与 Windows API

Rust 后台监听器
  ↓ 持久化成功后
clips_changed event（只发轻量失效通知）
  ↓
前端重新查询当前页
```

这是一个进程内的本地应用，不存在独立后端服务器，也不嵌入 HTTP 服务。

## 3. 文件结构

目录随真实功能增长。以下是 v1 目标结构，不要求在第一天创建所有空文件。

```text
ClipClop/
├── docs/
│   └── architecture.md
├── src/                              # Svelte 前端
│   ├── app.html
│   ├── routes/
│   │   ├── +layout.ts
│   │   ├── +page.svelte              # Quick Panel 页面组装
│   │   └── settings/
│   │       └── +page.svelte          # 设置窗口页面
│   └── lib/
│       ├── ipc/
│       │   ├── client.ts             # 唯一直接调用 invoke/listen 的位置
│       │   ├── types.ts              # IPC DTO 与错误类型
│       │   └── events.ts             # 监听、清理 Tauri event
│       ├── clips/
│       │   ├── api.ts                # list/get/copy/delete
│       │   ├── state.svelte.ts       # 当前页、选中项、搜索状态
│       │   ├── ClipList.svelte
│       │   ├── ClipRow.svelte
│       │   └── ClipPreview.svelte
│       ├── search/
│       │   └── SearchField.svelte
│       ├── settings/
│       │   ├── api.ts
│       │   └── SettingsForm.svelte
│       ├── shortcuts/
│       │   └── keyboard.ts            # 面板内快捷键映射
│       ├── ui/                         # 至少被两个 feature 使用才放入
│       │   ├── Keycap.svelte
│       │   └── EmptyState.svelte
│       └── styles/
│           ├── tokens.css
│           └── global.css
├── src-tauri/
│   ├── capabilities/
│   │   └── default.json               # 最小权限白名单
│   ├── migrations/
│   │   └── 0001_init.sql
│   ├── src/
│   │   ├── main.rs                    # 保持极薄，只调用 run()
│   │   ├── lib.rs                     # 组合根：初始化、manage、插件、commands
│   │   ├── state.rs                   # AppState 及共享资源句柄
│   │   ├── error.rs                   # AppError、ErrorCode、IPC 序列化
│   │   ├── commands/                  # Tauri 传输适配层
│   │   │   ├── mod.rs
│   │   │   ├── clips.rs
│   │   │   └── settings.rs
│   │   ├── clips/                     # 核心领域模块
│   │   │   ├── mod.rs                 # 对外公开 ClipService
│   │   │   ├── model.rs               # Clip、Flavor、CopyMode
│   │   │   ├── capture.rs             # 捕获编排、去重、持久化
│   │   │   ├── copy.rs                # rich/plain 写回规则
│   │   │   └── query.rs               # 列表与详情用例
│   │   ├── clipboard/                 # 系统剪贴板适配
│   │   │   ├── mod.rs
│   │   │   ├── macos.rs
│   │   │   └── windows.rs
│   │   ├── storage/                   # SQLite 与缓存
│   │   │   ├── mod.rs
│   │   │   ├── db.rs
│   │   │   ├── clips.rs
│   │   │   └── blobs.rs
│   │   ├── preview/                   # 缩略图与本地系统图标
│   │   │   ├── mod.rs
│   │   │   ├── thumbnail.rs
│   │   │   └── app_icon.rs
│   │   ├── settings/
│   │   │   ├── mod.rs
│   │   │   └── model.rs
│   │   └── window/
│   │       ├── mod.rs
│   │       └── hotkey.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.js
```

### 3.1 目录规则

- `commands/` 只做 DTO 转换、输入校验、权限边界和服务调用；禁止写 SQL、剪贴板判断或复制规则。
- `clips/` 拥有剪贴板历史的业务规则；不直接依赖 Tauri command 类型或 Svelte DTO。
- `clipboard/` 只解决 macOS/Windows 格式读取与写回差异；平台代码不得决定保留期、去重或 UI 类型。
- `storage/` 拥有事务、迁移和磁盘一致性；SQL 行模型不直接暴露给前端。
- 前端 `lib/ipc/client.ts` 是唯一直接使用 `invoke` 的位置；feature 通过自己的 `api.ts` 调用它。
- 前端状态只保存当前界面需要的数据；SQLite 是历史记录的唯一真相来源。
- 组件、类型或工具至少出现两个真实调用者后再提升到共享目录。

## 4. Rust 后端设计

### 4.1 组合根与状态

`lib.rs` 是 composition root，只负责：

1. 初始化应用数据目录和数据库迁移。
2. 创建 `AppState`。
3. 启动剪贴板监听与保留期清理任务。
4. 注册托盘、窗口、全局快捷键和 commands。
5. 启动 Tauri runtime。

```rust
pub struct AppState {
    pub clips: ClipService,
    pub settings: SettingsService,
}
```

command 通过 `tauri::State<'_, AppState>` 获取服务。Tauri 官方支持 managed state；并发可变资源必须使用合适的锁。数据库采用单连接所有者或小连接池，不把一个全局 `Mutex` 持有到文件读取、图片编码等慢操作结束。[Tauri State Management](https://v2.tauri.app/develop/state-management/)

### 4.2 领域模型

```rust
pub struct Clip {
    pub id: ClipId,
    pub kind: ClipKind,
    pub plain_text: Option<String>,
    pub flavors: Vec<Flavor>,
    pub source_app: Option<SourceApp>,
    pub created_at: Timestamp,
}

pub struct Flavor {
    pub format: FlavorFormat,
    pub payload: PayloadRef,
    pub byte_size: u64,
}

pub enum CopyMode {
    Rich,
    Plain,
}
```

`Clip` 是一次剪贴板变化的完整记录，`Flavor` 是同一次复制携带的一种表示。HTML、RTF、纯文本不是三条历史，也不是三个业务模块。

### 4.3 富文本复制

- 默认 `copy_clip(id, CopyMode::Rich)`：写回原记录中平台支持的 HTML、RTF 和纯文本 fallback。
- “复制为纯文本”调用 `copy_clip(id, CopyMode::Plain)`：只写 `text/plain`。
- 若记录只有纯文本，两种模式结果一致。
- 前端不解析、清洗或重建 HTML/RTF；预览使用纯文本，原始 payload 只在 Rust 中处理。
- Enter 默认带格式；`⌘⇧C` / `Ctrl+Shift+C` 复制纯文本；操作菜单同时展示两个动作。

两种模式必须共用一个 service 方法，通过 `CopyMode` 分支，不能维护两套复制流程。

### 4.4 用户控制与隐私

ClipClop **不识别、不分类、不拦截所谓敏感内容**。密码管理器或系统附加的 concealed/transient 标记不会触发自动丢弃。默认保存平台允许读取的全部受支持 flavor。

用户通过以下显式动作控制数据：

- 暂停/恢复记录。
- 忽略用户指定的来源应用。
- 删除单条或清空全部。
- 设置保留期限。

不提供预置忽略名单、内容敏感度检测、自动脱敏或规则引擎。输入因无法解析、资源上限或磁盘错误未保存时，必须记录明确错误原因；这属于系统健壮性，不属于内容审查。

应用不上传剪贴板内容、链接、文件路径或元数据，不因链接主动请求 favicon、SEO 或 Open Graph 信息。

### 4.5 存储与一致性

建议 SQLite 主表与 flavor 子表分离：

```sql
CREATE TABLE clips (
  id            TEXT PRIMARY KEY,
  kind          TEXT NOT NULL,
  plain_text    TEXT,
  preview       TEXT NOT NULL,
  source_id     TEXT,
  source_name   TEXT,
  created_at    TEXT NOT NULL
);

CREATE TABLE clip_flavors (
  clip_id       TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
  format        TEXT NOT NULL,
  inline_data   BLOB,
  blob_path     TEXT,
  byte_size     INTEGER NOT NULL,
  PRIMARY KEY (clip_id, format),
  CHECK ((inline_data IS NULL) <> (blob_path IS NULL))
);
```

- 可搜索纯文本和小型 HTML/RTF 可以入 SQLite；图片、较大 payload 和缩略图写入应用数据目录。
- 数据库只存应用数据目录内的相对路径，避免迁移目录后失效。
- 插入 Clip、Flavor 元数据和 FTS 索引必须处于同一事务。
- 文件缓存采用“先写临时文件 → `fsync`/关闭 → 原子重命名 → 提交数据库引用”；失败时不留下数据库悬空路径。
- 删除使用相反顺序：事务删除数据库记录并记录待清理文件，提交后删除缓存；下次启动清理孤儿文件。
- migrations 只向前执行；已经发布的 migration 不修改，新增编号文件。

文件引用不复制原文件。缩略图通过系统 API 异步生成，失败或超时回退到系统文件类型图标，不阻塞捕获事务。

### 4.6 并发模型

- 剪贴板监听器只负责读取快照并投递到单消费者 capture worker。
- capture worker 串行完成去重和写事务，避免同一 clipboard change 乱序。
- 缩略图生成进入有界后台队列，完成后单独更新 `thumb_ref`。
- 查询 command 可以并发执行，但不与长时间图片编码共享锁。
- 应用退出时停止接收新任务，等待正在提交的数据库事务完成；缩略图任务可以取消并在下次启动补做。
- event 只在事务成功后发送，前端不会看到尚未持久化的数据。

## 5. IPC 契约

IPC 使用少量粗粒度 commands，不把每个 Rust 函数暴露给前端。

```text
list_clips(request)             -> ClipPageDto
get_clip(id)                    -> ClipDetailDto
copy_clip(id, mode)             -> CopyResultDto
delete_clip(id)                 -> Unit
clear_history()                 -> Unit
get_settings()                  -> SettingsDto
update_settings(patch)          -> SettingsDto
set_capture_paused(paused)      -> CaptureStateDto
```

事件：

```text
clips_changed { revision, latest_id? }
settings_changed { revision }
capture_state_changed { paused }
```

规则：

- command 名称稳定，参数使用对象，便于以后增加可选字段。
- DTO 显式包含 `kind`，不依赖字段是否为空猜类型。
- 列表 DTO 不包含原始 flavor 或大二进制；详情按需返回安全预览信息。
- 原始文件路径、HTML/RTF payload 不发送给 WebView，除非某个已定义视图确实需要。
- 错误返回稳定 `code + message + details?`；UI 根据 `code` 决定恢复动作，不解析错误字符串。
- events 只做失效通知。Tauri 官方说明 event 无类型安全、无返回值且只支持 JSON；需要有序高吞吐数据时应使用 channel，但 ClipClop 的 UI 更新没有必要发送高吞吐流。[Tauri 前端事件](https://v2.tauri.app/develop/_sections/frontend-listen/)
- Svelte 组件卸载时必须执行 `unlisten`，防止 SPA 中重复监听和内存泄漏。

## 6. Svelte 前端设计

前端采用 feature-first，不采用前端版四层 DDD。

每个 feature 可以包含：

- `api.ts`：调用已类型化的 IPC client。
- `state.svelte.ts`：该功能的 UI 状态和异步状态机。
- 组件：只处理展示与用户意图。

前端状态分为三类：

| 状态 | 所有者 | 示例 |
|---|---|---|
| 持久化业务状态 | Rust/SQLite | 历史记录、设置、忽略应用 |
| 会话 UI 状态 | Svelte | 当前页、选中行、搜索词、菜单开关 |
| OS 状态 | Rust | 剪贴板、窗口可见性、全局快捷键 |

禁止把整份历史镜像进全局 store；列表按页加载，详情按选择加载。`clips_changed` 到达后重新请求当前查询，前端不自行合并数据库真相。

异步组件必须有 loading、empty、error 和 retry 状态。快速切换选中项时使用请求序号或 `AbortController` 思路忽略过期响应，避免旧详情覆盖新选择。

## 7. 扩展模块的方法

新增功能时按以下顺序判断：

1. 是否只是现有模块的一条新规则？是则留在现有模块。
2. 是否拥有独立数据、用例和生命周期？是则新增一个业务模块。
3. macOS 与 Windows 是否真的出现不同实现？出现后再新增平台文件或 trait。
4. 是否需要前端操作？需要时增加一个 command 和对应 feature API；后台内部功能不暴露 IPC。

例如以后增加 Pin：

```text
src-tauri/src/clips/pin.rs       # pin/unpin 领域规则
commands/clips.rs                # 新增 set_clip_pinned
src/lib/clips/api.ts             # 新增调用
ClipRow.svelte                   # 展示与触发
```

不需要新建 `pin` 服务、Repository、事件总线或独立 crate。只有当某个模块需要被 CLI、后台 daemon 或其他应用复用时，才将 Rust 核心抽为 workspace crate。

## 8. 生产级保障

### 8.1 错误与日志

- 所有 command 返回 `Result<T, AppError>`，错误码稳定、用户文案由前端本地化。
- 捕获循环的单条失败不能终止监听器；记录错误后继续处理下一次变化。
- 使用 Rust `tracing` 输出本地滚动日志，但不得记录剪贴板正文、HTML/RTF、文件路径或完整 URL。
- panic 只表示无法继续启动的配置/迁移错误；正常 I/O、格式和权限问题必须返回错误。

### 8.2 权限与安全

- `capabilities/default.json` 只开放当前窗口实际需要的命令和插件权限。
- WebView 不加载远程页面；设置严格 CSP。
- 不启用 shell、任意文件系统或 HTTP 权限，除非后续功能明确需要。
- IPC 入参全部校验：ID 格式、页大小、搜索长度、路径来源和枚举值。
- 数据库查询参数化，不拼接用户输入。

### 8.3 测试

- Rust 单元测试：flavor 归一化、rich/plain 选择、去重、保留期和资源上限。
- SQLite 集成测试：migration、事务回滚、分页、FTS、级联删除和孤儿缓存清理。
- 平台冒烟测试：macOS/Windows 各验证文本、HTML/RTF、图片、文件引用的读写回环；系统剪贴板测试串行执行。
- 前端测试：键盘映射、状态转换、IPC 错误恢复。Tauri 官方提供 `mockIPC` 和 event mock，可在不启动真实 Rust 后端时测试前端调用。[Tauri Mock APIs](https://v2.tauri.app/develop/tests/mocking/)
- 最小端到端路径：捕获富文本 → 列表出现 → 默认带格式复制 → 纯文本复制 → 删除。

### 8.4 CI 与发布门槛

每次合并至少运行：

```bash
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

macOS 与 Windows 分别构建安装包；发布前在两端执行剪贴板冒烟测试。锁定并提交 `pnpm-lock.yaml` 与 `src-tauri/Cargo.lock`，避免构建漂移。

## 9. 开源项目调研结论

### EcoPaste

[EcoPaste](https://github.com/EcoPasteHub/EcoPaste) 是与 ClipClop 最接近的参考。其 Rust 代码按 `clipboard`、`commands`、`db`、`settings`、`window`、`shortcut` 等能力组织，平台差异放在 `macos.rs`、`windows.rs`；前端按页面、hooks、stores 和 components 组织。它证明“能力模块 + 薄 commands + 平台文件”足以支撑成熟剪贴板工具，但其当前功能范围远大于 ClipClop，不应照搬全部目录和状态工具。

### PasteBar

[PasteBar](https://github.com/PasteBar/PasteBarApp) 使用 Tauri、Rust、Diesel、Tokio 与 React/TypeScript，并已经覆盖 macOS/Windows、搜索、备份、快捷键等生产功能。它说明 SQLite、Rust 后台和 WebView UI 的组合可落地；同时它的多种前端状态库和更大功能面不是 ClipClop v1 的必要依赖。

### Spacedrive

[Spacedrive](https://github.com/spacedriveapp/spacedrive) 的 core 明确划分 `domain`、`ops`、`infra`，并使用 CQRS、daemon、作业、同步和多个应用。这是完整 DDD/分层有合理收益的规模：多设备、P2P、长期任务、CLI/server/Tauri 多入口。ClipClop 没有这些条件，因此只借用领域边界和依赖方向，不复制其 workspace、CQRS、事件总线或扩展系统。

### 采用与拒绝

采用：

- Tauri 官方的前端/Rust 双工程边界。
- command 请求响应、event 失效通知。
- EcoPaste 的能力模块和平台差异隔离。
- Spacedrive 的领域不依赖基础设施原则。

拒绝：

- 完整 DDD 四层模板。
- CQRS、领域事件总线、依赖注入容器。
- 为 SQLite 建单实现 Repository trait。
- 多 crate workspace、内嵌 HTTP server、插件系统。
- 多套前端全局状态库。

当项目出现第二个客户端、独立 daemon、同步服务、可安装扩展或第二种存储实现时，再重新评估上述拒绝项。
