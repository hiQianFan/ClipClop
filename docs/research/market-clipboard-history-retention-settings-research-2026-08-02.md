---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 1
research_type: 'market'
research_topic: 'clipboard history retention settings'
research_goals: 'Evaluate smaller presets, custom values, and unlimited-by-default retention for ClipClop'
user_name: 'qianfan'
date: '2026-08-02'
web_research_enabled: true
source_verification: true
---

# Market Research: Clipboard History Retention Settings

## 执行摘要

本次研究核验了 Windows、Office、Paste、Alfred、Raycast、CopyQ、Maccy、Beetroot、Xfce Clipman、iClip 等产品。市场没有单一标准，但存在清晰共识：普通历史使用有限默认值，长期内容通过固定或片段库保护；“永久”或“不限制”可以提供，但通常需要用户主动选择。

ClipClop 推荐采用两项预设设置：保留时间为 `1 天 / 7 天 / 30 天（默认）/ 90 天 / 1 年 / 永久`；历史条数为 `100 / 500（默认）/ 1000 / 5000 / 不限制`。达到任一限制即清理最久未使用的普通记录。暂不提供任意自定义输入，因为预设已覆盖主要使用层级，而自定义会增加校验、文案和组合测试。

“使用后移到顶部”应覆盖文本、图片和文件，作为独立开关并默认开启。实现时应保留原始创建时间，新增最近使用语义用于排序与清理，避免把旧记录错误显示成刚创建。默认不采用无限保留，原因是剪贴板可能包含敏感信息，且 Maccy 已有大历史规模引发性能问题的公开案例。

## 目录

1. 研究初始化与范围
2. 用户行为与使用分层
3. 用户痛点与未满足需求
4. 用户决策路径与条数范围
5. 竞争格局
6. 研究综合与产品建议

## Research Initialization

### Research Understanding Confirmed

**Topic**: Clipboard history retention settings  
**Goals**: Evaluate whether ClipClop needs smaller presets, custom values, and unlimited-by-default retention  
**Research Type**: Market Research  
**Date**: 2026-08-02

### Research Scope

**Market Analysis Focus Areas:**

- Retention controls in mainstream macOS and Windows clipboard managers
- Time, item-count, and storage-size limits
- Presets versus custom values
- Default behavior and the risks of unlimited retention
- Product recommendation and UI copy for ClipClop

**Research Methodology:**

- Current web data with source verification
- Preference for official product documentation
- Multiple products across macOS and Windows
- Recommendations constrained by ClipClop's current local-first implementation

### Next Steps

1. ✅ Initialization and scope setting
2. Competitor behavior research
3. Pattern comparison
4. ClipClop recommendation and copy

**Research Status**: 范围已确认，用户行为与使用分层研究完成。

## 用户行为与使用分层

### 用户行为模式

剪贴板历史的核心行为不是“归档”，而是降低短期重复查找成本。用户通常希望系统自动保存、可以快速搜索，并在内容失去近期价值后自动清理。Windows 将历史限制为 25 条并在重启时清除未固定项目，说明系统级产品优先追求低维护和低隐私风险；Paste、Alfred、Raycast 则为更高频用户提供按时间选择的保留策略。

真正需要长期复用的内容通常会从“临时历史”升级为“固定内容”或“片段库”。Paste 明确规定固定项目不受保留期限影响；Alfred 建议把常用文本保存为 Snippet；Windows 和 CopyQ 也用固定功能保护重要项目。这种双层模型比无限扩大普通历史更符合用户心智。

