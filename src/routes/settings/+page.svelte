<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { applyTheme, getSettings, updateSettings, type Settings } from "$lib/settings/api";

  let settings = $state<Settings | null>(null);
  let saved = $state("");
  let error = $state("");

  onMount(async () => {
    try { settings = await getSettings(); applyTheme(settings.theme); }
    catch (reason) { error = String(reason); }
  });

  async function save() {
    if (!settings) return;
    error = "";
    try {
      settings = await updateSettings(settings);
      applyTheme(settings.theme);
      saved = "已保存";
      setTimeout(() => saved = "", 1200);
    } catch (reason) { error = String(reason); }
  }

  function removeIgnored(appId: string) {
    if (!settings) return;
    settings.ignored_apps = settings.ignored_apps.filter((item) => item !== appId);
  }

  function appLabel(appId: string) {
    return appId.split(/[\\/]/).pop()?.replace(/\.exe$/i, "") || appId;
  }
</script>

<svelte:window onkeydown={(event) => { if (event.key === "Escape") void getCurrentWindow().close(); }} />

<main class="settings">
  <header><div><h1>设置</h1><p>控制本地历史的保存方式。</p></div><button aria-label="关闭" onclick={() => getCurrentWindow().close()}>×</button></header>
  {#if settings}
    <section>
      <label><span><strong>开机启动</strong><small>登录系统后在后台启动 ClipClop。</small></span><input type="checkbox" bind:checked={settings.launch_at_login} /></label>
      <label><span><strong>保留期限</strong><small>超出期限的历史会在后续捕获时清理。</small></span><select bind:value={settings.retention_days}><option value={7}>7 天</option><option value={30}>30 天</option><option value={90}>90 天</option></select></label>
      <label><span><strong>外观</strong><small>跟随系统，或固定使用 Light/Dark。</small></span><select bind:value={settings.theme} onchange={() => applyTheme(settings!.theme)}><option value="system">跟随系统</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
      <div class="shortcut"><span><strong>全局快捷键</strong><small>当前版本使用平台默认值；快捷键自定义会在设置功能完善后开放。</small></span><kbd>{settings.hotkey}</kbd></div>
      {#if settings.ignored_apps.length > 0}
        <div class="ignored"><strong>已忽略的应用</strong>{#each settings.ignored_apps as appId}<div><code title={appId}>{appLabel(appId)}</code><button onclick={() => removeIgnored(appId)}>移除</button></div>{/each}</div>
      {/if}
    </section>
    <footer><span class:error>{error || saved}</span><button class="save" onclick={save}>保存</button></footer>
  {:else}<div class="loading">{error || "正在读取设置…"}</div>{/if}
</main>

<style>
  .settings { min-height:100vh; display:flex; flex-direction:column; background:var(--bg-shell); color:var(--text-1); }
  header { display:flex; justify-content:space-between; align-items:flex-start; padding:24px 24px 18px; border-bottom:1px solid var(--hairline); }
  h1 { margin:0 0 5px; font-size:18px; } p, small { margin:0; color:var(--text-3); font-size:12px; }
  header button { color:var(--text-2); background:transparent; font-size:22px; }
  section { flex:1; padding:8px 24px; }
  label, .shortcut { min-height:76px; display:flex; align-items:center; justify-content:space-between; gap:24px; border-bottom:1px solid var(--hairline); }
  label span, .shortcut span { display:flex; flex-direction:column; gap:5px; }
  strong { font-size:13px; font-weight:600; }
  select { min-width:112px; padding:7px 28px 7px 9px; border:1px solid var(--hairline); border-radius:6px; color:var(--text-1); background:var(--bg-raised); }
  input { width:18px; height:18px; accent-color:var(--text-1); }
  kbd { font:11px var(--mono); }
  .ignored { padding:18px 0; display:flex; flex-direction:column; gap:8px; }
  .ignored div { display:flex; justify-content:space-between; align-items:center; padding:7px 9px; border-radius:6px; background:var(--bg-raised); }
  .ignored code { max-width:320px; overflow:hidden; text-overflow:ellipsis; color:var(--text-2); font:11px var(--mono); }
  .ignored button { color:var(--text-2); background:transparent; font-size:12px; }
  footer { height:64px; display:flex; align-items:center; justify-content:flex-end; gap:16px; padding:0 24px; border-top:1px solid var(--hairline); }
  footer span { margin-right:auto; color:var(--text-2); font-size:12px; } footer span.error { color:var(--danger); }
  .save { padding:8px 18px; border-radius:6px; color:var(--action-on); background:var(--action); font-weight:600; }
  .loading { flex:1; display:grid; place-items:center; color:var(--text-3); }
</style>
