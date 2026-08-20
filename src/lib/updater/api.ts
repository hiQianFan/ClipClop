import { getVersion } from "@tauri-apps/api/app";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { error as logError } from "@tauri-apps/plugin-log";
import { getSettings, recordUpdateCheck, skipUpdateVersion } from "$lib/settings/api";

// Persist the raw failure cause to the diagnostic log before it is localized for
// display. This closes the updater blind spot: without it, install failures collapse
// into a generic message and leave no on-device trace to diagnose later.
function logUpdaterFailure(stage: string, reason: unknown) {
  if (!isTauri()) return;
  const detail = reason instanceof Error ? (reason.stack ?? reason.message) : String(reason);
  void logError(`updater ${stage} failed: ${detail}`).catch(() => {});
}

export const RELEASE_URL = "https://github.com/hiQianFan/ClipClop/releases";
export const DEVELOPMENT_VERSION = "__clipclop_development__";
type UpdaterErrorCode = "UPDATE_UNSUPPORTED" | "UPDATE_CHANGED";

function updaterError(code: UpdaterErrorCode) {
  return { code };
}

const RETRY_ATTEMPTS = 3;
const RETRY_BASE_DELAY_MS = 1500;

function delay(ms: number) {
  return new Promise<void>((resolve) => window.setTimeout(resolve, ms));
}

// Transient network failures (dropped connection, mid-stream decode error, timeout,
// DNS hiccup) are the common failure mode against GitHub's CDN, especially on flaky
// links. These are worth retrying. Our own control-flow errors (UPDATE_CHANGED,
// UPDATE_UNSUPPORTED) and signature failures are terminal — retrying cannot help.
export function isTransientNetworkError(reason: unknown): boolean {
  if (reason && typeof reason === "object" && "code" in reason) return false;
  const message = (reason instanceof Error ? reason.message : String(reason)).toLowerCase();
  if (message.includes("signature") || message.includes("verif")) return false;
  return [
    "error sending request",
    "error decoding response body",
    "timed out",
    "timeout",
    "connection",
    "connreset",
    "reset by peer",
    "network",
    "dns",
    "eof",
    "os error",
    "tls",
    "handshake",
  ].some((signature) => message.includes(signature));
}

// Retry an operation on transient network errors with linear backoff. onAttempt fires
// before each try (including the first) so callers can reset per-attempt UI state such
// as download progress, since a retried download restarts from zero (the updater has no
// resume/range support — a partial download is always discarded).
async function withRetry<T>(
  operation: (attempt: number) => Promise<T>,
  onAttempt?: (attempt: number) => void,
): Promise<T> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= RETRY_ATTEMPTS; attempt += 1) {
    onAttempt?.(attempt);
    try {
      return await operation(attempt);
    } catch (reason) {
      lastError = reason;
      if (attempt >= RETRY_ATTEMPTS || !isTransientNetworkError(reason)) throw reason;
      await delay(RETRY_BASE_DELAY_MS * attempt);
    }
  }
  throw lastError;
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

export type ReleaseNote = { version: string; publishedAt: string; notes: string; notesHtml: string | null; url: string; isLatest: boolean };
const RELEASES_API = "https://api.github.com/repos/hiQianFan/ClipClop/releases?per_page=30";

export async function listReleaseNotes(): Promise<ReleaseNote[]> {
  const response = await fetch(RELEASES_API, { headers: { Accept: "application/vnd.github.html+json" } });
  if (!response.ok) throw new Error(`GitHub releases request failed (${response.status})`);
  const releases = await response.json() as Array<{ tag_name?: string; published_at?: string; body?: string; body_html?: string; html_url?: string; draft?: boolean }>;
  return releases.filter((release) => !release.draft && release.tag_name && release.published_at).map((release, index) => ({
    // GitHub sanitizes Markdown when returning body_html through this media type.
    version: release.tag_name!, publishedAt: release.published_at!, notes: release.body?.trim() ?? "", notesHtml: release.body_html?.trim() || null, url: release.html_url ?? RELEASE_URL, isLatest: index === 0,
  }));
}

export type UpdateCheckResult =
  | { kind: "unsupported"; currentVersion: string }
  | { kind: "current"; currentVersion: string }
  | { kind: "skipped"; currentVersion: string; version: string }
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

