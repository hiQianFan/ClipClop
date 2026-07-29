import { describe, expect, it } from "vitest";
import { HistorySession, type HistorySessionApi } from "./session.svelte";
import type { ClipDetail, ClipSummary, HistoryPage } from "./types";

const item = (id: string): ClipSummary => ({
  id,
  content_type: "text",
  preview: id,
  source_app: null,
  created_at: "2026-01-01T00:00:00Z",
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
});
