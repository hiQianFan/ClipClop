---
stepsCompleted: [1, 2]
inputDocuments:
  - package.json
  - src-tauri/Cargo.toml
  - src/lib/history/ClipPreview.svelte
  - src/lib/history/types.ts
  - src-tauri/src/history/model.rs
  - src-tauri/src/preview/mod.rs
workflowType: 'research'
lastStep: 2
research_type: 'technical'
research_topic: 'ClipClop 复制字符串的代码与 JSON 预览和可选格式化'
research_goals: '判断字符串预览与格式化的最小实现；不读取、编辑或写回用户文件'
user_name: 'ClipClop team'
date: '2026-09-03'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-09-03
**Author:** ClipClop team
**Research Type:** technical

---

## Research Overview

评估 ClipClop 是否应为复制的代码、JSON 等字符串提供结构化预览与可选格式化，以及应复用浏览器原生能力还是引入专用解析器。文件继续保持现有只读预览/打开行为，不读取、编辑或写回文件内容。

---

## Technical Research Scope Confirmation

**Research Topic:** ClipClop 复制字符串的代码与 JSON 预览和可选格式化

**Research Goals:** 判断字符串预览与格式化的最小实现；不读取、编辑或写回用户文件

**Technical Research Scope:**

- Architecture Analysis — 现有 Tauri、Svelte、剪贴板历史和预览数据流
- Implementation Approaches — 只读预览、格式化、轻编辑的分层实现
- Technology Stack — 原生 JSON、CodeMirror、Monaco、Shiki、Prettier 与 Tree-sitter
- Integration Patterns — 内嵌 WebView、系统 Quick Look 和默认应用
- Performance Considerations — 安装体积、启动成本、大文本保护和离线运行

**Research Methodology:**

- 当前官方资料验证
- 与项目实际依赖和代码路径交叉核对
- 不采用无法由官方资料或本地构建验证的包体数字

**Scope Confirmed:** 2026-09-03

## Technology Stack Analysis

### Existing ClipClop Stack

ClipClop 已是 Svelte 5 + TypeScript + Vite 6 的 Tauri 2 桌面应用。前端没有编辑器或语法高亮依赖；Rust 端已经依赖 `serde_json`、`tauri-plugin-opener`，macOS 已集成 Quick Look，Windows 已适配外部 QuickLook。当前数据模型只区分 text/link/color/image/file，文本详情最终进入普通 `<pre>`，文件则优先展示缩略图或交给系统预览。

这意味着 JSON 剪贴板文本可以完全在现有前端完成识别、校验和美化。文件不在本研究范围内，继续使用现有 Quick Look/默认应用能力。现有 UI 详情会把文本截到 100,000 字符，所以大文本不能承诺完整格式化。

### Programming Languages

