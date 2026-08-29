# 实现与验收约束

## 阶段 1：Quick

- Tray 点击回调同步捕获 `NSEvent::mouseLocation()`，以该 point 选择 `NSScreen`；该值必须在任何异步调度前捕获。
- 目标屏为包含点击 point 的 screen；无法命中时依次尝试 `NSScreen::mainScreen` 和 `screens().first()`，但不得把 `mainScreen` 描述为菜单栏屏幕。
- 使用目标 screen 的实时 `visibleFrame`，以 360×604 points 为首选 content size，每边保留 6 points；工作区不足时缩小至可用范围。
- X 以点击 point 为锚点并夹紧，面板顶边贴 `visibleFrame.maxY`；验收语义是“点击锚点”，不是“托盘图标矩形中心”。
- 使用 `contentRectForFrameRect` / `frameRectForContentRect` 或等价 AppKit API 明确转换 content size 与 window frame。
- 通过 `Panel::as_panel()` 在主线程一次调用 `setFrame:display:`；macOS Quick 路径不得再调用 Tauri `set_position` 或 `set_size`。
- 若无法得到有效 screen 或有效正尺寸 frame，记录 warning 并不显示不可访问的新 frame。

## 阶段 2：主面板

- `cursor_screen_work_area()` 扩展为返回光标 screen 的完整 `visibleFrame` points。
- `panel_content_size()`、800×600 内容上限和双侧 `SHADOW_INSET` 计算保持原样。
- macOS 不再先异步 `set_size` 再同步移动；以同一尺寸计算结果和 `visibleFrame` origin 生成居中 content rect，再转换为 window frame，一次 `setFrame:display:`。
- 阶段 2 独立提交；阶段 1 的验证结果不能替代主面板真机验证。

## 自动化检查

- 纯几何测试覆盖：正常屏、负 X、上下排列、小工作区缩小、左右夹紧、非零 visibleFrame origin、无有效空间。
- 现有 Rust 与前端检查保持通过。

## 真机矩阵

- 1× 与 2× 显示器左右互换主屏。
- 1× 与 2× 显示器上下排列。
- 菜单栏位于不同屏幕；Dock 位于左、右、底。
- 连续跨屏打开、显示器拔插后再次打开、小分辨率显示器。
- Quick 完成后验证主面板完全未回归；主面板阶段完成后重复全矩阵。
