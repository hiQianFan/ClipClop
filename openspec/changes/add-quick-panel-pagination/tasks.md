## 1. 查询与键盘护栏

- [ ] 1.1 `api.ts` 允许 Quick 指定 page size，主面板默认仍为 10，并补 API 测试
- [ ] 1.2 `quick-keyboard.ts` 增加 PageUp/PageDown 和 Up/Down 页边界 action，并补纯函数测试

## 2. Quick 分页状态

- [ ] 2.1 `QuickPanel.svelte` 保存完整 HistoryPage，标准窗口每页十条
- [ ] 2.2 请求版本确保最后请求胜出；失败保留原页、页码和选择
- [ ] 2.3 Quick 重开与 history_changed 回第一页，翻页后选择目标页首项或末项
- [ ] 2.4 打开完整历史继续传递当前 selectedId

## 3. 固定布局

- [ ] 3.1 十槽网格填满列表可用高度；不足时渲染不可聚焦空槽，窗口和底部菜单不移动
- [ ] 3.2 在 header 右侧加入分页栏；单页显示 1/1 和禁用按钮
- [ ] 3.3 分页控件使用现有 token、原生按钮、i18n、焦点可见与 ARIA 标签

## 4. 验收

- [ ] 4.1 组件测试覆盖 1/10/11/23 条、末页空槽、按钮状态、失败和乱序
- [ ] 4.2 执行 `pnpm check`、`pnpm test`、`pnpm build`、`git diff --check`
- [ ] 4.3 执行 `openspec validate add-quick-panel-pagination --strict`
- [ ] 4.4 macOS/Windows 人工验证分页、键盘、主题、重开与固定布局
