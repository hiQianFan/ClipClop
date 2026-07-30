import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import HistoryList from "./HistoryList.svelte";
import type { HistoryPage } from "./types";

describe("HistoryList semantics", () => {
  it("announces positions across the full paged result", () => {
    const page: HistoryPage = {
      items: [{
        id: "clip-11",
        content_type: "text",
        preview: "text",
        source_app: null,
        created_at: "2026-01-01T00:00:00Z",
        byte_size: 4,
        metadata: {},
      }],
      page: 2,
      page_size: 10,
      total: 25,
      total_pages: 3,
    };
    const { body } = render(HistoryList, {
      props: {
        page,
        query: "",
        selectedId: "clip-11",
        expandedId: null,
        fileIndex: 0,
        loading: false,
        error: "",
        thumbnailUrls: {},
        reducedMotion: true,
        rowReorderMotion: false,
        onsearch() {},
        onsearchfocus() {},
        onsearchkeydown() {},
        onlistfocus() {},
        onselect() {},
        onpaste() {},
        onfile() {},
        onkeydown() {},
        onpage() {},
      },
    });
    expect(body).toContain('aria-posinset="11"');
    expect(body).toContain('aria-setsize="25"');
  });
});
