# ClipClop 前端架构评估

日期: 2026-08-30 · 范围: `src/` 全部前端模块 · 性质: **只评估,不修改代码**

所有数字均来自对当前工作区的实测(非估算)。后端 (`src-tauri/`) 按要求不在范围内。

---

## 1. 结论摘要

前端的**逻辑分层是健康的**,问题几乎全部集中在**视图层与样式层**:

| 维度 | 评价 | 说明 |
|---|---|---|
| 逻辑/状态分层 | 良好 | `session` / `preview-session` / `presentation` / `keyboard` / `pager` 纯逻辑已从组件剥离,且都有测试 |
| 组件拆分 | **不合格** | 2 个组件 800/669 行,承担 5~6 个职责;`src/lib/components/` 只有 1 个通用组件 |
| CSS 层级 | **不合格** | 无共享样式层,90 处 `:global()` 各自为政;单行 4318 字符的样式块 |
| Design Token | 部分不合格 | 色彩/字号/圆角落地良好;**间距 token 12 个全部零引用**,被 400+ 处硬编码 px 取代 |
| 架构文档 | 需补章节 | `docs/architecture.md` 无前端组件与样式边界的约定;`DESIGN.md` 的规则无代码层承载点 |

你对 Settings 页面的判断是准确的,而且它不是孤例——同样的问题在 `HistoryWorkspace` 上更严重。

---

## 2. 组件拆分

### 2.1 文件规模实测

| 文件 | 行数 | 其中样式 | 测试 |
|---|---|---|---|
| `history/HistoryWorkspace.svelte` | **814** | 44 | **无** |
| `settings/SettingsView.svelte` | **669** | **281** | 132 行 |
| `i18n/catalogs.ts` | 450 | — | 纯数据,合理 |
| `onboarding/OnboardingView.svelte` | 437 | ~150 | 31 行 |
| `history/HistoryList.svelte` | 425 | 68 | 57 行 |
| `updater/api.ts` | 311 | — | 69 行 |
| `updater/store.svelte.ts` | 262 | — | 87 行 |
| `history/preview-session.svelte.ts` | 217 | — | 164 行 |
| `history/QuickPanel.svelte` | 168 | 26 | **无** |
| `history/ClipPreview.svelte` | 161 | 45 | 100 行 |

### 2.2 `HistoryWorkspace.svelte` (814 行) — 最严重

它同时是:根路由容器、视图路由器、应用菜单、窗口级键盘路由器、删除确认流程、面板召唤生命周期响应器、设置/引导的宿主。

实测承载的状态:**30 个顶层 `let` 声明**(其中 `$state` 22 个),包含两组语义重叠的模式变量:

```ts
let mode = $state<InteractionMode>("browse");   // browse|search|menu|confirmation|file-tablist
let view = $state<"loading"|"history"|"settings"|"onboarding">("loading");
let menuOpen = $state(false);      // 与 mode==="menu" 重叠
let appMenuOpen = $state(false);   // 与 mode==="menu" 重叠
let deletePending = $state(false); // 与 mode==="confirmation" 重叠
```

`mode` 与 `menuOpen`/`appMenuOpen`/`deletePending` 表达的是同一件事的两套真相,需要在 8 处回调里手工同步(如 `onOpenChange={(open) => { menuOpen = open; mode = open ? "menu" : "browse"; if (open) appMenuOpen = false; }}`)。这是状态机被拆成布尔标志后的典型代价。

**它没有任何测试**——814 行、承担窗口键盘路由这种高风险逻辑,却是全项目唯一无测试的大组件。原因很直接:要测它必须 mock 掉 8 个模块。这是拆分缺失直接造成的可测试性损失。

建议的职责边界(不含实现):

```
HistoryWorkspace         视图路由 + 会话装配                       ~150
├─ AppTitleBar           标题栏 + 品牌 + 应用菜单                   ~90
├─ HistoryView           history 视图装配 (List + Preview + 动作栏)  ~200
│  └─ HistoryActionBar   动作栏 + 操作菜单 + 删除确认                ~150
└─ (既有) SettingsView / OnboardingView
```