- TypeScript 适合承担展示层 JSON 识别和格式化：浏览器原生 `JSON.parse()` 会校验 JSON，`JSON.stringify(value, null, 2)` 可直接生成缩进文本。[MDN JSON.parse](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse) 但解析后重新序列化可能改变超出 JavaScript 安全整数范围的数字，因此格式化只能作为派生视图，不能静默覆盖原文。
- Rust 端已有 `serde_json`，可用于受限文件读取后的 JSON 校验或格式化，但若文本本来已经传到前端，再走 IPC 没有收益。`serde_json` 已提供 `from_str` 和 `to_string_pretty`。[serde_json 文档](https://docs.rs/serde_json/latest/serde_json/)
- 代码预览无需先构建 AST。Tree-sitter 的价值是可增量语法树、容错解析和逐键更新；这些能力适合代码理解、结构导航或编辑器语义功能，不是首版预览的必要条件。[Tree-sitter 官方介绍](https://tree-sitter.github.io/)

### Development Frameworks and Libraries

| 方案 | 擅长 | 不负责 | 对 ClipClop 的判断 |
|---|---|---|---|
| 原生 `<pre>` + JSON API | JSON 校验、美化、纯文本预览 | 通用代码高亮、编辑器行为 | 首版首选，不新增依赖 |
| CodeMirror 6 | 模块化代码编辑、选择、撤销、搜索、语言扩展 | 任意语言自动格式化 | 真要轻编辑时首选 |
| Shiki fine-grained | 高准确度只读语法着色 | 编辑、格式化 | 只读代码着色需求成立后再加 |
| Monaco | 接近 VS Code 的编辑体验、language workers、语言服务 | 任意语言天然格式化 | 当前明显过重 |
| Prettier standalone | 浏览器内多种 Web 语言格式化 | 编辑器 UI；自动加载配置和插件 | 多语言格式化有明确需求后按语言加载 |
| Tree-sitter | 增量解析、语法树、结构查询 | 编辑 UI、代码格式化 | 当前跳过 |

CodeMirror 6 官方将 state、view、commands、language 等拆成 ES modules，允许只组装需要的能力；语言支持单独安装，因此适合 ClipClop 的轻编辑边界。[CodeMirror 系统指南](https://codemirror.net/docs/guide/)

Shiki 是基于 TextMate grammars/themes 的语法高亮器。官方明确建议浏览器和性能敏感场景使用 fine-grained bundle；其通用 web bundle 官方标注为 3.8 MB minified / 695 KB gzip，因此不能为了少数 JSON 预览直接引入整包。[Shiki bundles](https://shiki.style/guide/bundles)

Monaco 是 VS Code 的浏览器编辑器，并使用 Web Workers 承载部分语言能力；Vite/Tauri 集成需要管理 worker 与离线资源。除非产品目标升级为 schema 补全、诊断、IntelliSense 或 LSP，否则复杂度没有对应收益。[Monaco README](https://github.com/microsoft/monaco-editor/blob/main/README.md)

Prettier standalone 能在浏览器运行，但不会读取配置、ignore 文件或自动加载 parser plugins；调用者必须显式携带所需插件。这使它适合作为后续按语言启用的格式化器，而不是首版通用依赖。[Prettier browser 文档](https://prettier.io/docs/browser)

### Database and Storage Technologies

本需求不需要新数据库或索引。现有 SQLite 历史记录继续保存原始剪贴板内容；格式化预览应是派生视图，不应覆盖原始值。若后续允许编辑，最小语义应是“编辑当前剪贴板副本并重新复制/另存”，不要静默改写历史记录。

文件内容不进入字符串格式化流程，也不新增读取或写回接口。

### Development Tools and Platforms

现有 Vite 足以打包 CodeMirror、Shiki 或 Prettier 的 ESM 资源并保持离线。若未来引入候选包，应以实际生产构建的 chunk、启动时间和内存为准，而不是引用不同 bundler 条件下的第三方体积数字。

CodeMirror 可直接在 Svelte `onMount` 中创建 `EditorView` 并在销毁时清理，无需再引入 Svelte wrapper。复杂键盘和编辑行为由 CodeMirror 负责；外围按钮仍使用项目现有原生控件和 CSS tokens。

### Native Preview and External Applications

ClipClop 已经使用 Tauri opener 通过系统默认应用打开路径；官方 API 也明确支持默认或指定应用。因此“完整编辑”应继续交给用户已有编辑器，而不是在 ClipClop 内复制 IDE。[Tauri opener](https://tauri.app/reference/javascript/opener/)

macOS Quick Look 与现有 Windows QuickLook 集成适合作为真实文件的系统级预览兜底。内嵌预览的价值主要是减少跳出应用，并提供一致的 JSON 美化/搜索；它不应替代系统对未知格式的成熟预览能力。

### Technology Adoption Recommendation

建议按需求强度分三档：

1. **现在做：零新依赖。** 对剪贴板 text 尝试 JSON parse，成功后提供“原文 / 格式化”开关与“复制格式化结果”；文件仍沿用 Quick Look/默认应用。
2. **明确出现代码预览需求后做：** 选择少量语言的 Shiki fine-grained。Prettier 只在用户确实需要 JSON 以外的语言格式化时按语言加入。
3. **不做内嵌编辑。** 因此不引入 CodeMirror、Monaco 或 Tree-sitter；用户需要修改内容时使用现有系统编辑器或快捷记事本。

**置信度：高。** 工具能力和项目现状均由官方资料与代码验证；尚缺用户行为数据和加入候选包后的本项目生产构建数据，因此不能判断代码高亮或轻编辑是否值得立刻开发。

### Product Decision

首版只支持复制字符串中的标准 JSON：识别成功时允许在原文与两空格缩进的格式化预览之间切换，并可复制格式化结果。XML、HTML、CSS、JavaScript、TypeScript、YAML、Markdown、SQL 与 GraphQL 暂不识别或格式化。文件与内嵌编辑均不在范围内。

---

<!-- Content will be appended sequentially through research workflow steps -->
