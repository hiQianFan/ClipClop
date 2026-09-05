---
name: ClipClop macOS Permission Experience
status: final
updated: 2026-09-05
sources:
  - ../../../../../DESIGN.md
  - ../../../../../docs/architecture.zh-CN.md
  - ../../../implementation-artifacts/investigations/automatic-paste-logging-investigation.md
  - ../../../implementation-artifacts/spec-stabilize-unsigned-macos-auto-paste-permission.md
colors:
  bg-shell: "var(--bg-shell)"
  bg-raised: "var(--bg-raised)"
  bg-hover: "var(--bg-hover)"
  bg-selected: "var(--bg-selected)"
  hairline: "var(--hairline)"
  text-1: "var(--text-1)"
  text-2: "var(--text-2)"
  text-3: "var(--text-3)"
  action: "var(--action)"
  action-hover: "var(--action-hover)"
  action-on: "var(--action-on)"
  danger: "var(--danger)"
  danger-fill: "var(--danger-fill)"
  danger-on: "var(--danger-on)"
typography:
  heading:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 680
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  body:
    fontFamily: "ui-monospace, SF Mono, Cascadia Code, JetBrains Mono, Consolas, monospace"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: 1.5
  caption:
    fontFamily: "ui-monospace, SF Mono, Cascadia Code, JetBrains Mono, Consolas, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  xl: "14px"
  pill: "999px"
spacing:
  "1": "2px"
  "2": "4px"
  "3": "6px"
  "4": "8px"
  "6": "12px"
  "8": "16px"
  "10": "20px"
  "12": "24px"
components:
  button-primary:
    backgroundColor: "{colors.action}"
    textColor: "{colors.action-on}"
    rounded: "{rounded.md}"
    padding: "7px 15px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.text-2}"
    rounded: "{rounded.md}"
    padding: "7px 10px"
  row:
    backgroundColor: "transparent"
    textColor: "{colors.text-1}"
    rounded: "{rounded.lg}"
    padding: "7px 8px"
  menu:
    backgroundColor: "{colors.bg-raised}"
    textColor: "{colors.text-1}"
    rounded: "{rounded.lg}"
    padding: "6px"
---

# ClipClop macOS 权限体验设计

## Brand & Style

权限界面继承根目录 `DESIGN.md` 的 “Native Utility Drawer”：安静、紧凑、键盘优先，像操作系统自带的实用工具。它解释事实和下一步，不制造警报感，也不把授权包装成产品推广。

首次引导、权限中心和失败恢复共用同一种视觉语言。没有营销式 onboarding、教程轮播、大插画、渐变、玻璃效果、庆祝动画、嵌套卡片或强迫授权。根目录 `DESIGN.md` 与本文件冲突时，以根目录为准。

## Colors

只使用继承的中性色阶：`{colors.bg-shell}`、`{colors.bg-raised}`、`{colors.bg-hover}`、`{colors.bg-selected}` 和 `{colors.hairline}`。权限状态不引入绿、黄或新的品牌色。

- 标题与关键状态使用 `{colors.text-1}`。
- 解释、辅助文案和普通状态使用 `{colors.text-2}`。
- 路径、时间和次要元数据使用 `{colors.text-3}`，但不得承载唯一状态含义。
- `{colors.action}` 只用于当前表面的唯一主动作。
- `{colors.danger}` 仅用于“移除旧条目”等真正具有破坏性或不可逆含义的说明，不用于普通缺权状态。

状态必须同时由文字表达；颜色、图标和位置都只是辅助线索。

## Typography

界面标题、按钮、标签和步骤使用平台 sans，即 `{typography.heading}` 与 `{typography.label}`。当前 App 路径、快捷键和技术身份使用 `{typography.body}` 或 `{typography.caption}` 的 monospace。

- 页面标题：唯一的 20px heading。
- 权限名称与动作：13px UI label。
- 说明与状态：保持现有正文/Meta 层级，优先写成一行短句。
- `/Applications/ClipClop.app`、开发构建标记和按键使用 monospace。

