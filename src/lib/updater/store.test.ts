import { beforeEach, describe, expect, it, vi } from "vitest";

const update = { version: "0.5.0", currentVersion: "0.4.3", date: null, notes: "" };

async function loadStore(downloadUpdate: () => Promise<void> = async () => {}) {
  const installDownloadedUpdate = vi.fn(async () => {});
  const relaunchAfterUpdate = vi.fn(async () => {});
  const cancelUpdateDownload = vi.fn(async () => {});
  vi.doMock("./api", () => ({
    cachedUpdate: () => null,
    currentVersion: async () => "0.4.3",
    checkForUpdate: async () => ({ kind: "available", update }),
    downloadUpdate,
    installDownloadedUpdate,
    relaunchAfterUpdate,
    cancelUpdateDownload,
    discardDownloadedUpdate: vi.fn(async () => {}),
    skipUpdate: async () => {},
  }));
  const { updateStore } = await import("./store.svelte");
  await updateStore.check();
  return { updateStore, installDownloadedUpdate, relaunchAfterUpdate, cancelUpdateDownload };
}

beforeEach(() => {
  vi.resetModules();
});

describe("controllable updater state", () => {
  it("accepts only supported development preview states", async () => {
    const { parseDevelopmentUpdatePreview } = await import("./store.svelte");
    expect(parseDevelopmentUpdatePreview("?updatePreview=downloaded")).toBe("downloaded");
    expect(parseDevelopmentUpdatePreview("?updatePreview=installing")).toBe("installing");
    expect(parseDevelopmentUpdatePreview("?updatePreview=unknown")).toBeNull();
  });

  it("keeps a verified download ready for a later install", async () => {
    const { updateStore, installDownloadedUpdate } = await loadStore();
    await updateStore.download();
    expect(updateStore.phase).toBe("downloaded");
    expect(installDownloadedUpdate).not.toHaveBeenCalled();
  });

  it("continues into install only for download and install", async () => {
    const { updateStore, installDownloadedUpdate } = await loadStore();
    await updateStore.download(true);
    expect(updateStore.phase).toBe("installing");
    expect(installDownloadedUpdate).toHaveBeenCalledWith("0.5.0");
  });

  it("returns to the available state when a download is cancelled", async () => {
    let rejectDownload!: (reason: unknown) => void;
    const downloading = new Promise<void>((_, reject) => { rejectDownload = reject; });
    const { updateStore, cancelUpdateDownload } = await loadStore(() => downloading);
    cancelUpdateDownload.mockImplementation(async () => { rejectDownload("UPDATE_CANCELLED"); });
    const task = updateStore.download();
    await Promise.resolve();
    await updateStore.cancel();
    await task;
    expect(cancelUpdateDownload).toHaveBeenCalledOnce();
    expect(updateStore.phase).toBe("idle");
  });

  it("retries only relaunch after installation has succeeded", async () => {
    const { updateStore, installDownloadedUpdate, relaunchAfterUpdate } = await loadStore();
    relaunchAfterUpdate.mockRejectedValueOnce(new Error("restart failed"));
    await updateStore.download();
    await updateStore.install();
    expect(updateStore.errorSource).toBe("relaunch");
    await updateStore.retry();
    expect(installDownloadedUpdate).toHaveBeenCalledOnce();
    expect(relaunchAfterUpdate).toHaveBeenCalledTimes(2);
  });

  it("surfaces a cancel IPC failure as a download error", async () => {
    let rejectDownload!: (reason: unknown) => void;
    const downloading = new Promise<void>((_, reject) => { rejectDownload = reject; });
    const { updateStore, cancelUpdateDownload } = await loadStore(() => downloading);
    cancelUpdateDownload.mockRejectedValueOnce(new Error("cancel IPC failed"));
    const task = updateStore.download();
    await Promise.resolve();
    await updateStore.cancel();
    expect(updateStore.errorSource).toBe("download");
    rejectDownload("UPDATE_CANCELLED");
    await task;
  });
});
