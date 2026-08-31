---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 6
research_type: 'technical'
research_topic: 'ClipClop macOS 多显示器菜单与面板布局'
research_goals: '评估固定菜单大小是否能解决跨屏故障，并对照官方与开源实现提出方案'
user_name: 'qianfan'
date: '2026-08-28'
revision: 2
revision_date: '2026-08-28'
revision_note: '按 Cargo.lock 锁定依赖源码核验后，修正根因机制、降级链、修复范围与历史方案对比'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-08-28
**Revision:** 2（源码核验后修正）
**Author:** qianfan
**Research Type:** technical

---

## 修订说明（Revision 2）

第一版的方案方向成立：macOS Quick 采用专用 AppKit points 布局、一次性设置完整 NSPanel frame、不引入新依赖。但根因机制、降级链、修复范围和历史方案对比存在实质错误，照第一版落地会保留故障。本版按 `Cargo.lock` 锁定依赖的源码逐条复核后修正。

| 项 | 第一版结论 | 修正后结论 |
|---|---|---|
| Quick 故障机制 | tray 与 monitor 的 Y 原点不同，副屏匹配失败后函数静默返回 | 原点系一致。真实机制是 tao 对每屏各乘自身 scale，混合 DPI 下 physical 空间退化、矩形互相吞并，导致**确定性选错屏** |
| 降级链 | 失败时回退「Tray 事件 X 所在屏」 | **删除该级**。混合 DPI 下 X 同样被错误放大，该回退会原样保留故障 |
| 尺寸错误性质 | 分开发送 position/size 扩大 DPI 竞争窗口 | 两者是同一缺陷，零竞争时尺寸也必错，1×↔2× 间为 2 倍或 0.5 倍偏差 |
| 与自动 DPI 调整竞争 | 列为可能成因 | **排除**。Tauri 不回写 `new_inner_size`，tao 的 `setContentSize` 回写分支不成立 |
| 主面板范围 | 稳定基线，禁止改动 | 对**抖动**成立；对**位置**不成立。`center()` 无光标输入，跨屏会开在错屏 |
| f523f1c 病根 | 鼠标选屏 + 连续设置 position/size | 病根是 physical 往返，且有**两处**域混用。「按光标选屏」从未被回退，它在稳定基线中一直在用 |
| 实现风险 | 未识别 | `setFrame:` 绕过 Tauri 后，tao `shared_state` 缓存尺寸会与实际失同步 |

---

## Research Overview

调研 ClipClop 在 macOS 多显示器、不同分辨率和缩放比例下的菜单栏 Quick 面板与主面板布局。方法包括本地调用链审计、提交历史比对、`Cargo.lock` 锁定依赖源码核验、Apple 官方资料核对，以及 Maccy、ClipBin、AltTab 等开源实现对比。

Revision 2 将主面板的**定位**纳入范围（尺寸逻辑仍保持不变），因为用户报告的「显示位置错乱」包含主面板可能开在错屏这一条链。

---

## Technical Research Scope Confirmation

**Research Topic:** ClipClop macOS 多显示器菜单与面板布局
**Research Goals:** 评估固定菜单大小是否能解决跨屏故障，并对照官方与开源实现提出方案

**Technical Research Scope:**

- Architecture Analysis - 窗口创建、目标屏选择、坐标转换与布局职责
- Implementation Approaches - 固定首选尺寸、可见区域约束和跨屏定位
- Technology Stack - Tauri 2、Rust、NSPanel、AppKit NSScreen
- Integration Patterns - Tauri physical 坐标与 AppKit point 坐标的边界
- Performance Considerations - 每次显示实时计算，避免缓存屏幕布局

**Research Methodology:**

- 当前公开资料与原始源码核验
- 本地代码、提交历史与锁定依赖源码交叉验证
- 对关键根因标注置信度，并区分「源码可推导」与「需真机确认」

**Scope Confirmed:** 2026-08-28（Revision 2 扩展至主面板定位）

---

## 已核验的事实基线

以下均可从锁定版本源码直接读出，是后续所有推导的依据。

**Quick 已有固定首选尺寸**

`tauri.conf.json` 中 quick 为 360×604、`resizable: false`；`src-tauri/src/window.rs:29-30` 常量一致。运行时仅在目标工作区放不下时下调。**再固定一次像素无效，且在小屏上会越界。**

