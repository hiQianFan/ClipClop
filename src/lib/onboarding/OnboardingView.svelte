<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { ArrowLeft, ArrowRight, Check, Languages, Link, Search, Type } from "@lucide/svelte";
  import { currentPlatform, defaultShortcut, shortcutKeycaps, shortcutSpokenLabel } from "$lib/settings/shortcuts";
  import { languagePreference, localizedError, setLanguagePreference, t } from "$lib/i18n/index.svelte";
  import { getSettings, openFilePreviewSettings, setFilePreviewEnabled, type LanguagePreference } from "$lib/settings/api";
  import {
    getAutoPasteReadiness,
    openAutoPasteSettings,
    previewOnboardingExample,
    requestAutoPasteAccess,
    saveLanguagePreference,
    saveOnboardingState,
    supportsOnboardingPreview,
    type AutoPasteReadiness,
    type OnboardingExample,
    type OnboardingState,
    type OnboardingStep,
  } from "./api";

  type Mode = "first_run" | "quick_start" | "auto_paste";
  type AutoPasteViewState =
    | "checking" | "ready" | "requesting" | "request_initiated"
    | "opening_settings" | "check_failed" | "settings_open_failed";

  let { initial, mode = "first_run", onfinish }: {
    initial: OnboardingState;
    mode?: Mode;
    onfinish: (returnToSettings: boolean) => void;
  } = $props();

  const platform = currentPlatform();
  // File preview permission is macOS-only, so the step only exists there; Windows
  // keeps the original three-step flow (spec: no macOS permission entry on Windows).
  const steps: OnboardingStep[] = platform === "macos"
    ? ["overview", "practice", "auto_paste", "file_preview"]
    : ["overview", "practice", "auto_paste"];
  const examples: OnboardingExample[] = ["image", "text", "link"];
  const starting = untrack(() => $state.snapshot(initial)) as OnboardingState;
  let journey = $state<OnboardingState>(starting);
  let step = $state<OnboardingStep>(untrack(() => mode === "auto_paste") ? "auto_paste" : starting.current_step ?? "overview");
  let selected = $state<OnboardingExample>(starting.selected_example ?? "image");
  let inputValue = $state("");
  let pastedImage = $state(false);
  let readiness = $state<AutoPasteReadiness | null>(null);
  let autoState = $state<AutoPasteViewState>("checking");
  let error = $state("");
  let filePreviewEnabled = $state(false);
  let filePreviewSaving = $state(false);
  let sandboxList = $state<HTMLDivElement>();
  let languageWrap = $state<HTMLDivElement>();
  let languageButton = $state<HTMLButtonElement>();
  let languageMenuOpen = $state(false);
  let previewOpen = $state(false);
  let saveQueue = Promise.resolve();
  const announcement = $derived(t("onboarding.stepLabel", {
    current: steps.indexOf(step) + 1,
    total: steps.length,
    name: t(`onboarding.step.${step}` as "onboarding.step.overview"),
    status: t("onboarding.current"),
  }));
  const isLastStep = $derived(step === steps[steps.length - 1]);

  onMount(() => {
    void enter(step, false);
    if (platform === "macos") void loadFilePreviewSetting();
  });

  async function loadFilePreviewSetting() {
    try { filePreviewEnabled = (await getSettings()).file_preview_enabled; }
    catch (reason) { error = localizedError(reason); }
  }

  async function persist() {
    if (mode !== "first_run") return;
    const visited = journey.visited_steps.includes(step) ? journey.visited_steps : [...journey.visited_steps, step];
    const state = {
      completed_revision: null,
      current_step: step,
      visited_steps: visited,
      selected_example: selected,
    } satisfies OnboardingState;
    const request = saveQueue.then(() => saveOnboardingState(state));
    saveQueue = request.then(() => undefined, () => undefined);
    journey = await request;
  }

  async function enter(next: OnboardingStep, save = true) {
    if (step === "practice" && next !== "practice") await closePreviewIfOpen();
    step = next;
    if (!journey.visited_steps.includes(next)) journey.visited_steps = [...journey.visited_steps, next];
    if (save) {
      try { await persist(); } catch (reason) { error = localizedError(reason); }
    }
    if (next === "auto_paste") await checkReadiness();
    await tick();
    // The practice step puts focus straight on the sandbox so it is immediately
    // operable (up/down select rows; left/right still page between steps). Other
    // steps move no focus. A visually-hidden live region announces the step change.
    if (next === "practice") sandboxList?.focus();
  }

  async function selectExample(next: OnboardingExample) {
    await closePreviewIfOpen();
    selected = next;
    try { await persist(); } catch (reason) { error = localizedError(reason); }
  }

  function practiceKeydown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    const index = examples.indexOf(selected);
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      void selectExample(examples[(index + (event.key === "ArrowDown" ? 1 : -1) + examples.length) % examples.length]!);
    } else if (/^[1-3]$/.test(event.key)) {
      event.preventDefault();
      void selectExample(examples[Number(event.key) - 1]!);
    } else if (event.key === " " || event.code === "Space") {
      event.preventDefault();
      if (supportsOnboardingPreview(platform)) void togglePreview();
    } else if (event.key === "Enter") {
      event.preventDefault();
      pastedImage = selected === "image";
      inputValue = pastedImage ? "" : exampleText();
      // Focus stays on the sandbox list so the user can keep selecting and
      // pasting after Enter; the input remains independently editable.
    }
    // ArrowLeft/ArrowRight are intentionally not handled here so they bubble to
    // the window handler and page between steps.
  }

  async function togglePreview() {
    try {
      const outcome = await previewOnboardingExample(selected, !previewOpen);
      previewOpen = outcome === "native_opened";
    } catch (reason) {
      error = localizedError(reason);
    }
  }

  async function closePreviewIfOpen() {
    if (!previewOpen) return;
    try {
      await previewOnboardingExample(selected, false);
    } catch { /* best-effort close on step change */ }
    previewOpen = false;
  }

  async function checkReadiness() {
    autoState = "checking";
    error = "";
    try {
      readiness = await getAutoPasteReadiness();
      autoState = "ready";
    } catch (reason) {
      error = localizedError(reason);
      autoState = "check_failed";
    }
  }

  async function requestAccess() {
    autoState = "requesting";
    error = "";
    try {
      await requestAutoPasteAccess();
      autoState = "request_initiated";
    } catch (reason) {
      error = localizedError(reason);
      autoState = "check_failed";
    }
  }

  async function openSettings() {
    autoState = "opening_settings";
    try {
      await openAutoPasteSettings();
      autoState = "request_initiated";
    } catch (reason) {
      error = localizedError(reason);
      autoState = "settings_open_failed";
    }
  }

  // Pure shortcut to macOS Full Disk Access — never records or reflects state.
  async function openFilePreviewAccess() {
    error = "";
    try { await openFilePreviewSettings(); }
    catch (reason) { error = localizedError(reason); }
  }

  // The switch is the in-app gate. Onboarding has no save button, so it persists
  // immediately (matches the other onboarding actions).
  async function toggleFilePreview(enabled: boolean) {
    if (filePreviewSaving) return;
    filePreviewSaving = true;
    error = "";
    try { filePreviewEnabled = await setFilePreviewEnabled(enabled); }
    catch (reason) {
      filePreviewEnabled = !enabled;
      error = localizedError(reason);
    } finally {
      filePreviewSaving = false;
    }
  }

  async function finish() {
    if (mode === "first_run") {
      try {
        await saveQueue;
        await saveOnboardingState({
          completed_revision: 1,
          current_step: null,
          visited_steps: [],
          selected_example: null,
        });
      } catch (reason) {
        error = localizedError(reason);
        return;
      }
    }
    onfinish(mode !== "first_run");
  }

  async function changeLanguage(language: LanguagePreference) {
    const previous = languagePreference();
    try {
      await saveLanguagePreference(language);
      setLanguagePreference(language);
    } catch (reason) {
      setLanguagePreference(previous);
      error = localizedError(reason);
    }
  }

  function languageItems() {
    return Array.from(languageWrap?.querySelectorAll<HTMLButtonElement>("[role='menuitemradio']") ?? []);
  }

  async function openLanguageMenu(focus: "current" | "first" | "last" = "current") {
    languageMenuOpen = true;
    await tick();
    const items = languageItems();
    const index = focus === "first" ? 0 : focus === "last" ? items.length - 1
      : Math.max(0, items.findIndex((item) => item.dataset.language === languagePreference()));
    items[index]?.focus();
  }

  function closeLanguageMenu(focusButton = false) {
    languageMenuOpen = false;
    if (focusButton) requestAnimationFrame(() => languageButton?.focus());
  }

  async function chooseLanguage(language: LanguagePreference) {
    closeLanguageMenu(true);
    await changeLanguage(language);
  }

  function onLanguageButtonKeydown(event: KeyboardEvent) {
    if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
    event.preventDefault();
    void openLanguageMenu(event.key === 'ArrowUp' ? "last" : "current");
  }

  function onLanguageMenuKeydown(event: KeyboardEvent) {
    const items = languageItems();
    const index = items.indexOf(document.activeElement as HTMLButtonElement);
    if (event.key === "Escape") {
      event.preventDefault();
      closeLanguageMenu(true);
    } else if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1
        : (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      items[next]?.focus();
    } else if (event.key === "Tab") {
      closeLanguageMenu();
    }
  }

  function dismissLanguageMenu(event: PointerEvent) {
    if (languageMenuOpen && event.target instanceof Node && !languageWrap?.contains(event.target)) closeLanguageMenu();
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    if (languageMenuOpen) return;
    if (event.key === "Escape" && isLastStep) {
      event.preventDefault();
      void finish();
      return;
    }
    if (step === "practice" && !(event.target instanceof HTMLInputElement) && /^[1-3]$/.test(event.key)) {
      event.preventDefault();
      void selectExample(examples[Number(event.key) - 1]!);
      return;
    }
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    // Left/right page between steps everywhere except native form controls.
    if (mode === "auto_paste") return;
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
    const delta = event.key === "ArrowRight" ? 1 : -1;
    const nextIndex = steps.indexOf(step) + delta;
    if (nextIndex < 0 || nextIndex >= steps.length) return;
    event.preventDefault();
    void enter(steps[nextIndex]!);
  }

  function onWindowFocus() {
    if (step === "auto_paste") void checkReadiness();
    // Quick Look took keyboard focus while open; regaining window focus means the
    // user dismissed it (Space/Esc). The backend already reset PreviewState on
    // focus; clear our flag and return focus to the sandbox so keys work again.
    if (previewOpen) {
      previewOpen = false;
      if (step === "practice") void tick().then(() => sandboxList?.focus());
    }
  }

  function exampleText(example = selected) {
    return t(`onboarding.example.${example}` as "onboarding.example.image");
  }
