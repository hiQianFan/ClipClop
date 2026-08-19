import {
  getClipAsset,
  getClipFileAsset,
  getClipThumbnail,
  getSourceAppIcon,
} from "./api";
import type { ClipDetail, ClipSummary } from "./types";

export type PreviewResource = { data_url: string | null; byte_size: number | null; access_denied: boolean };

export type PreviewApi = {
  getClipAsset(id: string): Promise<PreviewResource>;
  getClipFileAsset(id: string, index: number): Promise<PreviewResource>;
  getClipThumbnail(id: string): Promise<PreviewResource>;
  getSourceAppIcon(id: string): Promise<PreviewResource>;
};

const defaultApi: PreviewApi = {
  getClipAsset,
  getClipFileAsset,
  getClipThumbnail,
  getSourceAppIcon,
};

const ASSET_DEBOUNCE_MS = 80;

export class PreviewSession {
  assetUrl = $state<string | null>(null);
  fileAccessDenied = $state(false);
  sourceIconUrl = $state<string | null>(null);
  thumbnailUrls = $state<Record<string, string>>({});
  fileThumbnailUrls = $state<Array<string | null>>([]);
  fileByteSizes = $state<Array<number | null>>([]);

  #api: PreviewApi;
  #assets = new Map<string, PreviewResource>();
  #thumbnails = new Map<string, string>();
  #sourceIcons = new Map<string, string | null>();
  #selectionVersion = 0;
  #fileVersion = 0;
  #pageVersion = 0;
  #assetTimer: ReturnType<typeof setTimeout> | undefined;
  #resolveTimer: (() => void) | undefined;

  constructor(api: PreviewApi = defaultApi) {
    this.#api = api;
  }

  resetSelection() {
    ++this.#selectionVersion;
    ++this.#fileVersion;
    this.#cancelTimer();
    this.assetUrl = null;
    this.fileAccessDenied = false;
    this.sourceIconUrl = null;
    this.fileThumbnailUrls = [];
    this.fileByteSizes = [];
  }

  resetPage() {
    ++this.#pageVersion;
    this.thumbnailUrls = {};
  }

  async loadSelection(id: string, detail: ClipDetail, readSelectedFile: boolean) {
    const version = this.#selectionVersion;
    void this.#loadSourceIcon(id, detail, version);
    if (detail.content_type === "image") {
      await this.#loadAsset(id, null, version, this.#fileVersion);
    } else if (detail.content_type === "file" && readSelectedFile) {
      await this.loadFile(id, 0);
    }
  }

  async loadFile(id: string, index: number) {
    const fileVersion = ++this.#fileVersion;
    this.assetUrl = null;
    this.fileAccessDenied = false;
    await this.#loadAsset(id, index, this.#selectionVersion, fileVersion);
  }

  async loadPageThumbnails(items: ClipSummary[]) {
    const version = this.#pageVersion;
    this.#applyCachedThumbnails(items);
    for (const item of items) {
      if (version !== this.#pageVersion) return;
      if (item.content_type !== "image" || this.#thumbnails.has(item.id)) continue;
      try {
        const thumbnail = await this.#api.getClipThumbnail(item.id);
        if (version !== this.#pageVersion) return;
        if (thumbnail.data_url) cacheSet(this.#thumbnails, item.id, thumbnail.data_url);
      } catch {
        // A neutral media icon is the intentional fallback.
      }
      if (version === this.#pageVersion) this.#applyCachedThumbnails(items);
    }
  }

  evict(id: string) {
    this.#thumbnails.delete(id);
    for (const key of this.#assets.keys()) {
      if (key.startsWith(`${id}:`)) this.#assets.delete(key);
    }
  }

  clear() {
    this.resetSelection();
    this.resetPage();
    this.#assets.clear();
    this.#thumbnails.clear();
    this.#sourceIcons.clear();
  }

  async #loadSourceIcon(id: string, detail: ClipDetail, version: number) {
    const source = detail.source_app;
    if (!source) return;
    if (this.#sourceIcons.has(source.id)) {
      this.sourceIconUrl = this.#sourceIcons.get(source.id) ?? null;
      return;
    }
    try {
      const icon = await this.#api.getSourceAppIcon(id);
      if (version !== this.#selectionVersion) return;
      cacheSet(this.#sourceIcons, source.id, icon.data_url);
      this.sourceIconUrl = icon.data_url;
    } catch {
      if (version !== this.#selectionVersion) return;
      cacheSet(this.#sourceIcons, source.id, null);
    }
  }

  #loadAsset(id: string, index: number | null, selectionVersion: number, fileVersion: number) {
    const key = `${id}:${index ?? "image"}`;
    const cached = this.#assets.get(key);
    if (cached) {
      this.#applyAsset(index, cached, selectionVersion, fileVersion);
      return Promise.resolve();
    }

    this.#cancelTimer();
    return new Promise<void>((resolve, reject) => {
      this.#resolveTimer = resolve;
      this.#assetTimer = globalThis.setTimeout(async () => {
        this.#assetTimer = undefined;
        this.#resolveTimer = undefined;
        try {
          const asset = index === null
            ? await this.#api.getClipAsset(id)
            : await this.#api.getClipFileAsset(id, index);
          if (!this.#isCurrent(selectionVersion, fileVersion)) return resolve();
          cacheSet(this.#assets, key, asset);
          this.#applyAsset(index, asset, selectionVersion, fileVersion);
          resolve();
        } catch (reason) {
          if (!this.#isCurrent(selectionVersion, fileVersion)) return resolve();
          if (index !== null) return resolve();
          reject(reason);
        }
      }, ASSET_DEBOUNCE_MS);
    });
  }

  #applyAsset(
    index: number | null,
    asset: PreviewResource,
    selectionVersion: number,
    fileVersion: number,
  ) {
    if (!this.#isCurrent(selectionVersion, fileVersion)) return;
    this.assetUrl = asset.data_url;
    this.fileAccessDenied = index !== null && asset.access_denied;
    if (index === null) return;
    this.fileThumbnailUrls[index] = asset.data_url;
    if (asset.byte_size !== null) this.fileByteSizes[index] = asset.byte_size;
  }

  #isCurrent(selectionVersion: number, fileVersion: number) {
    return selectionVersion === this.#selectionVersion && fileVersion === this.#fileVersion;
  }

  #applyCachedThumbnails(items: ClipSummary[]) {
    this.thumbnailUrls = Object.fromEntries(items.flatMap((item) => {
      const thumbnail = this.#thumbnails.get(item.id);
      return thumbnail ? [[item.id, thumbnail]] : [];
    }));
  }

  #cancelTimer() {
    if (this.#assetTimer !== undefined) globalThis.clearTimeout(this.#assetTimer);
    this.#assetTimer = undefined;
    this.#resolveTimer?.();
    this.#resolveTimer = undefined;
  }
}

function cacheSet<K, V>(cache: Map<K, V>, key: K, value: V) {
  if (!cache.has(key) && cache.size >= 100) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  cache.set(key, value);
}
