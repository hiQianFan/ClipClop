// Update-check/download state lives at module scope, not in the settings component,
// so an in-flight check or download survives the settings view being closed and
// reopened. The view only reads this store and renders it; the async task keeps
// running here regardless of component lifecycle. Messages are derived by the view
// from this raw state so they re-localize on language change.
import {
  cachedUpdate,
  cancelUpdateDownload,
  checkForUpdate,
  currentVersion,
  discardDownloadedUpdate,
  downloadUpdate,
  installDownloadedUpdate,
  relaunchAfterUpdate,
  skipUpdate,
  type AvailableUpdate,
} from "./api";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "current"
  | "skipped"
  | "downloading"
  | "downloaded"
  | "installing"
  | "error";
export type UpdateErrorSource = null | "check" | "download" | "install" | "relaunch" | "unsupported";
export type UpdateDisplayStatus = null | "current" | "available" | "skipped";

let phase = $state<UpdatePhase>("idle");
let update = $state<AvailableUpdate | null>(null);
let progress = $state<number | null>(null);
let errorReason = $state<unknown>(null);
let errorSource = $state<UpdateErrorSource>(null);
let appVersion = $state("…");
let hydrated = false;
let working = false;
let activeDownload: Promise<void> | null = null;
let skippedVersion = $state<string | null>(null);
let displayStatus = $state<UpdateDisplayStatus>(null);
let developmentPreview = false;

export type DevelopmentUpdatePreview = "available" | "downloading" | "downloaded" | "installing" | "download-error" | "install-error";

export function parseDevelopmentUpdatePreview(search: string): DevelopmentUpdatePreview | null {
  const value = new URLSearchParams(search).get("updatePreview");
  return value === "available" || value === "downloading" || value === "downloaded" || value === "installing" || value === "download-error" || value === "install-error"
    ? value
    : null;
}

function applyDevelopmentPreview(preview: DevelopmentUpdatePreview) {
  developmentPreview = true;
  appVersion = "0.4.3";
  update = { version: "0.5.0", currentVersion: appVersion, date: null, notes: "Development update preview" };
  progress = preview === "downloading" ? 42 : null;
  errorReason = preview.endsWith("error") ? "Development preview error" : null;
  errorSource = preview === "download-error" ? "download" : preview === "install-error" ? "install" : null;
  phase = preview === "available" ? "idle" : preview === "download-error" || preview === "install-error" ? "error" : preview;
  displayStatus = "available";
}

// Seed appVersion and any cached pending update exactly once per session. On later
// mounts the store already reflects the latest in-session truth (including an
// in-flight "checking" phase), so re-reading the cache would clobber it.
async function hydrate() {
  if (hydrated) return;
  hydrated = true;
  try {
    appVersion = await currentVersion();
  } catch {
    appVersion = "__clipclop_unknown__";
  }
  if (import.meta.env.DEV) {
    const preview = parseDevelopmentUpdatePreview(window.location.search);
    if (preview) {
      applyDevelopmentPreview(preview);
      return;
    }
  }
  // Only seed from cache when no check has been started this session.
  if (phase === "idle" && !update) {
    update = cachedUpdate(appVersion);
    if (update) displayStatus = "available";
  }
}

async function check() {
  if (phase === "checking" || working) return;
  if (developmentPreview) return;
  const previousVersion = update?.version;
  const previousPhase = phase;
  phase = "checking";
  errorReason = null;
  errorSource = null;
  try {
    const result = await checkForUpdate();
    if (result.kind === "available") {
      appVersion = result.update.currentVersion;
      update = result.update;
      if (previousVersion && previousVersion !== update.version) await discardDownloadedUpdate();
      displayStatus = "available";
      phase = previousPhase === "downloaded" && previousVersion === update.version ? "downloaded" : "idle";
    } else if (result.kind === "current") {
      await discardDownloadedUpdate();
      appVersion = result.currentVersion;
      update = null;
      displayStatus = "current";
      phase = "current";
    } else if (result.kind === "skipped") {
      await discardDownloadedUpdate();
      appVersion = result.currentVersion;
      update = null;
      skippedVersion = result.version;
      displayStatus = "skipped";
      phase = "skipped";
    } else {
      phase = "error";
      errorSource = "unsupported";
    }
  } catch (reason) {
    phase = "error";
    errorReason = reason;
    errorSource = "check";
  }
}

async function download(autoInstall = false) {
  if (!update || working) return;
  if (developmentPreview) {
    phase = autoInstall ? "installing" : "downloaded";
    progress = autoInstall ? null : 100;
    return;
  }
  working = true;
  phase = "downloading";
  progress = null;
  errorReason = null;
  errorSource = null;
  let changed = false;
  try {
    activeDownload = downloadUpdate(update.version, (value) => {
      progress = value;
    });
    await activeDownload;
    phase = "downloaded";
    if (autoInstall) {
      working = false;
      await install();
    }
  } catch (reason) {
    if (reason === "UPDATE_CANCELLED") phase = "idle";
    else if (reason && typeof reason === "object" && "code" in reason && reason.code === "UPDATE_CHANGED") {
      update = null;
      displayStatus = null;
      changed = true;
    }
    else {
      phase = "error";
      errorReason = reason;
      errorSource = "download";
    }
  } finally {
    activeDownload = null;
    working = false;
  }
  if (changed) await check();
}

async function cancel() {
  if (phase !== "downloading") return;
  if (developmentPreview) { progress = null; phase = "idle"; return; }
  try {
    await cancelUpdateDownload();
    await activeDownload;
  } catch (reason) {
    if (reason !== "UPDATE_CANCELLED") {
      phase = "error";
      errorReason = reason;
      errorSource = "download";
    }
  }
}

async function install() {
  if (!update || working || phase !== "downloaded") return;
  if (developmentPreview) { phase = "installing"; return; }
  working = true;
  phase = "installing";
  try {
    await installDownloadedUpdate(update.version);
    try {
      await relaunchAfterUpdate();
    } catch (reason) {
      phase = "error";
      errorReason = reason;
      errorSource = "relaunch";
    }
  } catch (reason) {
    phase = "error";
    errorReason = reason;
    errorSource = "install";
  } finally {
    working = false;
  }
}

async function skip() {
  if (!update || working) return;
  if (developmentPreview) { skippedVersion = update.version; update = null; phase = "skipped"; displayStatus = "skipped"; return; }
  await discardDownloadedUpdate();
  await skipUpdate(update);
  skippedVersion = update.version;
  update = null;
  displayStatus = "skipped";
  phase = "skipped";
}

// Retry after an error returns to the appropriate action for the last phase.
async function retry() {
  if (working) return;
  if (errorSource === "relaunch") {
    working = true;
    phase = "installing";
    try {
      await relaunchAfterUpdate();
    } catch (reason) {
      phase = "error";
      errorReason = reason;
      errorSource = "relaunch";
    } finally {
      working = false;
    }
    return;
  }
  if (errorSource === "install" && update) {
    phase = "downloaded";
    return await install();
  }
  if (errorSource === "download" && update) return await download();
  return await check();
}

export const updateStore = {
  get phase() { return phase; },
  get update() { return update; },
  get progress() { return progress; },
  get errorReason() { return errorReason; },
  get errorSource() { return errorSource; },
  get appVersion() { return appVersion; },
  get skippedVersion() { return skippedVersion; },
  get displayStatus() { return displayStatus; },
  get busy() { return phase === "downloading" || phase === "installing"; },
  hydrate,
  check,
  download,
  cancel,
  install,
  skip,
  retry,
};
