import { deleteClip, getClip, queryHistory } from "./api";
import type { ClipDetail, HistoryPage } from "./types";

export type HistorySessionApi = {
  queryHistory(query: string, page: number): Promise<HistoryPage>;
  getClip(id: string): Promise<ClipDetail>;
  deleteClip(id: string): Promise<void>;
};

const defaultApi: HistorySessionApi = { queryHistory, getClip, deleteClip };
const emptyPage = (page = 1): HistoryPage => ({
  items: [],
  page,
  page_size: 10,
  total: 0,
  total_pages: 0,
});

export class HistorySession {
  page = $state<HistoryPage>(emptyPage());
  query = $state("");
  selectedId = $state<string | null>(null);
  detail = $state<ClipDetail | null>(null);
  loading = $state(true);
  errorReason = $state<unknown>(null);
  detailPending = $state(false);

  #api: HistorySessionApi;
  #details = new Map<string, ClipDetail>();
  #refreshVersion = 0;
  #detailVersion = 0;

  constructor(api: HistorySessionApi = defaultApi) {
    this.#api = api;
  }

  async refresh(targetPage = this.page.page, selectLatest = false) {
    const version = ++this.#refreshVersion;
    this.loading = true;
    this.errorReason = null;
    try {
      let nextPage = await this.#api.queryHistory(this.query, targetPage);
      if (version !== this.#refreshVersion) return false;
      if (
        nextPage.items.length === 0
        && targetPage > 1
        && nextPage.total_pages < targetPage
      ) {
        nextPage = await this.#api.queryHistory(
          this.query,
          Math.max(1, nextPage.total_pages),
        );
        if (version !== this.#refreshVersion) return false;
      }
      this.page = nextPage;
      const nextId = !selectLatest && nextPage.items.some(({ id }) => id === this.selectedId)
        ? this.selectedId
        : nextPage.items[0]?.id ?? null;
      const nextSummary = nextPage.items.find(({ id }) => id === nextId);
      if (nextSummary && this.#details.get(nextSummary.id)?.last_used_at !== nextSummary.last_used_at) {
        this.#details.delete(nextSummary.id);
      }
      await this.select(nextId);
    } catch (reason) {
      if (version !== this.#refreshVersion) return false;
      this.errorReason = reason;
      this.page = emptyPage(targetPage);
      await this.select(null);
    } finally {
      if (version === this.#refreshVersion) this.loading = false;
    }
    return version === this.#refreshVersion;
  }

  async select(id: string | null) {
    const version = ++this.#detailVersion;
    this.selectedId = id;
    this.detail = null;
    this.detailPending = id !== null;
    if (!id) return;
    try {
      const detail = this.#details.get(id) ?? await this.#api.getClip(id);
      this.#details.set(id, detail);
      if (version === this.#detailVersion) {
        this.detail = detail;
        this.errorReason = null;
      }
    } catch (reason) {
      if (version === this.#detailVersion) this.errorReason = reason;
    } finally {
      if (version === this.#detailVersion) this.detailPending = false;
    }
  }

  async deleteSelected() {
    if (!this.selectedId) return;
    const id = this.selectedId;
    const index = this.page.items.findIndex((item) => item.id === id);
    const successor = this.page.items[index + 1]?.id ?? this.page.items[index - 1]?.id ?? null;
    const targetPage = this.page.items.length === 1 && this.page.page > 1
      ? this.page.page - 1
      : this.page.page;

    await this.#api.deleteClip(id);
    this.#details.delete(id);
    this.selectedId = successor;
    await this.refresh(targetPage);
  }

  clearCaches() {
    this.#details.clear();
  }

  evict(id: string) {
    this.#details.delete(id);
  }
}
