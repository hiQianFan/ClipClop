import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent } from "@tauri-apps/plugin-updater";
import { getSettings, recordUpdateCheck } from "$lib/settings/api";

export const RELEASE_URL = "https://github.com/hiQianFan/ClipClop/releases/latest";
export const DEVELOPMENT_VERSION = "__clipclop_development__";
type UpdaterErrorCode = "UPDATE_UNSUPPORTED" | "UPDATE_CHANGED";

function updaterError(code: UpdaterErrorCode) {
  return { code };
}

const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;
const AUTO_CHECK_DELAY_MS = 15_000;
const AUTO_CHECK_LOCK_MS = 60_000;
const CACHE_KEY = "clipclop.available-update";
const LOCK_KEY = "clipclop.update-check-lock";

export type AvailableUpdate = {
  version: string;
  currentVersion: string;
  date: string | null;
  notes: string;
};

export type UpdateCheckResult =
  | { kind: "unsupported"; currentVersion: string }
  | { kind: "current"; currentVersion: string }
  | { kind: "available"; update: AvailableUpdate };

let activeCheck: Promise<UpdateCheckResult> | null = null;

export function shouldAutoCheck(enabled: boolean, lastCheck: string | null, now = Date.now()) {
  if (!enabled) return false;
  if (!lastCheck) return true;
  const checkedAt = Date.parse(lastCheck);
  return !Number.isFinite(checkedAt) || now - checkedAt >= CHECK_INTERVAL_MS;
}

function readCachedUpdate(): AvailableUpdate | null {
  try {
    const value = localStorage.getItem(CACHE_KEY);
    return value ? (JSON.parse(value) as AvailableUpdate) : null;
  } catch {
    return null;
  }
}

function writeCachedUpdate(update: AvailableUpdate | null) {
  if (update) localStorage.setItem(CACHE_KEY, JSON.stringify(update));
  else localStorage.removeItem(CACHE_KEY);
}

export function cachedUpdate() {
  return readCachedUpdate();
}

export async function currentVersion() {
  return isTauri() ? getVersion() : DEVELOPMENT_VERSION;
}

export async function checkForUpdate(): Promise<UpdateCheckResult> {
  if (activeCheck) return activeCheck;
  activeCheck = performCheck().finally(() => {
    activeCheck = null;
  });
  return activeCheck;
}

async function performCheck(): Promise<UpdateCheckResult> {
  const version = await currentVersion();
  if (!isTauri() || import.meta.env.DEV) {
    return { kind: "unsupported", currentVersion: version };
  }

  // Record the attempt before networking so an offline launch does not retry on every startup.
  // Manual checks remain available regardless of this timestamp.
  await recordUpdateCheck();
  const found = await check({ timeout: 30_000 });

  if (!found) {
    writeCachedUpdate(null);
    return { kind: "current", currentVersion: version };
  }

  const update: AvailableUpdate = {
    version: found.version,
    currentVersion: found.currentVersion,
    date: found.date ?? null,
    notes: found.body?.trim() ?? "",
  };
  await found.close();
  writeCachedUpdate(update);
  return { kind: "available", update };
}

export async function downloadAndInstall(
  expectedVersion: string,
  onProgress: (percent: number | null) => void,
) {
  if (!isTauri() || import.meta.env.DEV) throw updaterError("UPDATE_UNSUPPORTED");
  const found = await check({ timeout: 30_000 });
  if (!found || found.version !== expectedVersion) {
    await found?.close();
    throw updaterError("UPDATE_CHANGED");
  }

  let downloaded = 0;
  let total: number | undefined;
  const progress = (event: DownloadEvent) => {
    if (event.event === "Started") total = event.data.contentLength;
    if (event.event === "Progress") downloaded += event.data.chunkLength;
    if (event.event === "Finished") onProgress(100);
    else onProgress(total ? Math.min(99, Math.round((downloaded / total) * 100)) : null);
  };

  try {
    await found.downloadAndInstall(progress, { timeout: 120_000 });
  } finally {
    try { await found.close(); } catch { /* Do not mask install failure or block relaunch. */ }
  }
  writeCachedUpdate(null);
  await relaunch();
}

export async function openLatestRelease() {
  await openUrl(RELEASE_URL);
}

export function scheduleAutomaticUpdateCheck() {
  if (!isTauri() || import.meta.env.DEV) return () => {};
  const timer = window.setTimeout(async () => {
    try {
      const settings = await getSettings();
      if (!shouldAutoCheck(settings.check_updates, settings.last_update_check)) return;

      const lock = Number(localStorage.getItem(LOCK_KEY) ?? 0);
      if (Date.now() - lock < AUTO_CHECK_LOCK_MS) return;
      localStorage.setItem(LOCK_KEY, String(Date.now()));
      await checkForUpdate();
    } catch {
      // Automatic checks are deliberately silent; manual checks expose recovery UI.
    }
  }, AUTO_CHECK_DELAY_MS);
  return () => window.clearTimeout(timer);
}