另需抽出的非组件逻辑:
- `window-router.ts` — 窗口级键盘路由(`onKeydown` / `onListKeydown` / `onWindowKeydown` 共 130 行纯分支,可脱离 DOM 单测,与既有 `keyboard.ts` 同层)
- `panel-lifecycle.ts` — `onPanelShown` / `resumeBrowse` / `resetToLatest` 的召唤策略

### 2.3 `SettingsView.svelte` (669 行) — 你发现的那个

6 个 tab 的模板、逻辑、样式全部同文件。其中:
- 模板层:`{#if tab === "general"}...{:else if}` 链,单个 tab 模板最长 **8 行超长单行**(如 line 515 更新头部为一行)
- 逻辑层:同时含更新任务编排、发布记录列表键盘导航、快捷键录制、主题/语言预览、清空确认
- 样式层:**281 行**,含一个 **4318 字符的单行**(line 590,内含 53 条规则)

按 tab 边界拆分是自然的(每个 tab 是独立的 `{#if}` 分支,无交叉引用),另需抽出:
- `SettingRow` — `DESIGN.md` 已用整节定义了"设置行两区契约",但代码中它只是散落的 `.row` class,**没有组件承载**,靠 6 处重复的 class 组合维持
- `ShortcutRecorder` — 录制状态机 + 键位显示
- `ReleaseNotes` — 发布记录浏览器(列表 + 详情 + 骨架屏,占样式约 90 行)
- `UpdatePanel` — 更新状态与动作

### 2.4 其余

- `OnboardingView` (437 行):其样式里明确写着"仿 HistoryList 行"并**重新实现了一遍** `.mini-row` / `.num` / `.lead` / `.snippet`,与 `HistoryList` 的真实行样式并行维护。这是缺少共享 Row 组件的直接后果,两边会各自漂移。
- `HistoryList` (425 行):职责集中(列表 + 筛选 + 分页拖拽),但分页 scrubber 的指针/滚轮/长按重复逻辑约 120 行,与列表渲染无关,可独立为 `PageScrubber`。
- `QuickPanel` (168 行):规模合理,但**无测试**,且重复实现了一遍行样式(第三份)。
- `src/lib/components/` 下只有 `AppSelect` 一个通用组件——这是"没有组件库"的最直接证据。

---

## 3. CSS 层级

### 3.1 缺少共享样式层

当前只有两层:`app.css`(token + 极少量 base)与**各组件内联 `<style>`**。中间缺失的"共享组件样式层"被 `:global()` 顶替,实测 **90 处**,分布:

| 文件 | `:global()` 处数 |
|---|---|
| `SettingsView.svelte` | 26 |
| `HistoryList.svelte` | 22 |
| `AppSelect.svelte` | 13 |
| `HistoryWorkspace.svelte` | 17 |
| `OnboardingView.svelte` | ~10 |
| `ClipPreview.svelte` | 5 |

两类用途混在一起,应区别对待:

1. **合法用途**:给 bits-ui 无头组件的渲染产物挂样式(`.select-trigger`、`.filter-popover`、`.settings-nav`)。Svelte 作用域无法覆盖第三方 DOM,这类 `:global()` 是必需的。
2. **问题用途**:把本该共享的样式声明在某个组件里,让别人隐式依赖。典型是 `HistoryWorkspace` 内声明的 `:global(.ghost)` / `:global(.destructive)` / `:global(.menu)` / `:global(.menu-separator)` —— `SettingsView` 与 `QuickPanel` 都在用 `.ghost`、`.menu-separator`,但样式来源在 `HistoryWorkspace`。**若 HistoryWorkspace 未挂载,这些样式不存在**;拆分它时会静默破坏其他组件。

这是当前 CSS 层级里风险最高的一条,且正好挡在 `HistoryWorkspace` 拆分的路上。

### 3.2 跨组件重复

实测重复定义:

- `kbd { font:var(--fs-caption)/var(--lh-snug) var(--mono); ... }` — 在 `ClipPreview` / `HistoryWorkspace` / `HistoryList` 中**逐字符相同地出现 3 次**,`OnboardingView` 里第 4 次(选择器不同,声明相同)
- 焦点环 `outline:2px solid var(--text-1); outline-offset:2px` — 9 处硬写,而 `app.css` 已有 `:focus-visible` 全局规则(用的是 `--text-2`),两套并存
- 行样式(`.row` / `.num` / `.lead` / `.snippet`)3 份:`HistoryList`、`QuickPanel`、`OnboardingView`
- 平台 sans 字体栈字面量在 `HistoryList` 内联 **6 次**,而 `app.css` 只在 `:root` 上设了 `font-family`,没有暴露为 token(缺 `--sans`)

