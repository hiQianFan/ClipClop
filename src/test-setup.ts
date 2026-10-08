import { afterAll } from "vitest";

// Bits UI restores body styles 24 ms after a scroll-locking overlay unmounts.
// Keep jsdom alive past that timer so slow CI runners don't hit a torn-down document.
afterAll(() => new Promise((resolve) => setTimeout(resolve, 50)));
