// Render the actual app frontend with isolated fixtures; no native backend is started.
// Usage: pnpm dev --host 127.0.0.1 --port 1420
// node scripts/capture-marketing.mjs /absolute/path/to/playwright/index.mjs
import assert from 'node:assert/strict';
import { mkdir, readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

const { chromium } = await import(pathToFileURL(process.argv[2]).href);
const output = new URL('../outputs/marketing-ui/', import.meta.url);
const png = await readFile(new URL('../static/app-icon.png', import.meta.url));
const imageInfo = { width: png.readUInt32BE(16), height: png.readUInt32BE(20), bytes: png.length };
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ channel: 'chrome', headless: true });
try {
  for (const theme of ['dark', 'light']) {
    const context = await browser.newContext({ viewport: { width: 1100, height: 700 }, deviceScaleFactor: 2, locale: 'zh-CN', reducedMotion: 'reduce' });
    // Only local frontend assets are allowed. No external enrichment or update traffic.
    await context.route('**/*', route => new URL(route.request().url()).origin === 'http://127.0.0.1:1420' ? route.continue() : route.abort());
    await context.addInitScript(({ theme, imageInfo }) => {
      const entries = [
        ['text', '发布前，再检查一遍。\n\n确认中英文文案\n检查下载链接\n准备产品截图\n\n让工具安静，让内容清晰。'],
        ['link', 'https://clipclop.io'],
        ['image', 'ClipClop 应用图标'],
        ['color', '#ECEEF0'],
        ['text', 'const greeting = "Hello, ClipClop";\nconsole.log(greeting);'],
        ['file', '/Users/demo/Documents/产品介绍.pdf'],
        ['text', '剪贴历史，本地保存，随手取用。'],
        ['link', 'https://github.com/hiQianFan/ClipClop'],
        ['color', '#17181A'],
        ['text', '周五 15:00 · 产品设计评审'],
      ];
      const items = entries.map(([content_type, text], index) => ({
        id: `demo-${index}`, content_type, preview: text, plain_text: text, source_app: null,
        created_at: '2026-09-08T02:00:00Z', last_used_at: '2026-09-08T02:15:00Z',
        byte_size: content_type === 'image' ? imageInfo.bytes : new TextEncoder().encode(text).length,
        metadata: content_type === 'image' ? { width: imageInfo.width, height: imageInfo.height } : content_type === 'file' ? { files: [text] } : { char_count: [...text].length }, flavors: [],
      }));
      const settings = { retention_days: 30, history_limit: 500, move_used_to_top: false, restore_browse_position: false, preserve_search_conditions: false, trim_whitespace: false, file_preview_enabled: false, launch_at_login: false, hotkey: 'Control+Super+C', theme, language: 'zh-CN', tray_click_action: 'recent', check_updates: false, last_update_check: null, skipped_update_version: null };
      const resource = { data_url: null, byte_size: null, access_denied: false, is_directory: false };
      window.captureBlocked = [];
      window.__TAURI_INTERNALS__ = {
        metadata: { currentWindow: { label: 'capture' }, currentWebview: { label: 'capture' } },
        transformCallback: () => 1,
        invoke: async (command, args = {}) => {
          if (command === 'get_settings') return settings;
          if (command === 'get_onboarding_state') return { completed_revision: 1, current_step: 'overview', visited_steps: ['overview'], selected_example: 'image' };
          if (command === 'get_preview_capability') return { provider: 'unavailable', reason: 'detection_failed', version: null };
          if (command === 'query_history') {
            const request = args.request;
            const matching = items.filter(item => item.preview.toLowerCase().includes(request.query.toLowerCase()));
            return { items: matching, page: 1, page_size: 10, total: matching.length, total_pages: matching.length ? 1 : 0 };
          }
          if (command === 'get_history_facets') return { type_total: items.length, type_counts: { text: 4, link: 2, image: 1, color: 2, file: 1 }, sources: [] };
          if (command === 'get_clip') return items.find(item => item.id === args.id);
          if (['get_clip_asset', 'get_clip_thumbnail'].includes(command)) return { ...resource, data_url: '/app-icon.png' };
          if (['get_clip_file_asset', 'get_source_app_icon'].includes(command)) return resource;
          if (['plugin:event|listen', 'plugin:event|unlisten'].includes(command)) return 1;
          window.captureBlocked.push(command);
          throw new Error(`Capture disallows native command: ${command}`);
        },
      };
    }, { theme, imageInfo });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto('http://127.0.0.1:1420');
    await page.getByRole('option').first().waitFor();
    assert.equal(await page.getByRole('option').count(), 10);
    await page.locator('.preview-body pre').waitFor();
    await page.screenshot({ path: new URL(`${theme}-text.png`, output).pathname, omitBackground: true });
    await page.locator('#clip-demo-2').click();
    await page.locator('.asset:not(.thumbnail)').waitFor();
    await page.locator('.asset').evaluate(async image => { await image.decode(); });
    await page.screenshot({ path: new URL(`${theme}-image.png`, output).pathname, omitBackground: true });
    await page.getByRole('textbox').fill('设计评审');
    await page.waitForFunction(() => document.querySelectorAll('[role="option"]').length === 1);
    assert.match(await page.getByRole('option').innerText(), /设计评审/);
    assert.deepEqual(await page.evaluate(() => window.captureBlocked), []);
    assert.deepEqual(errors, []);
    await context.close();
  }
  console.log(`Four 2x frontend screenshots saved to ${output.pathname}; selection/search and isolation checks passed.`);
} finally {
  await browser.close();
}