**tray 锚点的坐标域**

`tray-icon-0.24.1/src/platform_impl/macos/mod.rs:515-527`：

```rust
fn get_tray_rect(window: &NSWindow) -> Rect {
    let scale_factor = window.backingScaleFactor();   // tray 所在屏的 scale
    ...
    position: LogicalPosition::new(
        frame.origin.x,
        flip_window_screen_coordinates(frame.origin.y) - frame.size.height,
    ).to_physical(scale_factor),
}

fn flip_window_screen_coordinates(y: f64) -> f64 {   // 同文件 :610
    CGDisplayPixelsHigh(CGMainDisplayID()) as f64 - y
}
```

翻转基准是**主屏高度（points）**，得到「主屏左上角为原点」的 point 值，再乘 tray 所在屏 scale。

**monitor 几何的坐标域**

`tao-0.35.3/src/platform_impl/macos/monitor.rs:216-231`，`position()` 用 `CGDisplayBounds`（同为主屏左上角原点的 points），`size()` 用像素数，**两者都乘该屏自身的 scale**。

结论：**两者原点系一致，差异只在缩放乘数。** 这推翻了第一版「Y 原点错配」的假设。

**落地端消费的是「窗口当前屏」的 scale**

`tao-0.35.3/src/platform_impl/macos/window.rs`：

```rust
pub fn set_outer_position(&self, position: Position) {   // :728
    let scale_factor = self.scale_factor();              // 窗口当前屏
    let position = position.to_logical(scale_factor);
    ...
}
pub fn set_inner_size(&self, size: Size) {               // :755
    let scale_factor = self.scale_factor();              // 同一缺陷
    ...
}
pub fn scale_factor(&self) -> f64 {                      // :885
    NSWindow::backingScaleFactor(&self.ns_window) as _
}
```

**position 与 size 是同一个缺陷，不是两个独立竞争点。**

**work_area 是 Tauri 侧扩展，不在 tao 内**

`tauri-runtime-wry-2.11.4/src/monitor/macos.rs:7-35` 用 `MonitorExt` trait 提供 `work_area()`，内部读 `NSScreen.visibleFrame` 后按该屏 scale 转 physical。与 `position()` 同域，自身自洽。

**已排除的成因**

`tauri-runtime-wry-2.11.4/src/lib.rs:515-521` 转发 `ScaleFactorChanged` 时不修改 `new_inner_size`，因此 `tao-0.35.3/src/platform_impl/macos/app_state.rs:239-243` 的 `old_size != size` 不成立，`setContentSize` 不会触发。**抖动不来自 tao 的 DPI 回写。**

**方案可行性前置条件（均已确认）**

- `tauri-nspanel` 的 `as_panel()` 返回 `&objc2_app_kit::NSPanel`（`src/lib.rs:54`），可直接 `setFrame:display:`。
- macOS 上 `set_resizable(false)` 只改 styleMask，**不设** min/max content size（`tao window.rs:795-813`），因此不会阻挡 `setFrame:`。

---

## 根因：Quick 面板

**混合 DPI 下 Tauri 的 physical 全局空间是退化的。** tao 对每屏各乘自身 scale，屏幕矩形不再平铺，而是互相重叠或留空隙。

以最常见配置为例——内置 Retina A（points `0,0,1440×900` @2×）+ 外接 1× B（points `1440,0,1920×1080` @1×）：

| | points | Tauri physical |
|---|---|---|
| A | `0,0,1440×900` | `0,0,2880×1800` |
| B | `1440,0,1920×1080` | `1440,0,1920×1080` |

**A 的 physical 矩形吞掉了 B 的大半坐标区间。**

推导链（tray 在 B 上，points x≈2400）：

1. 锚点 → physical `(2400, 12)`（乘 B 的 scale 1）。
2. `window.rs:231-236` 的 `monitor_contains_point` 判定：x 2400 < 2880 ✓、y 12 < 1800 ✓ → **落进 A**。`CGGetActiveDisplayList` 主屏在前，A 先被命中 → **确定性选错屏**，而非匹配失败静默返回。
3. 用 A 的 work_area 与 scale 2 计算 → size `720×1208` physical，position 约 `(2040, 50)`。
4. `set_position` 按**窗口当前屏** scale 反算。窗口上次在 B（scale 1）→ 解读为 points `(2040, 50)`，落到 B 上。
5. `set_size` 同样按当前屏 scale 反算 → `720×1208` **points**，是意图尺寸的两倍，且超出 B 的屏高。

