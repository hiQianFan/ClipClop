---
stepsCompleted: [1, 2, 3, 4]
inputDocuments: []
workflowType: 'research'
lastStep: 4
research_type: 'market'
research_topic: 'clipboard manager metadata presentation'
research_goals: '调研竞品在剪贴板详情区展示的 metadata，并为 ClipClop 信息栏提出克制、可实施的内容建议'
user_name: 'qianfan'
date: '2026-08-23'
web_research_enabled: true
source_verification: true
---

# Market Research: clipboard manager metadata presentation

## Research Initialization

### Research Understanding Confirmed

**Topic**: clipboard manager metadata presentation
**Goals**: 调研竞品在剪贴板详情区展示的 metadata，并为 ClipClop 信息栏提出克制、可实施的内容建议
**Research Type**: Market Research
**Date**: 2026-08-23

### Research Scope

**Competitive Analysis Focus Areas:**

- Maccy、Paste、Raycast、Alfred、CopyQ、PastePal 与系统剪贴板的详情信息
- 文本、链接、图片、文件、代码等内容类型分别展示哪些 metadata
- 首次复制时间、最后使用时间、复制次数与来源应用的处理方式
- 默认可见信息与渐进披露信息的边界
- ClipClop 已存数据、可低成本补充数据及需要读取原文件的数据

**Research Methodology:**

- 优先使用官方文档、官方截图和开源源码
- 对商业软件未公开的信息明确标注证据限制
- 结合 ClipClop 当前 64/96px 信息栏与本地隐私边界评估

### Next Steps

1. ✅ 初始化与范围确认
2. 用户查看 metadata 的任务与行为分析
3. 竞品 metadata 对照
4. ClipClop 信息架构与优先级建议

**Research Status**: Scope ready for user confirmation

Scope confirmed by user on 2026-08-23.

## Customer Behavior and Segments

### Customer Behavior Patterns

用户查看剪贴板 metadata 的核心任务不是“阅读属性”，而是快速确认自己选中的是否为正确内容：来自哪个应用、何时复制、属于什么类型，以及图片/文件是否为预期尺寸。Paste 官方把来源应用、复制时间、设备与内容类型直接作为搜索过滤维度，说明这些属性承担的是找回与消歧，而不只是装饰。

_Behavior Drivers:_ 在相似文本、截图或文件之间快速确认来源与时效。

_Interaction Preferences:_ metadata 应贴近预览、默认可扫读；低频技术信息再渐进披露。

_Decision Habits:_ 用户通常先凭内容缩略图识别，再用来源、时间、尺寸做二次确认。

_Sources: https://pasteapp.io/help/search-and-filters · https://pasteapp.io/help/explore-paste_

### Demographic Segmentation

公开资料不足以可靠区分年龄、收入或教育程度；为 metadata 做人口统计画像没有产品价值。更合适的是按任务分群：近期快速回贴、跨应用查找、视觉素材复用、文件搬运和开发者富格式检查。ClipClop 应围绕任务密度设计，而不是假设某个年龄层偏爱更多属性。

_Confidence: High on the absence of useful demographic evidence; task segmentation is an inference from product workflows._

### Psychographic Profiles

- **速度优先型**：只想确认“这是刚才从 Chrome 复制的那条”，容忍的信息量最低。
- **可追溯型**：处理多个文档和应用，需要来源、首次/最近时间帮助恢复上下文。
- **素材型**：图片与文件用户更依赖尺寸、文件名、路径、大小和数量。
- **技术型**：关心纯文本/HTML/RTF 等 flavor，但这类数据不应默认占据主信息栏。

CopyQ 把复制时间、来源窗口标题和任意格式数据作为可选脚本/详情能力，而非强制塞入默认列表，支持“常用信息默认可见、技术信息按需展开”的层级。

_Sources: https://github.com/hluk/CopyQ/blob/master/docs/command-examples.rst · https://github.com/hluk/CopyQ/blob/master/docs/faq.rst_

### Customer Segment Profiles

1. **快速召回用户**：高频、短会话；需要来源应用、最近复制时间和内容类型，其他信息容易干扰。
2. **跨项目知识用户**：长期历史、相似条目多；需要来源、时间、设备或组织信息辅助搜索。ClipClop 目前无同步设备概念，不应虚构设备字段。
3. **图片/文件用户**：主要依靠缩略图和文件属性确认对象；需要尺寸、大小、文件序号，路径适合可复制或 tooltip。
4. **开发与排障用户**：偶尔需要 MIME/flavor、原始字节大小；适合操作菜单中的“内容格式”详情，而非常驻栏。

### Behavior Drivers and Influences

metadata 的价值随“内容本身是否足够辨认”上升或下降。独特短文本几乎不需要属性；外观相似的截图、同名文件、反复复制的内容则需要来源和时间消歧。CopyQ 允许把来源窗口标题和复制时间写入 tag，也反映了高级用户对可追溯性的需求，但需要用户主动启用。

