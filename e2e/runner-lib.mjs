/**
 * StreamGrab E2E —— chrome-devtools MCP 驱动库
 *
 * 直接通过 chrome-devtools-mcp 服务器的 stdio/JSON-RPC 调用其工具
 * （navigate_page / evaluate_script / wait_for / press_key / take_screenshot），
 * 不依赖任何第三方浏览器测试框架。
 */

import { spawn } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { homedir } from "node:os";

export const ROOT = process.cwd();
export const APP_BASE = "http://localhost:5173";

const SERVER_BIN = resolve(
  ROOT,
  "node_modules/chrome-devtools-mcp/build/src/bin/chrome-devtools-mcp.js",
);

/**
 * 解析 Chrome/Edge 可执行文件：
 * 1. E2E_CHROME_PATH 环境变量
 * 2. 常见安装路径（Windows/macOS/Linux）
 * 3. Puppeteer 缓存（CI 上 @puppeteer/browsers install chrome 的产物）
 * 找不到时省略参数，交给 chrome-devtools-mcp 的 Puppeteer 自行解析
 */
function resolveChrome() {
  const candidates = [];
  if (process.env.E2E_CHROME_PATH) {
    candidates.push(process.env.E2E_CHROME_PATH);
  }
  if (process.platform === "win32") {
    candidates.push(
      "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
      "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
      "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
      "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
    );
  } else if (process.platform === "darwin") {
    candidates.push(
      "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
      "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    );
  } else {
    candidates.push(
      "/usr/bin/google-chrome",
      "/usr/bin/google-chrome-stable",
      "/usr/bin/chromium",
      "/usr/bin/chromium-browser",
    );
    try {
      const cacheRoot = resolve(homedir(), ".cache/puppeteer/chrome");
      for (const dir of readdirSync(cacheRoot)) {
        candidates.push(
          resolve(cacheRoot, dir, "chrome-linux64/chrome"),
          resolve(
            cacheRoot,
            dir,
            "chrome-headless-shell-linux64/chrome-headless-shell",
          ),
        );
      }
    } catch {
      /* 无 puppeteer 缓存 */
    }
  }
  return candidates.find((p) => existsSync(p));
}

export function b64(str) {
  return Buffer.from(str, "utf8").toString("base64");
}

export function appUrl(seed, path = "/") {
  const u = new URL(path, APP_BASE);
  if (seed !== undefined && seed !== null) {
    u.searchParams.set("e2e_seed", b64(JSON.stringify(seed)));
  }
  return u.href;
}

export function extractText(res) {
  return (res.content || []).map((c) => c.text || "").join("\n");
}

function parseJsonBlock(text) {
  const m = text.match(/```json\n([\s\S]*?)\n```/);
  if (!m) return undefined;
  try {
    return JSON.parse(m[1]);
  } catch {
    return undefined;
  }
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/**
 * 完整指针点击序列（reka/radix 组件在 pointerdown 上打开弹层/响应交互，
 * 仅 el.click() 不够；纯 JS 点击不要求元素可见）
 */
export const POINTER_CLICK_JS = `
  function dispatchClick(el) {
    const base = { bubbles: true, cancelable: true, view: window, button: 0 };
    const pointer = { ...base, pointerId: 1, pointerType: 'mouse', isPrimary: true };
    el.dispatchEvent(new PointerEvent('pointerdown', pointer));
    el.dispatchEvent(new MouseEvent('mousedown', base));
    el.dispatchEvent(new PointerEvent('pointerup', pointer));
    el.dispatchEvent(new MouseEvent('mouseup', base));
    el.dispatchEvent(new MouseEvent('click', base));
  }
`;

/**
 * chrome-devtools-mcp 的 stdio MCP 客户端
 */
export class McpClient {
  constructor() {
    this.proc = null;
    this.buf = "";
    this.pending = new Map();
    this.nextId = 1;
    this.exited = null;
  }

  async start() {
    const chrome = resolveChrome();
    const args = [
      SERVER_BIN,
      "--headless",
      "--isolated",
      "--viewport=1440x900",
      "--blocked-url-pattern=https://api.github.com/*",
      "--allow-unrestricted-paths",
    ];
    if (chrome) {
      args.push(`--executablePath=${chrome}`);
    }
    this.proc = spawn(process.execPath, args, {
      env: { ...process.env, CI: "true" },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.proc.stdout.setEncoding("utf8");
    this.proc.stderr.setEncoding("utf8");
    this.proc.stdout.on("data", (d) => this._onData(d));
    this.proc.stderr.on("data", (d) => process.stderr.write("[mcp] " + d));
    this.proc.on("exit", (code) => {
      this.exited = code;
    });

    await this.request(
      "initialize",
      {
        protocolVersion: "2024-11-05",
        capabilities: {},
        clientInfo: { name: "streamgrab-e2e", version: "1.0.0" },
      },
      120000,
    );
    this.notify("notifications/initialized", {});
  }

  async stop() {
    if (this.proc && !this.proc.killed) {
      this.proc.kill();
    }
  }

  request(method, params = {}, timeoutMs = 120000) {
    const id = this.nextId++;
    const promise = new Promise((resolveFn, rejectFn) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        rejectFn(new Error(`MCP request timed out: ${method}`));
      }, timeoutMs);
      this.pending.set(id, {
        resolve: (v) => {
          clearTimeout(timer);
          resolveFn(v);
        },
        reject: (e) => {
          clearTimeout(timer);
          rejectFn(e);
        },
      });
    });
    this._write({ jsonrpc: "2.0", id, method, params });
    return promise;
  }

  notify(method, params = {}) {
    this._write({ jsonrpc: "2.0", method, params });
  }

  _write(msg) {
    this.proc.stdin.write(JSON.stringify(msg) + "\n");
  }

  _onData(chunk) {
    this.buf += chunk;
    let idx;
    while ((idx = this.buf.indexOf("\n")) >= 0) {
      const line = this.buf.slice(0, idx).trim();
      this.buf = this.buf.slice(idx + 1);
      if (!line) continue;
      let msg;
      try {
        msg = JSON.parse(line);
      } catch {
        continue;
      }
      if (msg.id !== undefined && this.pending.has(msg.id)) {
        const p = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        if (msg.error)
          p.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
        else p.resolve(msg.result || {});
      }
    }
  }

  async call(tool, args = {}) {
    const res = await this.request("tools/call", {
      name: tool,
      arguments: args,
    });
    if (res.isError) {
      throw new Error(`[${tool}] ${extractText(res)}`);
    }
    return res;
  }
}

/**
 * 页面驱动器：全部交互经 chrome-devtools MCP 工具完成
 */
export class Driver {
  constructor(client) {
    this.client = client;
    this.currentUrl = null;
  }

  /** 清空存储并用 seed 打开应用（每个测试开头调用） */
  async resetAndGo(seed, path = "/") {
    await this.evalOk(
      `() => { try { sessionStorage.clear(); } catch(e){} try { localStorage.clear(); } catch(e){} return 'ok'; }`,
    );
    this.currentUrl = appUrl(seed, path);
    await this.client.call("navigate_page", { url: this.currentUrl });
  }

  /** 重新加载当前页（模拟应用重启；mock 状态从 sessionStorage 恢复） */
  async reload() {
    await this.client.call("navigate_page", { url: this.currentUrl });
  }

  async eval(fnSource) {
    const res = await this.client.call("evaluate_script", {
      function: fnSource,
    });
    return parseJsonBlock(extractText(res));
  }

  async evalOk(fnSource) {
    const v = await this.eval(fnSource);
    if (v === "ok") return v;
    throw new Error(`eval 未返回 ok：${JSON.stringify(v)}`);
  }

  async bodyText() {
    return (await this.eval(`() => document.body.innerText`)) ?? "";
  }

  /** 等待文本出现（轮询 body.innerText，子串匹配；支持 RegExp） */
  async waitText(text, { timeout = 20000 } = {}) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const body = await this.bodyText();
      const hit =
        text instanceof RegExp ? text.test(body) : body.includes(String(text));
      if (hit) return;
      await sleep(200);
    }
    throw new Error(`文本在 ${timeout}ms 内未出现：${text}`);
  }

  async assertText(text, opts) {
    await this.waitText(text, opts);
  }

  async assertNoText(text, { timeout = 15000 } = {}) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const body = await this.bodyText();
      if (!body.includes(text)) return;
      await sleep(200);
    }
    throw new Error(`文本在 ${timeout}ms 后仍存在：${text}`);
  }

  async assertEval(fnSource, { timeout = 15000, message } = {}) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const v = await this.eval(fnSource);
      if (v) return v;
      await sleep(200);
    }
    throw new Error(message || `assertEval 超时：${fnSource}`);
  }

  // ============ 交互 ============

  async clickText(text, opts = {}) {
    const { exact = true, last = false, timeout = 8000 } = opts;
    const tag =
      opts.tag ||
      'button, [role="button"], a, label, [role="option"], [role="menuitem"], [role="switch"], [role="tab"], [role="checkbox"], [role="menuitemcheckbox"], [role="radio"]';
    const start = Date.now();
    while (true) {
      const v = await this.eval(`() => {
        const text = ${JSON.stringify(text)};
        const nodes = Array.from(document.querySelectorAll(${JSON.stringify(tag)}));
        const matches = nodes.filter(n => {
          const t = (n.textContent || '').trim();
          return ${exact ? "t === text" : "t.includes(text)"};
        });
        const el = ${last ? "matches[matches.length - 1]" : "matches[0]"};
        if (!el) return 'NOT_FOUND:' + text;
        ${POINTER_CLICK_JS}
        dispatchClick(el);
        return 'ok';
      }`);
      if (v === "ok") return;
      if (!String(v).startsWith("NOT_FOUND")) {
        throw new Error(`clickText 失败：${v}`);
      }
      if (Date.now() - start >= timeout) {
        throw new Error(`clickText 失败：${v}`);
      }
      await sleep(150);
    }
  }

  async clickTitle(title) {
    const v = await this.eval(`() => {
      const t = ${JSON.stringify(title)};
      const el = document.querySelector('[title="' + t + '"]');
      if (!el) return 'NOT_FOUND_TITLE:' + t;
      ${POINTER_CLICK_JS}
      dispatchClick(el);
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`clickTitle 失败：${v}`);
  }

  /** 在指定任务卡片内点按 title 的快速操作按钮（等待卡片渲染，消除导航后立即操作的偶发失败） */
  async clickTaskAction(title, fileName, { timeout = 8000 } = {}) {
    const start = Date.now();
    while (true) {
      const v = await this.eval(`() => {
        const name = ${JSON.stringify(fileName)};
        const t = ${JSON.stringify(title)};
        const cards = Array.from(document.querySelectorAll('.task-card'));
        const card = cards.find(c => (c.textContent || '').includes(name));
        if (!card) return 'NOT_FOUND_CARD:' + name;
        const el = card.querySelector('[title="' + t + '"]');
        if (!el) return 'NOT_FOUND_TITLE:' + t;
        ${POINTER_CLICK_JS}
        dispatchClick(el);
        return 'ok';
      }`);
      if (v === "ok") return;
      if (!String(v).startsWith("NOT_FOUND_CARD")) {
        throw new Error(`clickTaskAction 失败：${v}`);
      }
      if (Date.now() - start >= timeout) {
        throw new Error(`clickTaskAction 失败（等待 ${timeout}ms）：${v}`);
      }
      await sleep(200);
    }
  }

  /** 在任务卡片上触发右键菜单（等待卡片渲染，消除导航后立即操作的偶发失败） */
  async contextMenuOnCard(fileName, { timeout = 8000 } = {}) {
    const start = Date.now();
    while (true) {
      const v = await this.eval(`() => {
        const name = ${JSON.stringify(fileName)};
        const cards = Array.from(document.querySelectorAll('.task-card'));
        const card = cards.find(c => (c.textContent || '').includes(name));
        if (!card) return 'NOT_FOUND_CARD:' + name;
        card.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2 }));
        return 'ok';
      }`);
      if (v === "ok") return;
      if (Date.now() - start >= timeout) {
        throw new Error(`contextMenuOnCard 失败（等待 ${timeout}ms）：${v}`);
      }
      await sleep(200);
    }
  }

  /** 点击任务卡片（打开详情面板；等待卡片渲染，消除导航后立即操作的偶发失败） */
  async clickCard(fileName, { timeout = 8000 } = {}) {
    const start = Date.now();
    while (true) {
      const v = await this.eval(`() => {
        const name = ${JSON.stringify(fileName)};
        const cards = Array.from(document.querySelectorAll('.task-card'));
        const card = cards.find(c => (c.textContent || '').includes(name));
        if (!card) return 'NOT_FOUND_CARD:' + name;
        ${POINTER_CLICK_JS}
        dispatchClick(card);
        return 'ok';
      }`);
      if (v === "ok") return;
      if (Date.now() - start >= timeout) {
        throw new Error(`clickCard 失败（等待 ${timeout}ms）：${v}`);
      }
      await sleep(200);
    }
  }

  async clickFirstCheckbox() {
    const v = await this.eval(`() => {
      const el = document.querySelector('input[type="checkbox"]');
      if (!el) return 'NOT_FOUND_CHECKBOX';
      ${POINTER_CLICK_JS}
      dispatchClick(el);
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`clickFirstCheckbox 失败：${v}`);
  }

  /** 点开 reka Select（按触发器当前文本）并选择选项 */
  async selectOption(triggerText, optionText) {
    await this.clickText(triggerText, {
      tag: '[role="combobox"], button[role="combobox"], [role="button"], button',
    });
    await sleep(350);
    const v = await this.eval(`() => {
      const text = ${JSON.stringify(optionText)};
      const nodes = Array.from(document.querySelectorAll('[role="option"]'));
      const el = nodes.find(n => (n.textContent || '').trim() === text);
      if (!el) return 'NOT_FOUND_OPTION:' + text;
      el.focus();
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`selectOption 失败：${v}`);
    // 真实键盘事件：在聚焦的选项上按 Enter 完成选择
    await this.pressKey("Enter");
  }

  /** 点按标签所在设置行内的 Switch */
  async clickSwitch(label) {
    const v = await this.eval(`() => {
      const label = ${JSON.stringify(label)};
      const rows = Array.from(document.querySelectorAll('div.flex.items-center.justify-between'));
      const row = rows.find(r => (r.textContent || '').includes(label));
      const sw = row && row.querySelector('[role="switch"]');
      if (!sw) return 'NOT_FOUND_SWITCH:' + label;
      ${POINTER_CLICK_JS}
      dispatchClick(sw);
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`clickSwitch 失败：${v}`);
  }

  async focusFirst(selector) {
    const v = await this.eval(`() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return 'NOT_FOUND_SELECTOR:' + ${JSON.stringify(selector)};
      el.focus();
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`focusFirst 失败：${v}`);
  }

  async pressKey(key) {
    await this.client.call("press_key", { key });
  }

  async fillByPlaceholder(placeholder, value) {
    const v = await this.eval(`() => {
      const ph = ${JSON.stringify(placeholder)};
      const el = document.querySelector(
        'input[placeholder="' + ph + '"], textarea[placeholder="' + ph + '"]'
      );
      if (!el) return 'NOT_FOUND_PLACEHOLDER:' + ph;
      const proto = el instanceof HTMLTextAreaElement
        ? HTMLTextAreaElement.prototype
        : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, ${JSON.stringify(value)});
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`fillByPlaceholder 失败：${v}`);
  }

  async fillBySelector(selector, value) {
    const v = await this.eval(`() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return 'NOT_FOUND_SELECTOR:' + ${JSON.stringify(selector)};
      const proto = el instanceof HTMLTextAreaElement
        ? HTMLTextAreaElement.prototype
        : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, ${JSON.stringify(value)});
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`fillBySelector 失败：${v}`);
  }

  /** 向文本域派发拖放文本事件 */
  async dropTextOnTextarea(text) {
    const v = await this.eval(`() => {
      const ta = document.querySelector('textarea');
      if (!ta) return 'NOT_FOUND_TEXTAREA';
      const dt = new DataTransfer();
      dt.setData('text/plain', ${JSON.stringify(text)});
      ta.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
      return 'ok';
    }`);
    if (v !== "ok") throw new Error(`dropTextOnTextarea 失败：${v}`);
  }

  // ============ mock 控制 ============

  async mockState() {
    return this.eval(`() => window.__STREAMGRAB_MOCK__.getState()`);
  }

  async mockCalls() {
    return this.eval(`() => window.__STREAMGRAB_MOCK__.getCalls()`);
  }

  async mockCallsOf(command) {
    return this.eval(
      `() => window.__STREAMGRAB_MOCK__.getCalls().filter(c => c.command === ${JSON.stringify(command)})`,
    );
  }

  async mockClearCalls() {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.clearCalls(); return 'ok'; }`,
    );
  }

  async mockEmit(event, payload = null) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.emit(${JSON.stringify(event)}, ${JSON.stringify(payload)}); return 'ok'; }`,
    );
  }

  async mockSetClipboard(text) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.setClipboardText(${JSON.stringify(text)}); return 'ok'; }`,
    );
  }

  async mockSetParseResult(url, result) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.setParseResult(${JSON.stringify(url)}, ${JSON.stringify(result)}); return 'ok'; }`,
    );
  }

  async mockSetDialogResult(kind, value) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.setDialogResult(${JSON.stringify(kind)}, ${JSON.stringify(value)}); return 'ok'; }`,
    );
  }

  async mockSetFileExists(path, exists) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.setFileExists(${JSON.stringify(path)}, ${JSON.stringify(exists)}); return 'ok'; }`,
    );
  }

  async mockSetTrayStatus(created, error = null) {
    return this.evalOk(
      `() => { window.__STREAMGRAB_MOCK__.setTrayStatus(${JSON.stringify(created)}, ${JSON.stringify(error)}); return 'ok'; }`,
    );
  }

  async mockListenerCount(event) {
    return this.eval(
      `() => window.__STREAMGRAB_MOCK__.listenerCount(${JSON.stringify(event)})`,
    );
  }

  async screenshot(fileName) {
    try {
      await this.client.call("take_screenshot", {
        filePath: resolve(ROOT, `test-results/${fileName}.png`),
      });
    } catch (e) {
      process.stderr.write(`[screenshot 失败] ${e.message}\n`);
    }
  }
}

// ============ 极简测试框架 ============

export const tests = [];

export function test(name, fn) {
  tests.push({ name, fn });
}

export function assert(cond, message) {
  if (!cond) throw new Error(message || "断言失败");
}

export function assertEqual(actual, expected, message) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `${message || "断言失败"}：实际 ${JSON.stringify(actual)} ≠ 期望 ${JSON.stringify(expected)}`,
    );
  }
}
