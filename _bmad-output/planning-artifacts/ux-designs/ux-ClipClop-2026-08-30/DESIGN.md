---
name: ClipClop PowerToys Peek Integration
description: Windows 可选文件预览集成，继承 ClipClop 现有桌面设计系统。
status: draft
sources:
  - ../../../specs/spec-windows-powertoys-peek-integration/SPEC.md
colors:
  background: 'var(--bg-shell)'
  surface: 'var(--bg-raised)'
  surface-hover: 'var(--bg-hover)'
  surface-selected: 'var(--bg-selected)'
  border: 'var(--hairline)'
  text-primary: 'var(--text-1)'
  text-secondary: 'var(--text-2)'
  text-tertiary: 'var(--text-3)'
  action: 'var(--action)'
typography:
  heading:
    note: '继承 --fs-heading 与系统字体'
  body:
    note: '继承 --fs-body 与系统字体'
  ui:
    note: '继承 --fs-ui 与系统字体'
rounded:
  sm: 'var(--radius-sm)'
  md: 'var(--radius-md)'
  lg: 'var(--radius-lg)'
spacing:
  row-block: 12px
  content-gutter: 24px
  related-gap: 3px
components:
  integration-row:
    min-height: 68px
    border-bottom: '{colors.border}'
    action-radius: '{rounded.md}'
---

## Brand & Style

完全继承 ClipClop 当前设置页，不为 PowerToys 引入 Microsoft 品牌色、彩色徽章或独立卡片。它是一项安静的系统集成，而不是推广位。

## Colors

只使用现有 `--*` token。Ready、未安装和权限不可用主要通过文字表达，不依赖红绿颜色。只有真正的启动错误使用现有 `--danger`。

## Typography

标题、说明和按钮沿用设置行的 `strong`、`small`、`button` 层级，不增加新字号。

## Layout & Spacing

集成占 General 页中的一行，位于“快速上手”之后、macOS 专属权限行对应的位置。保持现有 68px 最小行高、24px 文本与动作间距和底部分隔线。

## Elevation & Depth

无新阴影、浮层或卡片。层级由设置页已有 surface 和分隔线承担。

## Shapes

按钮使用 `{rounded.md}`；不新增 pill 状态标签。

## Components

- **Peek integration row**：左侧名称和状态说明，右侧最多一个按钮。未安装显示“了解并安装”；ready 可显示低权重“了解 Peek”；权限不可用不显示动作。
- **Preview menu item**：只在 capability ready 且当前记录是文件时出现，沿用现有 DropdownMenu 样式与 Space keycap。
- **Shortcut row**：只在 capability ready 时把 Space 预览列入 Windows 快捷键说明。

## Do's and Don'ts

| Do | Don't |
|---|---|
| 明确写“由 Microsoft PowerToys 提供，需单独安装” | 暗示 Peek 随 ClipClop 安装 |
| 用普通设置行承载集成状态 | 用 hero、banner、onboarding 推广可选能力 |
| 只在能力可用时显示菜单和快捷键 | 显示 disabled 预览项让用户猜原因 |
| 启动失败使用现有错误样式 | 用绿色成功徽章制造视觉噪声 |
