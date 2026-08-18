// Update-check/download state lives at module scope, not in the settings component,
// so an in-flight check or download survives the settings view being closed and
// reopened. The view only reads this store and renders it; the async task keeps
// running here regardless of component lifecycle. Messages are derived by the view
// from this raw state so they re-localize on language change.
import {
  cachedUpdate,
  checkForUpdate,
  currentVersion,
  downloadAndInstall,
  skipUpdate,
  type AvailableUpdate,
} from "./api";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "current"
  | "skipped"
  | "downloading"
  | "installing"
  | "error";
export type UpdateErrorSource = null | "check" | "install" | "unsupported";

let phase = $state<UpdatePhase>("idle");
let update = $state<AvailableUpdate | null>(null);
let progress = $state<number | null>(null);
let errorReason = $state<unknown>(null);
let errorSource = $state<UpdateErrorSource>(null);
let appVersion = $state("…");
let hydrated = false;
let installing = false;
let skippedVersion = $state<string | null>(null);

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
  // Only seed from cache when no check has been started this session.
  if (phase === "idle" && !update) update = cachedUpdate(appVersion);
}

async function check() {
  if (phase === "checking") return; // a check is already running
  phase = "checking";
  errorReason = null;
  errorSource = null;
  try {
    const result = await checkForUpdate();
    if (result.kind === "available") {
      appVersion = result.update.currentVersion;
      update = result.update;
      phase = "idle";
    } else if (result.kind === "current") {
      appVersion = result.currentVersion;
      update = null;
      phase = "current";
    } else if (result.kind === "skipped") {
      appVersion = result.currentVersion;
      update = null;
      skippedVersion = result.version;
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

async function install() {
  if (!update || installing) return;
  installing = true;
  phase = "downloading";
  progress = null;
  errorReason = null;
  errorSource = null;
  try {
    await downloadAndInstall(update.version, (value) => {
      progress = value;
    });
    // The app relaunches on success; this phase shows until the process exits.
    phase = "installing";
  } catch (reason) {
    phase = "error";
    errorReason = reason;
    errorSource = "install";
  } finally {
    installing = false;
  }
}

async function skip() {
  if (!update) return;
  await skipUpdate(update);
  skippedVersion = update.version;
  update = null;
  phase = "skipped";
}

// Retry after an error returns to the appropriate action for the last phase.
function retry() {
  if (errorSource === "install" && update) return install();
  return check();
}

export const updateStore = {
  get phase() { return phase; },
  get update() { return update; },
  get progress() { return progress; },
  get errorReason() { return errorReason; },
  get errorSource() { return errorSource; },
  get appVersion() { return appVersion; },
  get skippedVersion() { return skippedVersion; },
  get busy() { return phase === "downloading" || phase === "installing"; },
  hydrate,
  check,
  install,
  skip,
  retry,
};