_Source: https://github.com/hluk/CopyQ/blob/master/docs/command-examples.rst_

### Customer Interaction Patterns

- **默认扫读**：来源应用 + 最近复制时间 + 1–2 个类型相关事实。
- **悬停/聚焦确认**：完整路径、完整时间、原始 URL 或较长来源名。
- **主动检查**：flavor/MIME、字节明细、首次复制时间、内部 ID 等诊断信息。
- **搜索过滤**：来源、内容类型、时间范围比在每一项上常驻更多标签更有价值；Paste 已明确使用这几类过滤条件。

_Source: https://pasteapp.io/help/search-and-filters_

### Customer Behavior Decision

信息栏“太干”并不意味着应该堆满通用字段。最有用的增强方向是把时间语义讲清，并按内容类型补充能帮助确认对象的事实。默认层建议保持 3–5 个视觉单元；技术 metadata 另行渐进披露。

## Customer Pain Points and Needs

### Customer Challenges and Frustrations

1. **置顶与旧时间冲突**：重复内容被提升后，如果仍显示 `created_at`，第一条可能显示数周前时间，用户会怀疑排序或去重出错。
2. **相似内容难以消歧**：相似截图、同名文件、近似文本只靠预览不足以确认，必须依靠来源、最近时间、尺寸或路径。
3. **字段含义不明确**：当前 ClipClop 直接显示日期、字符和大小，但没有说明日期是首次捕获还是最近使用；数值正确却可能传达错误语义。
4. **metadata 反客为主**：BetterTouchTool 的真实反馈同时出现了“希望显示大小、时间、字符数”和“metadata 占用空间远大于内容、希望关闭”两种诉求，说明无差别增加字段会损伤核心预览。
5. **富格式判断错误**：同一次复制可能包含文本、图片、HTML 等多个 flavor；用户眼中的“文本”与应用提供的底层格式可能不同。BTT 的 Word 案例中，复制文本却因剪贴板包含图片而显示为图片。

_Sources: https://community.folivora.ai/t/show-more-metadata-for-items-in-clipboard-manager/41204 · https://github.com/sabrogden/Ditto/issues/984_

### Unmet Customer Needs

**高优先：**

- 明确表达最近一次复制时间，并与 MRU 排序一致。
- 为每种内容类型提供真正有辨识力的 1–2 个指标。
- 完整时间、路径等长信息可访问但不常驻挤压。

**中优先：**

- 首次复制时间，用于判断内容历史跨度。
- 文本行数、链接域名、文件扩展名或种类。
- 多文件的总数和可得时的合计大小。

**低优先：**

- 复制/使用次数、完整 flavor 列表、窗口标题。
- 哈希、数据库 ID、每个 MIME 的字节大小。

PastePal 是丰富 metadata 的上界：默认可看到类型、来源、日期、字符数，并按内容类型扩展为文本行数、文件路径与大小、链接标题等。它证明这些数据有用，但不意味着 ClipClop 应复制其更重的信息密度。

_Source: https://github.com/IndieGoodies/PastePal_

### Barriers to Adoption

- **信任障碍**：时间与排序矛盾会让用户怀疑历史是否准确。
- **便利障碍**：信息过少导致必须逐项打开或悬停；信息过多又降低扫读速度。
- **隐私障碍**：为链接抓网页标题、favicon 或 social card 可能产生网络请求并暴露复制的域名。ClipClop 应坚持本地解析域名，不为 metadata 联网。
- **平台障碍**：文件大小和状态依赖原路径仍可访问；不可把读取失败显示成“0 B”。

### Service and Support Pain Points

metadata 若没有统一语义，会转化为支持成本：用户会询问“为什么第一条时间更旧”“为什么文字显示为图片”“大小是剪贴板数据还是源文件大小”。字段命名必须区分：剪贴内容大小、源文件大小、首次复制、最近复制。

### Customer Satisfaction Gaps

Paste 明确采用“内容预览 + 来源应用 + 复制时间”的可扫读组合；Ditto 则把日期放在 hover，将 Created、Last Used、格式、大小和哈希放在属性层。两者共同说明满意度不来自字段数量，而来自清楚的层级和一致语义。

_Sources: https://pasteapp.io/help/paste-on-mac · https://github.com/sabrogden/Ditto/issues/984 · https://github.com/sabrogden/Ditto/blob/master/Debug/Language/English.xml_

### Emotional Impact Assessment

metadata 缺失通常只是轻度摩擦；metadata 错误或与排序冲突则是信任问题。误认文件、图片或富文本可能导致粘贴错误，因此“准确、克制”优先于“显得丰富”。

### Pain Point Prioritization

| 优先级 | 痛点 | 建议 |
|---|---|---|
| 高 | 置顶后仍显示首次时间 | 主时间改为 `last_used_at` |
| 高 | 相似内容难区分 | 保留来源，按类型显示最多两个事实 |
| 高 | metadata 挤占内容 | 限定默认单元数量，长值省略/tooltip |
| 中 | 想知道首次复制 | 放入时间 tooltip 或次级详情 |
| 中 | 文件信息不足 | 文件名/路径 + 序号/大小，读取失败明确降级 |
| 低 | 想看格式、hash、ID | 仅诊断详情，不进常驻信息栏 |

