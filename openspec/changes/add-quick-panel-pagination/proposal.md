## Why

Quick 面板目前只查询并展示第一页。超过十条后必须切到完整历史，且当前十行没有填满列表区，在列表与底部应用菜单之间形成额外断层。

## What Changes

- 在 Quick header 右侧增加紧凑分页栏。
- 正常窗口固定每页十条；最后一页不足十条时保留空槽，不改变窗口或菜单位置。
- 十槽网格填满列表可用高度，消除列表底部额外 gap。
- 支持分页按钮、PageUp/PageDown，以及 Up/Down 在页边界连续浏览。
- Quick 重开或历史变化时回到第一页；打开完整历史继续携带当前选择。
- 翻页采用最后请求胜出，失败时保留原页面。

## Non-Goals

- 不复制主面板拖拽 scrubber、触觉反馈或复杂动画。
- 不修改 Quick 窗口尺寸、Rust 查询或数据库。
- 不引入 HistorySession、新依赖或新状态层。

## Impact

- `src/lib/history/api.ts`、`quick-keyboard.ts`、`QuickPanel.svelte` 及对应测试。
- i18n 分页文案。
- 不影响当前工作区其他发布流程改动。