对应用户症状：选错屏 → 位置错乱；scale 反算 → 尺寸异常；「旧几何显示 → 移动 → 改尺寸」三步 async 落地 → 可见中间态 → 抽搐。

**置信度：高（源码可推导，数学上确定）。** 同 DPI 时 physical 空间自洽，故障只在混合 DPI 出现——而「Retina 内置 + 1× 外接」是最常见配置。仍需混合 DPI 真机日志确认是否并存第二成因，这是实施第一步。

---

## 根因：主面板（第一版遗漏）

`src-tauri/src/window.rs:180-188`：

```rust
#[cfg(target_os = "macos")]
if let Some((width, height)) = macos::cursor_screen_work_area() {
    resize_panel(&window, width, height);   // 尺寸取自「光标所在屏」visibleFrame
} else {
    resize_panel_for_monitor(&window);
}
let _ = window.center();                    // 定位无任何光标输入
```

**尺寸链是正确的且必须保留。** `cursor_screen_work_area()` 返回 AppKit points，`resize_panel` 全程走 `LogicalSize`，`tao::set_inner_size` 对 Logical 入参不做 scale 换算，无往返 → 不抖动。这是「稳定基线」成立的真实原因。

**定位链有缺陷。** `center()` 在 macOS 上直连 AppKit `NSWindow::center()`（`tauri-runtime-wry-2.11.4/src/window/macos.rs:39-42`）：

- 入参中**不含任何光标信息**，因此不可能选到光标所在屏——这是逻辑必然，不依赖 Apple 文档细节。
- 按 full frame 而非 `visibleFrame` 居中，不扣除菜单栏与 Dock。

后果：光标在 B、窗口上次在 A 时，按 B 的工作区算尺寸却居中到 A。**与 DPI 无关，同 DPI 也会发生。**

因此 Revision 2 的修复范围为：**主面板只改定位，尺寸链一字不动。**

---

## 与被回退方案（f523f1c）的对比

这是判断「是否回老路」的关键。核验发现 `f523f1c` 的 `place_panel` 有**两处**坐标域混用，不是一处。

| | f523f1c（已回退） | 本方案 |
|---|---|---|
| 选屏输入 | `window.cursor_position()` → **physical** | `NSEvent::mouseLocation()` → **points** |
| 选屏比较 | `monitor_from_point` 内部比 `CGDisplayBounds` → **points** | `NSScreen.frame` → **points** |
| 选屏是否同域 | ❌ physical 比 points | ✅ points 比 points |
| 几何来源 | `Monitor.work_area()`，每屏各乘自身 scale | `NSScreen.visibleFrame`，原生 points |
| 落地方式 | `set_position` + `set_size` 两次 async，各按窗口当前屏 scale 反算 | `setFrame:display:` 一次，无反算 |
| 落地是否同域 | ❌ 目标屏 scale 产出、当前屏 scale 消费 | ✅ 全程 points |

选屏层的域混用为本轮新发现：`tauri-2.11.5/src/window/mod.rs:1748` 的 `cursor_position()` 明确返回 `PhysicalPosition`，而 `tao monitor.rs:163-173` 的 `from_point` 拿它去比 `CGDisplayBounds` 的 point 值。2× 主屏上光标 physical `(2000,500)` 会被当成 point `(2000,500)` 判定。这解释了该提交实机表现为「严重判断错误 + 尺寸抽搐」——两层同时错。

**关键区分：f523f1c 的病根是 physical 往返，不是「按光标选屏」。**

证据：当前被认定为稳定基线的 `f29c2cb` 中，`cursor_screen_work_area()` 用的正是 `NSEvent::mouseLocation()` + `NSScreen` 选屏（`window/macos.rs:3-22`）。**「按光标选屏」从未被回退，它一直在线上运行，且属于稳定的那一半。** 被回退的只有 physical 往返。

第一版把这两件事捆在一起否定，是它最大的逻辑跳跃——它自己在 Quick 方案里又用回了 `mouseLocation` 选屏。