### ClipClop Current Gap

ClipClop 已经拥有来源应用、`created_at`、`last_used_at`、字节大小、字符数、图片尺寸、文件路径与部分文件大小。主要缺口不是采集能力，而是 API 没有向前端返回 `last_used_at`，以及信息栏没有对时间和类型指标建立明确层级。第一阶段无需新增复制次数或远端 metadata。

## Customer Decision Processes and Journey

### Customer Decision-Making Process

用户从打开 ClipClop 到粘贴通常只经历四步：

1. **定位**：默认选择第一条，或用列表/搜索缩小范围。
2. **识别**：先看内容预览或缩略图。
3. **确认**：用来源应用、最近时间和类型属性排除相似项。
4. **行动**：直接粘贴，或打开操作菜单选择纯文本、查看、删除等动作。

Paste 把历史称为按最近顺序排列的视觉时间线，每项直接显示预览、来源和复制时间；Raycast 则支持按 Text、Images、Files、Links、Emails、Colors 过滤，并把格式切换等动作放入 Action Panel。这两种模式共同支持“内容先识别、metadata 再确认、复杂能力后置”的决策链。

_Sources: https://pasteapp.io/help/paste-on-mac · https://manual.raycast.com/clipboard-history_

### Decision Factors and Criteria

按用户实际选择顺序，信息权重应为：

1. **内容是否正确**：预览主体，权重最高。
2. **来源是否符合记忆**：应用图标和名称。
3. **时间是否符合记忆**：最近复制时间；历史搜索场景可显示完整日期。
4. **类型属性是否吻合**：文本字符/行数、图片尺寸、文件数量/大小、链接域名。
5. **能否按预期粘贴**：是否有富文本、多格式或纯文本能力；适合操作菜单而非常驻栏。

Raycast 保存原始格式并通过 “Paste as…” 让用户主动选择；这说明 flavor 信息会影响最终决定，但应作为动作层能力，不应以 MIME 字符串占据默认 metadata。

_Source: https://manual.raycast.com/clipboard-history_

### Selection Journey Mapping

| 阶段 | 用户问题 | ClipClop 应提供的信息 |
|---|---|---|
| 打开 | 最新的是哪条？ | MRU 排序，第一项默认选中 |
| 浏览 | 这是我要的内容吗？ | 大面积内容预览/缩略图 |
| 消歧 | 从哪里、什么时候复制？ | 来源应用 + `last_used_at` |
| 类型确认 | 图片/文件/文本是否正确？ | 最多两个类型相关事实 |
| 深入检查 | 首次时间、完整路径、有哪些格式？ | tooltip 或更多信息层 |
| 行动 | 如何粘贴？ | 主粘贴按钮 + 操作菜单 |

### Touchpoint Analysis

- **历史列表**：负责快速定位，不应增加大量 metadata。
- **右侧预览**：负责内容识别，是视觉主角。
- **底部信息栏**：负责确认和消歧，适合 3–5 个短单元。
- **tooltip/更多信息**：负责完整时间、路径和技术属性。
- **搜索/过滤**：当历史很长时，来源、类型和时间范围比增加更多常驻字段更有效。Paste 与 Raycast 都将类型或来源/时间用于过滤。

_Sources: https://pasteapp.io/help/search-and-filters · https://manual.raycast.com/clipboard-history_

### Information Gathering Patterns

用户通常不会逐字段阅读；他们用与记忆相符的线索快速排除。Ditto 用户要求日期常显的理由是“记得大概在一周或两周前复制”，逐项 hover 太慢；这说明时间在长期历史中是导航线索，而不仅是审计属性。

_Source: https://github.com/sabrogden/Ditto/issues/984_

### Decision Influencers

- **近期性**：最近复制和最近使用的内容更可能被选择。
- **视觉相似性**：越相似的内容越依赖 metadata。
- **格式风险**：开发者和写作者会在意富文本/纯文本差异。
- **隐私信任**：metadata 若触发网络请求或读取未授权文件，会降低信任。
- **信息密度**：BTT 用户反馈证明 metadata 过大时会遮蔽真正的复制内容。

_Source: https://community.folivora.ai/t/show-more-metadata-for-items-in-clipboard-manager/41204_

### Decision Optimization

- 主时间显示“最近复制”，与排序一致；完整 tooltip 同时提供“首次复制”。
- 右侧类型指标动态替换，不把所有字段同时展示。
- 值优先、标签弱化，但字段含义必须明确。
- 文件路径和完整时间支持悬停/聚焦读取；不要依赖只有鼠标可用的 tooltip。
- 不先做 metadata 自定义面板；默认策略稳定后，再根据真实反馈决定是否需要配置。
