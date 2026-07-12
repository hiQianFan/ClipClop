---
name: Clip-Clop
description: 克制、精确、以内容为中心的桌面剪贴板工具
colors:
  dark-shell: "#17181A"
  dark-raised: "#1E2022"
  dark-hover: "#242628"
  dark-selected: "#2A2D30"
  dark-hairline: "#2C2F32"
  dark-text-primary: "#ECEDEE"
  dark-text-secondary: "#9BA1A6"
  dark-text-tertiary: "#6B7075"
  dark-action: "#33363A"
  dark-action-hover: "#3D4045"
  light-shell: "#F2F3F4"
  light-raised: "#FAFBFB"
  light-hover: "#ECEEEF"
  light-selected: "#E2E5E8"
  light-hairline: "#D9DDE0"
  light-text-primary: "#1C1E20"
  light-text-secondary: "#5D6367"
  light-text-tertiary: "#737A80"
  light-action: "#2B2E31"
  light-action-hover: "#3A3D40"
  action-on: "#FFFFFF"
typography:
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
  content:
    fontFamily: "'SF Mono', 'Cascadia Mono', ui-monospace, Menlo, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.4
rounded:
  keycap: "4px"
  control: "6px"
  row: "8px"
  panel: "14px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  copy-button-dark:
    backgroundColor: "{colors.dark-action}"
    textColor: "{colors.dark-text-primary}"
    rounded: "{rounded.control}"
    padding: "7px 15px"
  copy-button-light:
    backgroundColor: "{colors.light-action}"
    textColor: "{colors.action-on}"
    rounded: "{rounded.control}"
    padding: "7px 15px"
  list-row:
    rounded: "{rounded.row}"
    padding: "8px 8px"
    height: "44px"
---

# Clip-Clop 设计系统

## 1. Overview（概览）

**设计北极星：“安静的剪贴板取景器”**

Clip-Clop 是随叫随到的桌面工具，不是内容管理系统。外壳使用无彩石墨灰并主动退后，让用户复制的文本、图片、颜色、文件和链接成为画面主体。界面密度偏高、层级明确、键盘优先，不使用装饰性配色建立品牌感。

主界面为约 `720 × 540px` 的浮动面板：左侧约 `300px`，包含搜索和每页 10 条的列表；右侧为可滚动预览和固定元数据栏；底部为 `48px` 状态栏。面板跟随系统切换 dark/light，布局、间距和交互保持一致。

- 快速识别优先于展示更多元数据。
- 列表负责定位，预览负责确认，Enter 负责复制并关闭。
- v1 不加入类型筛选、标签、侧边栏、AI 信息增强或直接粘贴。
- 系统字体负责界面，等宽字体负责剪贴板内容；中文自动回退到系统中文字体。

## 2. Colors（颜色）

配色策略是严格无彩色。产品 UI 自身只能使用灰阶；图片、色值色块、文件图标、应用图标和已缓存 favicon 属于用户内容，可以保留原色。

### Dark 模式

| Token | 色值 | 用途 |
|---|---:|---|
| `--bg-shell` | `#17181A` | 面板底色 |
| `--bg-raised` | `#1E2022` | 预览与抬升表面 |
| `--bg-hover` | `#242628` | 行悬停 |
| `--bg-selected` | `#2A2D30` | 选中行 |
| `--border-hairline` | `#2C2F32` | 分隔线 |
| `--text-primary` | `#ECEDEE` | 正文与主要内容 |
| `--text-secondary` | `#9BA1A6` | 控件、来源应用、可操作提示 |
| `--text-tertiary` | `#6B7075` | 非关键时间与元数据 |
| `--action` | `#33363A` | 主复制按钮 |
| `--action-hover` | `#3D4045` | 主按钮悬停 |

### Light 模式

Light 不是暗色反相：外壳使用浅灰而非纯白，预览表面略亮，选中态必须比 hover 深一个台阶，主要按钮使用深灰保证识别和对比度。

