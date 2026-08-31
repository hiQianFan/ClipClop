## Context

Quick 使用与主面板相同的 `query_history` 后端，但当前固定请求第一页并把条目截到窗口可见数量。标准 Quick 视觉为十条记录，但固定 40px 行没有填满列表的可用高度，因而在列表底部形成额外 gap。

## Decisions

### 分页栏位于 header 右侧

分页栏放在现有 header 右侧，不增加 header 高度。单页显示低权重 `1/1` 与禁用箭头；多页显示 `current/total`。使用原生按钮与现有 token。

### 十槽网格消除额外 gap

列表区使用十个等分槽位填满自身可用高度，而不是十个固定 40px 行后留下剩余空间。记录占据对应槽位；最后一页缺少的槽位只保留几何，不进入可访问树、不响应 hover/focus。底部应用菜单位置和面板高度保持不变。

### Quick 保持轻量分页状态

`queryHistory` 增加可选 `pageSize`，默认值仍为 10。Quick 保存完整 `HistoryPage`，不使用会加载 detail 的 `HistorySession`。标准窗口请求十条；若平台因工作区约束缩短窗口，则以实际槽位数请求，避免查询后隐藏记录。

### 原子页面切换

首次加载使用既有骨架；后续翻页保留当前列表直至新页成功返回。递增 request version，只允许最后一次请求提交。失败保留页面、页码和选择，并沿用 inline error。

### 连续键盘浏览

PageUp/PageDown 显式翻页；当前页末项 Down 进入下一页首项，首项 Up 进入上一页末项。Home/End 和数字键保持页内语义。Left/Right 保留，不新增占用。

### 新鲜度优先

每次 Quick 显示及 `history_changed` 都刷新第一页并选中最新记录。`onfull` 继续携带当前 `selectedId`，让完整历史接续当前上下文。

## Risks

| 风险 | 缓解 |
|---|---|
| 快速翻页旧响应覆盖新页 | request version |
| 分页失败造成列表闪空 | 成功后原子替换 |
| 小工作区隐藏同页尾部记录 | page size 使用实际槽位数 |
| 组件状态变重 | 只保存 HistoryPage、请求版本和 pending target |