### 3.3 可读性

`SettingsView` 的样式块被压成极少数超长行:

| 位置 | 字符数 | 内含规则数 |
|---|---|---|
| line 590 | **4318** | 53 |
| line 597 | 1641 | ~20 |
| line 650 | 936 | ~12 |

同文件顶部还有 `<!-- svelte-ignore css_unused_selector -->` ——说明其中已有死选择器,但被整块静音了,无法知道是哪些。这两点叠加,使这个样式块实际上不可审查。相比之下其他组件最长行 187~427 字符,尚可阅读。

---

## 4. Design Token

### 4.1 定义质量:好

`app.css` 的 token 体系设计是清晰的:4 级背景、3 级文字、语义色、6 级字号(固定 px,注释说明了"密集 UI 下 1px 是真实层级")、5 级行高、5 级圆角、3 档时长。命名一致,注释解释了取值理由。双主题通过 `prefers-color-scheme` + `[data-theme]` 覆盖,覆盖项完整对应。

### 4.2 落地质量:分裂

实测 token 引用次数(全 `src/`):

```
text-2   55    hairline 51    text-1   44    fs-ui    39    text-3   32
radius-md 23   mono     23    bg-hover 22    radius-sm 21   bg-selected 17
...
--space-1 ~ --space-20   全部 0
```

**间距 token 定义了 12 个,引用 0 次。** 与之对应,组件样式块内的硬编码 px 实测:

| 文件 | 硬编码 px | token 引用 |
|---|---|---|
| `SettingsView` | **178** | 131 |
| `OnboardingView` | 103 | 99 |
| `HistoryList` | 80 | 115 |
| `HistoryWorkspace` | 46 | 48 |
| `ClipPreview` | 41 | 53 |
| `QuickPanel` | 35 | 37 |
| `AppSelect` | 14 | 28 |
| **合计** | **497** | 511 |

即:**色彩/字号/圆角走 token,间距完全不走**。这不是"部分遗漏",是整个间距维度的 token 层没有接通。后果是 `app.css` 里那句"2px base grid; keeps the tuned dense rhythm"实际上没有任何强制力——实测出现的间距值包括 `9px 11px`、`7px`、`3px 6px`、`4px 8px 4px 50px` 等,已偏离 2px 栅格。

### 4.3 缺失的 token

代码中反复出现、语义稳定、但无 token 承载的量:

| 量 | 出现处 | 现状 |
|---|---|---|
| 标题栏高度 42px | `HistoryWorkspace` 网格 / `OnboardingView` / 搜索栏 | 三处硬编码 |
| 动作栏高度 48px | `HistoryWorkspace` 网格 / `SettingsView` 网格 | 两处硬编码 |
| 侧栏宽度 300/280/320px | `HistoryWorkspace` + 2 个媒体查询 | 硬编码 |
| 行高 44px / 行内缩略图 28px | `HistoryList` / `OnboardingView` | 两份 |
| 标准按钮 `min-height:32px; padding:0 12px` | `DESIGN.md` 明文定为唯一按钮尺寸 | 6 处重复硬写 |
| 面板外边距 20px(`calc(100vw - 40px)`) | `HistoryWorkspace` | 40 与 20 需手工保持一致 |
| 平台 sans 字体栈 | `HistoryList` 内联 6 次 | 无 `--sans` |

`--fs-emphasis`(2)、`--lh-relaxed`(2)、`--dur-mid`(1)、`--dur-slow`(1) 引用极低,但都名副其实,属正常长尾,不建议动。

---

## 5. 架构文档评估

### 5.1 `docs/architecture.md` (123 行)