| Token | 色值 | 用途 |
|---|---:|---|
| `--bg-shell` | `#F2F3F4` | 面板底色，减少纯白眩光 |
| `--bg-raised` | `#FAFBFB` | 预览与抬升表面 |
| `--bg-hover` | `#ECEEEF` | 行悬停 |
| `--bg-selected` | `#E2E5E8` | 选中行 |
| `--border-hairline` | `#D9DDE0` | 分隔线 |
| `--text-primary` | `#1C1E20` | 正文与主要内容 |
| `--text-secondary` | `#5D6367` | 控件、来源应用、可操作提示 |
| `--text-tertiary` | `#737A80` | 非关键时间与元数据 |
| `--action` | `#2B2E31` | 主复制按钮 |
| `--action-hover` | `#3A3D40` | 主按钮悬停 |
| `--action-on` | `#FFFFFF` | 主按钮文字 |

**纯灰规则。** v1 不使用品牌强调色、暖灰、冷灰染色、渐变或彩色选中条。状态不能只依赖颜色表达；焦点同时使用轮廓，错误同时提供文字。

## 3. Typography（字体）

界面字体使用 `-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif`；剪贴板片段和预览使用 `"SF Mono", "Cascadia Mono", ui-monospace, Menlo, monospace`。这种组合让操作界面保持原生，让内容呈现像精确的工作台。

- 搜索：`13px/1.4`，正文权重 400。
- 列表内容：`13px/1.5`，等宽；最多两行，默认单行截断。
- 行序号、时间、预览元数据：`11px/1.4`；关键操作信息用 secondary，非关键元数据才用 tertiary。
- 状态栏与按钮：`12px/1.4`，权重 500–600。
- 长文本预览：`13px/1.6`，保留换行并允许断词；单行阅读宽度尽量不超过 75 个字符。
- 不使用全大写眉题、展示字体或过度字距；界面中不需要营销式标题层级。

## 4. Elevation（层级）

界面以色阶和 `1px` 分隔线建立层级。列表行、预览区和普通控件不使用阴影；只有浮动 Quick Panel 相对桌面获得一层环境阴影。系统透明材质仅可作为外壳增强，关闭透明效果或 Windows 不支持时必须回退到实色 `--bg-shell`。

- Quick Panel：`box-shadow: 0 24px 70px rgba(0,0,0,.45), 0 2px 8px rgba(0,0,0,.30)`，只用于顶层浮窗。
- 分隔：统一 `1px solid var(--border-hairline)`。
- 焦点：`2px solid var(--text-secondary)`，并保留至少 `2px` 外偏移；不可只用色差表示。
- 面板出现：暗色/亮色均采用 `120ms` 淡入并从 `0.98` 缩放到 `1`；选择移动不做动画；遵循 `prefers-reduced-motion`。

## 5. Components（组件）

### Quick Panel

- 尺寸约 `720 × 540px`，圆角 `14px`；最小可用尺寸 `640 × 480px`。
- 两列布局为 `300px 1fr`，底栏 `48px`；竖向分隔线贯穿内容区与状态栏。
- Dark 为重点展示模式，Light 必须具备同等信息层级和 WCAG AA 可读性。

### 搜索

- 位于左栏顶部，高约 `40px`，左右内边距 `14px`，底部使用 hairline。
- 默认不抢焦点；按 `/` 或点击进入，Esc 清空焦点并返回列表。
- placeholder 使用 secondary 色以满足可读性；右侧 `/` 使用统一 keycap 样式。

### 列表与行

- 每页固定 10 条，最新在前；行高约 `44px`、圆角 `8px`、间距 `1px`。
- 结构为：`序号 + 28px 固定内容槽 + 片段 + 相对时间`。来源应用只在预览元数据中显示。
- text/code 的内容槽留空；颜色显示真实色块；图片显示缩略图；文件显示系统类型图标或缩略图；链接仅在本地已有 favicon 时显示，否则留空。
- hover 使用 `--bg-hover`；selected 只使用 `--bg-selected`，不加彩色边条、勾选图标或阴影。

### 预览

- 主体区域可滚动，内边距 `20px`；底部元数据栏固定，高约 `40px`，文件多路径时最多增至 `64px`。
- 普通文本、代码和富文本均以安全纯文本预览；富文本可在元数据中显示低干扰的“富文本”说明，不渲染不受信任 HTML。
- 图片等比缩放并限制在可视区；颜色显示大色块和原始色值；文件显示名称、路径、大小、缩略图及“源文件已移动或删除”状态。
- 链接预览显示原始 URL 和域名。v1 不抓取网页 title、description、Open Graph 等 SEO 信息，也不因剪贴板 URL 主动联网。

