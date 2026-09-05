---
name: ClipClop macOS Permission Experience
status: final
updated: 2026-09-05
sources:
  - DESIGN.md
  - ../../../../../DESIGN.md
  - ../../../../../docs/architecture.zh-CN.md
  - ../../../implementation-artifacts/investigations/automatic-paste-logging-investigation.md
  - ../../../implementation-artifacts/spec-stabilize-unsigned-macos-auto-paste-permission.md
---

# ClipClop macOS 权限体验

## Foundation

macOS 桌面端、键盘优先，使用现有 Svelte 5、Tauri、Bits UI 行为原语和 ClipClop token。本工作区的 `DESIGN.md` 定义权限表面的视觉增量；根目录 `DESIGN.md` 仍是视觉权威。

权限体验采用双入口：Settings → General → 权限与系统访问是长期管理区；自动粘贴和文件预览只在真实失败时给出就地恢复入口。首次引导解释价值与可选性，不强迫一次性授予全部权限。

辅助功能允许 ClipClop 注入 Command-V；完全磁盘访问只在文件预览实际需要且当前访问失败时请求。二者状态独立，不以“全部完成”作为使用剪贴板核心功能的前提。

## Information Architecture

| Surface | Reached from | Purpose |
|---|---|---|
| First-run onboarding | 首次启动 | 解释两项权限分别解锁什么，并让用户选择现在配置或稍后处理 |
| Settings → General → 权限与系统访问 | Settings 导航 | 集中查看和管理两项能力；不为两行内容新增一级侧栏分类 |
| Permission guide | 首次引导、权限区或恢复入口 | 一次处理一个权限；打开对应的系统设置后，引导仍保持显示 |
| Paste recovery notice | 自动粘贴返回 `copied_permission_required` | 告知内容已复制，提供手动 Command-V 和“处理权限” |
| File preview recovery notice | 预览因文件访问被拒 | 告知预览未打开，提供“处理权限”及不依赖预览的退路 |
| Restart notice | 本次引导观察到 denied → granted | 预告倒计时重启，允许取消或立即重启 |

权限引导是主面板内的明确模式，返回系统设置时保持上下文。退出引导后回到发起它的界面；从功能恢复入口进入时，保留当前选中项。

## Voice and Tone

语气直接、克制、可验证。先说发生了什么，再说下一步；不责怪用户，不把系统限制描述成 ClipClop 已经完成的动作。

| State | Recommended copy |
|---|---|
| Accessibility unknown | 正在检测辅助功能权限… |
| Accessibility required | 允许辅助功能后，ClipClop 才能自动按下 Command-V。 |
| Full Disk Access required | ClipClop 无法读取这个文件。配置完全磁盘访问后可再次预览。 |
| Waiting in System Settings | 在系统设置中允许当前 ClipClop，然后返回这里。 |
| Accessibility granted | 已检测到辅助功能权限。 |
| Full Disk Access unknown | macOS 不提供完整授权状态；ClipClop 会在预览具体文件时验证访问。 |
| Still denied | 尚未检测到权限。请确认添加的是当前 ClipClop。 |
| Paste degraded | 内容已复制，但没有自动粘贴。你仍可按 Command-V。 |
| Preview degraded | 无法预览这个文件，原文件和剪贴板历史未受影响。 |
| Old identity | 旧 ClipClop 条目不代表当前版本已获授权。请移除旧条目，再添加当前 App。 |
| Restart pending | 已检测到权限。ClipClop 将重新启动以完成配置。 |
| Restart cancelled | 已取消自动重新启动；你可以稍后手动重启。 |

按钮写实际动作：“打开系统设置”“重新检测”“在访达中显示”“处理权限”“稍后处理”“立即重新启动”“取消重新启动”。

## Component Patterns

| Component | Use | Behavioral rules |
|---|---|---|
| Permission row | Settings → General | 辅助功能读取实时能力；完全磁盘访问只显示“按需验证/上次访问结果”，每项独立提供“管理” |
| Permission guide | 单项授权 | 保持在 App 内；打开对应设置后，仅在引导处于活动状态时监听窗口重新获得焦点，并重新检测 |
| Inline recovery notice | 粘贴/预览真实失败 | 保留当前内容与选择；提供一个“处理权限”动作，不自动打开完整引导 |
| Current App identity | 更新后恢复 | 显示 `/Applications/ClipClop.app` 或“开发构建”；路径使用 `{typography.body}` |
| 在访达中显示 | 旧条目或手动添加 | 能解析 `.app` bundle 时显示；否则隐藏，不展示无效按钮 |
| Restart notice | denied → granted | 默认给出短暂预告；“立即重新启动”和“取消重新启动”均可键盘操作 |
| Live status | 所有检测结果 | `aria-live="polite"`，不主动移动焦点，不只依靠图标或颜色 |