</script>

<svelte:window onkeydown={onWindowKeydown} onfocus={onWindowFocus} onpointerdown={dismissLanguageMenu} />

{#snippet exampleIcon(example: OnboardingExample)}
  {#if example === "image"}<img src="/app-icon.png" alt={exampleText(example)} />
  {:else if example === "link"}<Link size={15} aria-hidden="true" />
  {:else}<Type size={15} aria-hidden="true" />{/if}
{/snippet}

{#snippet listRow(example: OnboardingExample, index: number)}
  <span class="num" aria-hidden="true">{index + 1}</span>
  <span class="lead">{@render exampleIcon(example)}</span>
  {#if example !== "image"}<span class="snippet">{exampleText(example)}</span>{/if}
{/snippet}

<header class="titlebar">
  <span class="brand"><span class="brand-mark" aria-hidden="true"></span>ClipClop</span>
  <div class="drag" data-tauri-drag-region></div>
  <div bind:this={languageWrap} class="language-menu-wrap" onfocusout={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) closeLanguageMenu(); }}>
    <button bind:this={languageButton} class="language-trigger" class:open={languageMenuOpen} aria-label={t("onboarding.language")} aria-haspopup="menu" aria-expanded={languageMenuOpen} onclick={() => languageMenuOpen ? closeLanguageMenu() : void openLanguageMenu()} onkeydown={onLanguageButtonKeydown}><Languages size={15} aria-hidden="true" /></button>
    {#if languageMenuOpen}
      <div class="language-menu" role="menu" tabindex="-1" aria-label={t("onboarding.language")} onkeydown={onLanguageMenuKeydown}>
        {#each [["system", t("settings.languageSystem")], ["zh-CN", t("settings.languageChinese")], ["en", t("settings.languageEnglish")]] as item}
          {@const value = item[0] as LanguagePreference}
          <button role="menuitemradio" aria-checked={languagePreference() === value} data-language={value} onclick={() => void chooseLanguage(value)}><span>{item[1]}</span>{#if languagePreference() === value}<Check size={13} aria-hidden="true" />{/if}</button>
        {/each}
      </div>
    {/if}
  </div>
</header>

<section class:practice={step === "practice"} class="body">
  <span class="sr-only" role="status" aria-live="polite">{announcement}</span>
  {#if step === "overview"}
    <div class="center">
      <h1>{t("onboarding.overview.title")}</h1>
      <p>{t("onboarding.overview.body")}</p>
      <div class="mini" role="img" aria-label={t("onboarding.overview.samples")}>
        <div class="mini-search" aria-hidden="true"><Search size={14} /><span class="ph">{t("history.searchPlaceholder")}</span><kbd>/</kbd></div>
        <div class="mini-list" aria-hidden="true">
          {#each examples as example, index}<div class="mini-row">{@render listRow(example, index)}</div>{/each}
        </div>
      </div>
      <kbd class="hotkey" aria-label={shortcutSpokenLabel(defaultShortcut(platform), platform)}>
        {#each shortcutKeycaps(defaultShortcut(platform), platform) as key, index}
          {#if index > 0}<span class="hotkey-plus" aria-hidden="true">+</span>{/if}
          <span class="hotkey-key" aria-hidden="true">{key}</span>
        {/each}
      </kbd>
    </div>
  {:else if step === "practice"}
    <div class="legend">
      <h1>{t("onboarding.practice.title")}</h1>
      <dl>
        <div><dt><kbd>↑</kbd><kbd>↓</kbd></dt><dd>{t("onboarding.practice.select")}</dd></div>
        <div><dt><kbd>1</kbd>–<kbd>3</kbd></dt><dd>{t("onboarding.practice.quickSelect")}</dd></div>
        {#if supportsOnboardingPreview(platform)}<div><dt><kbd>Space</kbd></dt><dd>{t("onboarding.practice.preview")}</dd></div>{/if}
        <div><dt><kbd>⏎</kbd></dt><dd>{t("onboarding.practice.paste")}</dd></div>
      </dl>
    </div>
    <div class="sandbox">
      <div bind:this={sandboxList} class="mini sandbox-list" role="listbox" tabindex="0" aria-label={t("onboarding.practice.sandbox")} aria-activedescendant={`sandbox-${selected}`} onkeydown={practiceKeydown}>
        {#each examples as example, index}
          <div id={`sandbox-${example}`} class="mini-row" role="option" tabindex="-1" aria-selected={selected === example} class:selected={selected === example} onclick={() => void selectExample(example)} onkeydown={practiceKeydown}>{@render listRow(example, index)}</div>
        {/each}
      </div>
      {#if pastedImage}
        <div class="mock pasted-image" role="img" aria-label={t("onboarding.example.image")}><img src="/app-icon.png" alt="" /></div>
      {:else}
        <label class="mock">
          <input type="text" bind:value={inputValue} placeholder={t("onboarding.practice.targetHint")} aria-label={t("onboarding.practice.target")} />
        </label>
      {/if}
    </div>
  {:else if step === "auto_paste"}
    {@const ready = readiness === "available" || readiness === "available_with_elevated_target_limit"}
    <div class="center">
      <h1>{t("onboarding.auto.title")}</h1>
      <p>{t("onboarding.auto.body")}</p>
      <p class="statecap" class:ok={ready} class:warn={!ready && autoState !== "checking" && autoState !== "requesting"} class:limit={readiness === "available_with_elevated_target_limit"} aria-live="polite">
        <span class="dot" aria-hidden="true"></span>
        {#if autoState === "checking"}{t("onboarding.auto.checking")}
        {:else if autoState === "requesting"}{t("onboarding.auto.requesting")}
        {:else if readiness === "available"}{t("onboarding.auto.available")}
        {:else if readiness === "available_with_elevated_target_limit"}{t("onboarding.auto.windows")}
        {:else if readiness === "unsupported"}{t("onboarding.auto.unsupported")}
        {:else}{t("onboarding.auto.permission")}{/if}
      </p>
      {#if autoState === "check_failed"}<button class="primary" onclick={() => void checkReadiness()}>{t("onboarding.auto.retry")}</button>
      {:else if autoState === "request_initiated" || autoState === "settings_open_failed"}<button class="primary" onclick={() => void openSettings()}>{t("onboarding.auto.openSettings")}</button>
      {:else if readiness === "permission_required" && autoState === "ready"}<button class="primary" onclick={() => void requestAccess()}>{t("onboarding.auto.enable")}</button>{/if}
      <small>{t("onboarding.auto.fallback")}</small>
    </div>
  {:else}
    <div class="center">
      <h1>{t("onboarding.filePreview.title")}</h1>
      <p>{t("onboarding.filePreview.body")}</p>
      <div class="permission-actions">
        <button onclick={() => void openFilePreviewAccess()}>{t("onboarding.filePreview.openSettings")}</button>
        <label class="switch-inline"><input type="checkbox" role="switch" bind:checked={filePreviewEnabled} disabled={filePreviewSaving} onchange={(event) => void toggleFilePreview(event.currentTarget.checked)} /><span class="switch-track"></span><span class="switch-label">{t("onboarding.filePreview.toggle")}</span></label>
      </div>
      <small>{t("onboarding.filePreview.fallback")}</small>
    </div>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<footer>
  <div class="navigation">
    <button aria-label={t("onboarding.previous")} disabled={step === "overview" || mode === "auto_paste"} onclick={() => void enter(steps[steps.indexOf(step) - 1]!)}><ArrowLeft size={17} /></button>
    <span class="dots" aria-hidden="true">
      {#each steps as item}
        <span class="dot" class:current={step === item} class:visited={journey.visited_steps.includes(item)}></span>
      {/each}
    </span>
    <button aria-label={t("onboarding.next")} disabled={isLastStep || mode === "auto_paste"} onclick={() => void enter(steps[steps.indexOf(step) + 1]!)}><ArrowRight size={17} /></button>
  </div>
  {#if isLastStep}<button class="primary" onclick={() => void finish()}>{t("onboarding.finish")}</button>{/if}
</footer>

<style>
  /* 外壳: 标题栏 + 语言下拉 (取自现有 app) */
  .titlebar{grid-column:1/-1;grid-row:1;display:flex;align-items:center;padding:0 14px;border-bottom:1px solid var(--hairline)}
  .brand{display:flex;align-items:center;gap:5px;color:var(--text-2);font-size:var(--fs-ui);font-weight:600}
  .brand-mark{width:14px;height:14px;background:currentColor;mask:url("/clipclop-mark.svg") center/contain no-repeat;-webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat}
  .drag{flex:1;align-self:stretch}
  .language-menu-wrap{position:relative}
  .language-trigger{width:26px;height:24px;display:grid;place-items:center;padding:0;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent}
  .language-trigger:hover,.language-trigger.open{color:var(--text-1);background:var(--bg-hover)}
  .language-menu{position:absolute;z-index:var(--z-menu);top:30px;right:0;width:150px;padding:5px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised);box-shadow:0 6px 18px rgba(0,0,0,.35)}
  .language-menu button{width:100%;display:flex;align-items:center;justify-content:space-between;gap:10px;padding:7px 9px;border:0;border-radius:var(--radius-md);color:var(--text-1);background:transparent;font-size:var(--fs-ui);text-align:left}
  .language-menu button:hover,.language-menu button:focus-visible{background:var(--bg-hover)}
  /* 主体 */
  .body{grid-column:1/-1;grid-row:2;min-height:0;position:relative;display:grid;place-items:center;padding:24px;overflow:auto}
  .center{max-width:60ch;display:flex;flex-direction:column;align-items:center;gap:16px;text-align:center}
  .center h1,.legend h1{margin:0;font-size:var(--fs-heading);font-weight:680;letter-spacing:-.01em}
  .center p{margin:0;max-width:60ch;color:var(--text-2);font-size:var(--fs-body);line-height:var(--lh-relaxed)}
  /* Step 4: Full Disk Access jump button + in-app enable switch, side by side. */
  .permission-actions{display:flex;align-items:center;gap:16px;margin-top:2px}
  .switch-inline{display:inline-flex;align-items:center;gap:9px;cursor:pointer}
  .switch-inline input{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);clip-path:inset(50%);white-space:nowrap}
  .switch-inline .switch-track{position:relative;flex:none;width:36px;height:20px;border:1px solid color-mix(in srgb,var(--text-2) 42%,var(--bg-selected));border-radius:var(--radius-pill);background:var(--bg-selected);transition:background var(--dur-fast) ease-out,border-color var(--dur-fast) ease-out}
  .switch-inline .switch-track:after{content:"";position:absolute;left:1px;top:1px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.22);transition:transform var(--dur-fast) ease-out}
  .switch-inline input:checked+.switch-track{border-color:var(--action);background:var(--action)}
  .switch-inline input:checked+.switch-track:after{transform:translateX(16px);background:var(--action-on)}
  .switch-inline input:focus-visible+.switch-track{outline:2px solid var(--text-1);outline-offset:2px}
  .switch-inline .switch-label{color:var(--text-2);font-size:var(--fs-ui)}
  @media(prefers-reduced-motion:reduce){.switch-inline .switch-track,.switch-inline .switch-track:after{transition:none}}
  @media(forced-colors:active){.switch-inline .switch-track{border:1px solid ButtonText;background:Canvas}.switch-inline .switch-track:after{background:ButtonText}.switch-inline input:checked+.switch-track{background:Highlight}.switch-inline input:checked+.switch-track:after{background:HighlightText}}
  /* 列表缩影 / 沙盒 (仿 HistoryList 行) */
  .mini{width:min(360px,82vw);border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-shell);overflow:hidden}
  .mini-search{height:38px;display:flex;align-items:center;gap:8px;padding:0 12px;color:var(--text-3);border-bottom:1px solid var(--hairline);font-size:var(--fs-ui)}
  .mini-search .ph{flex:1;text-align:left}
  .mini-search kbd,.legend kbd{font:var(--fs-caption)/var(--lh-snug) var(--mono);color:var(--text-2);border:1px solid var(--hairline);border-radius:var(--radius-sm);padding:1px 5px;white-space:nowrap}
  .mini-list{padding:6px;display:flex;flex-direction:column;gap:1px}
  .mini-row{min-height:44px;display:flex;align-items:center;gap:8px;padding:7px 8px;border-radius:var(--radius-lg);color:var(--text-1);background:transparent;text-align:left}
  .mini-row .num{width:16px;flex:none;color:var(--text-3);font:650 var(--fs-ui) var(--mono);text-align:center}
  .mini-row .lead{width:28px;height:28px;flex:none;display:flex;align-items:center;justify-content:center;overflow:hidden;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-raised)}
  .mini-row .lead img{width:100%;height:100%;object-fit:cover}
  .mini-row .snippet{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font:var(--fs-body)/var(--lh-normal) var(--mono)}
  .hotkey{display:flex;align-items:center;gap:8px;border:0;background:transparent}
  .hotkey-key{min-width:34px;padding:9px 12px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised);box-shadow:0 2px 0 var(--hairline);font:700 var(--fs-heading)/var(--lh-flush) var(--mono);text-align:center}
  .hotkey-plus{color:var(--text-3);font:var(--fs-emphasis)/var(--lh-flush) var(--mono)}
  /* 第2屏 42/58 分栏 */
  .body.practice{grid-template-columns:42% 58%;place-items:stretch;padding:0;gap:0}
  .legend{display:flex;flex-direction:column;justify-content:center;gap:14px;padding:28px 24px 28px 40px;border-right:1px solid var(--hairline)}
  .legend dl{display:flex;flex-direction:column;gap:9px;margin:0}
  .legend dl div{display:flex;align-items:center;gap:10px;font-size:var(--fs-ui);color:var(--text-2)}
  .legend dt{flex:none;display:inline-flex;gap:3px;align-items:center;color:var(--text-3)}
  .legend dd{margin:0}
  .sandbox{display:flex;flex-direction:column;justify-content:center;gap:20px;padding:24px 40px 24px 24px;min-height:0}
  .sandbox .mini{width:100%}
  .mini-row[role=option]{cursor:default}
  .mini-row[role=option]:hover{background:var(--bg-hover)}
  .sandbox-list:focus .mini-row.selected,.mini-row.selected{background:var(--bg-selected)}
  .sandbox-list:focus{outline:none}
  .mock{display:flex;padding:9px 11px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-shell)}
  .mock:focus-within{border-color:var(--action)}
  .mock input{width:100%;border:0;padding:0;background:transparent;color:var(--text-1);font:var(--fs-body)/var(--lh-normal) var(--mono);outline:none}
  .mock input::placeholder{color:var(--text-3)}
  .pasted-image{min-height:72px;align-items:center;justify-content:center}
  .pasted-image img{width:56px;height:56px;object-fit:contain}
  /* 第3屏 权限状态 */
  .statecap{display:flex;align-items:center;justify-content:center;gap:7px;margin:0;color:var(--text-2);font-size:var(--fs-ui)}
  .statecap .dot{width:8px;height:8px;flex:none;border-radius:50%;background:var(--text-3)}
  .statecap.warn{color:#e6b968}.statecap.warn .dot{background:#e0a53f}
  .statecap.ok{color:var(--ok,#5fd39a)}.statecap.ok .dot{background:var(--ok,#5fd39a)}
  .statecap.limit{color:var(--text-2)}.statecap.limit .dot{background:var(--text-3)}
  .primary{color:var(--action-on)!important;background:var(--action)!important;font-weight:650}
  .center>button{padding:7px 10px;border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-hover);font-size:var(--fs-ui)}
  .center small{color:var(--text-3);font-size:var(--fs-meta)}
  .error{position:absolute;bottom:8px;margin:0;color:var(--danger);font-size:var(--fs-ui)}
  /* 工具栏 */
  footer{grid-column:1/-1;grid-row:3;display:flex;align-items:center;justify-content:space-between;padding:0 16px;border-top:1px solid var(--hairline)}
  footer button{min-height:30px;padding:7px 12px;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}
  .navigation{display:flex;align-items:center;gap:8px}
  .navigation>button:first-child,.navigation>button:last-child{width:36px;height:30px;padding:0;display:grid;place-items:center;border:1px solid var(--hairline);border-radius:var(--radius-sm)}
  /* 进度圆点: 纯状态展示, 不可点击/聚焦 */
  .dots{display:flex;align-items:center;gap:8px;padding:0 4px}
  .dots .dot{width:7px;height:7px;border-radius:50%;background:var(--hairline)}
  .dots .dot.visited{background:var(--text-3)}
  .dots .dot.current{width:9px;height:9px;background:var(--action);box-shadow:0 0 0 2px var(--bg-shell)}
  button:hover:not(:disabled){background:var(--bg-hover)}
  button:disabled{opacity:.4}
  .language-trigger:focus-visible,.language-menu button:focus-visible,.body button:focus-visible,footer button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
  .sandbox-list:focus-visible,.mini-row[role=option]:focus-visible{outline:none}
  .sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  @media(max-width:780px){.body.practice{grid-template-columns:1fr;grid-template-rows:auto minmax(250px,1fr)}.legend{padding:16px 22px 8px;border-right:0}.sandbox{padding:8px 22px 16px}}
  @media(prefers-reduced-motion:no-preference){.body>.center,.body>.legend,.body>.sandbox{animation:enter .17s ease-out}@keyframes enter{from{opacity:.25;transform:translateY(3px)}}}
</style>