### 富文本 flavor

“Flavor”是同一次复制里并存的数据格式。例如从网页复制一段带链接的文字，剪贴板可能同时包含 `text/plain`、`text/html` 和 `text/rtf`：纯文本用于搜索和安全预览，HTML/RTF 保存字体、粗体、链接、列表等格式。Clip-Clop 要保存原始已知 flavor 的实际 payload，而不只是保存格式名称；用户复制回剪贴板时原样写回这些格式，使目标应用自行选择最合适的格式。

- v1 保存 `plain + HTML/RTF` 中实际存在的格式；不主动转换或补造缺失格式。
- 搜索、列表摘要和去重基于规范化纯文本；原始 flavor 不进入 FTS。
- 预览只显示纯文本，避免引入富文本编辑器和 HTML 安全风险。
- 遇到密码管理器标记的 concealed/transient flavor 时整项跳过，不落盘。

### 文件缩略图

- 文件主体仍是引用：只保存路径和元数据，不复制源文件到应用目录。
- Clip-Clop 可以读取源文件一次，仅用于调用系统 Quick Look/Thumbnail API 生成小尺寸本地缩略图；缩略图是缓存，不代表托管文件。
- 建议输出最长边 `128px` 的 WebP/PNG，单张设置大小上限，源文件变化或删除后允许显示旧缩略图并标记引用失效。
- 无法读取、不支持预览或生成超时时，回退为系统文件类型图标；不能阻塞剪贴板捕获主流程。

### favicon 与网页元信息

favicon 是网站的小图标，通常显示在浏览器标签页，例如 GitHub 的章鱼猫图标。它与 SEO 信息不同：SEO/网页元信息通常指页面标题、描述、Open Graph 图片等，需要请求网页并解析内容。

- v1 不主动联网获取 favicon 或 SEO 信息，避免泄露用户复制过哪些网址，并保持离线优先。
- 若操作系统或本地浏览器缓存已能提供 favicon，可使用本地缓存；否则链接内容槽留空。
- 如未来增加联网增强，必须默认关闭、显式授权、限定请求数据和缓存周期；这不是 v1 范围。

### 状态栏与操作

- 左侧为 `[←]  x/N  [→]`；箭头是 keycap，页码是普通信息文本。
- 右侧为 `⌘K 操作` 和 `⏎ 复制`。主按钮使用灰阶实色填充、圆角 `6px`。
- 键盘：`↑/↓` 选择，`←/→` 翻页，`1–0` 跳转可见行，Enter 复制并关闭，`/` 搜索，`⌘K/Ctrl+K` 操作，Esc 返回列表或关闭。
- 操作菜单只含当前需要的删除、忽略来源应用、清空历史；不做可搜索命令面板。

## 6. Do's and Don'ts（应做与禁用）

### 应做

- **应**让内容成为唯一稳定的彩色来源，并保持所有产品控件为灰阶。
- **应**在 Dark 与 Light 中维持相同布局和状态层级，分别校验文本与背景对比度。
- **应**为所有交互状态提供 default、hover、focus、active、disabled；加载使用局部骨架，不遮挡整个面板。
- **应**使用系统图标和标准键盘行为；文件缩略图失败时安静回退到文件类型图标。
- **应**让空历史只显示一句说明和呼出快捷键，搜索无结果只显示“无匹配结果”。

### 禁用

- **禁止**加入彩色强调色、渐变、玻璃卡片、发光效果、彩色类型图标或彩色选中条。
- **禁止**把 Clip-Clop 做成 AI 剪贴板、知识库、笔记系统、重型效率仪表盘或通用命令启动器。
- **禁止**加入永久应用导航、类型侧边栏、密集筛选 chips、过大的操作栏或元数据表格。
- **禁止**为每条记录增加独立卡片、宽阴影或超过 `16px` 的内容容器圆角。
- **禁止**自动渲染不受信任的 HTML、自动请求被复制的 URL、生成摘要、解释、标题或分类。
- **禁止**使用自定义滚动条、非标准弹窗和不表达状态的装饰动画。