_行为驱动：避免刚复制过的内容丢失、快速复用近期内容、保护少量长期高价值内容。_  
_交互偏好：自动记录与自动清理；仅在有明确长期价值时主动固定。_  
_决策习惯：普通用户接受默认值，高频用户才会进入设置调整。_  
_来源：[Microsoft Clipboard](https://support.microsoft.com/en-au/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)、[Paste Help](https://pasteapp.io/help/control-history-retention)、[Alfred Help](https://www.alfredapp.com/help/features/clipboard/)、[CopyQ Documentation](https://copyq.readthedocs.io/en/latest/pin-items.html)_

### 用户分层

本议题缺少可信的公开人口统计数据，因此不按年龄、收入等属性虚构细分；更适合按任务强度和风险偏好划分：

1. **轻度用户**：偶尔找回刚复制的内容，希望零配置、记录少、隐私风险低。Windows 的 25 条固定上限代表这一层。
2. **日常效率用户**：每天跨应用复制文本、链接、图片和文件，希望可搜索数周到数月。Paste 默认 1 个月，Alfred 提供 24 小时、7 天、1 个月、3 个月。
3. **重度知识工作者与开发者**：希望检索半年以上甚至无限历史，并愿意承担本地存储与隐私成本。Raycast 将 6 个月、1 年、Unlimited 作为主动选择，而非自动开启。
4. **长期素材用户**：少量内容需要永久复用。市场主流做法是固定、Pinboard 或 Snippet，而不是把全部流水历史永久化。

_来源：[Windows Clipboard](https://support.microsoft.com/en-au/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)、[Paste Retention](https://pasteapp.io/help/control-history-retention)、[Alfred Clipboard History](https://www.alfredapp.com/help/features/clipboard/)、[Raycast Clipboard History](https://manual.raycast.com/clipboard-history)_

### 心理与价值取向

- **省心**：不想理解数据库、容量或清理算法，默认值应直接可用。
- **可找回**：担心需要的旧内容突然消失，因此清理规则必须可预测。
- **隐私**：剪贴板可能包含敏感信息；保留越久，暴露窗口越大。Apple 在启用剪贴板历史时会提示其中可能出现敏感信息。
- **控制感**：高级用户需要延长保留期，但不一定需要任意数字输入。

_来源：[Apple Clipboard History](https://support.apple.com/zh-cn/guide/mac-help/mchl40d5b86b/mac)、[Raycast Clipboard History](https://manual.raycast.com/clipboard-history)_

### 行为驱动与影响因素

理性驱动主要是检索效率、磁盘占用与隐私；情绪驱动是“别把我刚才复制的东西弄丢”。条数对用户来说容易理解，却不能准确反映图片和文件造成的空间占用；时间更贴近日常回忆方式，例如“昨天复制过”或“上个月用过”。因此时间适合作为主设置，条数更像安全护栏。

当前证据不足以支持“用户普遍需要自定义到任意天数或任意条数”。已核验产品主要采用少量预设；CopyQ 的最大条数属于高级配置，不能直接证明大众产品也需要相同自由度。

### 研究质量与缺口

- **高置信度**：主流产品普遍区分临时历史与长期固定内容；时间预设比任意自定义更常见。
- **中置信度**：ClipClop 用户可按使用强度分为上述四类，这是基于产品行为的任务分层，不是人口统计调查。
- **待验证**：ClipClop 实际用户的历史规模分布、搜索旧记录的时间跨度、图片/文件占用量。上线遥测受隐私定位限制，更适合使用本地匿名统计或用户访谈验证。

## 用户痛点与未满足需求

### 主要痛点

1. **需要的内容被过早删除**：不同用户的工作周期不同。24 小时对部分用户足够，但周报、月报或阶段性开发任务可能需要数周。市场因此普遍提供 1 天、1 周、1 个月、3 个月等阶梯，而不是只有 30/90 天。
2. **敏感内容保存过久**：用户可能从浏览器复制 API Key、验证码或密码，无法仅靠排除整个应用来避免记录。Raycast 用户明确要求比最短 24 小时更短的保留方式，并表示自己通常只需要最近 10 条。
3. **删除规则不可预测**：用户更在意“为什么这条不见了”，而不是算法本身。Windows 明确说明 25 条、每项 4 MB、重启清理未固定项目；这种限制虽然严格，但容易理解。
4. **历史过大影响性能**：Maccy 曾尝试把上限从 999 提高到 9999，后因多个性能问题撤回。这是反对“默认无限”的直接产品证据。
5. **设置本身成为负担**：轻量工具用户反感为了剪贴板历史再维护复杂系统。增加时间、条数、容量、例外、自定义输入等多个控件，会把低频设置变成管理任务。

_来源：[Raycast 用户对更短保留期的需求](https://www.reddit.com/r/raycastapp/comments/1ip5wic)、[Microsoft Clipboard](https://support.microsoft.com/en-au/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)、[Maccy 2.0 Discussion](https://github.com/p0deje/Maccy/discussions/818)、[轻量剪贴板工具讨论](https://www.reddit.com/r/macapps/comments/1v5d8jo/os_batch_clipboard_v23_easy_paste_queue_to_your/)_

### 未满足需求

- **更小单位确有必要，但只需一个更小预设**：市场证据支持“1 天”和“1 周”；尚不支持小时级自定义。小时级需求更多是敏感数据治理，应优先通过忽略应用、快速清理或未来的敏感内容策略解决。
- **长期保存需要明确出口**：用户需要“永久保留某几条”，而不是默认永久保留全部。固定项目或收藏比无限历史更精确。
- **高级用户需要更长跨度**：可提供 1 年或不限制，但应是主动选择，不宜默认开启。Raycast 明确说明更长保留不会自动启用，Paste 默认仍是 1 个月。
- **条数应作为保护上限**：条数不能准确代表磁盘空间，但能限制查询、滚动和数据库规模。它适合作为内部默认保护或一个高级设置，不应与保留时间争夺主层级。

_来源：[Paste 保留策略](https://pasteapp.io/help/control-history-retention)、[Raycast Clipboard History](https://manual.raycast.com/clipboard-history)、[Maccy 性能讨论](https://github.com/p0deje/Maccy/discussions/818)_

### 采用障碍与满意度缺口

**信任障碍**是最重要的：ClipClop 是本地优先，但本地保存并不等于没有隐私风险；无限保留会扩大设备被访问或备份泄露时的暴露窗口。其次是**便利障碍**：如果用户必须理解两个清理维度以及两套“无限”状态，设置会变得难以解释。

市场也显示一个期望差异：重度用户把“Unlimited”视为价值，隐私敏感用户则希望低于 24 小时甚至只保留最近若干条。因此不存在适合所有人的“不限制”默认值。

_来源：[Raycast 敏感历史讨论](https://www.reddit.com/r/raycastapp/comments/16o7b3d)、[Maccy 隐私问题](https://github.com/p0deje/Maccy/issues/1335)、[Raycast 官方设置](https://manual.raycast.com/clipboard-history)_

### 痛点优先级

| 优先级 | 痛点 | 对 ClipClop 的含义 |
|---|---|---|
| 高 | 旧内容过早消失或删除规则不透明 | 增加 1 天、7 天，并写清清理规则 |
| 高 | 敏感数据被长期保存 | 不采用默认不限制；保留清空入口和本地说明 |
| 高 | 无限历史导致性能退化 | 必须保留安全上限，哪怕暂时不暴露给用户 |
| 中 | 少量内容需要永久保存 | 后续用固定功能解决，不靠全局无限 |
| 中 | 高级用户需要更长历史 | 提供“不限制”作为主动选项即可 |
| 低 | 任意天数或任意条数 | 暂无足够市场证据，先不增加自定义输入 |

### 本阶段判断

- **更小单位：必要**，优先增加 1 天和 7 天；不需要小时。
- **自定义时间：暂不必要**，主流产品以预设为主，预设已覆盖主要工作周期。
- **自定义条数：暂不必要**，它增加校验、文案和组合状态，却没有对应的强用户证据。
- **默认不限制：不建议**，与主流默认、隐私最小化和性能证据相冲突。

## 用户决策路径与条数范围

### 用户如何选择保留策略

用户通常不会根据数据库规模计算合适上限，而是从一个具体失败经历出发：找不到昨天复制的内容、担心密码仍在历史里，或发现应用变慢。决策顺序通常是：

1. 先接受产品默认值；
2. 内容过早消失时延长时间或提高条数；
3. 隐私担忧时缩短时间或降低条数；
4. 真正重要的内容改为固定或收藏；
5. 只有重度用户会主动选择不限制。

因此设置应让用户通过少量档位完成选择，而不是要求其输入任意数字。时间对应“我多久前复制过”，条数对应“我大约需要多少条”，两者同时出现时必须解释采用任一条件触发清理。

_来源：[Paste 保留设置](https://pasteapp.io/help/control-history-retention)、[Raycast Clipboard History](https://manual.raycast.com/clipboard-history)、[CopyQ 内存建议](https://copyq-de.readthedocs.io/de/latest/faq.html)_

### 市场条数范围

| 产品或类型 | 条数策略 | 观察 |
|---|---:|---|
| Windows Clipboard | 25 条 | 系统级、低维护、固定项例外 |
| Office Clipboard | 24 条 | 面向即时跨文档操作 |
| Xfce Clipman | 文本默认 10，可设 5–100；图片默认 0，可设 0–5 | 极轻量，严格控制资源 |
| VS Code Clipboard Manager 扩展 | 默认 100 条 | 单一应用场景 |
| CopyQ | 默认每标签 200 条 | 官方建议通过降低条数减少内存 |
| Beetroot | 100 / 250 / 500 / 1000 / Unlimited，默认 500 | 桌面独立工具的清晰阶梯 |
| Maccy | 常见配置 500–999；9999 试验因性能问题撤回 | 约 1000 是已验证边界，10000 风险明显 |
| Clipboard Manager（macOS） | 20 / 50 / 100 / 200 / 500 / 1000 / 5000 | 覆盖轻量到重度的完整档位 |
| iClip | 自动历史最高 1000 条 | 成熟桌面产品的上限参考 |

_来源：[Microsoft Clipboard](https://support.microsoft.com/en-au/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)、[Office Clipboard](https://support.microsoft.com/en-us/office/copy-and-paste-using-the-office-clipboard-714a72af-1ad4-450f-8708-c2931e73ec8a)、[Xfce Clipman](https://docs.xfce.org/panel-plugins/clipman/start)、[CopyQ FAQ](https://copyq-de.readthedocs.io/de/latest/faq.html)、[Beetroot User Guide](https://max.nardit.com/beetroot/docs/user-guide)、[Maccy 2.0 Discussion](https://github.com/p0deje/Maccy/discussions/818)、[Clipboard Manager Tutorial](https://clipboard-manager.mac-application.com/tutorial.html)、[iClip Help](https://iclipapp.com/dl/iClip%20Help.pdf)_

### ClipClop 条数方案评估

#### 推荐公开档位

**100 / 500 / 1000 / 5000 / 不限制**

- `100`：隐私敏感或轻度使用者；比 500 更小的需求由此覆盖，无需再加入 20、25、50。
- `500`：日常用户；是多个独立工具采用的中间值。
- `1000`：高频用户；接近 Maccy、iClip 的成熟边界。
- `5000`：重度用户；需要清楚提示可能增加磁盘占用。
- `不限制`：专家选择；仍需单条大小限制和数据库性能保护。

不建议加入 `250`：它虽有竞品采用，但在 ClipClop 中不能形成新的明确用户层级。也不建议任意自定义：100–5000 的数量级档位已覆盖主要需求，自定义只会引入输入校验、最小最大值、错误文案和组合测试。

#### 默认值判断

**推荐默认 500 条，不推荐默认不限制。**

理由：ClipClop 同时保存文本、图片和多种剪贴板格式，单条成本差异很大；500 比纯文本工具的 1000 更保守，又明显高于系统级的 25 条。若本地性能测试证明 1000 条混合数据稳定，可把默认提高到 1000，但不能仅凭文本场景推断。

如果产品坚持默认“不限制”，最低要求是后台仍保留一个不可见的磁盘容量保护和单条大小限制，否则“不限制”会把产品承诺变成数据与性能风险。与其设置一个用户看不到的假无限，不如直接使用明确的 500 或 1000 默认值。

### 时间与条数的组合决策

若两项都开放，建议规则为：**记录达到任一上限时自动清理，固定记录除外。**

推荐组合：

- 保留时间：1 天 / 7 天 / 30 天（默认）/ 90 天 / 1 年 / 永久
- 历史记录上限：100 / 500（默认）/ 1000 / 5000 / 不限制

但从设置复杂度看，首版更推荐只公开“保留时间”，内部先采用 1000 条安全上限。等用户明确反馈“30 天内记录也被条数上限删掉”或需要主动控制规模，再公开条数设置。这是当前证据支持的最小方案。

### 决策优化

- 下拉菜单使用预设，不提供数字输入框。
- 在说明文字中明确“达到时间或条数任一限制即清理”。
- 选择“永久”或“不限制”时提示历史可能包含敏感内容并增加磁盘占用。
- 设置页显示当前记录数与占用空间，比任意自定义值更能帮助用户做决定。
- 长期保护应最终由“固定”承担；它能让用户放心选择较短的普通历史期限。

## 竞争格局

### 主要竞品矩阵

| 产品 | 时间限制 | 条数限制 | 永久或无限 | 固定保护 | 使用后移到顶部 | 默认策略 |
|---|---|---:|---|---|---|---|
| Windows Clipboard | 无可调时间 | 25 | 否 | 有 | 重新选择即成为当前剪贴板 | 重启清理未固定项 |
| Paste | 1 天、1 周、1 月、1 年 | 未公开 | 永久 | 有 | 未在保留文档中说明 | 30 天 |
| Alfred | 24 小时、7 天、1 月、3 月 | 未公开 | 否 | 转为 Snippet | 工作流可控制是否回到顶部 | 历史默认因隐私关闭 |
| Raycast | 1 天、1 周、1 月、3 月、6 月、1 年 | 未公开 | Unlimited（Pro） | 有 | 有独立开关 | 长期保留不自动开启 |
| CopyQ | 无主时间策略 | 默认每标签 200 | 可提高 | 有 | 可移动或固定 | 200 |
| Maccy | 无主时间策略 | 常见 500–999 | 受性能约束 | 有 | 按最近复制排序 | 有限条数 |
| Beetroot | Never、1/7/30 天 | 100/250/500/1000 | Unlimited | 收藏保护 | 未明确 | 500 条、时间不限 |

_来源：[Microsoft Clipboard](https://support.microsoft.com/en-au/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)、[Paste Help](https://pasteapp.io/help/control-history-retention)、[Alfred Help](https://www.alfredapp.com/help/features/clipboard/)、[Raycast Manual](https://manual.raycast.com/clipboard-history)、[CopyQ FAQ](https://copyq.readthedocs.io/en/latest/faq.html)、[Maccy Discussion](https://github.com/p0deje/Maccy/discussions/818)、[Beetroot Guide](https://max.nardit.com/beetroot/docs/user-guide)_

### 竞争定位

ClipClop 更适合定位在 Windows 原生历史与 Raycast/Paste 之间：比系统工具保留更多、搜索更强；比综合效率平台更轻、更本地、更容易理解。设置不应追求 CopyQ 式的全面可编程，而应提供经过验证的预设。

### 优势与弱点

**优势：** 本地优先、跨平台、文本/图片/文件统一、已有分页和搜索、复制旧文本后已有回到顶部的基础实现。  
**弱点：** 当前只有 7/30/90 天；文本与图片/文件的“使用后提升”行为不一致；没有条数保护；尚无固定功能；时间字段同时承担创建时间和最近使用时间会损失原始信息。

### 差异化机会

1. 用很少的设置覆盖从隐私敏感到重度用户的主要范围。
2. 统一所有内容类型的“使用后移到顶部”，并提供默认开启的开关。
3. 同时显示历史条数和本地占用空间，让用户理解清理选择的影响。
4. 后续增加固定功能，把临时历史与长期资料明确分层。

### 竞争威胁

macOS 已加入系统剪贴板历史，Windows 也有原生方案；ClipClop 不能只靠“能看历史”形成价值。过多设置会削弱轻量定位，而默认无限又会制造隐私和性能负担。

### 最终竞品判断

- 预设优于任意自定义值。
- 永久/不限制可以提供，但不应作为默认。
- 条数档位推荐 `100 / 500 / 1000 / 5000 / 不限制`，默认 `500`。
- 时间档位推荐 `1 天 / 7 天 / 30 天 / 90 天 / 1 年 / 永久`，默认 `30 天`。
- “使用后移到顶部”应提供开关，默认开启，并覆盖文本、图片和文件。
- 固定功能是长期正确方向，但不应阻塞本轮保留策略落地。

## 研究综合与产品建议

### 最终设置方案

| 设置项 | 选项 | 默认值 |
|---|---|---|
| 保留时间 | 1 天、7 天、30 天、90 天、1 年、永久 | 30 天 |
| 历史记录上限 | 100、500、1000、5000、不限制 | 500 |
| 使用后移到顶部 | 开启、关闭 | 开启 |

清理规则：**超过保留时间或历史条数任一限制时，删除最久未使用的普通记录。**未来加入固定功能后，固定记录不受两项限制影响。

### 推荐中文文案

**保留时间**  
选项：`1 天 / 7 天 / 30 天 / 90 天 / 1 年 / 永久`  
说明：`超过保留时间的历史记录会自动删除。再次使用后将重新计算。`

**历史记录上限**  
选项：`100 条 / 500 条 / 1,000 条 / 5,000 条 / 不限制`  
说明：`达到上限后，优先删除最久未使用的记录。`

**使用后移到顶部**  
说明：`复制或粘贴一条历史记录后，将它移到列表顶部。`

当用户选择“永久”或“不限制”时显示同一条辅助提示：`长期保留可能增加磁盘占用，并保存更多敏感内容。`

### 实现原则

1. **保留原始时间**：`created_at` 只表示首次捕获时间；新增或明确使用 `last_used_at` 负责排序和清理。
2. **统一内容类型**：文本、图片、文件在复制与粘贴成功写入剪贴板后，都按开关决定是否更新 `last_used_at`。
3. **任一条件清理**：先清理过期记录，再按最近使用顺序裁剪到条数上限。
4. **设置变更可预测**：用户降低限制并保存后应立即清理，不必等待下一次捕获；执行前说明影响，避免无提示删除。
5. **输入边界**：不提供任意数值，自然消除非法值、极端值和单位歧义。

### 分阶段落地建议

#### 第一阶段：本轮完成

- 增加保留时间预设与永久选项。
- 增加条数预设与不限制选项，默认 500。
- 增加“使用后移到顶部”开关，默认开启。
- 统一文本、图片和文件行为。
- 增加最小清理测试：时间限制、条数限制、关闭移动开关各一条关键路径。

#### 第二阶段：有真实需求时再做

- 固定记录，并让固定项免于自动清理。
- 显示当前历史条数和磁盘占用。
- 按单条大小或总磁盘容量限制。
- 只有预设不能满足明确用户案例时，才增加自定义值。

### 风险与缓解

| 风险 | 影响 | 缓解方式 |
|---|---|---|
| 永久与不限制同时开启 | 数据和敏感信息持续累积 | 非默认、明确提示、保留清空入口 |
| 图片或多格式记录体积过大 | 磁盘增长与性能下降 | 保留单条大小边界；后续按实际数据增加容量上限 |
| 更新排序时间覆盖创建时间 | 来源信息失真 | 分离 `created_at` 与 `last_used_at` |
| 保存设置立即大量删除 | 用户意外丢失历史 | 保存前显示将删除的数量并二次确认，或仅在确有删除时确认 |
| 三项设置增加认知负担 | 设置页变复杂 | 使用预设、简短说明，不增加高级自定义输入 |

剪贴板可能包含密码、令牌和商业信息。OWASP 建议避免不必要地存储敏感数据，并在不再需要时清理缓存或临时副本；这进一步支持“不限制不作为默认”的决定。[OWASP Developer Guide](https://devguide.owasp.org/en/04-design/02-web-app-checklist/08-protect-data/)

### 验收指标

- 默认设置下，混合文本、图片、文件累计 500 条后仍能稳定查询、翻页和搜索。
- 第 501 条捕获后，最久未使用的普通记录被删除。
- 使用旧记录后，在开关开启时回到第一位，关闭时保持原顺序。
- 选择永久或不限制时不会发生对应维度的清理。
- 原始捕获时间始终不变。
- 中英文设置文案含义一致，键盘与读屏能够操作所有控件。

### 研究限制

公开资料能确认产品功能和部分默认值，但没有可靠的行业市场份额或 ClipClop 用户实际历史规模数据，因此本文不虚构市场占比。500 条默认值是基于竞品范围、混合内容成本和保守性能策略的产品判断，应在实现后用本地混合数据压力测试验证。

---

**研究完成日期：** 2026-08-02  
**研究结论置信度：** 高（竞品功能与默认策略）；中（ClipClop 默认条数，需性能测试验证）