第一版不提供 App 图标拖入系统设置。用户需要手动添加时，调用系统设置的 `+`，并通过“在访达中显示”定位真实 `.app`。

## State Patterns

### Per-permission state

| State | Treatment |
|---|---|
| checking | 显示“正在检测…”，相关功能暂按 unavailable；不重复打开设置 |
| not_requested | 解释价值并提供“打开系统设置”；允许“稍后处理” |
| waiting | 引导保持显示，说明应在系统设置中允许当前 App |
| granted | 原位显示“已检测到权限”；只在本次引导发生状态转换时进入 restart_pending |
| denied | 保留引导和“重新检测”；不循环弹窗，不自动重启 |
| stale_identity_suspected | 展示当前 App 身份、Finder 定位和移除旧条目后重新添加的步骤 |
| unsupported | 从非 macOS 表面移除；不显示不可执行动作 |
| open_settings_failed | 引导内显示本地化错误，保留重试和退出 |

### Combined permission status

权限区不使用单一“全部授权”开关，也不显示“2 项已就绪”这类汇总，因为完全磁盘访问无法可靠全局检测。每一行独立陈述事实；任一权限缺失不改变另一项的状态。

### Restart state

只有引导激活期间实时结果从 denied 变为 granted 才进入 `restart_pending`。预告期间：

- 显示将重新启动的原因与短倒计时。
- 主动作是“立即重新启动”，次动作是“取消重新启动”。
- 用户有未完成的面板交互、关闭提示或按 Escape 时取消倒计时。
- 取消后不再自动重启；状态保留为“已授权，需要重新启动”。
- relaunch 失败时停止自动尝试，显示“重新启动 ClipClop”。
- 重复回焦事件不得触发第二次自动重启。

## Interaction Primitives

- Settings 行和引导按钮使用原生 button；需要 modal 时使用现有 Bits UI Dialog 以保留焦点圈、Escape 和焦点返回。
- 打开系统设置是显式用户动作；ClipClop 不自动点击开关、`+` 或列表项。
- 引导打开系统设置后保持存在；窗口回焦触发一次复检，不做定时轮询或持久化权限布尔值。
- Escape 只退出一层：取消重启预告，或关闭权限引导并回到来源；不会撤销已授予权限。
- 粘贴缺权时不隐藏面板，内容已经写入剪贴板；用户可按 Command-V 完成任务。
- 预览缺权时不删除、移动或重新导入文件，也不修改历史记录。
- 每次检查针对当前运行实例；不得根据系统设置列表中存在同名条目推断已授权。

## Accessibility Floor

- 页面、权限行、状态、错误和倒计时均有完整文本，不只依赖颜色或图标。
- 状态变化使用 polite live region；倒计时不逐秒播报，只播报开始、取消和即将执行。
- 打开系统设置后返回 App，焦点留在发起动作或状态区域；不会跳到页面顶部。
- 对话框打开时，焦点移到标题后的第一个可执行动作；对话框关闭后，焦点返回触发控件。使用 Tab 键时，焦点保持在对话框内。
- 所有按钮都有完整可访问名称，例如“打开辅助功能设置”“在访达中显示当前 ClipClop”。
- “稍后处理”和“取消重新启动”始终可由键盘到达，不用低对比或隐蔽链接弱化。
- 强制颜色、深浅主题、缩放和 `prefers-reduced-motion` 继承现有 token；关闭动画时倒计时仍以文本表达。

## Responsive & Platform

权限中心和引导仅在 macOS 展示对应系统权限。Windows 不显示辅助功能或完全磁盘访问行，保持现有行为。

840×640 主面板沿用现有 Settings 布局。窄窗口中左侧说明可以换行，右侧按钮不可压缩；步骤纵向排列，不改为横向卡片。系统设置在外部窗口打开，ClipClop 不尝试控制其尺寸或焦点。

## Inspiration & Anti-patterns

- 采用 Alfred 式集中 Permissions 管理：不同能力分开请求和展示状态。
- 采用 Raycast 式功能现场恢复：真实缺权时显示 Grant/处理入口，完成后继续 setup。
- 遵循 Apple 系统路径：用户在 Privacy & Security 中打开开关，或用 `+` 添加当前 App。
- 拒绝一次性索要全部权限、首启强制弹窗、循环提醒、假成功状态、disabled 死路和自动操作系统设置。
- 拒绝原生拖拽的第一版实现；只有真实可用性测试证明它显著降低失败率时再评估。

## Key Flows

### Flow 1 — 林然首次安装

