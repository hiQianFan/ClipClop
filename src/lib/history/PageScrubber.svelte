<script lang="ts">
  import { onDestroy } from "svelte";
  import { performPagerHaptic } from "./api";
  import { draggedPage, PAGE_DRAG_STEP, visiblePageTicks } from "./pager";

  let {
    page,
    totalPages,
    reducedMotion,
    disabled = false,
    displayPage = $bindable<number | null>(null),
    onpage,
  }: {
    page: number;
    totalPages: number;
    reducedMotion: boolean;
    disabled?: boolean;
    displayPage?: number | null;
    onpage: (page: number) => void;
  } = $props();

  let draggingPage = $state(false);
  let dragPointer = 0;
  let dragStartX = 0;
  let dragStartPage = 1;
  let dragPage = $state(1);
  let dragOffset = $state(0);
  let visualPage = $state<number | null>(null);
  let keyDirection = $state<-1 | 0 | 1>(0);
  let keyTicksVisible = $state(false);
  let keyMotionTimer: number | undefined;
  let keyHideTimer: number | undefined;
  let repeatDelay: number | undefined;
  let repeatTimer: number | undefined;
  let repeatDirection: -1 | 0 | 1 = 0;
  let wheelDistance = 0;
  let wheelEndTimer: number | undefined;
  let lastHapticAt = Number.NEGATIVE_INFINITY;
  const currentPage = () => draggingPage ? dragPage : visualPage ?? page;

  $effect(() => {
    if (visualPage === page) visualPage = null;
    displayPage = draggingPage ? dragPage : visualPage;
  });

  function tickPhase() {
    return draggingPage && repeatDirection === 0 ? dragOffset / PAGE_DRAG_STEP : keyDirection;
  }

  function tickStyle(tickPage: number) {
    const position = tickPage - currentPage() + tickPhase();
    const distance = Math.abs(position);
    const height = Math.max(5, 16 - distance * 3.5);
    const opacity = Math.max(.2, 1 - distance * .2);
    return `height:${height}px;opacity:${opacity};transform:translate(${position * 18}px,-50%)`;
  }

  function performStepHaptic() {
    const now = performance.now();
    if (now - lastHapticAt < 40) return;
    lastHapticAt = now;
    void performPagerHaptic();
  }

  function startPageDrag(event: PointerEvent) {
    if (disabled || event.button !== 0) return;
    event.preventDefault();
    draggingPage = true;
    dragPointer = event.pointerId;
    dragStartX = event.clientX;
    dragStartPage = currentPage();
    dragPage = currentPage();
    dragOffset = 0;
    document.documentElement.classList.add("pager-dragging");
    if (event.currentTarget instanceof HTMLElement) event.currentTarget.setPointerCapture(event.pointerId);
  }

  function stopRepeat(resetX?: number) {
    window.clearTimeout(repeatDelay);
    window.clearInterval(repeatTimer);
    repeatDirection = 0;
    if (resetX !== undefined) {
      dragStartX = resetX;
      dragStartPage = dragPage;
      dragOffset = 0;
    }
  }

  function repeatPage(direction: -1 | 1) {
    const next = dragPage + direction;
    if (next < 1 || next > totalPages) return stopRepeat();
    dragPage = next;
    onpage(next);
    performStepHaptic();
    animatePageStep(direction);
  }

  function startRepeat(direction: -1 | 1) {
    if (repeatDirection === direction) return;
    stopRepeat();
    repeatDirection = direction;
    repeatDelay = window.setTimeout(() => {
      repeatPage(direction);
      repeatTimer = window.setInterval(() => repeatPage(direction), 220);
    }, 500);
  }

  function movePageDrag(event: PointerEvent) {
    if (!draggingPage || event.pointerId !== dragPointer) return;
    event.preventDefault();
    if (event.currentTarget instanceof HTMLElement) {
      const bounds = event.currentTarget.getBoundingClientRect();
      if (event.clientX < bounds.left - 24) startRepeat(1);
      else if (event.clientX > bounds.right + 24) startRepeat(-1);
      else if (repeatDirection) stopRepeat(event.clientX);
    }
    if (repeatDirection) return;
    const delta = event.clientX - dragStartX;
    const next = draggedPage(dragStartPage, delta, totalPages);
    const remainder = delta + (next - dragStartPage) * PAGE_DRAG_STEP;
    dragOffset = next === 1 || next === totalPages
      ? Math.max(-18, Math.min(remainder * 0.25, 18))
      : remainder;
    if (next === dragPage) return;
    dragPage = next;
    onpage(next);
    performStepHaptic();
  }

  function stopPageDrag(event: PointerEvent) {
    if (event.pointerId !== dragPointer) return;
    stopRepeat();
    draggingPage = false;
    dragOffset = 0;
    document.documentElement.classList.remove("pager-dragging");
  }

  function animatePageStep(direction: -1 | 1) {
    window.clearTimeout(keyMotionTimer);
    window.clearTimeout(keyHideTimer);
    keyTicksVisible = true;
    keyHideTimer = window.setTimeout(() => keyTicksVisible = false, 700);
    if (reducedMotion) return;
    keyDirection = 0;
    requestAnimationFrame(() => {
      keyDirection = direction;
      keyMotionTimer = window.setTimeout(() => keyDirection = 0, 180);
    });
  }

  export function turnPage(direction: -1 | 1) {
    if (disabled) return;
    const next = currentPage() + direction;
    if (next < 1 || next > totalPages) return;
    visualPage = next;
    onpage(next);
    performStepHaptic();
    animatePageStep(direction);
  }

  export function reset() {
    visualPage = null;
    if (!draggingPage) displayPage = null;
  }

  function onScrubberWheel(event: WheelEvent) {
    if (disabled || Math.abs(event.deltaX) <= Math.abs(event.deltaY) * 1.25) return;
    event.preventDefault();
    window.clearTimeout(wheelEndTimer);
    wheelDistance += event.deltaX;
    while (Math.abs(wheelDistance) >= PAGE_DRAG_STEP) {
      const direction = wheelDistance > 0 ? 1 : -1;
      const before = currentPage();
      turnPage(direction);
      if (currentPage() === before) {
        wheelDistance = 0;
        break;
      }
      wheelDistance -= direction * PAGE_DRAG_STEP;
    }
    wheelEndTimer = window.setTimeout(() => wheelDistance = 0, 120);
  }

  onDestroy(() => {
    if (typeof window === "undefined") return;
    stopRepeat();
    window.clearTimeout(keyMotionTimer);
    window.clearTimeout(keyHideTimer);
    window.clearTimeout(wheelEndTimer);
    document.documentElement.classList.remove("pager-dragging");
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions Pointer-only enhancement; keyboard pagination stays on the owning list. -->
<div class="scrubber" class:dragging={draggingPage} class:disabled aria-disabled={disabled} onwheel={onScrubberWheel} onpointerdown={startPageDrag} onpointermove={movePageDrag} onpointerup={stopPageDrag} onpointercancel={stopPageDrag}>
  <span class="ticks" class:key-visible={keyTicksVisible} class:key-motion={keyDirection !== 0} aria-hidden="true">
    {#each visiblePageTicks(currentPage(), totalPages) as tick}<i class:current={tick === currentPage()} style={tickStyle(tick)}></i>{/each}
  </span>
</div>

<style>
  .scrubber{height:30px;min-width:70px;position:relative;overflow:hidden;border-radius:var(--radius-sm);cursor:grab;touch-action:none;user-select:none;-webkit-user-select:none}
  .scrubber.dragging{cursor:grabbing}
  .scrubber.disabled{cursor:default;opacity:.35}
  .ticks{position:absolute;inset:0;opacity:0;pointer-events:none;transition:opacity 220ms ease-out}
  .scrubber:hover .ticks,.scrubber.dragging .ticks,.ticks.key-visible{opacity:1}
  .ticks i{width:2px;position:absolute;left:50%;top:50%;margin-left:-1px;border-radius:var(--radius-pill);background:var(--text-2);transform-origin:center;transition:none}
  .ticks.key-motion i{transition:height var(--dur-fast) var(--ease-out),opacity var(--dur-fast) ease-out,transform var(--dur-fast) var(--ease-out)}
  @media(prefers-reduced-motion:reduce){.ticks,.ticks.key-motion i{transition:none}}
  :global(html.pager-dragging),:global(html.pager-dragging *){cursor:grabbing!important}
</style>