// Compare dotted numeric versions (0.1.2). Returns >0 if a>b, <0 if a<b, 0 if equal.
// Any non-numeric/prerelease suffix is ignored; unparseable input yields NaN segments
// that the caller treats as "cannot confirm newer".
export function compareVersions(a: string, b: string): number {
  const parse = (v: string) => v.split(".").map((n) => parseInt(n, 10));
  const pa = parse(a);
  const pb = parse(b);
  for (let i = 0; i < 3; i += 1) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (Number.isNaN(x) || Number.isNaN(y)) return 0;
    if (x !== y) return x - y;
  }
  return 0;
}

// Return the cached pending update only when it is strictly newer than the currently
// running version. A stale entry (e.g. left over after a manual DMG install, or pointing
// at an equal/older version) is discarded so the UI never prompts a downgrade.
export function cachedUpdate(current?: string): AvailableUpdate | null {
  const cached = readCachedUpdate();
  if (!cached) return null;
  if (current && compareVersions(cached.version, current) > 0) return cached;
  writeCachedUpdate(null);
  return null;
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
  let found;
  try {
    found = await withRetry(() => check({ timeout: 30_000 }));
  } catch (reason) {
    logUpdaterFailure("check", reason);
    throw reason;
  }

  if (!found) {
    writeCachedUpdate(null);
    return { kind: "current", currentVersion: version };
  }

  // Defense in depth: the plugin only surfaces newer versions, but never cache or offer a
  // version that is not strictly ahead of what is running — a stale/misconfigured endpoint
  // must not produce a downgrade prompt.
  if (compareVersions(found.version, version) <= 0) {
    await found.close();
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
  const settings = await getSettings();
  if (settings.skipped_update_version === update.version) {
    writeCachedUpdate(null);
    return { kind: "skipped", currentVersion: version, version: update.version };
  }
  writeCachedUpdate(update);
  return { kind: "available", update };
}

export async function skipUpdate(update: AvailableUpdate) {
  await skipUpdateVersion(update.version);
  writeCachedUpdate(null);
}

type ControlledDownloadEvent = {
  requestId: string;
  kind: "progress" | "finished" | "error";
  percent: number | null;
  error: string | null;
};

export async function downloadUpdate(
  expectedVersion: string,
  onProgress: (percent: number | null) => void,
) {
  if (!isTauri() || import.meta.env.DEV) throw updaterError("UPDATE_UNSUPPORTED");
  const requestId = crypto.randomUUID();
  try {
    onProgress(null);
    let finish: ((event: ControlledDownloadEvent) => void) | null = null;
    const unlisten = await listen<ControlledDownloadEvent>("clipclop://update-download", ({ payload }) => {
        if (payload.requestId !== requestId) return;
        if (payload.kind === "progress") onProgress(payload.percent);
        else finish?.(payload);
    });
    try {
      await new Promise<void>((resolve, reject) => {
        finish = (payload) => payload.kind === "finished" ? resolve() : reject(payload.error ?? "Update download failed");
        void invoke("start_update_download", { expectedVersion, requestId }).catch(reject);
      });
    } finally {
      unlisten();
    }
  } catch (reason) {
    if (reason !== "UPDATE_CANCELLED") logUpdaterFailure("download", reason);
    if (reason === "UPDATE_CHANGED") throw updaterError("UPDATE_CHANGED");
    throw reason;
  }
}

export async function cancelUpdateDownload() {
  await invoke("cancel_update_download");
}

export async function discardDownloadedUpdate() {
  await invoke("discard_downloaded_update");
}

export async function installDownloadedUpdate(expectedVersion: string) {
  if (!isTauri() || import.meta.env.DEV) throw updaterError("UPDATE_UNSUPPORTED");
  try {
    await invoke("install_downloaded_update", { expectedVersion });
  } catch (reason) {
    logUpdaterFailure("install", reason);
    if (reason === "UPDATE_CHANGED") throw updaterError("UPDATE_CHANGED");
    throw reason;
  }
  writeCachedUpdate(null);
}

export async function relaunchAfterUpdate() {
  try {
    await relaunch();
  } catch (reason) {
    logUpdaterFailure("relaunch", reason);
    throw reason;
  }
}

export async function openLatestRelease() {
  await invoke("open_release_page");
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
