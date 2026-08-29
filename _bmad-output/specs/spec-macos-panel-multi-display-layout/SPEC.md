---
id: SPEC-macos-panel-multi-display-layout
companions:
  - implementation.md
sources:
  - ../../planning-artifacts/research/technical-macos-multi-display-panel-research-2026-08-28.md
---

> **规范契约。** 本 SPEC 与 `companions:` 中的文件共同定义待实现、测试和验收的完整范围。

# macOS 面板多显示器布局

## Why

ClipClop 的 Quick 与主面板在混合缩放、多显示器环境中可能选错屏幕、尺寸异常或出现可见几何跳变。布局必须避开 macOS 上跨显示器 physical 坐标的缩放往返，同时保留现有 WebView UI、窗口尺寸策略和 Windows 行为。

## Capabilities

- id: CAP-1
  intent: 用户点击 macOS 托盘图标时，Quick 面板在点击所在显示器的安全可见区域内稳定展开。
  success: 在左右、上下排列的 1×/2× 混合缩放显示器上，Quick 始终完整位于点击所在屏幕的 `visibleFrame` 内，首选内容尺寸保持 360×604 points，小屏仅按既有边距规则缩小，显示过程无可见移动或尺寸跳变。

- id: CAP-2
  intent: 用户通过快捷键或应用入口打开 macOS 主面板时，面板在光标所在显示器的安全可见区域内居中。
  success: 主面板继续使用现有 `panel_content_size()` 结果，在光标屏 `visibleFrame` 内一次应用最终尺寸和位置；不同缩放显示器之间连续呼出时尺寸策略不变且无抖动。

- id: CAP-3
  intent: 布局失败可被诊断且不会把面板移动到不可访问位置。
  success: 选屏或原生 frame 应用失败会记录不含剪贴板内容的 warning；显示器变化后不复用已离屏的旧 frame。

## Constraints

- macOS 的选屏、约束和最终 frame 计算必须全程使用 AppKit points，并在主线程一次应用完整 frame；不得在同一路径混用 Tauri physical `set_position`/`set_size`。
- Quick 与主面板仍是现有 Tauri WebView + NSPanel，不重写为原生 AppKit UI。
- Quick 首选内容尺寸 360×604、主面板 `panel_content_size()`、`SHADOW_INSET` 及小工作区缩小规则保持不变。
- CAP-1 与 CAP-2 分为独立实现和验证步骤；先完成 Quick，主面板不得搭车修改。
- Windows 路径、窗口生命周期、焦点、粘贴、预览和前端行为保持不变。
- 原生 frame 尺寸必须明确区分 content rect 与 window frame，不得假设两者恒等。

## Non-goals

- 不修复 Tauri/tao 上游的 macOS 混合 DPI monitor 表示。
- 不恢复历史 `place_panel` physical 往返方案。
- 不新增窗口定位依赖，不缓存显示器对象或几何。
- 不在 CAP-1 中修改主面板定位。

## Success signal

在 Retina 内屏与 1× 外屏组成的左右及上下布局中，连续从不同屏幕打开 Quick 与主面板：两者均出现在预期屏幕、完整位于 `visibleFrame`、视觉尺寸稳定且无抽搐；Windows 行为和所有现有自动化检查保持通过。
