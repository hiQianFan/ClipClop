import { describe, expect, it } from "vitest";
import { HistorySession, type HistorySessionApi } from "./session.svelte";
import type { ClipDetail, ClipSummary, HistoryPage } from "./types";

const item = (id: string): ClipSummary => ({
  id,
  content_type: "text",
  preview: id,
  source_app: null,
  created_at: "2026-01-01T00:00:00Z",
  last_used_at: "2026-01-01T00:00:00Z",
  byte_size: 1,
  metadata: {},
});
const detail = (id: string): ClipDetail => ({ ...item(id), plain_text: id, flavors: [] });
const page = (ids: string[], current = 1, totalPages = 1): HistoryPage => ({
  items: ids.map(item),
  page: current,
  page_size: 10,
  total: ids.length,
  total_pages: totalPages,
});

function api(overrides: Partial<HistorySessionApi> = {}): HistorySessionApi {
  return {
    queryHistory: async () => page(["a", "b"]),
    getClip: async (id) => detail(id),
    deleteClip: async () => {},
    ...overrides,
  };
}

describe("HistorySession", () => {
  it("preserves a selection that exists after refresh", async () => {
    const session = new HistorySession(api());
    await session.refresh();
    await session.select("b");
    await session.refresh();
    expect(session.selectedId).toBe("b");
  });

  it("reloads cached detail when its last-used timestamp changes", async () => {
    let lastUsed = "2026-01-01T00:00:00Z";
    let detailRequests = 0;
    const session = new HistorySession(api({
      queryHistory: async () => ({ ...page(["a"]), items: [{ ...item("a"), last_used_at: lastUsed }] }),
      getClip: async (id) => ({ ...detail(id), last_used_at: lastUsed, plain_text: `${++detailRequests}` }),
    }));
    await session.refresh();
    lastUsed = "2026-01-02T00:00:00Z";
    await session.refresh();
    expect(session.detail?.last_used_at).toBe(lastUsed);
    expect(detailRequests).toBe(2);
  });

  it("ignores stale detail responses", async () => {
    const resolvers = new Map<string, (value: ClipDetail) => void>();
    const session = new HistorySession(api({
      getClip: (id) => new Promise((resolve) => resolvers.set(id, resolve)),
    }));
    const first = session.select("a");
    const second = session.select("b");
    resolvers.get("b")?.(detail("b"));
    await second;
    resolvers.get("a")?.(detail("a"));
    await first;
    expect(session.detail?.id).toBe("b");
  });

  it("reports stale refreshes so callers do not restore obsolete focus", async () => {
    const resolvers: Array<(value: HistoryPage) => void> = [];
    const session = new HistorySession(api({
      queryHistory: () => new Promise((resolve) => resolvers.push(resolve)),
    }));
    const first = session.refresh(1);
    const second = session.refresh(2);
    resolvers[1]?.(page(["new"], 2, 2));
    await expect(second).resolves.toBe(true);
    resolvers[0]?.(page(["old"], 1, 2));
    await expect(first).resolves.toBe(false);
    expect(session.page.page).toBe(2);
  });

  it("selects the next row after deletion", async () => {
    const session = new HistorySession(api());
    await session.refresh();
    await session.deleteSelected();
    expect(session.selectedId).toBe("b");
  });

  it("clamps deletion of the final row to the previous page", async () => {
    const requested: number[] = [];
    const session = new HistorySession(api({
      queryHistory: async (_query, current) => {
        requested.push(current);
        return current === 2 ? page(["a"], 2, 2) : page(["z"], 1, 1);
      },
    }));
    await session.refresh(2);
    await session.deleteSelected();
    expect(requested).toEqual([2, 1]);
    expect(session.page.page).toBe(1);
  });

  it("clamps a page invalidated by concurrent retention or deletion", async () => {
    const requested: number[] = [];
    const session = new HistorySession(api({
      queryHistory: async (_query, current) => {
        requested.push(current);
        return current === 3 ? page([], 3, 2) : page(["last"], 2, 2);
      },
    }));
    await session.refresh(3);
    expect(requested).toEqual([3, 2]);
    expect(session.page.page).toBe(2);
    expect(session.selectedId).toBe("last");
  });

});