现状:质量高,但**几乎只覆盖 Rust 侧与跨进程边界**。前端相关内容仅两处:
- "Runtime map" 一段:声明 `src/` 拥有 DOM 焦点/键盘/渲染/无障碍,并定义了 `HistorySession` / `PreviewSession` / `HistoryWorkspace` 的三方所有权
- "Keyboard command priority" 一节:定义了窗口键盘路由的优先级

这两条都是**准确且仍然成立**的。问题是缺口:文档没有任何一条约定覆盖本次评估发现的全部问题——组件拆分粒度、样式层级、`:global()` 的许可边界、token 的强制范围。因此这些问题的产生并不违反文档,文档也无法阻止其复发。

值得注意:文档写 `HistoryWorkspace` "orchestrates their call order and continues to own DOM focus, keyboard routing, and the multi-file fileIndex cursor" —— 描述是对的,但它没有给这个"orchestrator"设上限,于是它长到了 814 行并额外接管了应用菜单、视图路由、删除流程。**这是文档缺少约束而非文档写错。**

建议补充的章节(与既有风格一致,写边界而非清单):
- **Frontend composition** — 视图/编排组件/展示组件/纯逻辑模块四层的职责与依赖方向;编排组件不得直接承载表单与菜单模板;纯逻辑必须可脱离 DOM 单测(项目已有 8 个这样的模块,是既成事实,只需写下来)
- **Style layering** — `app.css`(token) → 共享组件样式 → 组件内联三层;`:global()` 仅许可用于第三方无头组件的渲染产物,不得用于跨组件共享自有样式
- **Token authority** — 哪些维度必须走 token(色彩/字号/行高/圆角/时长/间距),哪些可硬编码(一次性几何)
- **Frontend verification gates** — 现有 "Verification gates" 全是原生行为的真机项,可补充组件层的检查项(如新组件需带测试)

### 5.2 `DESIGN.md` (262 行)

设计系统本身写得相当好:有北极星、命名规则(One Action Rule / Hairline Rule / Two-Zone Rule 等)、Do/Don't 清单。**问题是它的规则在代码里没有承载点**:

| DESIGN.md 规则 | 代码现状 |
|---|---|
| "Settings Row 两区契约"(整节 + ASCII 图 + 表格) | 无 `SettingRow` 组件,靠 6 处重复 class 维持 |
| "The One-Button-Size Rule:每个动作按钮都是 `min-height:32px; padding:0 12px`" | 无 `Button` 组件,该尺寸在 6 处重复硬写 |
| "Keycaps: mono 10px, 1px hairline, sm radius, 1px 5px" | 无组件,`kbd` 规则重复 4 次 |
| "2px spacing base" | `--space-*` 零引用,已出现 9px/11px/7px |
| "Rows (signature component)" 明确称其为签名组件 | 行样式 3 份平行实现 |

即:`DESIGN.md` 描述的设计系统是真实存在于设计意图中的,但在代码里**没有单一实现点**,靠人工复述维持。这解释了为什么 Settings 页面会长成现在这样——契约存在于文档,不存在于类型或组件里。

文档本身几乎不需要改;需要改的是让代码出现能承载这些规则的组件,并在 `architecture.md` 里写明"DESIGN.md 的组件契约由 `src/lib/ui/` 承载"这类映射。

### 5.3 `docs/interaction-contract.md` (239 行)

未发现与代码的明显偏离,本次不作为待办。

### 5.4 `_bmad-output/.../ux-ClipClop-2026-08-30/DESIGN.md` (76 行)

与根 `DESIGN.md`(262 行)同名不同内容,是 UX 工作流的产物。**存在两份 DESIGN.md 会造成引用歧义**,建议在架构文档里明确哪一份是权威。

---

## 6. 测试覆盖

| 组件 | 源码 | 测试 |
|---|---|---|
| `HistoryWorkspace` | 814 | **无** |
| `QuickPanel` | 168 | **无** |
| `routes/+layout.svelte` | 59 | 无(薄,可接受) |
| `SettingsView` | 669 | 132 |
| `OnboardingView` | 437 | 31 |
| `HistoryList` | 425 | 57 |
| `ClipPreview` | 161 | 100 |
| `AppSelect` | 55 | 21 |

纯逻辑模块测试覆盖良好(`preview-session` 217/164、`presentation` 103/102、`session` 123/130、`shortcuts` 123/50、`pager` 11/18 等),这是项目的强项。