1. 林然首次打开 ClipClop，看到快速说明：核心剪贴板历史无需额外权限；自动粘贴和受保护文件预览会分别按需请求。
2. 页面列出“自动粘贴”和“文件预览”的价值，不显示“全部允许”。
3. 他选择现在配置自动粘贴；ClipClop 进入辅助功能引导。
4. 他也可以选择“稍后处理”并直接进入历史面板。
5. **高潮：** 林然清楚知道每项权限为何存在，并只配置此刻需要的一项，没有被首启阻断。

### Flow 2 — 周敏在设置中管理权限

1. 周敏打开 Settings → General → 权限与系统访问。
2. 页面显示辅助功能实时状态；完全磁盘访问显示“预览文件时验证”，不声称全局已授权。
3. 她点击完全磁盘访问行的“管理”，权限引导保持显示，同时打开 macOS 对应设置。
4. 她返回 ClipClop；窗口重新获得焦点时，页面重新检测并原位更新状态，焦点仍在“管理”动作附近。
5. **高潮：** 两项权限的真实状态清晰可见，周敏无需猜测系统列表中的同名 App 是否有效。

### Flow 3 — 陈望从粘贴失败恢复

1. 陈望在另一个 App 中呼出 ClipClop，选中记录并按 Enter。
2. 当前实例缺少 Post Event 权限；ClipClop 写入剪贴板但不隐藏面板。
3. 原位提示告诉他“内容已复制，但没有自动粘贴”，同时提供“处理权限”。
4. 他可以立刻回到目标 App 手动按 Command-V，也可以进入辅助功能引导。
5. **高潮：** 即使授权失败，陈望的当前任务仍可完成，且恢复入口不是错误文本死路。

### Flow 4 — 顾乔从文件预览失败恢复

1. 顾乔选中一个受保护位置的文件并请求预览。
2. ClipClop 确认是文件访问权限失败，而非把所有预览错误都归因于磁盘权限。
3. 当前表面显示“无法预览这个文件”，保留选中项并提供“处理权限”。
4. 顾乔进入完全磁盘访问引导，或选择稍后处理并继续复制、打开其他记录。
5. **高潮：** 顾乔不会因为一次预览失败被迫离开核心流程，也不会误以为文件已损坏。

### Flow 5 — 许诺更新后遇到旧授权

1. 许诺覆盖更新 ad-hoc 构建后，系统设置仍有旧 ClipClop 条目，但当前实例实时检测为 denied。
2. UI 不显示“已授权”，而是说明旧条目不代表当前版本已获授权，并展示当前 App 路径或“开发构建”。
3. 他点击“在访达中显示”，确认当前 `.app`，按说明在系统设置中移除旧条目并用 `+` 添加当前 App。
4. 返回 ClipClop 后，窗口重新获得焦点并触发重新检测。
5. **高潮：** 状态变为已授权；整个过程没有承诺旧授权迁移，也没有把 debug 与安装版混为一谈。

### Flow 6 — 唐宁拒绝或稍后处理

1. 唐宁在首次引导中不想开放完全磁盘访问，点击“稍后处理”。
2. ClipClop 不重复弹窗；设置权限中心继续如实显示“需要处理”。
3. 只有当她预览受保护文件并真实失败时，当前表面再次提供一次非阻断恢复入口。
4. 她关闭提示后仍能浏览文本、复制和手动粘贴。
5. **高潮：** 唐宁保有控制权，拒绝一项权限不会被惩罚，也不会关闭无关能力。

### Flow 7 — 沈一授权后取消自动重启

1. 沈一从辅助功能引导打开系统设置并允许当前 ClipClop。
2. 返回 App 后，实时复检观察到本次 denied → granted。
3. ClipClop 显示重启预告、原因和“立即重新启动 / 取消重新启动”。
4. 她正在核对一条内容，于是点击“取消重新启动”；倒计时停止且不会再次自动出现。
5. 她稍后从状态行手动重启；若不取消，倒计时结束后 App 只 relaunch 一次。
6. **高潮：** 授权闭环可以自动完成，但不会突然打断沈一尚未结束的任务。

## Verification Journeys

- 使用 `/Applications/ClipClop.app` 验证首次拒绝、稍后处理、授权返回、取消重启、自动重启和 relaunch 失败。
- 用旧构建条目与当前安装版验证旧身份文案、访达定位和重新添加。
- 分别制造 Post Event 权限失败与真实文件访问失败，确认两种恢复入口不会混淆。
- 验证手动 Command-V、历史数据、当前选中项和来源表面在失败与返回后保持可用。
- 使用 VoiceOver、键盘、强制颜色、深浅主题和 reduced motion 检查状态播报、焦点返回与倒计时取消。
- 测试只使用虚构剪贴内容和非私人路径；日志不记录剪贴板或预览载荷。
