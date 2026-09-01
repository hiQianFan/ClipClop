<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { DropdownMenu } from "bits-ui";
  import { ArrowLeft, ArrowRight, Check, Languages, Link, LoaderCircle, Search, Type } from "@lucide/svelte";
  import ShortcutHint from "$lib/components/ShortcutHint.svelte";
  import PageScrubber from "$lib/history/PageScrubber.svelte";
  import { currentPlatform, defaultShortcut } from "$lib/settings/shortcuts";
  import { languagePreference, localizedError, setLanguagePreference, t } from "$lib/i18n/index.svelte";
  import { openFilePreviewSettings, type LanguagePreference } from "$lib/settings/api";
  import {
    openAutoPasteSettings,
    previewOnboardingExample,
    saveLanguagePreference,
    saveOnboardingState,
    supportsOnboardingPreview,
    type OnboardingExample,
    type OnboardingState,
    type OnboardingStep,
  } from "./api";

  type Mode = "first_run" | "quick_start" | "auto_paste";
  let { initial, mode = "first_run", onfinish }: {
    initial: OnboardingState;
    mode?: Mode;
    onfinish: (returnToSettings: boolean) => void;
  } = $props();

  const platform = currentPlatform();
  const steps: OnboardingStep[] = platform === "macos"
    ? ["overview", "practice", "auto_paste"]
    : ["overview", "practice"];
  const examples: OnboardingExample[] = ["image", "text", "link"];
  const practicePages = 3;
  const starting = untrack(() => $state.snapshot(initial)) as OnboardingState;
  let journey = $state<OnboardingState>(starting);
  let step = $state<OnboardingStep>(untrack(() => mode === "auto_paste" || starting.current_step === "file_preview") ? "auto_paste" : starting.current_step ?? "overview");
  let selected = $state<OnboardingExample>(starting.selected_example ?? "image");
  let practicePage = $state(1);
  let inputValue = $state("");
  let pastedImage = $state(false);
  let error = $state("");
  let sandboxList = $state<HTMLDivElement>();
  let languageMenuOpen = $state(false);
  let languageButton = $state<HTMLButtonElement | null>(null);
  let languageOpenFocus: "current" | "last" = "current";
  let languageTabExit = false;
  let previewOpen = $state(false);
  let finishing = $state(false);
  let reducedMotion = $state(false);
  let saveQueue = Promise.resolve();
  const announcement = $derived(t("onboarding.stepLabel", {
    current: steps.indexOf(step) + 1,
    total: steps.length,
    name: t(`onboarding.step.${step}` as "onboarding.step.overview"),
    status: t("onboarding.current"),
  }));
  const isLastStep = $derived(mode === "auto_paste" || step === steps[steps.length - 1]);

  onMount(() => {
    reducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
    void enter(step, false);
  });

  async function persist() {
    if (mode !== "first_run") return;
    const visited = journey.visited_steps.includes(step) ? journey.visited_steps : [...journey.visited_steps, step];
    const state = {
      ...journey,
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
    const visible = practiceExamples();
    const index = visible.indexOf(selected);
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const direction = event.key === "ArrowDown" ? 1 : -1;
      const next = index + direction;
      if (next >= 0 && next < visible.length) void selectExample(visible[next]!);
      else turnPracticePage(practicePage + direction, direction > 0 ? "first" : "last");
    } else if (event.key === "ArrowLeft" || event.key === "PageUp") {
      event.preventDefault();
      turnPracticePage(practicePage - 1, "last");
    } else if (event.key === "ArrowRight" || event.key === "PageDown") {
      event.preventDefault();
      turnPracticePage(practicePage + 1, "first");
    } else if (/^[1-3]$/.test(event.key)) {
      event.preventDefault();
      void selectExample(visible[Number(event.key) - 1]!);
    } else if (event.key === " " || event.code === "Space") {
      event.preventDefault();
      if (supportsOnboardingPreview(platform)) void togglePreview();
    } else if (event.key === "Enter") {
      event.preventDefault();
      pastedImage = selected === "image";
      inputValue = pastedImage ? "" : practiceExampleText(selected);
      // Focus stays on the sandbox list so the user can keep selecting and
      // pasting after Enter; the input remains independently editable.
    }
  }

  function onWindowClick(event: MouseEvent) {
    if (step !== "practice") return;
    if (event.target instanceof Element && event.target.closest("input, button, [role=menuitemradio]")) return;
    sandboxList?.focus();
  }

  function practiceExamples() {
    const offset = practicePage - 1;
    return examples.map((_, index) => examples[(index + offset) % examples.length]!);
  }

  function turnPracticePage(page: number, edge?: "first" | "last") {
    const next = Math.max(1, Math.min(page, practicePages));
    if (next === practicePage) return;
    practicePage = next;
    const visible = practiceExamples();
    void selectExample(visible[edge === "last" ? visible.length - 1 : 0]!);
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

  async function openPermissionSettings(kind: "auto_paste" | "file_preview") {
    error = "";
    try {
      await (kind === "auto_paste" ? openAutoPasteSettings() : openFilePreviewSettings());
    } catch (reason) {
      error = localizedError(reason);
    }
  }

  async function finish() {
    if (finishing) return;
    finishing = true;
    if (mode === "first_run") {
      try {
        await saveQueue;
        await saveOnboardingState({
          ...journey,
          completed_revision: 1,
          current_step: null,
          visited_steps: [],
          selected_example: null,
        });
      } catch (reason) {
        error = localizedError(reason);
        finishing = false;
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

  async function chooseLanguage(language: LanguagePreference) {
    languageMenuOpen = false;
    await changeLanguage(language);
  }

  function onLanguageButtonKeydown(event: KeyboardEvent) {
    if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
    event.preventDefault();
    languageOpenFocus = event.key === 'ArrowUp' ? "last" : "current";
    languageMenuOpen = true;
  }

  function onLanguageMenuKeydown(event: KeyboardEvent) {
    if (event.key === "Tab") languageTabExit = true;
  }

  function restoreLanguageFocus(event: Event) {
    event.preventDefault();
    if (languageTabExit) {
      languageTabExit = false;
      return;
    }
    languageButton?.focus();
  }

  function focusLanguageItem(event: Event) {
    event.preventDefault();
    requestAnimationFrame(() => {
      const items = Array.from(document.querySelectorAll<HTMLElement>("[data-language]"));
      const target = languageOpenFocus === "last"
        ? items.at(-1)
        : items.find((item) => item.dataset.language === languagePreference());
      target?.focus();
    });
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
      void selectExample(practiceExamples()[Number(event.key) - 1]!);
    }
  }

  function onWindowFocus() {
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

  function practiceExampleText(example: OnboardingExample) {
    if (practicePage === 1 || example !== "text") return exampleText(example);
    return t(`onboarding.practice.page${practicePage}.text` as "onboarding.practice.page2.text");
  }
</script>

<svelte:window onkeydown={onWindowKeydown} onfocus={onWindowFocus} onclick={onWindowClick} />

{#snippet exampleIcon(example: OnboardingExample, label = exampleText(example))}
  {#if example === "image"}<img src="/app-icon.png" alt={label} />
  {:else if example === "link"}<Link size={15} aria-hidden="true" />
  {:else}<Type size={15} aria-hidden="true" />{/if}
{/snippet}

{#snippet listRow(example: OnboardingExample, index: number, label = exampleText(example))}
  <span class="num" aria-hidden="true">{index + 1}</span>
  <span class="lead">{@render exampleIcon(example, label)}</span>
  {#if example !== "image"}<span class="snippet">{label}</span>{/if}
{/snippet}

<header class="titlebar">
  <span class="brand"><span class="brand-mark" aria-hidden="true"></span>ClipClop</span>
  <div class="drag" data-tauri-drag-region></div>
  <DropdownMenu.Root bind:open={languageMenuOpen}>
  <div class="language-menu-wrap">
    <DropdownMenu.Trigger bind:ref={languageButton} class={`language-trigger${languageMenuOpen ? " open" : ""}`} aria-label={t("onboarding.language")} onclick={() => languageOpenFocus = "current"} onkeydown={onLanguageButtonKeydown}><Languages size={15} aria-hidden="true" /></DropdownMenu.Trigger>
    <DropdownMenu.ContentStatic class="language-menu" aria-label={t("onboarding.language")} loop={true} onkeydown={onLanguageMenuKeydown} onOpenAutoFocus={focusLanguageItem} onCloseAutoFocus={restoreLanguageFocus}>
      <DropdownMenu.RadioGroup value={languagePreference()}>
        {#each [["system", t("settings.languageSystem")], ["zh-CN", t("settings.languageChinese")], ["en", t("settings.languageEnglish")]] as item}
          {@const value = item[0] as LanguagePreference}
          <DropdownMenu.RadioItem {value} data-language={value} onclick={() => void chooseLanguage(value)}><span>{item[1]}</span>{#if languagePreference() === value}<Check size={13} aria-hidden="true" />{/if}</DropdownMenu.RadioItem>
        {/each}
      </DropdownMenu.RadioGroup>
    </DropdownMenu.ContentStatic>
  </div>
  </DropdownMenu.Root>
</header>

<section class:practice={step === "practice"} class="body">
  <span class="sr-only" role="status" aria-live="polite">{announcement}</span>
  {#if step === "overview"}
    <div class="center">
      <h1>{t("onboarding.overview.title")}</h1>
      <p>{t("onboarding.overview.body")}</p>
      <div class="mini" role="img" aria-label={t("onboarding.overview.samples")}>
        <div class="mini-search" aria-hidden="true"><Search size={14} /><span class="ph">{t("history.searchPlaceholder")}</span><ShortcutHint shortcut="/" {platform} variant="compact" /></div>
        <div class="mini-list" aria-hidden="true">
          {#each examples as example, index}<div class="mini-row">{@render listRow(example, index)}</div>{/each}
        </div>
      </div>
      <ShortcutHint shortcut={defaultShortcut(platform)} {platform} variant="keycaps" />
    </div>
  {:else if step === "practice"}
    <div class="legend">
      <h1>{t("onboarding.practice.title")}</h1>
      <dl>
        <div><dt><ShortcutHint shortcut="ArrowUp" {platform} variant="compact" /><ShortcutHint shortcut="ArrowDown" {platform} variant="compact" /></dt><dd>{t("onboarding.practice.select")}</dd></div>
        <div><dt><ShortcutHint shortcut="ArrowLeft" {platform} variant="compact" /><ShortcutHint shortcut="ArrowRight" {platform} variant="compact" /></dt><dd>{t("onboarding.practice.page")}</dd></div>
        <div><dt><ShortcutHint shortcut="1" {platform} variant="compact" />–<ShortcutHint shortcut="3" {platform} variant="compact" /></dt><dd>{t("onboarding.practice.quickSelect")}</dd></div>
        {#if supportsOnboardingPreview(platform)}<div><dt><ShortcutHint shortcut="Space" {platform} variant="compact" /></dt><dd>{t("onboarding.practice.preview")}</dd></div>{/if}
        <div><dt><ShortcutHint shortcut="Enter" {platform} variant="compact" /></dt><dd>{t("onboarding.practice.paste")}</dd></div>
      </dl>
    </div>
    <div class="sandbox">
      <div class="practice-pager">
        <span class="pager-hint">{t("onboarding.practice.scrubberHint")}</span>
        <div class="pager-controls">
          <button aria-label={t("history.previousPage")} disabled={practicePage <= 1} onclick={() => turnPracticePage(practicePage - 1, "last")}><span aria-hidden="true">‹</span></button>
          <PageScrubber page={practicePage} totalPages={practicePages} {reducedMotion} onpage={(page) => turnPracticePage(page)} />
          <span aria-live="polite">{t("quick.pageStatus", { current: practicePage, total: practicePages })}</span>
          <button aria-label={t("history.nextPage")} disabled={practicePage >= practicePages} onclick={() => turnPracticePage(practicePage + 1, "first")}><span aria-hidden="true">›</span></button>
        </div>
      </div>
      <div bind:this={sandboxList} class="mini sandbox-list" role="listbox" tabindex="0" aria-label={t("onboarding.practice.sandbox")} aria-activedescendant={`sandbox-${selected}`} onkeydown={practiceKeydown}>
        {#each practiceExamples() as example, index}
          <div id={`sandbox-${example}`} class="mini-row" role="option" tabindex="-1" aria-selected={selected === example} class:selected={selected === example} onclick={() => void selectExample(example)} onkeydown={practiceKeydown}>{@render listRow(example, index, practiceExampleText(example))}</div>
        {/each}
      </div>
      {#if pastedImage}
        <div class="mock pasted-image" role="img" aria-label={practiceExampleText("image")}><img src="/app-icon.png" alt="" /></div>
      {:else}
        <label class="mock">
          <input type="text" bind:value={inputValue} placeholder={t("onboarding.practice.targetHint")} aria-label={t("onboarding.practice.target")} />
        </label>
      {/if}
    </div>
  {:else if step === "auto_paste"}
    <div class="center capability-step">
      <h1>{t("onboarding.auto.title")}</h1>
      <p>{t("onboarding.auto.body")}</p>
      <div class="capabilities">
        <div class="capability-row">
          <span><strong>{t("onboarding.auto.autoPasteTitle")}</strong><small>{t("onboarding.auto.autoPasteHelp")}</small></span>
          <button onclick={() => void openPermissionSettings("auto_paste")}>{t("onboarding.auto.openSettings")}</button>
        </div>
        <div class="capability-row">
          <span><strong>{t("onboarding.auto.filePreviewTitle")}</strong><small>{t("onboarding.auto.filePreviewHelp")}</small></span>
          <button onclick={() => void openPermissionSettings("file_preview")}>{t("onboarding.auto.manageFileAccess")}</button>
        </div>
      </div>
      <small class="capability-note">
        {t("onboarding.auto.fallback")} {t("onboarding.auto.systemPreviewPrefix")} <ShortcutHint shortcut="Space" {platform} variant="compact" /> {t("onboarding.auto.systemPreviewSuffix")}
      </small>
    </div>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<footer>
  <button class="step-button previous" disabled={step === "overview" || mode === "auto_paste"} onclick={() => void enter(steps[steps.indexOf(step) - 1]!)}><ArrowLeft size={15} aria-hidden="true" />{t("onboarding.previous")}</button>
  {#if mode !== "auto_paste"}<span class="step-progress" aria-live="polite">{steps.indexOf(step) + 1} / {steps.length}</span>{/if}
  {#if isLastStep}
    <button class="primary finish" onclick={() => void finish()} disabled={finishing} aria-busy={finishing}>{#if finishing}<LoaderCircle size={14} class="finish-spinner" />{t("onboarding.finishing")}{:else}{t("onboarding.finish")}{/if}</button>
  {:else}
    <button class="step-button next" onclick={() => void enter(steps[steps.indexOf(step) + 1]!)}>{t("onboarding.next")}<ArrowRight size={15} aria-hidden="true" /></button>
  {/if}
</footer>

<style>
  /* 外壳: 标题栏 + 语言下拉 (取自现有 app) */
  .titlebar{grid-column:1/-1;grid-row:1;display:flex;align-items:center;padding:0 14px;border-bottom:1px solid var(--hairline)}
  .brand{display:flex;align-items:center;gap:5px;color:var(--text-2);font-size:var(--fs-ui);font-weight:600}
  .brand-mark{width:14px;height:14px;background:currentColor;mask:url("/clipclop-mark.svg") center/contain no-repeat;-webkit-mask:url("/clipclop-mark.svg") center/contain no-repeat}
  .drag{flex:1;align-self:stretch}
  .language-menu-wrap{position:relative}
  :global(.language-trigger){width:26px;height:24px;display:grid;place-items:center;padding:0;border:0;border-radius:var(--radius-md);color:var(--text-2);background:transparent}
  :global(.language-trigger:hover),:global(.language-trigger.open){color:var(--text-1);background:var(--bg-hover)}
  :global(.language-menu){position:absolute;z-index:var(--z-menu);top:30px;right:0;width:150px;padding:5px;border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-raised);box-shadow:0 6px 18px rgba(0,0,0,.35)}
  :global(.language-menu [role="menuitemradio"]){width:100%;display:flex;align-items:center;justify-content:space-between;gap:10px;padding:7px 9px;border:0;border-radius:var(--radius-md);color:var(--text-1);background:transparent;font-size:var(--fs-ui);text-align:left}
  :global(.language-menu [role="menuitemradio"]:hover),:global(.language-menu [role="menuitemradio"]:focus-visible),:global(.language-menu [role="menuitemradio"][data-highlighted]){background:var(--bg-hover)}
  /* 主体 */
  .body{grid-column:1/-1;grid-row:2;min-height:0;position:relative;display:grid;place-items:center;padding:24px;overflow:auto;scrollbar-gutter:stable both-edges}
  .center{max-width:60ch;display:flex;flex-direction:column;align-items:center;gap:16px;text-align:center}
  .center h1,.legend h1{margin:0;font-size:var(--fs-heading);font-weight:680;letter-spacing:-.01em}
  .center p{margin:0;max-width:60ch;color:var(--text-2);font-size:var(--fs-body);line-height:var(--lh-relaxed)}
  /* 列表缩影 / 沙盒 (仿 HistoryList 行) */
  .mini{width:min(360px,82vw);border:1px solid var(--hairline);border-radius:var(--radius-lg);background:var(--bg-shell);overflow:hidden}
  .mini-search{height:38px;display:flex;align-items:center;gap:8px;padding:0 12px;color:var(--text-3);border-bottom:1px solid var(--hairline);font-size:var(--fs-ui)}
  .mini-search .ph{flex:1;text-align:left}
  .mini-list{padding:6px;display:flex;flex-direction:column;gap:1px}
  .mini-row{min-height:44px;display:flex;align-items:center;gap:8px;padding:7px 8px;border-radius:var(--radius-lg);color:var(--text-1);background:transparent;text-align:left}
  .mini-row .num{width:16px;flex:none;color:var(--text-3);font:650 var(--fs-ui) var(--mono);text-align:center}
  .mini-row .lead{width:28px;height:28px;flex:none;display:flex;align-items:center;justify-content:center;overflow:hidden;border-radius:var(--radius-sm);color:var(--text-2);background:var(--bg-raised)}
  .mini-row .lead img{width:100%;height:100%;object-fit:cover}
  .mini-row .snippet{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font:var(--fs-body)/var(--lh-normal) var(--mono)}
  /* 第2屏 42/58 分栏 */
  .body.practice{grid-template-columns:42% 58%;place-items:stretch;padding:0;gap:0}
  .legend{display:flex;flex-direction:column;justify-content:center;gap:14px;padding:28px 24px 28px 40px;border-right:1px solid var(--hairline)}
  .legend dl{display:flex;flex-direction:column;gap:9px;margin:0}
  .legend dl div{display:flex;align-items:center;gap:10px;font-size:var(--fs-ui);color:var(--text-2)}
  .legend dt{flex:none;display:inline-flex;gap:3px;align-items:center;color:var(--text-3)}
  .legend dd{margin:0}
  .sandbox{display:flex;flex-direction:column;justify-content:center;gap:20px;padding:24px 40px 24px 24px;min-height:0}
  .practice-pager{min-height:30px;display:flex;align-items:center;justify-content:space-between;gap:12px;color:var(--text-3);font:500 var(--fs-meta)/1 var(--mono)}
  .pager-hint{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif;font-weight:400;white-space:nowrap}
  .pager-controls{display:flex;align-items:center;gap:6px}
  .practice-pager button{width:28px;height:28px;padding:0;border-radius:var(--radius-sm);color:var(--text-2);background:transparent;font-size:20px;line-height:1}
  .practice-pager :global(.ticks){opacity:1}
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
  .primary{color:var(--action-on)!important;background:var(--action)!important;font-weight:650}
  .center small{color:var(--text-3);font-size:var(--fs-meta)}
  .capability-step{width:min(640px,82vw);max-width:none;gap:0}
  .capability-step h1{margin-bottom:10px}
  .capabilities{width:100%;margin-top:28px;border-top:1px solid var(--hairline)}
  .capability-row{min-height:76px;display:flex;align-items:center;gap:24px;border-bottom:1px solid var(--hairline);text-align:left}
  .capability-row>span{min-width:0;flex:1;display:flex;flex-direction:column;gap:4px}
  .capability-row strong{color:var(--text-1);font-size:var(--fs-ui)}
  .capability-row small{color:var(--text-3);font-size:var(--fs-meta);line-height:var(--lh-normal)}
  .capability-row button{flex:none;min-height:32px;padding:0 12px;border:1px solid var(--hairline);border-radius:var(--radius-md);color:var(--text-2);background:var(--bg-raised);font-size:var(--fs-ui);font-weight:600;white-space:nowrap}
  .capability-row button:hover{color:var(--text-1);background:var(--bg-hover)}
  .capability-row button:active{background:var(--bg-selected)}
  .capability-note{margin-top:14px}
  .error{position:absolute;bottom:8px;margin:0;color:var(--danger);font-size:var(--fs-ui)}
  /* 工具栏 */
  footer{grid-column:1/-1;grid-row:3;display:grid;grid-template-columns:1fr auto 1fr;align-items:center;padding:0 16px;border-top:1px solid var(--hairline)}
  footer button{min-height:30px;padding:7px 12px;border-radius:var(--radius-md);color:var(--text-2);background:transparent;font-size:var(--fs-ui)}
  footer .finish{grid-column:3;justify-self:end;display:inline-flex;align-items:center;justify-content:center;gap:6px}
  footer .finish :global(.finish-spinner){animation:finish-spin .8s linear infinite}
  .step-button{display:flex;align-items:center;gap:5px;border:1px solid var(--hairline)}
  .step-button.previous{justify-self:start}
  .step-button.next{grid-column:3;justify-self:end}
  .step-progress{grid-column:2;color:var(--text-3);font:500 var(--fs-meta)/1 var(--mono);font-variant-numeric:tabular-nums}
  button:hover:not(:disabled){background:var(--bg-hover)}
  button:disabled{opacity:.4}
  @keyframes finish-spin{to{transform:rotate(360deg)}}
  @media(prefers-reduced-motion:reduce){footer .finish :global(.finish-spinner){animation:none}}
  :global(.language-trigger:focus-visible),:global(.language-menu [role="menuitemradio"]:focus-visible),.body button:focus-visible,footer button:focus-visible{outline:2px solid var(--text-1);outline-offset:2px}
  .sandbox-list:focus-visible,.mini-row[role=option]:focus-visible{outline:none}
  .sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  @media(max-width:780px){.body.practice{grid-template-columns:1fr;grid-template-rows:auto minmax(250px,1fr)}.legend{padding:16px 22px 8px;border-right:0}.sandbox{padding:8px 22px 16px}}
  @media(prefers-reduced-motion:no-preference){.body>.center,.body>.legend,.body>.sandbox{animation:enter .17s ease-out}@keyframes enter{from{opacity:.25}}}
</style>