组件层则与规模**倒挂**:最大的组件没有测试,`SettingsView`/`OnboardingView`/`HistoryList` 的测试与源码比在 5:1~14:1。这与拆分问题同源——组件越大越难测,越不测越敢长。

---

## 7. 待办清单(供 OpenSpec 立项)

按依赖顺序排列。序号即建议的实施顺序,前项是后项的前提。

### A. 建立共享层(其他项的前提)

- **A1** 建立 `src/lib/ui/`,承载 `DESIGN.md` 已定义的契约:`Button`(One-Button-Size Rule)、`SettingRow`(Two-Zone Rule)、`Switch`、`Keycap`、`Row`
- **A2** 建立共享样式层,把 `HistoryWorkspace` 里被跨组件依赖的 `:global(.ghost)` / `.destructive` / `.menu` / `.menu-separator` 迁出去。**必须先做,否则 B1 拆分会静默破坏 SettingsView 与 QuickPanel 的样式**
- **A3** 接通间距 token:12 个 `--space-*` 从 0 引用变为实际使用;补 `--sans`、标题栏/动作栏高度、侧栏宽度、标准按钮尺寸等语义 token

### B. 拆分超大组件

- **B1** 拆 `HistoryWorkspace` (814 → 目标 ~150):抽出 `AppTitleBar`、`HistoryView`、`HistoryActionBar`;抽出 `window-router.ts` 与 `panel-lifecycle.ts` 为可单测的纯逻辑;收敛 `mode` 与 3 个布尔标志为单一状态机。**补测试**
- **B2** 拆 `SettingsView` (669 → 目标 ~120):6 个 tab 各自成文件;抽出 `ShortcutRecorder`、`ReleaseNotes`、`UpdatePanel`;样式随之下沉,消除 4318 字符行,移除整块 `svelte-ignore css_unused_selector` 并清理暴露出的死选择器
- **B3** `OnboardingView` 改用 A1 的共享 Row,删除"仿 HistoryList"的第二份行样式;`QuickPanel` 同理(第三份)并补测试
- **B4** `HistoryList` 抽出 `PageScrubber`(约 120 行指针/滚轮/长按逻辑)

### C. 文档打磨

- **C1** `docs/architecture.md` 补 Frontend composition / Style layering / Token authority / 前端验证门四节(含中文版 `architecture.zh-CN.md` 同步)
- **C2** 在架构文档中写明 `DESIGN.md` 契约与 `src/lib/ui/` 组件的映射关系,并澄清两份 `DESIGN.md` 的权威归属

### 量化目标

| 指标 | 当前(实测) | 目标 |
|---|---|---|
| 最大组件行数 | 814 | < 250 |
| 最大样式块行数 | 281 | < 100 |
| 最长样式单行字符 | 4318 | < 200 |
| `:global()` 处数 | 90 | 仅保留第三方无头组件所需 |
| `--space-*` 引用 | **0** | > 200 |
| 样式块硬编码 px | 497 | < 150(仅一次性几何) |
| 无测试组件(>150 行) | 2 | 0 |
| `kbd` / 行样式重复实现 | 4 / 3 | 1 / 1 |

---

## 8. 风险提示

1. **A2 必须先于 B1。** `.ghost` / `.menu-separator` 等样式的定义点在 `HistoryWorkspace`,而消费者在 `SettingsView` 和 `QuickPanel`。先拆组件会导致样式在运行时静默消失——不报错、不失败构建,只在特定视图下表现为样式丢失。
2. **`svelte-ignore css_unused_selector` 掩盖了未知数量的死选择器。** B2 移除它之后暴露的清理量目前无法预估,立项时应留出余量。
3. **`--space-*` 接通会产生大范围视觉 diff。** 当前实际间距(9px、11px、7px)不在 2px 栅格上,对齐到 token 会有像素级位移。建议按组件分批,每批配合视觉核对,而非一次性替换。
4. 逻辑层(`session` / `preview-session` / `presentation` / `keyboard` / `pager`)**不建议在本轮改动**。它们分层清晰、测试充分,是项目当前最健康的部分。

---

*本报告仅为评估,未修改任何源码或文档。*
