// @vitest-environment jsdom
import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const performPagerHaptic = vi.hoisted(() => vi.fn());
vi.mock("./api", () => ({ performPagerHaptic }));

import PageScrubber from "./PageScrubber.svelte";

function pointerEvent(type: string, { button = 0, pointerId, clientX }: { button?: number; pointerId: number; clientX: number }) {
  const event = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperties(event, {
    button: { value: button }, pointerId: { value: pointerId }, clientX: { value: clientX },
  });
  return event;
}

beforeEach(() => performPagerHaptic.mockReset());
afterEach(() => {
  cleanup();
  document.documentElement.classList.remove("pager-dragging");
});

describe("PageScrubber", () => {
  it("keeps the imperative turnPage contract used by HistoryList", () => {
    const onpage = vi.fn();
    const view = render(PageScrubber, { props: { page: 5, totalPages: 6, reducedMotion: true, onpage } });
    view.component.turnPage(1);
    expect(onpage).toHaveBeenLastCalledWith(6);
    view.component.turnPage(1);
    expect(onpage).toHaveBeenCalledTimes(1);
  });

  it("turns horizontal wheel distance into consecutive pages", async () => {
    const onpage = vi.fn();
    const view = render(PageScrubber, { props: { page: 5, totalPages: 20, reducedMotion: true, onpage } });
    const scrubber = view.container.querySelector(".scrubber")!;
    await fireEvent.wheel(scrubber, { deltaX: 96, deltaY: 0 });
    expect(onpage.mock.calls).toEqual([[6], [7]]);
    expect(performPagerHaptic).toHaveBeenCalled();
    await fireEvent.wheel(scrubber, { deltaX: 0, deltaY: 96 });
    expect(onpage).toHaveBeenCalledTimes(2);
  });

  it("preserves drag paging and clears the global dragging cursor", async () => {
    const onpage = vi.fn();
    const view = render(PageScrubber, { props: { page: 5, totalPages: 20, reducedMotion: true, onpage } });
    const scrubber = view.container.querySelector(".scrubber") as HTMLElement;
    scrubber.setPointerCapture = vi.fn();
    vi.spyOn(scrubber, "getBoundingClientRect").mockReturnValue({ left: 0, right: 100 } as DOMRect);
    await fireEvent(scrubber, pointerEvent("pointerdown", { pointerId: 1, clientX: 100 }));
    expect(document.documentElement.classList.contains("pager-dragging")).toBe(true);
    await fireEvent(scrubber, pointerEvent("pointermove", { pointerId: 1, clientX: 4 }));
    expect(onpage).toHaveBeenLastCalledWith(3);
    await fireEvent(scrubber, pointerEvent("pointerup", { pointerId: 1, clientX: 4 }));
    expect(document.documentElement.classList.contains("pager-dragging")).toBe(false);
  });

  it("ignores gestures while disabled and cleans up an interrupted drag", async () => {
    const onpage = vi.fn();
    const disabled = render(PageScrubber, { props: { page: 1, totalPages: 3, reducedMotion: false, disabled: true, onpage } });
    await fireEvent.wheel(disabled.container.querySelector(".scrubber")!, { deltaX: 96 });
    expect(onpage).not.toHaveBeenCalled();
    disabled.unmount();

    const active = render(PageScrubber, { props: { page: 1, totalPages: 3, reducedMotion: false, onpage } });
    const scrubber = active.container.querySelector(".scrubber") as HTMLElement;
    scrubber.setPointerCapture = vi.fn();
    await fireEvent(scrubber, pointerEvent("pointerdown", { pointerId: 2, clientX: 50 }));
    active.unmount();
    expect(document.documentElement.classList.contains("pager-dragging")).toBe(false);
  });
});
