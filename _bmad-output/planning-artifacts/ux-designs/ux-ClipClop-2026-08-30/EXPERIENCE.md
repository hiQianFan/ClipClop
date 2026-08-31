---
name: ClipClop PowerToys Peek Integration
status: draft
sources:
  - ../../../specs/spec-windows-powertoys-peek-integration/SPEC.md
updated: 2026-08-30
---

# Windows PowerToys Peek — Experience Spine

## Foundation

Windows 桌面端、键盘优先，使用现有 Svelte 5、Bits UI 菜单和 ClipClop token。`DESIGN.md` 是视觉依据；本文件定义发现、状态和交互。PowerToys Peek 是用户自行安装的外部 prerequisite，不是 ClipClop 内置功能。

## Information Architecture

| Surface | Reached from | Purpose |
|---|---|---|
| Settings → General | 主面板设置 | 展示 Peek 状态与 Microsoft 官方安装入口 |
| History action menu | 当前文件的操作菜单 | ready 时提供显式预览动作 |
| Quick panel | 托盘 Quick | ready 时用 Space 直接预览当前文件 |
| Settings → Shortcuts | 设置导航 | ready 时说明 Windows Space 行为 |

→ 交互演示：`.working/demo-powertoys-peek.html`。规范文件与 spine 冲突时，以 spine 为准。

## Voice and Tone

| 状态 | 中文 | English |
|---|---|---|
| 未安装 | 安装 PowerToys 后，可在 ClipClop 中按 Space 预览文件。 | Install PowerToys to preview files with Space in ClipClop. |
| 已就绪 | 已检测到 PowerToys Peek。选择文件后按 Space 即可预览。 | PowerToys Peek is ready. Select a file and press Space to preview it. |
| 权限不可用 | 管理员模式下无法使用 Peek。请以普通权限重新启动 ClipClop。 | Peek isn't available while ClipClop runs as administrator. Restart ClipClop normally. |
| 启动失败 | 无法启动 PowerToys Peek。 | PowerToys Peek couldn't be opened. |

按钮使用“了解并安装”而非“安装”，因为 ClipClop 只打开 Microsoft 官方页面，不执行安装。

## Component Patterns

| Component | Use | Behavioral rules |
|---|---|---|
| Integration row | Windows General | mount、重新进入 General、面板重新呼出时读取 capability；状态文本使用 `aria-live="polite"`，不抢焦点。 |
| Official-link button | 未安装 | 打开 Microsoft 官方安装说明；不得写成自动安装。 |
| Preview menu item | 文件操作菜单 | ready + File 时存在；其他状态从 DOM 移除，不用 disabled。 |
| Space route | History / Quick | ready + File 才拦截；不可用时不 `preventDefault()`。 |
| Inline error | 真正启动失败 | 使用现有错误区域；不 fallback，不弹 modal。 |

## State Patterns

| State | Treatment |
|---|---|
| Capability loading | 设置行显示“正在检测 PowerToys Peek…”；菜单和 Space 暂按 unavailable 处理。 |
| Not installed | 设置行显示说明与“了解并安装”；其他表面完全不出现预览能力。 |
| Ready | 设置行显示已就绪；文件菜单、Space 和快捷键说明出现。 |
| Elevated | 设置行解释普通权限要求；无动作按钮，其他表面不出现预览能力。 |
| Installed while ClipClop stays open | 用户返回或再次进入 General/呼出面板后变为 Ready；不弹庆祝通知。 |
| Executable removed after detection | 后端返回 NotPreviewable；前端刷新 capability 并静默移除入口。 |
| Spawn failed | 当前表面显示 inline error；入口保留，允许再次尝试。 |

## Interaction Primitives

- Settings 使用原生按钮；无需新增复杂控件。
- History 的现有 Bits UI DropdownMenu 保持键盘和焦点行为。
- Space 只在浏览模式、当前记录为真实文件且 Peek ready 时生效。
- Escape 由 Peek 自己关闭其窗口；ClipClop 不试图遥控 Peek。
- 禁止：首次启动弹窗、Space 首次触发教育 toast、自动安装、重复提醒、disabled 菜单项。

## Accessibility Floor

- 状态不只依靠颜色或图标表达。
- 官方链接按钮具备完整可访问名称：“了解并安装 Microsoft PowerToys”。
- capability 变化使用 polite live region，不主动移动焦点。
- 菜单项出现后遵循 Bits UI 的 roving focus；消失时焦点回到菜单 trigger 或历史列表。
- 键盘说明只陈述当前可用能力，避免屏幕阅读器读出无法执行的快捷键。
- 强制颜色和深浅主题全部继承现有 token。

## Responsive & Platform

该触点仅在 Windows 出现；macOS 不显示 PowerToys 设置行。840×640 主面板保持现有设置布局；窄窗口中文案可换行，右侧按钮不可压缩。

## Inspiration & Anti-patterns

- **采用 Files 模式**：已安装即零配置工作，未安装只在集成入口说明。
- **采用 Zotero 的 prerequisite 表达**：明确需要外部工具。
- **拒绝推广式 onboarding**：可选预览不应阻断剪贴板核心流程。
- **拒绝 disabled action**：隐藏不存在的能力比展示一个无法解释的灰色按钮更诚实。

## Key Flows

### Flow 1 — 林然发现并安装 Peek

1. 林然在 Windows 设置的 General 页看到“PowerToys Peek 文件预览”。
2. 状态说明它需要单独安装，按钮写“了解并安装”。
3. 他打开 Microsoft 官方说明并自行安装 PowerToys。
4. 返回 ClipClop，再次进入 General。
5. **高潮：** 状态变为“已就绪”；选中文件后按 Space，Peek 打开且没有启动默认应用。

### Flow 2 — 周敏不需要预览

1. 周敏从未安装 PowerToys，也不进入设置。
2. 她在主面板和 Quick 中浏览、复制、粘贴。
3. 预览菜单和快捷键从不出现。
4. **高潮：** 她的日常流程没有被缺失的可选能力打扰。

### Flow 3 — 陈望以管理员权限运行

1. 陈望进入 General，看到管理员模式说明。
2. 页面不再误导他重新安装 PowerToys。
3. 他正常权限重启 ClipClop。
4. **高潮：** 状态恢复 ready，Space 预览可用。