本方案沿用 `f29c2cb` 中被证明稳定的部分（AppKit 取点选屏），丢弃 `f523f1c` 中被证明有害的部分（physical 往返）。**方向与回退教训一致，不是绕回去。**

补充：`b784734` 引入的当前 `layout_quick_panel`，用的正是 `f523f1c` 被淘汰的同一套 `set_position` + `set_size` physical 模式。主面板已脱离该模式，Quick 尚未。本方案是把这条已被淘汰的路径从 Quick 上一并移除。

---

## 开源实现对照

共同模式是让选屏、约束和最终 frame 留在**同一个坐标域**，而不在 logical/physical 与不同屏 scale 之间往返。

- **Maccy**：用 `NSEvent.mouseLocation` 选 `NSScreen`，按 `visibleFrame` 夹紧弹窗；状态栏局部位置经 window-to-screen 转换。[PopupPosition.swift](https://github.com/p0deje/Maccy/blob/master/Maccy/PopupPosition.swift)
- **ClipBin**：Tauri/Rust 实现中显式处理 AppKit bottom-left 与跨平台 top-left 坐标，按目标 `visibleFrame` 布局。[tray.rs](https://github.com/wwwppp0801/clipbin/blob/main/src-tauri/src/tray.rs#L155)
- **AltTab**：显示时选鼠标所在 `NSScreen`，按该屏尺寸限制并居中。[SwitcherPanel.swift](https://github.com/sergio-farfan/alttab-macos/blob/main/AltTab/AltTab/SwitcherPanel.swift#L234)
- **Apple**：`visibleFrame` 为排除菜单栏、Dock 等不可用区域后的区域，应按当前状态实时读取。[NSScreen.visibleFrame](https://developer.apple.com/documentation/appkit/nsscreen/visibleframe)

---

## 推荐架构

**核心原则：让一次 Tray 点击只产生一个坐标域里的一个矩形，一次落地。**

```
Tray 点击（主线程）
  → NSEvent::mouseLocation()                    ← AppKit points，唯一取点
  → NSScreen::screens() 找包含该点的屏           ← 同域比较，不跨 scale
  → 读该屏实时 visibleFrame                      ← 已扣菜单栏 / Dock
  → 以 360×604 points 为首选，放不下才按边距缩小
  → 锚点 X 居中、贴 visibleFrame 顶边下方、四边夹紧
  → panel.as_panel().setFrame:display:          ← 一次落地，无中间态
```

**三个不变量**

1. 全程 points，不进 physical。
2. 选屏、约束、落地用同一块 `NSScreen`。
3. 几何只写一次。

不变量 3 消除抖动（抖动源是可见中间态）；不变量 1、2 消除位置与尺寸错误（错误源是跨 scale 换算）。Windows 保持现有 physical 路径——该平台 monitor API 自洽，无此缺陷。

**降级链（相对第一版的关键修正）**

`mouseLocation` 所在屏 → `NSScreen::mainScreen`（菜单栏所在屏）→ `screens().first()`，每级失败记 warning。

**不得加入「按 Tray 事件 X 选屏」这一级** —— 混合 DPI 下 X 同样被错误放大，该回退会原样保留故障。这是第一版必须删除的内容。

**实现约束**

- 面板顶边直接取 `visibleFrame.maxY`，不自行计算菜单栏高度（`visibleFrame` 已扣除）。
- macOS Quick 路径清零 Tauri 几何调用。绕过 Tauri 直接 `setFrame:` 后，tao 内部 `shared_state` 缓存尺寸会与实际失同步，后续 `set_size`/`inner_size` 将读到脏值。当前 `layout_quick_panel` 是唯一处，整体替换即可。
- frame 修改必须在主线程执行。
- 不缓存 `NSScreen` 或 `visibleFrame`；显示器排列、Dock、菜单栏均可能变化。

**主面板**

`center()` 换成对光标屏 `visibleFrame` 的显式居中，同样在 points 域一次 `setFrame`。**尺寸链保持不动。** 这不会带回 `place_panel` 的抖动——抖动源是 physical 往返，不是选屏方式。

**可观测性**

`window.rs:268-269` 两个 `let _ =` 与匹配失败的静默返回补 warning。诊断日志同时记录 physical 与 points 两套数值，否则无法区分本文根因与并存的第二成因。日志含目标 screen frame、visibleFrame、scale 与最终 frame；不记录剪贴板内容。

---

## 方案对比

| 方案 | 结果 | 决策 |
|---|---|---|
| 固定绝对像素尺寸 | 不解决选屏与坐标错误；小屏会越界 | 拒绝 |
| 恢复共享 `place_panel` | 混用两个域，已验证有害 | 拒绝 |
| 继续修补 Tauri physical 计算 | physical 空间在混合 DPI 下本身退化 | 无解 |
| 全部窗口迁移 AppKit | 统一坐标，但会触碰主面板稳定的尺寸链 | 过度 |
| macOS Quick 走 AppKit points，主面板只改定位 | 改动小、单一坐标域、不触碰稳定链 | **推荐** |

---

## 实施路线

三个独立提交，后两个可单独回滚：

**① 诊断日志（可立即做，零风险）**

补充记录 physical 与 points 双套数值，在混合 DPI 真机上点击一次，确认本文根因或识别并存的第二成因。

**② Quick 的 AppKit 布局**

- `src-tauri/src/window/macos.rs`：Quick 专用的 screen 与 frame 计算。
- `src-tauri/src/window.rs`：macOS Quick 路由到原生布局；Windows 保持 `layout_quick_panel`。
- 增加纯几何测试。

**③ 主面板定位**

用光标屏 `visibleFrame` 显式居中，一次 `setFrame`。独立提交，便于单独回滚。

**测试策略**

- 纯几何函数覆盖 points 域：负 X、上下排列、小屏缩小、左右边缘夹紧。
- 真机矩阵（后附）。

**真机测试矩阵**

| 场景 | 验证点 |
|---|---|
| 副屏在左 / 右 | Tray 所在屏正确、X 夹紧 |
| 副屏在上 / 下 | Y 匹配不依赖仅 X |
| 1× ↔ 2× 混合 DPI | 选屏正确、视觉尺寸稳定、无抖搐（**本次核心**） |
| 小尺寸屏幕 | 在 `visibleFrame` 内缩小 |
| 菜单栏位于副屏 | Quick 出现在该菜单栏下方 |
| Dock 左 / 右 / 底 | 不覆盖不可用区域 |
| 拔插显示器 | 下次点击重新读取，无缓存 |
| 连续开关 Quick | 位置尺寸稳定 |
| 主面板跨屏快捷键 | 光标所在屏打开并在 `visibleFrame` 内居中 |
| Windows Tray | 原有 physical 布局无回归 |

---

## 修复后预期效果

**能达成**

- Tray 点击后 Quick 稳定出现在 Tray 所在屏，混合 DPI 不再选错。
- 1× / 2× 视觉尺寸一致锁定 360×604 points，不再出现 2 倍或半尺寸。
- 从「旧几何显示 → 移动 → 改尺寸」的可见中间态消失，抖动消除。
- 面板完整落在实时 `visibleFrame` 内，不压菜单栏和 Dock。
- 拔插显示器、改排列、移 Dock 后下次点击即生效（不缓存几何）。
- 主面板在光标屏打开并在 `visibleFrame` 内居中。
- Windows 与前端零改动。

**不覆盖**

Tauri/tao 在 macOS 混合 DPI 下 monitor API 的 physical 空间退化**依然存在**，本方案是绕行而非上游修复。其他代码若用 `available_monitors()` 做几何判断，同样的坑还在。

**因果链确认**

推导链通过源码可直接验证，数学上确定。仍需混合 DPI 真机日志确认是否并存第二成因——这是实施第一步。

---

## 风险与置信度

| 论断 | 置信度 | 依据 |
|---|---|---|
| Quick 已固定首选尺寸，问题不是缺少固定尺寸 | 高 | 配置与常量直读 |
| tray 与 monitor 原点系一致，差异仅在缩放乘数 | 高 | 双侧源码直读 |
| 混合 DPI 下 physical 空间退化 → 确定性选错屏 | 高 | 源码可推导 |
| position 与 size 是同一缺陷，尺寸错误确定发生 | 高 | 源码直读 |
| 抖动不来自 tao 的 DPI 回写 | 高 | 回写分支不成立 |
| `center()` 无光标输入，主面板跨屏会开错屏 | 高 | 入参逻辑必然 |
| f523f1c 病根是 physical 往返而非按光标选屏 | 高 | 两处域混用 + 稳定基线仍在用光标选屏 |
| 上述机制是当前实机故障的**唯一**主因 | 中高 | 需混合 DPI 真机日志确认无第二成因 |

**风险缓解**

- 鼠标点不在任何屏幕：按修正后的降级链回退并记 warning。
- frame 修改必须在主线程。
- macOS Quick 路径不得残留 Tauri 几何调用（避免 `shared_state` 脏值）。
- 主面板尺寸链不得改动；定位改动独立提交，便于单独回滚。
- 不复用到 Windows。

**回退策略**

若原生 layout 失败，保留窗口上次稳定 frame 并记 warning；**不要回退到跨 scale 的 physical 重算。**

---

## 结论

ClipClop 不需要重新设计所有窗口，也不需要再次固定菜单像素大小。最小且稳健的修复是切断 macOS 上的跨屏 physical 坐标往返：让一次 Tray 点击产生一个 `NSScreen`，在该屏的 points / `visibleFrame` 中一次生成并应用完整 NSPanel frame；主面板同法只修正定位，保留其已稳定的 points 尺寸链。

本方案不共用任何 `f523f1c` 的失败机制——它沿用的是当前稳定基线中一直在运行的 AppKit 取点选屏，丢弃的是被回退的 physical 往返。

---

## 参考资料

**官方文档**

- [Apple NSScreen](https://developer.apple.com/documentation/appkit/nsscreen)
- [Apple visibleFrame](https://developer.apple.com/documentation/appkit/nsscreen/visibleframe)
- [Apple window-to-screen conversion](https://developer.apple.com/documentation/appkit/nswindow/convertpoint%28toscreen%3A%29)
- [Tauri Monitor API](https://docs.rs/tauri/latest/tauri/window/struct.Monitor.html)

**开源实现**

- [Maccy PopupPosition](https://github.com/p0deje/Maccy/blob/master/Maccy/PopupPosition.swift)
- [ClipBin tray layout](https://github.com/wwwppp0801/clipbin/blob/main/src-tauri/src/tray.rs#L155)
- [AltTab SwitcherPanel](https://github.com/sergio-farfan/alttab-macos/blob/main/AltTab/AltTab/SwitcherPanel.swift#L234)

**锁定依赖源码核验位置**

| 论断 | 位置 |
|---|---|
| tray 锚点乘 tray 屏 scale、按主屏高度翻转 | `tray-icon-0.24.1/src/platform_impl/macos/mod.rs:515-527, 610` |
| monitor position/size 各乘自身 scale | `tao-0.35.3/src/platform_impl/macos/monitor.rs:216-239` |
| `from_point` 比 `CGDisplayBounds`（points） | `tao-0.35.3/src/platform_impl/macos/monitor.rs:163-173` |
| `set_outer_position` / `set_inner_size` 用窗口当前屏 scale | `tao-0.35.3/src/platform_impl/macos/window.rs:728-760, 885` |
| `set_resizable(false)` 只改 styleMask | `tao-0.35.3/src/platform_impl/macos/window.rs:795-813` |
| DPI 回写分支 | `tao-0.35.3/src/platform_impl/macos/app_state.rs:220-244` |
| `work_area()` 为 Tauri 扩展 trait | `tauri-runtime-wry-2.11.4/src/monitor/macos.rs:7-35` |
| `ScaleFactorChanged` 不回写 size | `tauri-runtime-wry-2.11.4/src/lib.rs:515-521` |
| `center()` 直连 AppKit | `tauri-runtime-wry-2.11.4/src/window/macos.rs:39-42` |
| `cursor_position()` 返回 physical | `tauri-2.11.5/src/window/mod.rs:1748` |
| `as_panel()` 返回 NSPanel | `tauri-nspanel@a3122e8/src/lib.rs:54` |

---

**Revision 2 Date:** 2026-08-28
**Source Verification:** Apple 官方文档、开源原始代码、`Cargo.lock` 锁定依赖源码逐条核验
**Confidence Level:** High（机制源码可推导）；「唯一主因」需混合 DPI 实机日志确认

