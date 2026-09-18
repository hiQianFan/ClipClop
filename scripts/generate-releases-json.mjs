#!/usr/bin/env node
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";

const REPO = process.env.GITHUB_REPOSITORY || "hiQianFan/ClipClop";
const OUTPUT = process.argv[2] || ".release-assets/releases.json";
const MAX_PAGES = 100;

function hasNext(link) {
  return /(?:^|,\s*)<[^>]+>;\s*rel="next"(?:,|$)/.test(link || "");
}

function assertSafeGitHubHtml(html) {
  if (!html) return;
  const lower = html.toLowerCase();
  assert(!lower.includes("<script"), "release HTML contains script markup");
  assert(!/\son[a-z]+\s*=/.test(lower), "release HTML contains event attributes");
  assert(!/(href|src)\s*=\s*["']?\s*javascript:/i.test(html), "release HTML contains javascript URL");
}

export function buildReleaseFeed(input, generatedAt = new Date().toISOString()) {
  const byTag = new Map();
  for (const release of input) {
    if (release.draft || release.prerelease || !release.tag_name || !release.published_at) continue;
    if (byTag.has(release.tag_name)) continue;
    assertSafeGitHubHtml(release.body_html);
    byTag.set(release.tag_name, release);
  }
  const releases = [...byTag.values()]
    .sort((a, b) => Date.parse(b.published_at) - Date.parse(a.published_at) || Number(b.id || 0) - Number(a.id || 0))
    .map((release, index) => ({
      version: release.tag_name,
      publishedAt: release.published_at,
      name: release.name || release.tag_name,
      notes: (release.body || "").trim(),
      notesHtml: release.body_html?.trim() || null,
      url: release.html_url || `https://github.com/${REPO}/releases/tag/${release.tag_name}`,
      isLatest: index === 0,
    }));
  const contentHash = createHash("sha256").update(JSON.stringify(releases)).digest("hex");
  return { schemaVersion: 1, generatedAt, contentHash, total: releases.length, releases };
}

async function fetchAllReleases() {
  const headers = { Accept: "application/vnd.github.html+json", "User-Agent": "clipclop-release-sync" };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  const releases = [];
  for (let page = 1; page <= MAX_PAGES; page += 1) {
    const response = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100&page=${page}`, { headers });
    if (!response.ok) throw new Error(`GitHub releases request failed (${response.status})`);
    releases.push(...await response.json());
    if (!hasNext(response.headers.get("Link"))) return releases;
  }
  throw new Error(`GitHub releases exceeded ${MAX_PAGES} pages`);
}

async function writeFeed() {
  const feed = buildReleaseFeed(await fetchAllReleases());
  await mkdir(dirname(OUTPUT), { recursive: true });
  await writeFile(OUTPUT, `${JSON.stringify(feed, null, 2)}\n`);
}

function selfTest() {
  const feed = buildReleaseFeed([
    { id: 1, tag_name: "v0.1.0", published_at: "2026-01-01T00:00:00Z", body: "old", body_html: "<p>old</p>", html_url: "https://example.test/old" },
    { id: 2, tag_name: "v0.2.0", published_at: "2026-02-01T00:00:00Z", body: "new", body_html: "<p>new</p>", html_url: "https://example.test/new" },
    { id: 3, tag_name: "v0.3.0", published_at: "2026-03-01T00:00:00Z", draft: true },
  ], "2026-09-17T00:00:00.000Z");
  assert.equal(feed.schemaVersion, 1);
  assert.equal(feed.total, 2);
  assert.equal(feed.releases[0].version, "v0.2.0");
  assert.equal(feed.releases[0].isLatest, true);
  assert.equal(feed.releases[1].isLatest, false);
  assert.throws(() => buildReleaseFeed([{ tag_name: "v0.1.0", published_at: "2026-01-01T00:00:00Z", body_html: "<script></script>" }]));
}

if (process.argv.includes("--self-test")) selfTest();
else await writeFeed();