## Layout & Spacing

权限区放在 Settings → General，沿用单列 ruled-list，不新增侧栏分类或卡片网格。每项权限是一条两区设置行：左侧为名称、用途与当前可验证状态，右侧为不可压缩的单一动作。

- 行内间距只使用 `{spacing.*}`；区间关系沿用根设计系统的 2px 基准。
- 文本区 `flex: 1 1 auto; min-width: 0`；动作区 `flex: none`。
- 设置行之间用 1px `{colors.hairline}` 分隔。
- 行说明优先保持一行；窄窗口可换行，但不能挤压右侧按钮。
- 完整引导页按“标题 → 为什么需要 → 当前状态 → 下一步 → 次要恢复”纵向排列，不并排展示两个权限请求。

## Elevation & Depth

状态通过色阶变化表达，不通过阴影。页面、行、按钮均在同一平面；只有现有主面板和 Bits UI 浮层可以使用根设计系统已有阴影。

权限引导优先作为主面板内的页面或既有模式呈现。若必须使用对话框，其阴影和焦点圈沿用现有浮层，不定义新的 elevation。

## Shapes

按钮使用 `{rounded.md}`，设置行和原位错误区域使用 `{rounded.lg}`，主面板使用 `{rounded.xl}`。所有边框保持 1px；不得用粗边、彩色侧条或新的 radius 强调缺权。

## Components

### Permission row

沿用 Settings Row 两区结构。左侧依次显示权限名称、价值说明与文本状态；右侧只显示与当前状态对应的动作：

- 未请求、已拒绝、已失效：ghost “管理”。
- 检测中：无按钮或禁用当前动作，状态写“正在检测…”。
- 辅助功能已授权：文本写“已就绪”，不显示庆祝图形；可保留 ghost “管理”供撤销或检查。
- 完全磁盘访问：不显示不可验证的全局“已就绪”；写“预览文件时验证”或最近一次具体访问结果。
- 不适用：从该平台或能力对应表面移除，不显示假 disabled 控件。

### Permission guide

一次只处理一个权限。顶部显示权限名称和它解锁的用户价值，中部展示当前状态和不超过三步的系统操作，底部放一个 filled 主动作与必要的 ghost 次动作。

主动作随状态变化：

- “打开辅助功能设置”
- “打开完全磁盘访问设置”
- “重新检测”
- “重新启动 ClipClop”

“稍后处理”“在访达中显示”“取消重新启动”均为 ghost。无法在访达中定位当前 App 时，隐藏该动作。

### Status feedback

使用短文本与可选的 Lucide 状态图标；图标为装饰时 `aria-hidden="true"`。检测、等待、仍未授权、已授权、重启失败均在原位置更新，不弹庆祝 toast。

### Recovery notice

粘贴或预览失败时使用现有 inline error/notice 区域：一句结果、一句降级说明、一个“处理权限”动作。它不覆盖内容，也不自动跳转到完整引导。

## Do's and Don'ts

### Do

- 解释权限给用户带来的直接价值，并允许稍后处理。
- 明确展示正在授权的是当前安装版还是开发构建。
- 使用简短、准确的动作名称；打开系统设置不能写成“完成授权”。
- 授权成功后先预告重新启动，给用户取消机会。
- 在回焦时原位更新状态，并保持键盘焦点稳定。
- 继承根目录 token、强制颜色、深浅主题与 reduced-motion 行为。

### Don't

- 不要求用户一次性授予所有权限。
- 不自动点击系统设置，不读取或修改 TCC 数据库。
- 不把普通拒绝或缺权画成危险状态。
- 不实现 WebView 到系统设置的 App 图标拖拽；第一版使用系统 `+`、Finder 定位和清晰步骤。
- 不声称旧授权可迁移，也不把 debug 与安装版称作同一个授权对象。
- 不新增颜色、卡片体系、权限专用按钮尺寸或共享抽象。
