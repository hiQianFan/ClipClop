<script lang="ts">
  import { onMount, tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { clearHistory } from "$lib/clips/api";
  import { applyTheme, getSettings, updateSettings, type Settings } from "./api";
  import { cachedUpdate, checkForUpdate, currentVersion, downloadAndInstall, openLatestRelease, type AvailableUpdate } from "$lib/updater/api";

  let { onclose, oncleared }: { onclose: () => void; oncleared: () => void } = $props();
  let settings = $state<Settings | null>(null);
  let tab = $state<"general" | "updates" | "about">("general");
  let status = $state("");
  let appVersion = $state("…");
  let update = $state<AvailableUpdate | null>(null);
  let updateState = $state<"idle" | "checking" | "current" | "downloading" | "installing" | "error">("idle");
  let updateMessage = $state("");
  let updateProgress = $state<number | null>(null);
  let confirmClear = $state(false);
  let firstControl = $state<HTMLInputElement>();

  onMount(() => {
    void load();
  });

  async function load() {
    try { appVersion = await currentVersion(); } catch { appVersion = "未知"; }
    update = cachedUpdate();
    try { settings = await getSettings(); await tick(); firstControl?.focus(); }
    catch (reason) { status = message(reason); }
  }

  async function save() {
    if (!settings) return;
    status = "";
    try {
      settings = await updateSettings(settings);
      applyTheme(settings.theme);
      status = "已保存";
    } catch (reason) { status = message(reason); }
  }

  async function checkUpdates() {
    updateState = "checking";
    updateMessage = "";
    try {
      const result = await checkForUpdate();
      if (result.kind === "available") {
        appVersion = result.update.currentVersion;
        update = result.update;
        updateState = "idle";
      } else if (result.kind === "current") {
        appVersion = result.currentVersion;
        update = null;
        updateState = "current";
        updateMessage = "当前已是最新版本";
      } else {
        updateState = "error";
        updateMessage = "开发环境不执行自动更新";
      }
    } catch (reason) {
      updateState = "error";
      updateMessage = `检查失败：${message(reason)}`;
    }
  }

  async function installUpdate() {
    if (!update) return;
    updateState = "downloading";
    updateProgress = null;
    try {
      await downloadAndInstall(update.version, (progress) => {
        updateProgress = progress;
        updateMessage = progress === null ? "正在下载更新…" : `正在下载更新 ${progress}%`;
      });
      updateState = "installing";
      updateMessage = "正在安装并重新启动…";
    } catch (reason) {
      updateState = "error";
      updateMessage = `安装失败：${message(reason)}`;
    }
  }

  async function removeAll() {
    try {
      await clearHistory();
      confirmClear = false;
      status = "历史已清空";
      oncleared();
    } catch (reason) { confirmClear = false; status = `清空失败：${message(reason)}`; }
  }

  function message(reason: unknown) {
    return typeof reason === "object" && reason && "message" in reason
      ? String((reason as { message: unknown }).message) : String(reason ?? "未知错误");
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (confirmClear) confirmClear = false; else onclose();
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault(); void save();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="settings-shell">
  <div class="settings-body">
    <nav class="settings-nav" aria-label="设置分类">
      <button class:active={tab === "general"} onclick={() => tab = "general"}>常规</button>
      <button class:active={tab === "updates"} onclick={() => tab = "updates"}>软件更新</button>
      <button class:active={tab === "about"} onclick={() => tab = "about"}>关于</button>
    </nav>
    <section class="settings-content">
      {#if settings}
        {#if tab === "general"}
          <label><span><strong>开机启动</strong><small>登录系统后在后台启动 ClipClop。</small></span><input bind:this={firstControl} type="checkbox" bind:checked={settings.launch_at_login} /></label>
          <label><span><strong>保留期限</strong><small>超出期限的历史会在后续捕获时清理。</small></span><select bind:value={settings.retention_days}><option value={7}>7 天</option><option value={30}>30 天</option><option value={90}>90 天</option></select></label>
          <label><span><strong>外观</strong><small>跟随系统，或固定使用 Light/Dark。</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">跟随系统</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
          <div class="row"><span><strong>全局快捷键</strong><small>当前版本使用平台默认值。</small></span><kbd>{settings.hotkey}</kbd></div>
          <div class="row"><span><strong>数据管理</strong><small>仅删除 ClipClop 保存的历史。</small></span><button class="danger" onclick={() => confirmClear = true}>清空全部历史</button></div>
        {:else if tab === "updates"}
          <div class="update-head"><span><strong>保持 ClipClop 为最新版本</strong><small>当前版本 {appVersion}；最多每天自动检查一次。</small></span><label><span>自动检查</span><input type="checkbox" bind:checked={settings.check_updates} /></label></div>
          {#if update}
            <div class="update-card"><strong>ClipClop {update.version} 可用</strong>{#if update.notes}<p>{update.notes}</p>{/if}{#if updateState === "downloading" && updateProgress !== null}<progress max="100" value={updateProgress}></progress>{/if}<div><button onclick={() => void openLatestRelease()}>查看发布页</button><button class="primary" disabled={updateState === "downloading" || updateState === "installing"} onclick={installUpdate}>下载并安装</button></div></div>
          {:else}<div class="update-check"><span class:error={updateState === "error"}>{updateMessage}</span><button disabled={updateState === "checking"} onclick={checkUpdates}>{updateState === "checking" ? "正在检查…" : "检查更新"}</button></div>{/if}
          {#if update && updateMessage}<small class:error={updateState === "error"}>{updateMessage}</small>{/if}
        {:else}
          <div class="about"><img src="/app-icon.png" alt="ClipClop 图标" /><h2>ClipClop</h2><p>轻量、离线优先的跨平台剪贴板历史工具。</p><small>版本 {appVersion} · MIT License</small><button aria-label="在 GitHub 查看 ClipClop" onclick={() => void openUrl("https://github.com/hiQianFan/ClipClop")}>GitHub</button></div>
        {/if}
      {:else}<div class="loading">{status || "正在读取设置…"}</div>{/if}
    </section>
  </div>
  <footer>
    {#if confirmClear}<strong>清空全部历史？</strong><button onclick={() => confirmClear = false}>取消</button><button class="danger" onclick={() => void removeAll()}>清空</button>
    {:else}<span>{status}</span><button onclick={onclose}>返回</button>{#if tab !== "about"}<button class="primary" onclick={() => void save()} disabled={!settings}>保存</button>{/if}{/if}
  </footer>
</div>

<style>
  .settings-shell{grid-column:1/-1;grid-row:2/4;min-height:0;display:grid;grid-template-rows:1fr 48px}.settings-body{min-height:0;display:grid;grid-template-columns:148px 1fr}.settings-nav{display:flex;flex-direction:column;gap:2px;padding:12px 10px;border-right:1px solid var(--hairline)}button{padding:8px 10px;border-radius:6px;color:var(--text-2);background:transparent}.settings-nav button{text-align:left;font-weight:600}.settings-nav button:hover,.settings-nav button.active,button:hover{color:var(--text-1);background:var(--bg-hover)}.settings-nav button.active{background:var(--bg-selected)}.settings-content{min-height:0;overflow:auto;padding:0 20px}.settings-content>label,.row,.update-head{min-height:68px;display:flex;align-items:center;justify-content:space-between;gap:24px;border-bottom:1px solid var(--hairline)}label>span,.row>span,.update-head>span{display:flex;flex-direction:column;gap:3px}strong{font-size:13px}small{color:var(--text-3);font-size:11px}select{min-width:116px;padding:7px;border:1px solid var(--hairline);border-radius:6px;color:var(--text-1);background:var(--bg-raised)}input{width:18px;height:18px}.update-head label{display:flex;align-items:center;gap:8px}.update-card{display:flex;flex-direction:column;gap:10px;margin-top:16px;padding:14px;border-radius:8px;background:var(--bg-raised)}.update-card p{max-height:120px;overflow:auto;white-space:pre-wrap;color:var(--text-2);font-size:12px}.update-card>div,.update-check{display:flex;justify-content:flex-end;gap:8px}.update-check{justify-content:space-between;margin-top:16px}.about,.loading{height:100%;display:grid;place-content:center;justify-items:center;gap:8px;text-align:center}.about img{width:56px;height:56px}.about h2,.about p{margin:0}.about p{color:var(--text-2);font-size:12px}footer{display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;border-top:1px solid var(--hairline)}footer span,footer strong{margin-right:auto}.primary{color:var(--action-on);background:var(--action)}.danger,.error{color:var(--danger)}
</style>
