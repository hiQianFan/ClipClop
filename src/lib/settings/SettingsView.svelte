<script lang="ts">
  import { onMount, tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/clips/api";
  import { applyTheme, getSettings, updateSettings, type Settings } from "./api";
  import { currentPlatform, defaultShortcut, shortcutFromKeyboardEvent, shortcutKeycaps, type ShortcutPlatform } from "./shortcuts";
  import { cachedUpdate, checkForUpdate, currentVersion, downloadAndInstall, openLatestRelease, type AvailableUpdate } from "$lib/updater/api";

  type Tab = "general" | "shortcuts" | "updates" | "about";
  type ShortcutRow = { name: string; description: string; keys: string[][] };
  const tabs: Tab[] = ["general", "shortcuts", "updates", "about"];

  let { onclose, oncleared }: { onclose: () => void; oncleared: () => void } = $props();
  let settings = $state<Settings | null>(null);
  let tab = $state<Tab>("general");
  let status = $state("");
  let saving = $state(false);
  let appVersion = $state("…");
  let update = $state<AvailableUpdate | null>(null);
  let updateState = $state<"idle" | "checking" | "current" | "downloading" | "installing" | "error">("idle");
  let updateMessage = $state("");
  let updateProgress = $state<number | null>(null);
  let confirmClear = $state(false);
  let recording = $state(false);
  let shortcutError = $state("");
  let savedHotkey = $state("");
  let navButtons = $state<HTMLButtonElement[]>([]);
  let settingsContent = $state<HTMLElement>();
  let sectionHeading = $state<HTMLHeadingElement>();
  let clearTrigger = $state<HTMLButtonElement>();
  let confirmClearButton = $state<HTMLButtonElement>();
  let recorder = $state<HTMLButtonElement>();
  const platform: ShortcutPlatform = currentPlatform();

  const panelShortcuts: ShortcutRow[] = [
    { name: "搜索", description: "将焦点移到搜索框", keys: [[platform === "macos" ? "Command" : "Ctrl", "F"], ["/"]] },
    { name: "打开设置", description: "从历史面板进入设置", keys: [[platform === "macos" ? "Command" : "Ctrl", ","]] },
    { name: "当前记录操作", description: "打开所选记录的操作菜单", keys: [[platform === "macos" ? "Command" : "Ctrl", "K"], ...(platform === "windows" ? [["Shift", "F10"]] : [])] },
    { name: "操作菜单导航", description: "在菜单中移动到上一项、下一项或首末项", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
    { name: "关闭面板或菜单", description: "Escape 按层级返回；窗口快捷键直接关闭面板", keys: [["Escape"], [platform === "macos" ? "Command" : "Ctrl", "W"]] },
  ];
  const listShortcuts: ShortcutRow[] = [
    { name: "移动选择", description: "选择上一条或下一条记录", keys: [["ArrowUp"], ["ArrowDown"]] },
    { name: "首条或末条", description: "跳到当前页首尾", keys: [["Home"], ["End"]] },
    { name: "翻页", description: "切换上一页或下一页", keys: [["ArrowLeft"], ["ArrowRight"], ["PageUp"], ["PageDown"]] },
    { name: "跳到可见记录", description: "1–9 对应前九条，0 对应第十条", keys: [["1"], ["…"], ["0"]] },
    { name: "粘贴", description: "保留原格式；加 Shift 仅粘贴纯文本", keys: [["Enter"], ["Shift", "Enter"]] },
    { name: "在默认应用查看", description: "打开当前记录", keys: [["Space"]] },
    { name: "复制为纯文本", description: "仅在记录包含纯文本时生效", keys: [[platform === "macos" ? "Command" : "Ctrl", "Shift", "C"]] },
    { name: "删除", description: "先进入删除确认", keys: [[platform === "macos" ? "Command" : "Ctrl", platform === "macos" ? "Backspace" : "Delete"]] },
    { name: "切换组内文件", description: "列表聚焦且记录含多个文件时生效", keys: [[platform === "macos" ? "Command" : "Ctrl", "ArrowLeft"], [platform === "macos" ? "Command" : "Ctrl", "ArrowRight"]] },
  ];
  const fileShortcuts: ShortcutRow[] = [
    { name: "上一个或下一个文件", description: "文件导航取得焦点时生效", keys: [["ArrowLeft"], ["ArrowRight"]] },
    { name: "首个或末个文件", description: "文件导航取得焦点时生效", keys: [["Home"], ["End"]] },
  ];
  const settingsShortcuts: ShortcutRow[] = [
    { name: "切换分类", description: "侧栏聚焦时循环切换或跳到首末分类", keys: [["ArrowUp"], ["ArrowDown"], ["Home"], ["End"]] },
    { name: "进入或返回详情", description: "从侧栏进入详情，或回到当前分类", keys: [["ArrowRight"], ["Tab"], ["ArrowLeft"]] },
    { name: "保存设置", description: "保存当前设置", keys: [[platform === "macos" ? "Command" : "Ctrl", "S"]] },
    { name: "返回历史", description: "取消确认或离开设置", keys: [["Escape"]] },
  ];
  const shortcutGroups: [string, ShortcutRow[]][] = [
    ["面板操作", panelShortcuts],
    ["历史列表", listShortcuts],
    ["文件预览", fileShortcuts],
    ["设置", settingsShortcuts],
  ];

  onMount(() => {
    requestAnimationFrame(() => navButtons[0]?.focus());
    void load();
  });

  async function load() {
    try { appVersion = await currentVersion(); } catch { appVersion = "未知"; }
    update = cachedUpdate();
    try { settings = await getSettings(); savedHotkey = settings.hotkey; }
    catch (reason) { status = `读取设置失败：${message(reason)}`; }
  }

  function selectTab(next: Tab) {
    tab = next;
    recording = false;
    shortcutError = "";
  }

  async function focusDetail() {
    await tick();
    const first = settingsContent?.querySelector<HTMLElement>(
      'input:not([disabled]), select:not([disabled]), button:not([disabled]), [tabindex="0"]',
    );
    const target = first ?? sectionHeading;
    target?.focus();
    target?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }

  async function onNavKeydown(event: KeyboardEvent) {
    const current = tabs.indexOf(tab);
    let next = current;
    if (event.key === "ArrowDown") next = (current + 1) % tabs.length;
    else if (event.key === "ArrowUp") next = (current - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else if (event.key === "ArrowRight" || (event.key === "Tab" && !event.shiftKey)) {
      event.preventDefault();
      await focusDetail();
      return;
    } else return;
    event.preventDefault();
    selectTab(tabs[next]);
    await tick();
    navButtons[next]?.focus();
  }

  function onContentKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowLeft" || recording) return;
    event.preventDefault();
    event.stopPropagation();
    const target = navButtons[tabs.indexOf(tab)];
    target?.focus();
    target?.scrollIntoView({ block: "nearest" });
  }

  async function save() {
    if (!settings || saving) return;
    saving = true;
    status = "正在保存…";
    try {
      settings = await updateSettings(settings);
      savedHotkey = settings.hotkey;
      applyTheme(settings.theme);
      recording = false;
      shortcutError = "";
      status = "设置已保存";
    } catch (reason) {
      settings.hotkey = savedHotkey;
      status = `保存失败：${message(reason)}`;
      if (tab === "shortcuts") recorder?.focus();
    } finally {
      saving = false;
    }
  }

  function recordShortcut(event: KeyboardEvent) {
    if (!recording || !settings) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      recording = false;
      shortcutError = "";
      status = "已取消快捷键录制";
      return;
    }
    const result = shortcutFromKeyboardEvent(event, platform);
    if (!result.valid) {
      shortcutError = result.message;
      return;
    }
    settings.hotkey = result.shortcut;
    recording = false;
    shortcutError = "";
    status = `已录制 ${displayShortcut(result.shortcut)}，保存后生效`;
  }

  function restoreDefaultShortcut() {
    if (!settings) return;
    settings.hotkey = defaultShortcut(platform);
    recording = false;
    shortcutError = "";
    status = "已恢复默认快捷键，保存后生效";
  }

  async function requestClear() {
    confirmClear = true;
    await tick();
    confirmClearButton?.focus();
  }

  async function cancelClear() {
    confirmClear = false;
    await tick();
    clearTrigger?.focus();
  }

  async function checkUpdates() {
    updateState = "checking";
    updateMessage = "正在检查更新…";
    try {
      const result = await checkForUpdate();
      if (result.kind === "available") {
        appVersion = result.update.currentVersion;
        update = result.update;
        updateState = "idle";
        updateMessage = `发现 ClipClop ${result.update.version}`;
      } else if (result.kind === "current") {
        appVersion = result.currentVersion; update = null; updateState = "current"; updateMessage = "当前已是最新版本";
      } else { updateState = "error"; updateMessage = "开发环境不执行自动更新"; }
    } catch (reason) { updateState = "error"; updateMessage = `检查失败：${message(reason)}`; }
  }

  async function installUpdate() {
    if (!update) return;
    updateState = "downloading"; updateProgress = null;
    try {
      await downloadAndInstall(update.version, (progress) => {
        updateProgress = progress;
        updateMessage = progress === null ? "正在下载更新…" : `正在下载更新 ${progress}%`;
      });
      updateState = "installing"; updateMessage = "正在安装并重新启动…";
    } catch (reason) { updateState = "error"; updateMessage = `安装失败：${message(reason)}`; }
  }

  async function removeAll() {
    try {
      await clearHistory(); confirmClear = false; status = "历史已清空"; oncleared();
    } catch (reason) { confirmClear = false; status = `清空失败：${message(reason)}`; await tick(); clearTrigger?.focus(); }
  }

  function message(reason: unknown) {
    return typeof reason === "object" && reason && "message" in reason
      ? String((reason as { message: unknown }).message) : String(reason ?? "未知错误");
  }

  function displayShortcut(shortcut: string) { return shortcutKeycaps(shortcut, platform).join(" "); }
  function displayKeys(keys: string[]) { return shortcutKeycaps(keys.join("+"), platform); }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (recording) { recording = false; shortcutError = ""; status = "已取消快捷键录制"; }
      else if (confirmClear) void cancelClear();
      else onclose();
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault(); void save();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="settings-shell">
  <div class="settings-body">
    <div class="settings-nav" role="tablist" aria-orientation="vertical" aria-label="设置分类">
      <button bind:this={navButtons[0]} id="settings-tab-general" role="tab" aria-controls="settings-panel" aria-selected={tab === "general"} tabindex={tab === "general" ? 0 : -1} class:active={tab === "general"} onclick={() => selectTab("general")} onkeydown={onNavKeydown}>常规</button>
      <button bind:this={navButtons[1]} id="settings-tab-shortcuts" role="tab" aria-controls="settings-panel" aria-selected={tab === "shortcuts"} tabindex={tab === "shortcuts" ? 0 : -1} class:active={tab === "shortcuts"} onclick={() => selectTab("shortcuts")} onkeydown={onNavKeydown}>快捷键</button>
      <button bind:this={navButtons[2]} id="settings-tab-updates" role="tab" aria-controls="settings-panel" aria-selected={tab === "updates"} tabindex={tab === "updates" ? 0 : -1} class:active={tab === "updates"} onclick={() => selectTab("updates")} onkeydown={onNavKeydown}>软件更新</button>
      <button bind:this={navButtons[3]} id="settings-tab-about" role="tab" aria-controls="settings-panel" aria-selected={tab === "about"} tabindex={tab === "about" ? 0 : -1} class:active={tab === "about"} onclick={() => selectTab("about")} onkeydown={onNavKeydown}>关于</button>
    </div>
    <div bind:this={settingsContent} id="settings-panel" class="settings-content" role="tabpanel" aria-labelledby={`settings-tab-${tab}`} tabindex="-1" onkeydown={onContentKeydown}>
      {#if settings}
        {#if tab === "general"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">常规</h1>
          <label><span><strong>开机启动</strong><small>登录系统后在后台启动 ClipClop。</small></span><input type="checkbox" bind:checked={settings.launch_at_login} /></label>
          <label><span><strong>保留期限</strong><small>超出期限的历史会在后续捕获时清理。</small></span><select bind:value={settings.retention_days}><option value={7}>7 天</option><option value={30}>30 天</option><option value={90}>90 天</option></select></label>
          <label><span><strong>外观</strong><small>跟随系统，或固定使用 Light/Dark。</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">跟随系统</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
          <div class="row"><span><strong>数据管理</strong><small>仅删除 ClipClop 保存的历史。</small></span><button bind:this={clearTrigger} class="danger" onclick={requestClear}>清空全部历史</button></div>
        {:else if tab === "shortcuts"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">快捷键</h1>
          <p class="section-intro">集中查看 ClipClop 的键盘操作。只有全局呼出快捷键可以修改。</p>
          <section class="shortcut-group" aria-labelledby="global-shortcut-title">
            <h2 id="global-shortcut-title">全局</h2>
            <div class="shortcut-row editable">
              <span><strong>呼出或隐藏 ClipClop</strong><small>在其他应用中也能使用。</small></span>
              <div class="shortcut-actions">
                <kbd aria-label={`当前快捷键 ${displayShortcut(settings.hotkey)}`}>{displayShortcut(settings.hotkey)}</kbd>
                <button bind:this={recorder} class:recording onclick={() => { recording = true; shortcutError = ""; status = "请按下新的快捷键，按 Escape 取消"; }} onkeydown={recordShortcut}>{recording ? "请按快捷键…" : "更改"}</button>
                <button onclick={restoreDefaultShortcut} disabled={settings.hotkey === defaultShortcut(platform)}>恢复默认</button>
              </div>
            </div>
            {#if shortcutError}<p class="inline-error" role="alert">{shortcutError}</p>{/if}
          </section>
          {#each shortcutGroups as group}
            <section class="shortcut-group" aria-labelledby={`shortcut-${group[0]}`}>
              <h2 id={`shortcut-${group[0]}`}>{group[0]}</h2>
              {#each group[1] as item}
                <div class="shortcut-row"><span><strong>{item.name}</strong><small>{item.description}</small></span><div class="key-list">{#each item.keys as keys}<kbd>{#each displayKeys(keys) as key}<span>{key}</span>{/each}</kbd>{/each}</div></div>
              {/each}
            </section>
          {/each}
        {:else if tab === "updates"}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1">软件更新</h1>
          <div class="update-head"><span><strong>保持 ClipClop 为最新版本</strong><small>当前版本 {appVersion}；最多每天自动检查一次。</small></span><label><span>自动检查</span><input type="checkbox" bind:checked={settings.check_updates} /></label></div>
          {#if update}
            <div class="update-card"><strong>ClipClop {update.version} 可用</strong>{#if update.notes}<p>{update.notes}</p>{/if}{#if updateState === "downloading" && updateProgress !== null}<progress max="100" value={updateProgress}></progress>{/if}<div><button onclick={() => void openLatestRelease()}>查看发布页</button><button class="primary" disabled={updateState === "downloading" || updateState === "installing"} onclick={installUpdate}>下载并安装</button></div></div>
          {:else}<div class="update-check"><span class:error={updateState === "error"} aria-live="polite">{updateMessage}</span><button disabled={updateState === "checking"} onclick={checkUpdates}>{updateState === "checking" ? "正在检查…" : "检查更新"}</button></div>{/if}
          {#if update && updateMessage}<small class:error={updateState === "error"} aria-live="polite">{updateMessage}</small>{/if}
        {:else}
          <h1 bind:this={sectionHeading} id="settings-section-title" tabindex="-1" class="visually-hidden">关于</h1>
          <div class="about"><img src="/app-icon.png" alt="ClipClop 图标" /><h2>ClipClop</h2><p>轻量、离线优先的跨平台剪贴板历史工具。</p><small>版本 {appVersion} · MIT License</small><button aria-label="在 GitHub 查看 ClipClop" onclick={() => void openUrl("https://github.com/hiQianFan/ClipClop")}>GitHub</button></div>
        {/if}
      {:else}<div class="loading" role="status">{status || "正在读取设置…"}</div>{/if}
    </div>
  </div>
  <footer>
    {#if confirmClear}<strong>清空全部历史？</strong><button onclick={cancelClear}>取消</button><button bind:this={confirmClearButton} class="danger" onclick={() => void removeAll()}>清空</button>
    {:else}<span aria-live="polite" aria-atomic="true">{status}</span><button onclick={onclose}>返回</button>{#if tab !== "about"}<button class="primary" onclick={() => void save()} disabled={!settings || saving}>{saving ? "正在保存…" : "保存"}</button>{/if}{/if}
  </footer>
</div>

<style>
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}.settings-body{min-height:0;display:grid;grid-template-columns:clamp(168px,22%,192px) minmax(0,1fr)}.settings-nav{display:flex;flex-direction:column;gap:3px;padding:14px 12px;border-right:1px solid var(--hairline)}button{padding:8px 10px;border-radius:6px;color:var(--text-2);background:transparent;font-size:12px;line-height:1.4}.settings-nav button{min-height:40px;padding:0 12px;text-align:left;font-size:13px;font-weight:600}.settings-nav button:hover,.settings-nav button.active,button:hover{color:var(--text-1);background:var(--bg-hover)}.settings-nav button.active{background:var(--bg-selected)}button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}.settings-nav button:focus-visible{outline:none;box-shadow:inset 0 0 0 2px var(--text-1)}.settings-content{min-width:0;min-height:0;overflow:auto;padding:0 24px 20px}.settings-content h1{margin:18px 0 4px;font-size:18px;line-height:1.3}.settings-content h1:focus{outline:none}.section-intro{margin:0 0 18px;color:var(--text-2);font-size:12px;line-height:1.5}.settings-content>label,.row,.update-head{min-height:68px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}label>span,.row>span,.update-head>span,.shortcut-row>span{display:flex;flex-direction:column;gap:3px}strong{font-size:13px}small{color:var(--text-3);font-size:12px;line-height:1.4}select{min-width:116px;padding:7px;border:1px solid var(--hairline);border-radius:6px;color:var(--text-1);background:var(--bg-raised);font-size:12px}input{width:18px;height:18px}.shortcut-group{margin-top:18px}.shortcut-group h2{margin:0;padding-bottom:6px;border-bottom:1px solid var(--hairline);font-size:12px;color:var(--text-2)}.shortcut-row{min-height:56px;display:flex;align-items:center;justify-content:space-between;gap:18px;border-bottom:1px solid var(--hairline)}.shortcut-actions,.key-list{display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:6px}.shortcut-actions kbd{min-width:92px;text-align:center}.key-list kbd{display:flex;gap:3px;border:0;background:transparent}.key-list kbd span,.shortcut-actions kbd{padding:3px 6px;border:1px solid var(--hairline);border-radius:4px;color:var(--text-1);background:var(--bg-raised);font:12px/1.3 ui-monospace,monospace;white-space:nowrap}.recording{color:var(--text-1);background:var(--bg-selected)}.inline-error{margin:8px 0 0;color:var(--danger);font-size:12px}.update-head label{display:flex;align-items:center;gap:8px}.update-card{display:flex;flex-direction:column;gap:10px;margin-top:16px;padding:14px;border-radius:8px;background:var(--bg-raised)}.update-card p{max-height:120px;overflow:auto;white-space:pre-wrap;color:var(--text-2);font-size:12px}.update-card>div,.update-check{display:flex;justify-content:flex-end;gap:8px}.update-check{justify-content:space-between;margin-top:16px}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about img{width:56px;height:56px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:12px}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{margin-right:auto}.primary{color:var(--action-on);background:var(--action)}.danger,.error{color:var(--danger)}.visually-hidden{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
</style>
